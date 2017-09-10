use std::io::{Read, Write};
use std::path::PathBuf;
use clap::ArgMatches;
use config::Config;
use errors::*;
use std::fs;
use remove;
use config;
use add;

pub fn run(matches: &ArgMatches) -> Result<()> {
    // Make sure the /chrome folder exists
    let config = Config::load()?;
    fs::create_dir_all(config.chrome_path)?;

    let styles = match matches.values_of_lossy("STYLE") {
        Some(styles) => styles,
        None => Config::load()?
            .styles
            .iter()
            .map(|s| s.name.clone())
            .collect(),
    };
    let edit = matches.is_present("edit");

    for style in styles {
        if matches.is_present("toggle") {
            println!("");
            toggle_style(&style)?;
        } else {
            println!("");
            update_style(&style, edit)?;
        }
    }

    Ok(())
}

// Toggles a style
// Enable if Disabled, Disable if Enabled
fn toggle_style(style: &str) -> Result<()> {
    println!("Toggling '{}'", style);

    // Load config and backup initial state
    let mut config = Config::load()?;
    let config_backup = config.clone();

    // Get the id of the style that will be toggled
    let id = config
        .style_id_from_str(style)
        .ok_or("Invalid style id or name")?;

    // Toggle the style in the config
    config.toggle_style(id)?;

    config.write()?;

    // Update target file
    let result = update_style(style, false);

    // Recover config if update failed
    if let Err(error) = result {
        config::restore_config(&config_backup, &error)?;
    }

    println!("Toggled style '{}'", style);

    Ok(())
}

// Update a style
// Asks about settings again if `edit` is true
fn update_style(style: &str, edit: bool) -> Result<()> {
    println!("Updating '{}'", style);

    // Load config and backup initial state
    let mut config = Config::load()?;
    let config_backup = config.clone();

    // Get the id of the style that will be updated
    let id = config
        .style_id_from_str(style)
        .ok_or("Invalid style id or name")?;

    // Get current style
    let current_style = config
        .remove_style(id)
        .ok_or("Unable to find style in config")?;

    // Load initial state of the target file as backup
    let target_path = current_style.path.clone();
    let mut file_backup = String::new();
    {
        // If file could not be found, leave the file backup empty
        // This will just create an empty file after recovery
        let _ = fs::File::open(&target_path).and_then(|mut f| f.read_to_string(&mut file_backup));
    }

    // Get the chrome path and check target file
    let path_str = target_path.to_str().ok_or("Invalid file path")?;
    let user_chrome = path_str.ends_with("userChrome.css");

    // Remove old style
    remove::remove_style(&id.to_string())?;

    let enabled = current_style.enabled;
    // Add new updated style
    let result = if edit {
        add::add_style(&current_style.uri, user_chrome, None, !enabled)
    } else {
        add::add_style(
            &current_style.uri.clone(),
            user_chrome,
            Some(current_style),
            !enabled,
        )
    };

    // Recover both config and target file if add failed
    if result.is_err() {
        recover_failure(&config_backup, &file_backup, &target_path)?;
    }

    println!("Updated style '{}'", style);

    Ok(())
}

fn recover_failure(config_backup: &Config, file_backup: &str, target_path: &PathBuf) -> Result<()> {
    eprintln!("\x1b[0;31;40mUnable to update style\x1b[0m");
    println!("Attempting to restore config and target file");

    // Recover config
    let config_result = config_backup.write().map_err(
        |_| "Unable to recover config\nPlease ensure the config is not corrupted",
    );

    // Recover target file
    let file_result = fs::File::create(&target_path)
        .and_then(|mut f| f.write_all(file_backup.as_bytes()))
        .map_err(
            |_| "Unable to recover target file\nPlease ensure the file is not corrupted",
        );

    // Propagate recovery attempt failure
    match (config_result, file_result) {
        (Err(e1), Err(e2)) => Err((e1.to_string() + e2).into()),
        (Err(e), _) | (_, Err(e)) => Err(e.into()),
        _ => Ok(()),
    }
}
