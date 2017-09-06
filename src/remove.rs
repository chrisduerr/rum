use std::io::{Read, Write};
use std::path::PathBuf;
use clap::ArgMatches;
use std::fs::File;
use errors::*;
use config;

pub fn run(matches: &ArgMatches) -> Result<()> {
    let styles = matches.values_of_lossy("STYLE").unwrap();

    for style in styles {
        println!("Removing '{}'", style);
        remove_style(&style)?;
        println!("Removed style '{}'\n", style);
    }

    println!("Removed all styles!");

    Ok(())
}

pub fn remove_style(style: &str) -> Result<()> {
    // Get current config
    let mut config = config::Config::load()?;
    let config_backup = config.clone();

    // Get the id of the style that will be removed
    let id = config
        .style_id_from_str(style)
        .ok_or("Invalid style id or name")?;

    let path = config.file_path_by_id(id).ok_or("Invalid file path")?;

    // Remove style from config
    config.remove_style(id);

    // Save config
    config.write()?;

    // Remove from file
    let result = remove_from_file(id, &path);

    // Restore config if style could not be removed from file
    if let Err(e) = result {
        config::restore_config(&config_backup, &e)?;
    }

    Ok(())
}

fn remove_from_file(id: i32, path: &PathBuf) -> Result<()> {
    // Read current content
    let mut content = String::new();
    {
        match File::open(path) {
            Ok(mut file) => {
                file.read_to_string(&mut content)?;
            }
            Err(e) => {
                println!("Unable to find '{}': {}", path.to_string_lossy(), e);
                println!("Removing style only from config");
                return Ok(());
            }
        };
    }

    // Get new content
    content = remove_style_from_str(&content, id);

    // Write new file
    File::create(path)?.write_all(content.as_bytes())?;

    Ok(())
}

fn remove_style_from_str(user_content: &str, id: i32) -> String {
    let start_str = config::RUM_START.replace("{}", &id.to_string());
    let end_str = config::RUM_END.replace("{}", &id.to_string());

    if let Some(start) = user_content.find(&start_str) {
        if let Some(end) = user_content.find(&end_str) {
            if start < end {
                let mut result = user_content[..start].to_owned();
                result.push_str(&user_content[end + end_str.len()..]);
                return result;
            }
        }
    }

    user_content.to_owned()
}

#[test]
#[allow(non_snake_case)]
fn remove_style_from_str__with_id_zero__removes_style_zero() {
    let user_content = "foobar\n\n/* RUM START 0 */\nstyle\n\
                        /* RUM END 0 */\n\n/* RUM START 1 */\n\n/* RUM END 1 */\n";

    let result = remove_style_from_str(user_content, 0);

    assert_eq!(result, "foobar\n\n/* RUM START 1 */\n\n/* RUM END 1 */\n");
}
