use clap::ArgMatches;
use config::Config;
use errors::*;
use remove;
use add;

pub fn run(matches: &ArgMatches) -> Result<()> {
    let styles = match matches.values_of_lossy("STYLE") {
        Some(styles) => styles,
        None => Config::load()?.styles.iter().map(|s| s.id.to_string()).collect(),
    };
    let edit = matches.is_present("edit");

    for style in styles {
        update_style(&style, edit)?;
    }

    Ok(())
}

fn update_style(style: &str, edit: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Get the id of the style that will be updated
    let id = config.style_id_from_str(style).ok_or("Invalid style id or name")?;

    // Get current style
    let current_style = config.pop_style(id).ok_or("Unable to find style in config")?;

    // Remove old style
    remove::remove_style(&id.to_string())?;

    // Add new updated style
    if edit {
        add::add_style(&current_style.uri, None)?;
    } else {
        add::add_style(&current_style.uri.clone(), Some(current_style))?;
    }

    Ok(())
}
