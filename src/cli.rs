use clap::Parser;

/// Simple program to forword GMail email TO another email
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// mandatory GMAIL account that receive emails
    #[clap(short, long)]
    pub username: String,

    /// mandatory GMAIL account's password or access key
    #[clap(short, long)]
    pub password: String,

    /// mandatory recipient email (use , to set many)
    #[clap(short, long)]
    pub to: String,

    /// folder to watch for new email, default is the INBOX
    #[clap(short, long, default_value = "INBOX")]
    pub folder: String,

    /// if the sender of an email matches (find) this regular expression, the email can be forwarded
    #[clap(short, long, default_value = "")]
    pub sender: String,

    /// if the subject of an email matches (find) this regular expression, the email can be forwarded
    #[clap(short = 'j', long, default_value = "")]
    pub subject: String,
}

pub fn parse() -> Args {
    Args::parse()
}
