use std::io::{self, BufRead, BufReader, Read, Write};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use errors::*;
use std::env;
use toml;

const CONFIG_PATH: &str = ".config/rum.toml";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub user_content: String,
    pub styles: Vec<Style>,
}

impl Config {
    fn new(user_content: String) -> Config {
        Config {
            user_content,
            styles: Vec::new(),
        }
    }

    pub fn next_style_id(&self) -> i32 {
        let mut ids: Vec<i32> = self.styles.iter().map(|s| s.id).collect();
        ids.sort_by(|a, b| a.cmp(b));
        let mut id = 0;
        for i in ids {
            if i != id {
                return id;
            }
            id += 1;
        }
        id
    }

    pub fn load() -> Result<Config> {
        let path = get_config_path()?;
        let mut content = String::new();
        File::open(path)?.read_to_string(&mut content)?;
        Ok(toml::from_str::<Config>(&content)?)
    }

    pub fn write(&self) -> Result<()> {
        let output = toml::to_string(self)?;
        let config_path = get_config_path()?;
        File::create(config_path)?.write_all(output.as_bytes())?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Style {
    pub id: i32,
    pub domain: String,
    pub style_type: StyleType,
    pub settings: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub enum StyleType {
    Userstyle,
    Local,
    Remote,
}

// Check if the config file exists
pub fn config_exists() -> bool {
    if let Ok(path) = get_config_path() {
        path.as_path().exists()
    } else {
        false
    }
}

// Create a new config file
pub fn create_config() -> Result<()> {
    // Get the profile
    let profiles_ini = get_profiles_ini()?;
    let ini_buf = BufReader::new(File::open(&profiles_ini)?);
    let profiles = get_profiles(ini_buf);

    let stdin = io::stdin();
    let profile = get_profile_selection(&profiles, &mut stdin.lock())?;

    let mut user_content = profiles_ini;
    user_content.pop();
    user_content.push(profile);
    user_content.push("chrome/userContent.css");
    let user_content = user_content.to_str().ok_or("UserContent path invalid.")?;

    // Create new config
    let config = Config::new(user_content.to_owned());
    config.write()?;

    println!("Successfully created new profile.\n");

    Ok(())
}

// Return the location of the `pofiles.ini` file
fn get_profiles_ini() -> Result<PathBuf> {
    let mut path = env::home_dir().ok_or("Unable to locate home directory")?;
    path.push(".mozilla/firefox/profiles.ini");
    Ok(path)
}

// Get list with all profile names, first is the default
fn get_profiles<T: BufRead>(profiles_buf: T) -> Vec<String> {
    let mut profiles = Vec::new();
    let mut default = false;

    for line in profiles_buf.lines() {
        let line = line.unwrap_or_else(|_| String::new());

        if line == "Name=default" {
            default = true;
        } else if line.starts_with("Path=") {
            let profile = (&line[5..]).to_owned();
            if default {
                default = false;
                profiles.insert(0, profile);
            } else {
                profiles.push(profile);
            }
        }
    }

    profiles
}

// Interact with the user to check which profile he wants
fn get_profile_selection<T: BufRead>(profiles: &[String], input: &mut T) -> Result<String> {
    println!("Select a profile:\n");

    // Iterate over all profiles
    for (i, profile) in profiles.iter().enumerate() {
        println!("    ({}) {}", i, profile);
    }

    println!("\nEnter profile number:");
    print!("[Default: 0] > ");
    io::stdout().flush()?;

    // Read user choice
    let mut user_input = String::new();
    input.read_line(&mut user_input)?;
    let user_input = user_input.trim();

    let index = if !user_input.is_empty() {
        usize::from_str_radix(user_input, 10)?
    } else {
        0
    };

    Ok(
        profiles
            .get(index)
            .ok_or("Profile number out of range.")?
            .to_owned(),
    )
}

// Get pat of config file
fn get_config_path() -> Result<PathBuf> {
    let mut path = env::home_dir().ok_or("Unable to find home directory.")?;
    path.push(CONFIG_PATH);
    Ok(path)
}


////////// TESTS //////////


#[test]
#[allow(non_snake_case)]
fn get_profiles__with_one_profile__returns_profile_name() {
    let cursor: io::Cursor<&[u8; 14]> = io::Cursor::new(b"Path=MyProfile");

    let profiles = get_profiles(cursor);

    assert_eq!(profiles[0], "MyProfile");
}

#[test]
#[allow(non_snake_case)]
fn get_profiles__with_empty_ini__returns_empty_vec() {
    let cursor: io::Cursor<&[u8; 0]> = io::Cursor::new(b"");

    let profiles = get_profiles(cursor);

    assert_eq!(profiles.len(), 0);
}

#[test]
#[allow(non_snake_case)]
fn get_profiles__with_multiple_profiles__returns_correct_default() {
    let content = "Path=nondefault\nName=default\nPath=default\nPath=nondefault";
    let cursor = io::Cursor::new(content.as_bytes());

    let profiles = get_profiles(cursor);

    assert_eq!(profiles[0], "default");
}

#[test]
#[allow(non_snake_case)]
fn get_profiles__with_multiple_profiles__returns_all_profiles() {
    let cursor = io::Cursor::new(b"Path=one\nPath=two");

    let profiles = get_profiles(cursor);

    assert_eq!(profiles[0], "one");
    assert_eq!(profiles[1], "two");
}

#[test]
#[allow(non_snake_case)]
fn get_profile_selection__with_no_user_input__returns_first_profile() {
    let mut input = io::Cursor::new(b"");
    let profiles = vec![String::from("0"), String::from("1"), String::from("2")];

    let profile = get_profile_selection(&profiles, &mut input).unwrap();

    assert_eq!(profile, "0");
}

#[test]
#[allow(non_snake_case)]
fn get_profile_selection__with_user_input_one__returns_second_profile() {
    let mut input = io::Cursor::new(b"1");
    let profiles = vec![String::from("0"), String::from("1"), String::from("2")];

    let profile = get_profile_selection(&profiles, &mut input).unwrap();

    assert_eq!(profile, "1");
}

#[test]
#[allow(non_snake_case)]
fn get_profile_selection__with_user_input_letters__returns_error() {
    let mut input = io::Cursor::new(b"aoeu");
    let profiles = vec![String::from("0"), String::from("1"), String::from("2")];

    let result = get_profile_selection(&profiles, &mut input);

    assert!(result.is_err());
}

#[test]
#[allow(non_snake_case)]
fn get_profile_selection__with_user_input_out_of_range__returns_error() {
    let mut input = io::Cursor::new(b"99");
    let profiles = vec![String::from("0"), String::from("1"), String::from("2")];

    let result = get_profile_selection(&profiles, &mut input);

    assert!(result.is_err());
}

#[test]
#[allow(non_snake_case)]
fn get_profile_selection__with_empty_vec__returns_error() {
    let mut input = io::Cursor::new(b"");

    let result = get_profile_selection(&[], &mut input);

    assert!(result.is_err());
}

#[test]
#[allow(non_snake_case)]
fn next_style_id__with_two_styles__returns_minimal_id() {
    let style_one = Style {
        id: 0,
        domain: String::new(),
        style_type: StyleType::Local,
        settings: HashMap::new(),
    };
    let style_two = Style {
        id: 2,
        domain: String::new(),
        style_type: StyleType::Local,
        settings: HashMap::new(),
    };
    let config = Config {
        user_content: String::new(),
        styles: vec![style_one, style_two],
    };

    let id = config.next_style_id();

    assert_eq!(id, 1);
}
