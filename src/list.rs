use config::{Config, Style};
use clap::ArgMatches;
use errors::*;

pub fn run(matches: &ArgMatches) -> Result<()> {
    // Check if user wants verbose information
    let verbose = matches.is_present("verbose");

    // Load current config file
    let config = Config::load()?;

    // Sort styles by ID to make it easier on the eyes
    let mut styles = config.styles.clone();
    styles.sort_by_key(|s| s.id);

    // Print output for every style
    for style in styles {
        if verbose {
            print_verbose(style);
        } else {
            print(style);
        }
    }

    Ok(())
}

// Print non-verbose information about a style
fn print(style: Style) {
    // Get the ID as a string, this makes formatting easier
    let id_str = ["(", &style.id.to_string(), ")"].concat();

    // Print the information based on status and target file
    if !style.enabled {
        println!("{:5} [DISABLED] {}", id_str, style.name);
    } else if style.path.to_string_lossy().ends_with("userChrome.css") {
        println!("{:5} [CHROME]   {}", id_str, style.name);
    } else {
        println!("{:5} [CONTENT]  {}", id_str, style.name);
    };
}

// Print verbose information about a style
fn print_verbose(style: Style) {
    // Shorten the target to the bare minimum
    let target = if style.path.to_string_lossy().ends_with("userChrome.css") {
        "userChrome"
    } else {
        "userContent"
    };

    // Print the information
    println!("{}", style.name);
    println!("    ID: {}", style.id);
    println!("    URI: {}", style.uri);
    println!("    TARGET: {}", target);
    println!("    TYPE: {:?}", style.style_type);
    println!("    DOMAIN: {}", style.domain.unwrap_or_default());
    println!("    ENABLED: {}", style.enabled);
    println!("");
}
