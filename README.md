# GMAIL Forwarder

This project allow to automatically forward emails that arrive in a particular GMAIL IMAP folder to a list of recipients.
**Motivation:** allow to transfer email without configuring the email forward feature of GMAIL and also works for email created by API (not the case for the GMAIL forward feature).

## Usage

```
# from source: cargo run -- --help
# from binary:
./gmail_forwarder --help
gmail_forwarder 0.1.0
Simple program to forword GMail email TO another email

USAGE:
    gmail_forwarder [OPTIONS] --username <USERNAME> --password <PASSWORD> --to <TO>

OPTIONS:
    -f, --folder <FOLDER>        folder to watch for new email, default is the INBOX [default:
                                 INBOX]
    -h, --help                   Print help information
    -j, --subject <SUBJECT>      if the subject of an email matches (find) this regular expression,
                                 the email can be forwarded [default: ]
    -p, --password <PASSWORD>    mandatory GMAIL account's password or access key
    -s, --sender <SENDER>        if the sender of an email matches (find) this regular expression,
                                 the email can be forwarded [default: ]
    -t, --to <TO>                mandatory recipient email (use , to set many)
    -u, --username <USERNAME>    mandatory GMAIL account that receive emails
    -V, --version                Print version information
```

Sample of usage:

```
./gmail_forwarder -u gmailaccount -p gmailpassword -t recipient@mail.com
```

## Build

You need a **RUST** environment whith **cargo** to build:

```
cargo build --release
```