extern crate config;
extern crate dirs;

use std::{error::Error, fs, path::Path};
use termion::event::Key;

pub struct Paths {
    pub classic: String,
    pub retail: String,
}

pub struct KeyBindings {
    pub update_addon: Key,
    pub update_all_addons: Key,
    pub remove_addon: Key,
    pub download_addon: Key,
    pub install_addon: Key,
    pub select_retail_version: Key,
    pub select_classic_version: Key,
    pub search_addon: Key,
    pub next_tab: Key,
    pub prev_tab: Key,
    pub next_table_item: Key,
    pub prev_table_item: Key,
    pub quit: Key,
    pub scroll_down_log: Key,
    pub scroll_up_log: Key,
}

const FILENAME: &str = "Config.toml";
const APP_DIR: &str = "wowAddonManager";

pub struct Settings {
    pub paths: Paths,
    pub key_bindings: KeyBindings,
}

impl Settings {
    pub fn new() -> Settings {
        let mut s = config::Config::default();
        match Settings::init_config_file() {
            Ok(path) => {
                s.merge(config::File::with_name(&path)).unwrap();
            }
            Err(_) => {
                s.merge(config::File::with_name("Config")).unwrap();
            }
        };

        let paths = Paths {
            classic: s.get::<String>("paths.classic").unwrap_or("".to_string()),
            retail: s.get::<String>("paths.retail").unwrap_or("".to_string()),
        };
        let update_addon = Settings::parse_key(
            s.get::<String>("keybindings.update_addon")
                .unwrap_or("".to_string()),
        );
        let update_all_addons = Settings::parse_key(
            s.get::<String>("keybindings.update_all_addons")
                .unwrap_or("".to_string()),
        );
        let remove_addon = Settings::parse_key(
            s.get::<String>("keybindings.remove_addon")
                .unwrap_or("".to_string()),
        );
        let download_addon = Settings::parse_key(
            s.get::<String>("keybindings.download_addon")
                .unwrap_or("".to_string()),
        );
        let install_addon = Settings::parse_key(
            s.get::<String>("keybindings.install_addon")
                .unwrap_or("".to_string()),
        );
        let select_retail_version = Settings::parse_key(
            s.get::<String>("keybindings.select_retail_version")
                .unwrap_or("".to_string()),
        );
        let select_classic_version = Settings::parse_key(
            s.get::<String>("keybindings.select_classic_version")
                .unwrap_or("".to_string()),
        );
        let search_addon = Settings::parse_key(
            s.get::<String>("keybindings.search_addon")
                .unwrap_or("".to_string()),
        );
        let next_tab = Settings::parse_key(
            s.get::<String>("keybindings.next_tab")
                .unwrap_or("".to_string()),
        );
        let prev_tab = Settings::parse_key(
            s.get::<String>("keybindings.prev_tab")
                .unwrap_or("".to_string()),
        );
        let next_table_item = Settings::parse_key(
            s.get::<String>("keybindings.next_table_item")
                .unwrap_or("".to_string()),
        );
        let prev_table_item = Settings::parse_key(
            s.get::<String>("keybindings.prev_table_item")
                .unwrap_or("".to_string()),
        );
        let quit = Settings::parse_key(
            s.get::<String>("keybindings.quit")
                .unwrap_or("".to_string()),
        );
        let scroll_down_log = Settings::parse_key(
            s.get::<String>("keybindings.scroll_down_log")
                .unwrap_or("".to_string()),
        );
        let scroll_up_log = Settings::parse_key(
            s.get::<String>("keybindings.scroll_up_log")
                .unwrap_or("".to_string()),
        );
        let key_bindings = KeyBindings {
            update_addon: update_addon,
            update_all_addons: update_all_addons,
            remove_addon: remove_addon,
            download_addon: download_addon,
            install_addon: install_addon,
            select_retail_version: select_retail_version,
            select_classic_version: select_classic_version,
            search_addon: search_addon,
            next_tab: next_tab,
            prev_tab: prev_tab,
            next_table_item: next_table_item,
            prev_table_item: prev_table_item,
            quit: quit,
            scroll_down_log: scroll_down_log,
            scroll_up_log: scroll_up_log,
        };
        Settings {
            paths: paths,
            key_bindings: key_bindings,
        }
    }

    pub fn init_config_file() -> Result<String, Box<dyn Error>> {
        match dirs::config_dir() {
            Some(config) => {
                let path = Path::new(&config);
                let app_config_dir = path.join(APP_DIR);
                if !app_config_dir.exists() {
                    fs::create_dir(&app_config_dir)?;
                }
                let config_file_path = app_config_dir.join(FILENAME);
                let path_string =
                    config_file_path.to_str().unwrap().to_string();
                if !config_file_path.exists() {
                    fs::copy("Config.toml", config_file_path)?;
                }
                Ok(path_string)
            }
            None => {
                panic!("Config directory not found!");
            }
        }
    }

    pub fn parse_key(key: String) -> Key {
        fn get_single_char(string: &str) -> char {
            match string.chars().next() {
                Some(c) => c,
                None => '\0',
            }
        }

        match key.len() {
            1 => Key::Char(get_single_char(key.as_str())),
            _ => {
                let tokens: Vec<&str> = key.split('-').collect();

                match tokens[0].to_lowercase().as_str() {
                    "ctrl" => Key::Ctrl(get_single_char(tokens[1])),
                    "alt" => Key::Alt(get_single_char(tokens[1])),
                    "left" => Key::Left,
                    "right" => Key::Right,
                    "up" => Key::Up,
                    "down" => Key::Down,
                    "backspace" => Key::Backspace,
                    "del" => Key::Delete,
                    "esc" => Key::Esc,
                    "pageup" => Key::PageUp,
                    "pagedown" => Key::PageDown,
                    "space" => Key::Char(' '),
                    _ => Key::Null,
                }
            }
        }
    }
}
