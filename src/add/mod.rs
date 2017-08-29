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
fn add_style(uri: &str) -> Result<()> {
    // Get current config
    let mut config = Config::load()?;

    let id = config.next_style_id();

    // Get css and settings
    let stdin = io::stdin();
    let mut style = if uri.starts_with('/') {
        local_style(uri, id, &mut stdin.lock())?
    } else if uri.contains('.') {
        remote_style(uri, id, &mut stdin.lock())?
    } else {
        userstyle::style(uri, id, &mut stdin.lock())?
    };

    // Save new userContent
    if let Some(ref domain) = style.domain {
        style.css = format!("@-moz-document {} {{\n{}\n}}", domain, style.css);
    }
    let content = ["\n", &style.css].concat();

    let mut openopts = OpenOptions::new();
    openopts.write(true).append(true).create(true);
    openopts
        .open(&config.user_content)?
        .write_all(content.as_bytes())?;

    // Add style to config
    config.styles.push(style);

    // Save new config
    config.write()?;

    Ok(())
}

// Get the domain a style should apply to
fn read_domain<T: BufRead>(input: &mut T) -> Option<String> {
    println!("Do you want to add a domain?");
    print!("[y/N] > ");
    let _ = io::stdout().flush();

    // Return None if user doesn't want a domain
    let mut add_domain = String::new();
    let _ = input.read_line(&mut add_domain);
    if add_domain.to_lowercase().trim() != "y" {
        return None;
    }

    // Ask for the domain name that should be selected
    println!("Please select a target domain:");
    println!("Example: 'domain(\"kernel.org\")'");
    print!(" > ");
    let _ = io::stdout().flush();

    loop {
        let mut choice = String::new();
        if input.read_line(&mut choice).is_err() {
            println!("Invalid input. Please try again");
        } else {
            choice = choice.trim().to_owned();
            return Some(choice);
        }
    }
}

// Load a local style
fn local_style<T: BufRead>(path: &str, id: i32, input: &mut T) -> Result<Style> {
    let mut css = String::new();
    File::open(path)?.read_to_string(&mut css)?;
    let domain = read_domain(input);

    Ok(Style {
        id,
        domain,
        style_type: StyleType::Local,
        settings: HashMap::new(),
        css
    })
}

// Load a remote style
fn remote_style<T: BufRead>(url: &str, id: i32, input: &mut T) -> Result<Style> {
    let mut css = String::new();
    reqwest::get(url)?.read_to_string(&mut css)?;
    let domain = read_domain(input);

    Ok(Style {
        id,
        domain,
        style_type: StyleType::Remote,
        settings: HashMap::new(),
        css
    })
}


////////// TESTS //////////


#[test]
#[allow(non_snake_case)]
fn read_domain__with_add_domain_true_and_domain_input__returns_input() {
    let mut cursor = io::Cursor::new(b"y\ninput");

    let result = read_domain(&mut cursor);

    assert_eq!(result, Some(String::from("input")));
}

#[test]
#[allow(non_snake_case)]
fn read_domain__with_add_domain_false__returns_none() {
    let mut cursor = io::Cursor::new(b"n");

    let result = read_domain(&mut cursor);

    assert_eq!(result, None);
}
