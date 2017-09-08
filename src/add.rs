use config::{self, Config, Style, StyleType};
use std::io::{self, BufRead, Read, Write};
use std::fs::{self, File, OpenOptions};
use std::collections::HashMap;
use std::path::PathBuf;
use clap::ArgMatches;
use errors::*;
use userstyle;
use reqwest;

pub fn run(matches: &ArgMatches) -> Result<()> {
    // Make sure the /chrome folder exists
    let config = Config::load()?;
    fs::create_dir_all(config.chrome_path)?;

    let uris = matches.values_of_lossy("STYLE").unwrap();

    for uri in uris {
        println!("");
        add_style(&uri, matches.is_present("userchrome"), None, false)?;
    }

    Ok(())
}

// Add a single style to config and userContent
pub fn add_style(
    uri: &str,
    user_chrome: bool,
    current_style: Option<Style>,
    config_only: bool,
) -> Result<()> {
    println!("Adding '{}':", uri);

    // Get current config
    let mut config = Config::load()?;
    let config_backup = config.clone();

    let id = config.next_style_id();

    // Get correct file path
    let mut file_path = PathBuf::from(&config.chrome_path);
    if user_chrome {
        file_path.push("userChrome.css");
    } else {
        file_path.push("userContent.css");
    }

    // Get css and settings
    let stdin = io::stdin();
    let mut style = if uri.starts_with('/') {
        local_style(uri, id, current_style, file_path.clone(), &mut stdin.lock())?
    } else if uri.contains('.') {
        remote_style(uri, id, current_style, file_path.clone(), &mut stdin.lock())?
    } else {
        userstyle::style(uri, id, current_style, file_path.clone(), &mut stdin.lock())?
    };

    // Add domain to CSS
    if let Some(ref domain) = style.domain {
        style.css = format!("@-moz-document {} {{\n{}\n}}", domain, style.css);
    }

    // Add style to config
    config.styles.push(style.clone());

    // Save new config
    config.write()?;

    // Return if it should not be added to the target file
    if config_only {
        println!("Added style '{}'", uri);
        return Ok(());
    }

    // Save new File
    let start = config::RUM_START.replace("{}", &id.to_string());
    let end = config::RUM_END.replace("{}", &id.to_string());
    let content = start + &style.css + &end;

    let mut openopts = OpenOptions::new();
    openopts.write(true).append(true).create(true);
    let result = openopts
        .open(&file_path)
        .and_then(|mut f| f.write_all(content.as_bytes()));

    // Restore config if style could not be written
    if let Err(e) = result {
        config::restore_config(&config_backup, &Error::from(e))?;
    }

    println!("Added style '{}'", uri);

    Ok(())
}

// Read any text input from the user
// Loops untile input is valid
fn read_text<T: BufRead>(text: &str, input: &mut T) -> String {
    print!("{}", text);
    let _ = io::stdout().flush();

    loop {
        let mut choice = String::new();
        if input.read_line(&mut choice).is_err() {
            eprintln!("\x1b[0;31;40mInvalid input. Please try again.\x1b[0m");
            print!(" > ");
            let _ = io::stdout().flush();
        } else {
            choice = choice.trim().to_owned();
            return choice;
        }
    }
}

// Get the name for a style from the user
fn read_name<T: BufRead>(input: &mut T) -> String {
    read_text("Please select a name for this style:\n > ", input)
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
    let helptext = "Please select a target domain:\nExample: 'domain(\"kernel.org\")'\n > ";
    Some(read_text(helptext, input))
}

// Load a local style
fn local_style<T: BufRead>(
    path: &str,
    id: i32,
    style: Option<Style>,
    file_path: PathBuf,
    input: &mut T,
) -> Result<Style> {
    let mut css = String::new();
    File::open(path)?.read_to_string(&mut css)?;

    generic_style(path, id, css, style, file_path, input)
}

// Load a remote style
fn remote_style<T: BufRead>(
    url: &str,
    id: i32,
    style: Option<Style>,
    file_path: PathBuf,
    input: &mut T,
) -> Result<Style> {
    let mut css = String::new();
    reqwest::get(url)?.read_to_string(&mut css)?;

    generic_style(url, id, css, style, file_path, input)
}

// Generic method that creates style from CSS only
fn generic_style<T: BufRead>(
    uri: &str,
    id: i32,
    css: String,
    style: Option<Style>,
    path: PathBuf,
    input: &mut T,
) -> Result<Style> {
    // Update existing style
    if let Some(mut style) = style {
        style.css = css;
        return Ok(style);
    }

    // Add new style
    let name = read_name(input);
    let domain = if path.ends_with("userChrome.css") {
        Some(String::from("url(chrome://browser/content/browser.xul)"))
    } else {
        read_domain(input)
    };

    Ok(Style {
        id,
        name,
        domain,
        path,
        enabled: true,
        uri: uri.to_owned(),
        style_type: StyleType::Local,
        settings: HashMap::new(),
        css,
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

#[test]
#[allow(non_snake_case)]
fn read_text__with_invalid_bytes__return_next_valid_input() {
    let mut cursor = io::Cursor::new(&[255, 10, 98]);

    let result = read_text("", &mut cursor);

    assert_eq!(result, "b");
}
