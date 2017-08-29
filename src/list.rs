use config::Config;
use errors::*;

pub fn run() -> Result<()> {
    let config = Config::load()?;
    for style in config.styles {
        println!("({}) {}", style.id, style.name);
    }

    Ok(())
}
