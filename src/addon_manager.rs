use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;

const FILE_NAME: &str = ".addons.json";

#[derive(Serialize, Deserialize)]
pub struct Addons {
    pub addons: Vec<Addon>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Addon {
    pub addon_id: String,
    pub name: String,
    pub file_id: String,
    pub file_date: String,
    pub modules: Vec<String>,
    pub download_url: String,
    pub version: String,
    pub game_version: String,
    pub download_count: String,
}

pub struct AddonManager {}

impl AddonManager {
    pub fn init_addon_db(path: &str) -> Result<(), Box<dyn Error>> {
        let filepath = format!("{}/{}", path, FILE_NAME);
        if !Path::new(&filepath).exists() {
            let addons = Addons { addons: Vec::new() };
            AddonManager::save_addon_db(path, addons)?;
        }
        Ok(())
    }

    pub fn load_addon_db(path: &str) -> Result<Addons, Box<dyn Error>> {
        let filepath = format!("{}/{}", path, FILE_NAME);
        let content = fs::read_to_string(filepath)?;
        let addons: Addons = serde_json::from_str(&content)?;
        Ok(addons)
    }

    pub fn save_addon_db(
        path: &str,
        addons: Addons,
    ) -> Result<(), Box<dyn Error>> {
        let filepath = format!("{}/{}", path, FILE_NAME);
        let j = serde_json::to_string(&addons)?;
        fs::write(filepath, j)?;
        Ok(())
    }

    pub fn add_to_db(path: &str, addon: Addon) -> Result<(), Box<dyn Error>> {
        let mut addons = AddonManager::load_addon_db(path)?;
        addons.addons.push(addon);
        AddonManager::save_addon_db(path, addons)?;
        Ok(())
    }
    
    pub fn delete(path: &str, addon: &Addon) -> Result<(), Box<dyn Error>> {
        let mut addons = AddonManager::load_addon_db(path)?;
        match addons
            .addons
            .iter()
            .position(|a| a.addon_id == addon.addon_id)
        {
            Some(index) => {
                addons.addons.remove(index);
                for module in addon.modules.iter() {
                    let p = format!("{}/{}", path, module);
                    fs::remove_dir_all(p)?;
                }
            }
            None => {}
        }
        AddonManager::save_addon_db(path, addons)?;
        Ok(())
    }
}
