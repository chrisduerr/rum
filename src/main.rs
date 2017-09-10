#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

extern crate base64;
extern crate reqwest;
extern crate toml;
extern crate userstyles;

// Macro for printing red text to stderr
macro_rules! error {
    ($fmt:expr) => (eprintln!("\x1b[0;31;40m{}\x1b[0m\n", $fmt));
    ($fmt:expr, $($arg:tt)*) => (eprintln!($fmt, $($arg)*));
}

mod add;
mod list;
mod remove;
mod config;
mod update;
mod userstyle;
mod errors {
    error_chain!{
        foreign_links {
            IoError(::std::io::Error);
            TomlError(::toml::de::Error);
            ReqwestError(::reqwest::Error);
            TomlSerError(::toml::ser::Error);
            ParseIntError(::std::num::ParseIntError);
        }
    }
}

use std::sync::{Arc, Mutex};
use std::process::exit;
use errors::*;
use clap::App;
use std::io;

// Setup a mock for stdin to make testing possible without excessive parameters
#[cfg(not(test))]
lazy_static! {
    static ref READER: Arc<Mutex<io::Stdin>> = {
        Arc::new(Mutex::new(io::stdin()))
    };
}

#[cfg(test)]
lazy_static! {
    static ref READER: Arc<Mutex<io::Cursor<Vec<u8>>>> = {
        Arc::new(Mutex::new(io::Cursor::new(Vec::new())))
    };
}

quick_main!(run);
fn run() -> Result<()> {
    // Parse CLI parameters
    let yaml = load_yaml!("clap.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // Execute subcommnd with CLI parameters
    if let Some(subcommand) = matches.subcommand_name() {
        match subcommand {
            "add" => add::run(submatches(&matches, "add"))?,
            "list" => list::run(submatches(&matches, "list"))?,
            "remove" => remove::run(submatches(&matches, "remove"))?,
            "update" => update::run(submatches(&matches, "update"))?,
            _ => (),
        };
    } else {
        // Complain when no parameters are specified
        error!("No operation specified (use -h for help)");
        exit(1);
    }

    Ok(())
}

#[inline]
fn submatches<'a>(matches: &'a clap::ArgMatches, subcommand: &str) -> &'a clap::ArgMatches<'a> {
    // This is unwrapped because it's only executed when the subcommand is called
    matches.subcommand_matches(subcommand).unwrap()
}
