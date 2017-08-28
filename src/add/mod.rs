mod userstyle;

use clap::ArgMatches;
use errors::*;

pub fn run(matches: &ArgMatches) -> Result<()> {
    let uris = matches.values_of_lossy("STYLE").unwrap();

    for uri in uris {
        add_style(&uri)?;
    }

    Ok(())
}

fn add_style(uri: &str) -> Result<()> {
    // Load style css
    //
    // Get current config
    // Add style to config
    //
    // Get current userContent
    // Add style to config

    unimplemented!();

    Ok(())
}

