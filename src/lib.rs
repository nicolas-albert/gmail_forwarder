mod cli;

use cli::Args;
use imap::extensions::idle::WaitOutcome::MailboxChanged;
use imap::types::UnsolicitedResponse;
use imap::Session;
use lettre::message::header::*;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use log::{error, info};
use regex::Regex;
use retry::delay::Fixed;
use retry::retry;
use rustls_connector::TlsStream;
use std::error::Error;
use std::net::TcpStream;
use std::time::Duration;

pub struct App {
    args: Args,
    connected: bool,
    exists: u32,
    host_imap: String,
    host_smtp: String,
    port: u16,
    session: Option<Session<TlsStream<TcpStream>>>,
    re_sender: Option<Regex>,
    re_subject: Option<Regex>,
}

impl std::ops::Deref for App {
    type Target = Args;
    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl App {
    fn session(&mut self) -> Result<&mut Session<TlsStream<TcpStream>>, String> {
        if let Some(session) = &mut self.session {
            Ok(session)
        } else {
            Err(String::from("no session"))
        }
    }

    pub fn parse() -> App {
        let args = cli::parse();
        App {
            connected: false,
            exists: 0,
            host_imap: String::from("imap.gmail.com"),
            host_smtp: String::from("smtp.gmail.com"),
            port: 993,
            session: None,
            re_sender: match Regex::new(&args.sender.as_str()) {
                Ok(r) => Some(r),
                _ => {
                    println!("invalid sender expression");
                    None
                }
            },
            re_subject: match Regex::new(&args.subject.as_str()) {
                Ok(r) => Some(r),
                _ => {
                    println!("invalid subject expression");
                    None
                }
            },
            args: args,
        }
    }

    fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let client = imap::ClientBuilder::new(&self.host_imap, self.port).rustls()?;
        self.session = Some(
            client
                .login(&self.username, &self.password)
                .map_err(|e| e.0)?,
        );
        let mailbox = self.session()?.select("INBOX")?;
        self.exists = mailbox.exists;
        self.connected = true;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Starting the run loop");
        loop {
            let result = retry(Fixed::from_millis(30000).take(20), || self.connect());
            match result {
                Ok(_) => info!("Connected to IMAP!"),
                Err(e) => error!("Failed to connect IMAP: {:?}", e),
            }
            if let Err(e) = self.watch_and_forward() {
                self.connected = false;
                error!("Error: {}", e);
            }
            if !self.connected {
                if let Ok(session) = self.session() {
                    let _ = session.close();
                }
            }
        }
    }

    fn watch_and_forward(&mut self) -> Result<(), Box<dyn Error>> {
        let mut exists = Some(self.exists);
        let mut bye = false;
        if let Ok(MailboxChanged) = self
            .session()?
            .idle()
            .timeout(Duration::from_secs(60 * 10))
            .wait_while(|response| {
                info!("IDLE response: {:?}", response);
                if let UnsolicitedResponse::Exists(id) = response {
                    let last_exists = exists.unwrap();
                    exists = Some(id);
                    if id > last_exists {
                        return false;
                    }
                } else if let UnsolicitedResponse::Bye { .. } = response {
                    bye = true;
                    exists = None;
                    return false;
                }
                true
            })
        {
        } else {
            info!("Wait not OK, do bye");
            bye = true;
            exists = None;
        }

        self.connected = !bye;

        if let Some(exists) = exists {
            let messages = self
                .session()?
                .fetch(exists.to_string(), "(BODY[HEADER] BODY[TEXT])")?;
            if let Some(m) = messages.iter().next() {
                let headers = m.header().ok_or("no header")?;
                let headers = std::str::from_utf8(headers).expect("message was not valid utf-8");
                let headers: std::collections::HashMap<&str, &str> = headers
                    .lines()
                    .map(|l| match l.splitn(2, ":").collect::<Vec<&str>>()[..] {
                        [a, b, ..] => (a, b.trim()),
                        _ => (l, ""),
                    })
                    .collect();

                if let Some(re) = &self.re_subject {
                    if !re.is_match(headers["Subject"]) {
                        info!("Subject not match: {}", headers["Subject"]);
                        return Ok(());
                    }
                }
                if let Some(re) = &self.re_sender {
                    if !re.is_match(headers["From"]) {
                        info!("Sender not match: {}", headers["From"]);
                        return Ok(());
                    }
                }
                if let Some(text) = m.text() {
                    self.send_email(&headers, text)?;
                }
            }
        }
        Ok(())
    }

    fn send_email(
        &self,
        headers: &std::collections::HashMap<&str, &str>,
        body: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let creds = Credentials::new(self.username.clone(), self.password.clone());
        let mailer = SmtpTransport::relay(self.host_smtp.as_str())?
            .credentials(creds)
            .build();
        for to in self.to.split(',') {
            let body = lettre::message::Body::new_with_encoding(
                body.to_vec(),
                lettre::message::header::ContentTransferEncoding::Binary,
            )
            .ok()
            .ok_or("failed to encode body")?;
            let email = Message::builder()
                .from(headers["From"].parse()?)
                .to(to.parse()?)
                .subject(headers["Subject"])
                .header(ContentType::parse(headers["Content-Type"])?)
                .body(body)?;
            let result = retry(Fixed::from_millis(30000).take(20), || mailer.send(&email));
            match result {
                Ok(_) => info!("Email sent successfully to {}!", to),
                Err(e) => error!("Could not send email: {:?}", e),
            }
        }
        Ok(())
    }
}
