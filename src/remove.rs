use std::io::{Read, Write};
use std::path::PathBuf;
use clap::ArgMatches;
use std::fs::File;
use errors::*;
use config;

pub fn run(matches: &ArgMatches) -> Result<()> {
    let styles = matches.values_of_lossy("STYLE").unwrap();

    for style in styles {
        println!("");
        remove_style(&style)?;
    }

    Ok(())
}

pub fn remove_style(style: &str) -> Result<()> {
    println!("Removing '{}'", style);

    // Get current config
    let mut config = config::Config::load()?;
    let config_backup = config.clone();

    // Remove style from config
    let removed_style = config.remove_style(style).ok_or("Invalid style id or name")?;

    // Save config
    config.write()?;

    // Remove from file
    let result = remove_from_file(removed_style.id, &removed_style.path);

    // Restore config if style could not be removed from file
    if let Err(e) = result {
        config::restore_config(&config_backup, &e)?;
    }

    println!("Removed style '{}'", style);

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
                error!("Unable to find '{}': {}", path.to_string_lossy(), e);
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

// Remove a style with RUM tags from a string slice
fn remove_style_from_str(user_content: &str, id: i32) -> String {
    // Replace placeholders with ID
    let start_str = config::RUM_START.replace("{}", &id.to_string());
    let end_str = config::RUM_END.replace("{}", &id.to_string());

    // Get index of start and end tags
    if let Some(start) = user_content.find(&start_str) {
        if let Some(end) = user_content.find(&end_str) {
            if start < end {
                // Substring the slice to remove the
                // contents between start and end tags
                let mut result = user_content[..start].to_owned();
                result.push_str(&user_content[end + end_str.len()..]);
                return result;
            }
        }
    }

    // Return the original in case something didn't work out
    user_content.to_owned()
}


////////// TESTS //////////


#[cfg(test)]
fn mock_config(config: config::Config) {
    let mut mock = config::MOCK_CONFIG.lock().unwrap();
    (*mock) = config;
}

#[test]
#[allow(non_snake_case)]
fn remove_style_from_str__with_id_zero__removes_style_zero() {
    let user_content = "foobar\n\n/* RUM START 0 */\nstyle\n\
                        /* RUM END 0 */\n\n/* RUM START 1 */\n\n/* RUM END 1 */\n";

    let result = remove_style_from_str(user_content, 0);

    assert_eq!(result, "foobar\n\n/* RUM START 1 */\n\n/* RUM END 1 */\n");
}

#[test]
#[allow(non_snake_case)]
fn remove_style_from_str__with_start_before_end__returns_original() {
    let user_content = "foobar\n\n/* RUM END 0 */\nstyle\n\
                        /* RUM START 0 */\n\n/* RUM START 1 */\n\n/* RUM END 1 */\n";

    let result = remove_style_from_str(user_content, 0);

    assert_eq!(result, user_content);
}

#[test]
#[allow(non_snake_case)]
fn remove_style_from_str__with_tags_missing__returns_original() {
    let user_content = "no tags";

    let result = remove_style_from_str(user_content, 0);

    assert_eq!(result, user_content);
}

#[test]
#[allow(non_snake_case)]
fn remove_style__with_style_name_one__removes_style_from_config() {
    config::clear_writer();
    let mut style = config::dummy_style();
    style.name = String::from("one");
    let config = config::dummy_config(vec![style]);
    mock_config(config);

    remove_style("one").unwrap();

    let writer = config::WRITER.lock().unwrap();
    let content = String::from_utf8_lossy(&(*writer));
    assert_eq!(content, "chrome_path = \"\"\nstyles = []\n");
}

#[test]
#[allow(non_snake_case)]
fn remove_style__with_style_id_one__removes_style_from_config() {
    config::clear_writer();
    let mut style = config::dummy_style();
    style.id = 1;
    let config = config::dummy_config(vec![style]);
    mock_config(config);

    remove_style("1").unwrap();

    let writer = config::WRITER.lock().unwrap();
    let content = String::from_utf8_lossy(&(*writer));
    assert_eq!(content, "chrome_path = \"\"\nstyles = []\n");
}

#[test]
#[should_panic]
#[allow(non_snake_case)]
fn remove_style__with_invalid_style__panics() {
    let config = config::dummy_config(Vec::new());
    mock_config(config);

    remove_style("1").unwrap();
}
