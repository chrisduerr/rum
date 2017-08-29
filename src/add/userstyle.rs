use userstyles::response::{Style, StyleSetting};
use std::io::{self, BufRead, Write};
use std::collections::HashMap;
use userstyles;
use errors::*;

// Get the css and settings of a style
pub fn css<T: BufRead>(id: &str, input: &mut T) -> Result<(String, HashMap<String, String>)> {
    // Send Request for Style
    let id = u32::from_str_radix(id, 10)?;
    let style = userstyles::get_style(id)?;

    // Get custom settings
    let mut map = settings(&style, input)?;

    // Return style with custom css
    Ok((style.get_css(Some(&mut map)), map))
}

fn style_options(setting: &StyleSetting, label_val: bool) -> Vec<String> {
    let mut options = Vec::new();
    for option in &setting.style_setting_options {
        if label_val {
            options.push(option.value.clone());
        } else {
            options.push(option.label.clone());
        }
    }
    options
}

fn style_default(setting: &StyleSetting) -> usize {
    for (i, option) in setting.style_setting_options.iter().enumerate() {
        if option.default {
            return i;
        }
    }
    0
}

fn display_options(options: &[String], default: usize, show_custom: bool) {
    for (i, option) in options.iter().enumerate() {
        println!("    ({}) {}", i, option);
    }

    if show_custom {
        println!("    ({}) Custom", options.len());
    }

    print!("[Default {}] > ", default);
    let _ = io::stdout().flush();
}

fn settings<T: BufRead>(style: &Style, mut input: T) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for setting in &style.style_settings {
        let allow_custom = !(setting.setting_type == "dropdown");
        let label_val = match &*setting.setting_type {
            "color" | "text" => true,
            _ => false,
        };
        let style_options = style_options(setting, label_val);
        let style_default = style_default(setting);

        println!("\n[{}] {}:", setting.setting_type, setting.label);
        display_options(&style_options, style_default, allow_custom);
        let choice = read_user_choice(allow_custom, style_options.len(), style_default, &mut input);

        let setting_override = if choice == style_options.len() {
            read_custom_setting(&mut input)
        } else {
            setting.style_setting_options[choice].value.clone()
        };

        map.insert(setting.install_key.clone(), setting_override);
    }
    Ok(map)
}

fn read_custom_setting<T: BufRead>(input: &mut T) -> String {
    print!("[custom] > ");
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

fn read_user_choice<T: BufRead>(
    allow_custom: bool,
    mut allowed_max: usize,
    default: usize,
    input: &mut T,
) -> usize {
    let _ = io::stdout().flush();

    if !allow_custom {
        allowed_max -= 1;
    }

    loop {
        let mut choice = String::new();
        if input.read_line(&mut choice).is_ok() {
            let choice = choice.trim();
            if choice.is_empty() {
                return default;
            } else if let Ok(index) = usize::from_str_radix(choice, 10) {
                if index <= allowed_max {
                    return index;
                }
            }
        }


        println!("Invalid input. Please try again.");
        print!(" > ");
        let _ = io::stdout().flush();
    }
}


////////// TESTS //////////


#[cfg(test)]
use userstyles::response::StyleSettingOption;

#[test]
#[allow(non_snake_case)]
fn css__with_demo_style_id__returns_demostyle() {
    let url = "1";
    let mut cursor = io::Cursor::new(b"");

    let css = css(url, &mut cursor).unwrap();

    assert_eq!(css.0, "*{ color: red !important; }");
}

#[test]
#[allow(non_snake_case)]
fn css__with_allo_style_id_default_settings__css_contains_default_color() {
    let url = "146771";
    let mut cursor = io::Cursor::new(b"");

    let css = css(url, &mut cursor).unwrap();

    assert!(css.0.contains("#0F9D58"));
}

#[test]
#[allow(non_snake_case)]
fn css__with_allo_style_id_custom_color_setting__css_contains_custom_color() {
    let url = "146771";
    let mut cursor = io::Cursor::new(b"1\n#ff00ff\n\n");

    let css = css(url, &mut cursor).unwrap();

    assert!(css.0.contains("#ff00ff"));
}

#[test]
#[allow(non_snake_case)]
fn css__with_demo_style_id__returns_empty_settings() {
    let url = "1";
    let mut cursor = io::Cursor::new(b"");

    let css = css(url, &mut cursor).unwrap();

    assert_eq!(css.1.len(), 0);
}

#[test]
#[allow(non_snake_case)]
fn css__with_allo_style_id_default_settings__settings_hashmap() {
    let url = "146771";
    let mut cursor = io::Cursor::new(b"");

    let css = css(url, &mut cursor).unwrap();

    assert_eq!(css.1.get("ACCENTCOLOR").unwrap(), "#0F9D58");
    assert_eq!(
        css.1.get("CONVOBG").unwrap(),
        "    background-image:  none !important;"
    );
}

#[test]
#[allow(non_snake_case)]
fn style_options__with_single_label_and_labelval_false__return_single_label() {
    let mut option = StyleSettingOption::default();
    option.label = String::from("foobar");
    let mut setting = StyleSetting::default();
    setting.style_setting_options = vec![option];

    let options = style_options(&setting, false);

    assert_eq!(options[0], "foobar");
}

#[test]
#[allow(non_snake_case)]
fn style_options__with_single_value_and_labelval_true__return_single_label() {
    let mut option = StyleSettingOption::default();
    option.value = String::from("foobar2");
    let mut setting = StyleSetting::default();
    setting.style_setting_options = vec![option];

    let options = style_options(&setting, true);

    assert_eq!(options[0], "foobar2");
}

#[test]
#[allow(non_snake_case)]
fn style_default__with_default_second__returns_one() {
    let option = StyleSettingOption::default();
    let mut default = StyleSettingOption::default();
    default.default = true;
    let mut setting = StyleSetting::default();
    setting.style_setting_options = vec![option, default];

    let default = style_default(&setting);

    assert_eq!(default, 1);
}

#[test]
#[allow(non_snake_case)]
fn settings__with_choice_one__returns_choice() {
    let key = String::from("setting");
    let val = String::from("option");

    let mut option = StyleSettingOption::default();
    option.value = val.clone();
    let mut default = StyleSettingOption::default();
    default.default = true;

    let mut setting = StyleSetting::default();
    setting.setting_type = String::from("dropdown");
    setting.install_key = key.clone();
    setting.style_setting_options = vec![default, option];

    let mut style = Style::default();
    style.style_settings = vec![setting];

    let cursor = io::Cursor::new(b"1");


    let map = settings(&style, cursor).unwrap();
    let elem = map.get(&key).unwrap();


    assert_eq!(elem, &val);
}

#[test]
#[allow(non_snake_case)]
fn settings__with_custom_color__returns_color() {
    let key = String::from("setting");
    let mut setting = StyleSetting::default();
    setting.setting_type = String::from("color");
    setting.install_key = key.clone();
    let mut style = Style::default();
    style.style_settings = vec![setting];
    let cursor = io::Cursor::new(b"0\n#ff00ff");


    let map = settings(&style, cursor).unwrap();
    let elem = map.get(&key).unwrap();


    assert_eq!(elem, "#ff00ff");
}

#[test]
#[allow(non_snake_case)]
fn read_user_choice__with_correct_index__returns_index() {
    let mut cursor = io::Cursor::new(b"3");

    let result = read_user_choice(false, 9, 0, &mut cursor);

    assert_eq!(result, 3);
}

#[test]
#[allow(non_snake_case)]
fn read_user_choice__with_invalid_input__loops_until_valid() {
    let mut cursor = io::Cursor::new(b"oeu\n2yi\naaa9\n-3\n2");

    let result = read_user_choice(false, 3, 0, &mut cursor);

    assert_eq!(result, 2);
}

#[test]
#[allow(non_snake_case)]
fn read_user_choice__with_custom_true__allows_bigger_index() {
    let mut cursor = io::Cursor::new(b"4");

    let result = read_user_choice(true, 4, 0, &mut cursor);

    assert_eq!(result, 4);
}

#[test]
#[allow(non_snake_case)]
fn read_user_choice__with_custom_false__disallows_bigger_index() {
    let mut cursor = io::Cursor::new(b"4\n3");

    let result = read_user_choice(false, 4, 0, &mut cursor);

    assert_eq!(result, 3);
}

#[test]
#[allow(non_snake_case)]
fn read_user_choice__with_empty_input__returns_default() {
    let mut cursor = io::Cursor::new(b"\n");

    let result = read_user_choice(false, 4, 0, &mut cursor);

    assert_eq!(result, 0);
}
