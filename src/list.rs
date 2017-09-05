use config::{Config, Style};
use clap::ArgMatches;
use errors::*;

pub fn run(matches: &ArgMatches) -> Result<()> {
    let verbose = matches.is_present("verbose");
    let config = Config::load()?;

    let mut styles = config.styles.clone();
    styles.sort_by_key(|s| s.id);

    for style in styles {
        if verbose {
            print_verbose(style);
        } else {
            let id_str = ["(", &style.id.to_string(), ")"].concat();
            if style.path.to_string_lossy().ends_with("userChrome.css") {
                println!("{:5} [CONTENT] {}", id_str, style.name);
            } else {
                println!("{:5} [CHROME]  {}", id_str, style.name);
            };
        }
    }

    Ok(())
}

fn print_verbose(style: Style) {
    // Shorten the target to the bare minimum
    let target = if style.path.to_string_lossy().ends_with("userChrome.css") {
        "userChrome"
    } else {
        "userContent"
    };

    println!("{}", style.name);
    println!("    ID: {}", style.id);
    println!("    URI: {}", style.uri);
    println!("    TARGET: {}", target);
    println!("    TYPE: {:?}", style.style_type);
    println!("    DOMAIN: {}", style.domain.unwrap_or_default());
    println!("");
}
