mod userstyle;

use std::io::{self, BufRead, Read, Write};
use config::{Config, Style, StyleType};
use std::fs::{File, OpenOptions};
use std::collections::HashMap;
use clap::ArgMatches;
use errors::*;
use reqwest;

pub fn run(matches: &ArgMatches) -> Result<()> {
    let uris = matches.values_of_lossy("STYLE").unwrap();

    for uri in uris {
        println!("Adding '{}':", uri);
        add_style(&uri)?;
        println!("Added style '{}'\n", uri);
    }

    println!("Added all styles!");

    Ok(())
}

// Add a single style to config and userContent
// TODO: Add domain as option for local and remote
// FIXME: For userstyle it might be possible to use the style object
// FIXME: So css() only overwrites the css of the style object
// FIXME: This makes getting settings and domain easier
fn add_style(uri: &str) -> Result<()> {
    // Get css and settings
    let stdin = io::stdin();
    let ((css, settings), style_type) = if uri.starts_with('/') {
        (local_css(uri)?, StyleType::Local)
    } else if uri.contains('.') {
        (remote_css(uri)?, StyleType::Remote)
    } else {
        let res = userstyle::css(uri, &mut stdin.lock())?;
        (res, StyleType::Userstyle)
    };

    // Get current config
    let mut config = Config::load()?;

    // Add style to config
    let id = config.next_style_id();
    let domain = read_domain(&mut stdin.lock());
    let style = Style {
        id,
        domain,
        style_type: style_type,
        settings,
    };
    config.styles.push(style);

    // Save new config
    config.write()?;

    // Save new userContent
    let content = ["\n", &css].concat();

    let mut openopts = OpenOptions::new();
    openopts.write(true).append(true).create(true);
    openopts
        .open(&config.user_content)?
        .write_all(content.as_bytes())?;

    Ok(())
}

fn read_domain<T: BufRead>(input: &mut T) -> String {
    println!("Please select a target domain:");
    print!(" > ");
    let _ = io::stdout().flush();

    loop {
        let mut choice = String::new();
        if input.read_line(&mut choice).is_err() {
            println!("Invalid input. Please try again");
        } else {
            choice = choice.trim().to_owned();
            return choice;
        }
    }
}

fn local_css(path: &str) -> Result<(String, HashMap<String, String>)> {
    let mut css = String::new();
    File::open(path)?.read_to_string(&mut css)?;
    Ok((css, HashMap::new()))
}

fn remote_css(url: &str) -> Result<(String, HashMap<String, String>)> {
    let mut css = String::new();
    reqwest::get(url)?.read_to_string(&mut css)?;
    Ok((css, HashMap::new()))
}


////////// TESTS //////////


#[test]
#[allow(non_snake_case)]
fn read_domain__with_input__returns_input() {
    let mut cursor = io::Cursor::new(b"input");

    let result = read_domain(&mut cursor);

    assert_eq!(result, "input");
}
