mod cli;

use cli::Args;
use imap::extensions::idle::WaitOutcome::MailboxChanged;
use imap::types::UnsolicitedResponse;
use imap::Session;
use lettre::message::header::*;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use regex::Regex;
use rustls_connector::TlsStream;
use std::error::Error;
use std::net::TcpStream;

pub struct App {
    args: Args,
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

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let client = imap::ClientBuilder::new(&self.host_imap, self.port).rustls()?;
        self.session = Some(
            client
                .login(&self.username, &self.password)
                .map_err(|e| e.0)?,
        );
        let mailbox = self.session()?.select("INBOX")?;
        self.exists = mailbox.exists;
        loop {
            if let Err(e) = self.watch_and_forward() {
                println!("Error: {}", e);
            }
        }
    }

    fn watch_and_forward(&mut self) -> Result<(), Box<dyn Error>> {
        let mut exists = self.exists;
        if let Ok(MailboxChanged) = self.session()?.idle().wait_while(|response| {
            println!("IDLE response #{}: {:?}", 1, response);
            if let UnsolicitedResponse::Exists(id) = response {
                let last_exists = exists;
                exists = id;
                if id > last_exists {
                    return false;
                }
            }
            true
        }) {
        } else {
            return Ok(());
        }

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
                    println!("Subject not match: {}", headers["Subject"]);
                    return Ok(());
                }
            }
            if let Some(re) = &self.re_sender {
                if !re.is_match(headers["From"]) {
                    println!("Sender not match: {}", headers["From"]);
                    return Ok(());
                }
            }
            if let Some(text) = m.text() {
                self.send_email(&headers, text)?;
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
            let email = Message::builder()
                .from(headers["From"].parse()?)
                .to(to.parse()?)
                .subject(headers["Subject"])
                .header(ContentType::parse(headers["Content-Type"])?)
                .body(String::from_utf8(body.to_vec())?)?;
            match mailer.send(&email) {
                Ok(_) => println!("Email sent successfully to {}!", to),
                Err(e) => panic!("Could not send email: {:?}", e),
            }
        }
        Ok(())
    }
}
