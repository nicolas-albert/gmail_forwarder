use gmail_forwarder::App;
use simple_logger::SimpleLogger;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().env().init().unwrap();
    App::parse().run()?;
    Ok(())
}
