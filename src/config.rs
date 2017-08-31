use std::io::{self, BufRead, BufReader, Read, Write};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use errors::*;
use std::env;
use toml;

const CONFIG_PATH: &str = ".config/rum.toml";
pub const RUM_START: &str = "\n/* RUM START {} */\n";
pub const RUM_END: &str = "\n/* RUM END {} */\n";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub chrome_path: String,
    pub styles: Vec<Style>,
}

impl Config {
    fn new(chrome_path: String) -> Config {
        Config {
            chrome_path,
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

    pub fn file_path_by_id(&self, id: i32) -> Option<PathBuf> {
        for style in &self.styles {
            if style.id == id {
                return Some(style.path.clone());
            }
        }
        None
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

    pub fn remove_style(&mut self, id: i32) {
        self.styles.retain(|s| s.id != id);
    }

    pub fn pop_style(&mut self, id: i32) -> Option<Style> {
        let mut index = None;
        for (i, style) in self.styles.iter().enumerate() {
            if style.id == id {
                index = Some(i);
            }
        }

        if let Some(index) = index {
            Some(self.styles.swap_remove(index))
        } else {
            None
        }
    }

    pub fn style_id_from_str(&self, name_or_id: &str) -> Option<i32> {
        if let Ok(id) = i32::from_str_radix(name_or_id, 10) {
            if !self.contains_style(id) {
                None
            } else {
                Some(id)
            }
        } else {
            for style in &self.styles {
                if style.name == name_or_id {
                    return Some(style.id);
                }
            }
            None
        }
    }

    pub fn contains_style(&self, id: i32) -> bool {
        for style in &self.styles {
            if style.id == id {
                return true;
            }
        }
        false
    }
}

#[derive(Serialize, Deserialize)]
pub struct Style {
    #[serde(skip_serializing, skip_deserializing)] pub css: String,
    pub id: i32,
    pub uri: String,
    pub name: String,
    pub path: PathBuf,
    pub style_type: StyleType,
    pub domain: Option<String>,
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

    let mut chrome_path = profiles_ini;
    chrome_path.pop();
    chrome_path.push(profile);
    chrome_path.push("chrome");
    let chrome_path = chrome_path.to_str().ok_or("Profile chrome path invalid.")?;

    // Create new config
    let config = Config::new(chrome_path.to_owned());
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


#[cfg(test)]
fn dummy_style() -> Style {
    Style {
        id: 0,
        domain: None,
        uri: String::new(),
        name: String::new(),
        path: PathBuf::new(),
        style_type: StyleType::Local,
        settings: HashMap::new(),
        css: String::new(),
    }
}

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
    let mut style_zero = dummy_style();
    let mut style_two = dummy_style();
    style_zero.id = 0;
    style_two.id = 2;
    let config = Config {
        chrome_path: String::new(),
        styles: vec![style_zero, style_two],
    };

    let id = config.next_style_id();

    assert_eq!(id, 1);
}

#[test]
#[allow(non_snake_case)]
fn remove_style__with_id_one__removes_style_one() {
    let mut style_zero = dummy_style();
    let mut style_one = dummy_style();
    let mut style_two = dummy_style();
    style_zero.id = 0;
    style_one.id = 1;
    style_two.id = 2;
    let mut config = Config {
        chrome_path: String::new(),
        styles: vec![style_zero, style_one, style_two],
    };

    config.remove_style(1);

    assert_eq!(config.styles.len(), 2);
    assert_eq!(config.styles[0].id, 0);
    assert_eq!(config.styles[1].id, 2);
}

#[test]
#[allow(non_snake_case)]
fn style_id_by_name__with_name_one__returns_one() {
    let mut style_zero = dummy_style();
    let mut style_one = dummy_style();
    let mut style_two = dummy_style();
    style_zero.name = String::from("zero");
    style_one.name = String::from("one");
    style_two.name = String::from("two");
    style_zero.id = 0;
    style_one.id = 1;
    style_two.id = 2;
    let config = Config {
        chrome_path: String::new(),
        styles: vec![style_zero, style_one, style_two],
    };

    let id = config.style_id_from_str("one").unwrap();

    assert_eq!(id, 1);
}

#[test]
#[allow(non_snake_case)]
fn contains_style__with_style__returns_true() {
    let mut style_zero = dummy_style();
    style_zero.id = 0;
    let config = Config {
        chrome_path: String::new(),
        styles: vec![style_zero],
    };

    let contains_style = config.contains_style(0);

    assert!(contains_style);
}

#[test]
#[allow(non_snake_case)]
fn contains_style__without_style__returns_false() {
    let config = Config {
        chrome_path: String::new(),
        styles: vec![],
    };

    let contains_style = config.contains_style(0);

    assert!(!contains_style);
}
