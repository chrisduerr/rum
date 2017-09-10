use std::io::{self, BufRead, BufReader, Read, Write};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use errors::*;
use std::env;
use READER;
use toml;

const CONFIG_PATH: &str = ".config/rum.toml";
pub const RUM_START: &str = "\n/* RUM START {} */\n";
pub const RUM_END: &str = "\n/* RUM END {} */\n";

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub chrome_path: String,
    pub styles: Vec<Style>,
}

impl Config {
    // Get the next free style id
    pub fn next_style_id(&self) -> i32 {
        // Get all currently used ids
        let mut ids: Vec<i32> = self.styles.iter().map(|s| s.id).collect();
        ids.sort_by(|a, b| a.cmp(b));

        // Get the next free id and return it
        let mut id = 0;
        for i in ids {
            if i != id {
                return id;
            }
            id += 1;
        }
        id
    }

    // Load the config file
    pub fn load() -> Result<Config> {
        // Create config if it does not exist already
        if !config_exists() {
            error!("No config file found");
            create_config()?;
        }

        // Load the content of the file
        let path = config_path()?;
        let mut content = String::new();
        File::open(path)?.read_to_string(&mut content)?;

        // Parse the file content
        Ok(toml::from_str::<Config>(&content)?)
    }

    // Write the current config to the file
    pub fn write(&self) -> Result<()> {
        // Convert struct to toml string
        let output = toml::to_string(self)?;

        // Write the string to the file
        let config_path = config_path()?;
        File::create(config_path)?.write_all(output.as_bytes())?;

        Ok(())
    }

    // Remove a style and return the removed style
    pub fn remove_style(&mut self, id: i32) -> Option<Style> {
        // Get the index of the style
        let mut index = None;
        for (i, style) in self.styles.iter().enumerate() {
            if style.id == id {
                index = Some(i);
                break;
            }
        }

        // Remove and return the style
        if let Some(index) = index {
            Some(self.styles.swap_remove(index))
        } else {
            None
        }
    }

    // Get the id from a string that's either the id or the name
    pub fn style_id_from_str(&self, name_or_id: &str) -> Option<i32> {
        if let Ok(id) = i32::from_str_radix(name_or_id, 10) {
            // If the str is a valid int, return the style with that id
            if self.contains_style(id) {
                Some(id)
            } else {
                None
            }
        } else {
            // If it's not an int, return the style with that name
            for style in &self.styles {
                if style.name == name_or_id {
                    return Some(style.id);
                }
            }
            None
        }
    }

    // Change the status of a style
    // ENABLED  -> DISABLED
    // DISABLED -> ENABLED
    pub fn toggle_style(&mut self, id: i32) -> Result<()> {
        for style in &mut self.styles {
            if style.id == id {
                style.enabled = !style.enabled;
                return Ok(());
            }
        }

        Err("Style with this id does not exist")?
    }

    // Create a new style
    fn new(chrome_path: String) -> Config {
        Config {
            chrome_path,
            styles: Vec::new(),
        }
    }

    // Check if a style with the specified id exists
    fn contains_style(&self, id: i32) -> bool {
        for style in &self.styles {
            if style.id == id {
                return true;
            }
        }
        false
    }
}

// Every information that is required for a Style
#[derive(Serialize, Deserialize, Clone)]
pub struct Style {
    #[serde(skip_serializing, skip_deserializing)] pub css: String,
    pub id: i32,
    pub uri: String,
    pub name: String,
    pub path: PathBuf,
    #[serde(default = "default_true")] pub enabled: bool,
    pub style_type: StyleType,
    pub domain: Option<String>,
    pub settings: HashMap<String, String>,
}

// Used for serde to set the default of `enabled`
// Required for backwards compatibility
fn default_true() -> bool {
    true
}

// The type of a style
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StyleType {
    Userstyle,
    Local,
    Remote,
}

// Check if the config file exists
pub fn config_exists() -> bool {
    if let Ok(path) = config_path() {
        path.as_path().exists()
    } else {
        false
    }
}

// Create a new config file
pub fn create_config() -> Result<()> {
    // Get the profile.ini file
    let profiles_ini = profiles_ini_path()?;
    let ini_buf = BufReader::new(File::open(&profiles_ini)?);
    let profiles = get_profiles(ini_buf);

    // Read the chosen profile
    let profile = get_profile_selection(&profiles)?;

    // Create the path from the user's choice
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
fn profiles_ini_path() -> Result<PathBuf> {
    let mut path = env::home_dir().ok_or("Unable to locate home directory")?;
    path.push(".mozilla/firefox/profiles.ini");
    Ok(path)
}

// Get list with all profile names, first is the default
fn get_profiles<T: BufRead>(profiles_buf: T) -> Vec<String> {
    let mut profiles = Vec::new();
    let mut default = false;

    // Iterate over all lines in the profiles.ini
    for line in profiles_buf.lines() {
        let line = line.unwrap_or_else(|_| String::new());

        if line == "Name=default" {
            // If the line contains Name=default, the next entry is the default
            default = true;
        } else if line.starts_with("Path=") {
            // If it starts with Path=, it's one possible profile option
            let profile = (&line[5..]).to_owned();
            if default {
                // Insert the default as the first element
                default = false;
                profiles.insert(0, profile);
            } else {
                profiles.push(profile);
            }
        }
    }

    // Return all profiles
    profiles
}

// Interact with the user to check which profile he wants
#[allow(unused_mut)]
fn get_profile_selection(profiles: &[String]) -> Result<String> {
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
    let mut input = READER.lock().map_err(|_| "Unable to lift reader lock")?;
    input.read_line(&mut user_input)?;
    let user_input = user_input.trim();

    // Convert choice to integer
    let index = if !user_input.is_empty() {
        usize::from_str_radix(user_input, 10)?
    } else {
        0
    };

    // Return the name of the profile
    Ok(
        profiles
            .get(index)
            .ok_or("Profile number out of range.")?
            .to_owned(),
    )
}

// Get pat of config file
fn config_path() -> Result<PathBuf> {
    let mut path = env::home_dir().ok_or("Unable to find home directory.")?;
    path.push(CONFIG_PATH);
    Ok(path)
}

// Attempts to recover an old config
pub fn restore_config(backup: &Config, error: &Error) -> Result<()> {
    error!("Error: {}", error);
    println!("Attempting to recover config");
    match backup.write() {
        Ok(_) => {
            println!("Successfully recovered config");
            println!("Style has not been added");
            Ok(())
        }
        error => {
            error!("Unable to recover config");
            error!("Please ensure the config is not corrupted");
            error
        }
    }
}


////////// TESTS //////////


#[cfg(test)]
fn dummy_style() -> Style {
    Style {
        id: 0,
        domain: None,
        enabled: true,
        uri: String::new(),
        name: String::new(),
        path: PathBuf::new(),
        style_type: StyleType::Local,
        settings: HashMap::new(),
        css: String::new(),
    }
}

#[cfg(test)]
fn dummy_config(styles: Vec<Style>) -> Config {
    Config {
        chrome_path: String::new(),
        styles: styles,
    }
}

#[cfg(test)]
fn write_reader(text: &str) {
    let mut input = READER.lock().unwrap();
    (*input) = io::Cursor::new(text.as_bytes().to_vec());
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
    write_reader("");
    let profiles = vec![String::from("0"), String::from("1"), String::from("2")];

    let profile = get_profile_selection(&profiles).unwrap();

    assert_eq!(profile, "0");
}

#[test]
#[allow(non_snake_case)]
fn get_profile_selection__with_user_input_one__returns_second_profile() {
    write_reader("1");
    let profiles = vec![String::from("0"), String::from("1"), String::from("2")];

    let profile = get_profile_selection(&profiles).unwrap();

    assert_eq!(profile, "1");
}

#[test]
#[allow(non_snake_case)]
fn get_profile_selection__with_user_input_letters__returns_error() {
    write_reader("aoeu");
    let profiles = vec![String::from("0"), String::from("1"), String::from("2")];

    let result = get_profile_selection(&profiles);

    assert!(result.is_err());
}

#[test]
#[allow(non_snake_case)]
fn get_profile_selection__with_user_input_out_of_range__returns_error() {
    write_reader("99");
    let profiles = vec![String::from("0"), String::from("1"), String::from("2")];

    let result = get_profile_selection(&profiles);

    assert!(result.is_err());
}

#[test]
#[allow(non_snake_case)]
fn get_profile_selection__with_empty_vec__returns_error() {
    write_reader("");

    let result = get_profile_selection(&[]);

    assert!(result.is_err());
}

#[test]
#[allow(non_snake_case)]
fn next_style_id__with_two_styles__returns_minimal_id() {
    let mut style_zero = dummy_style();
    let mut style_two = dummy_style();
    style_zero.id = 0;
    style_two.id = 2;
    let config = dummy_config(vec![style_zero, style_two]);

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
    let mut config = dummy_config(vec![style_zero, style_one, style_two]);

    config.remove_style(1);

    assert_eq!(config.styles.len(), 2);
    assert_eq!(config.styles[0].id, 0);
    assert_eq!(config.styles[1].id, 2);
}

#[test]
#[allow(non_snake_case)]
fn remove_style__with_id_one__returns_style_one() {
    let mut style_zero = dummy_style();
    let mut style_one = dummy_style();
    let mut style_two = dummy_style();
    style_zero.id = 0;
    style_one.id = 1;
    style_two.id = 2;
    let mut config = dummy_config(vec![style_zero, style_one, style_two]);

    let result = config.remove_style(1).unwrap();

    assert_eq!(result.id, 1);
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
    let config = dummy_config(vec![style_zero, style_one, style_two]);

    let id = config.style_id_from_str("one").unwrap();

    assert_eq!(id, 1);
}

#[test]
#[allow(non_snake_case)]
fn contains_style__with_style__returns_true() {
    let mut style_zero = dummy_style();
    style_zero.id = 0;
    let config = dummy_config(vec![style_zero]);

    let contains_style = config.contains_style(0);

    assert!(contains_style);
}

#[test]
#[allow(non_snake_case)]
fn contains_style__without_style__returns_false() {
    let config = dummy_config(Vec::new());

    let contains_style = config.contains_style(0);

    assert!(!contains_style);
}

#[test]
#[allow(non_snake_case)]
fn toggle_style__with_style_enabled__disables_style() {
    let mut style = dummy_style();
    style.enabled = true;
    style.id = 0;
    let mut config = dummy_config(vec![style]);

    config.toggle_style(0).unwrap();

    assert!(!config.styles[0].enabled);
}

#[test]
#[allow(non_snake_case)]
fn toggle_style__with_style_disabled__enables_style() {
    let mut style = dummy_style();
    style.enabled = false;
    style.id = 3;
    let mut config = dummy_config(vec![style]);

    config.toggle_style(3).unwrap();

    assert!(config.styles[0].enabled);
}

#[test]
#[should_panic]
#[allow(non_snake_case)]
fn toggle_style__with_invalid_id__returns_error() {
    let mut config = dummy_config(Vec::new());

    config.toggle_style(15).unwrap();
}

#[test]
#[allow(non_snake_case)]
fn profiles_ini_path__returns_pathbuf_ending_profileini() {
    let pathbuf = profiles_ini_path().unwrap();

    let expected = "/.mozilla/firefox/profiles.ini";
    assert!(pathbuf.to_str().unwrap().ends_with(expected));
}

#[test]
#[allow(non_snake_case)]
fn config_path__returns_pathbuf_ending_rumconfig() {
    let pathbuf = config_path().unwrap();

    assert!(pathbuf.to_str().unwrap().ends_with("/.config/rum.toml"));
}

#[test]
#[allow(non_snake_case)]
fn new__with_path__returns_config_with_path() {
    let path = String::from("aoeuaoeu");

    let config = Config::new(path.clone());

    assert_eq!(config.chrome_path, path);
}
