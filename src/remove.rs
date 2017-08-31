use std::io::{Read, Write};
use std::fs::OpenOptions;
use clap::ArgMatches;
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
    let mut config = config::Config::load()?;

    // Get the id of the style that will be removed
    let id = config
        .style_id_from_str(style)
        .ok_or("Invalid style id or name")?;

    // Remove style from config
    config.remove_style(id);

    // Save config
    config.write()?;

    // Remove from userContent.css
    remove_from_usercontent(id, &config.user_content)?;

    Ok(())
}

fn remove_from_usercontent(id: i32, path: &str) -> Result<()> {
    // Open usercontent file
    let mut options = OpenOptions::new();
    options.write(true).read(true).truncate(true);
    let mut file = options.open(path)?;

    // Remove style from userContent
    let mut user_content = String::new();
    file.read_to_string(&mut user_content)?;
    user_content = remove_style_from_str(&user_content, id);

    // Write new userContent
    file.write_all(user_content.as_bytes())?;

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
