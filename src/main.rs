use gmail_forwarder::App;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    App::parse().run()?;
    Ok(())
}
