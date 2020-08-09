extern crate tempfile;
use crate::addon_manager::{Addon};
use crate::app::TableItem;
use crate::app::Version;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use tempfile::Builder;

const BASE_URL: &str = "https://addons-ecs.forgesvc.net/api/v2";
const GAMEID: &str = "1";
const STABLE_RELEASE: usize = 1;
const BETA_RELEASE: usize = 2;
const ALPHA_RELEASE: usize = 3;

pub struct CurseForgeAPI {}

impl CurseForgeAPI {
    #[tokio::main]
    pub async fn search(
        addon: &str,
        game_version: Version,
    ) -> Result<Vec<TableItem>, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/addon/search?gameId={}&searchFilter={}",
            BASE_URL, GAMEID, addon
        );
        let resp = reqwest::get(&url).await?.text().await?;
        let data: serde_json::Value = serde_json::from_str(&resp)?;
        let mut items = Vec::new();
        for addon in data.as_array().unwrap().iter() {
            match CurseForgeAPI::parse_json(addon, game_version) {
                Some(a) => {
                    items.push(TableItem {
                        cells: vec![
                            a.name.clone(),
                            a.game_version.clone(),
                            a.file_date.clone(),
                            a.download_count.clone(),
                        ],
                        download_url: a.download_url.clone(),
                        addon: a.clone(),
                    });
                }
                None => {}
            }
        }
        Ok(items)
    }

    fn parse_json(
        json: &serde_json::Value,
        game_version: Version,
    ) -> Option<Addon> {
        let latest_file = CurseForgeAPI::latest_file(json, game_version);
        match latest_file {
            Some(file) => {
                let filedate = CurseForgeAPI::parse_date(&file["fileDate"]);
                let download_count =
                    CurseForgeAPI::parse_download_count(&json["downloadCount"]);
                let modules: Vec<String> = file["modules"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|m| {
                        m["foldername"].as_str().unwrap_or_default().to_string()
                    })
                    .collect();
                let addon = Addon {
                    addon_id: json["id"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    file_id: file["id"].as_str().unwrap_or_default().to_string(),
                    name: json["name"].as_str().unwrap_or_default().to_string(),
                    file_date: filedate.to_string(),
                    modules: modules,
                    version: file["displayName"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    game_version: file["gameVersion"][0]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    download_url: file["downloadUrl"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    download_count: download_count,
                };
                return Some(addon);
            }
            None => {
                return None;
            }
        };
    }

    pub fn parse_date(filedate: &serde_json::Value) -> String {
        let filedate = filedate.as_str().unwrap_or_default();
        let ymd = filedate.chars().take(10).collect();
        ymd
    }

    pub fn parse_download_count(download_count: &serde_json::Value) -> String {
        let mut s = String::new();
        let i_str = download_count
            .as_f64()
            .unwrap_or_default()
            .to_string()
            .replace(".0", "");
        let a = i_str.chars().rev().enumerate();
        for (idx, val) in a {
            if idx != 0 && idx % 3 == 0 {
                s.insert(0, ',');
            }
            s.insert(0, val);
        }
        s
    }

    pub fn latest_file(
        json: &serde_json::Value,
        game_version: Version,
    ) -> Option<&serde_json::Value> {
        let game_version_flavor = match game_version {
            Version::Classic => "wow_classic",
            Version::Retail => "wow_retail",
        };
        let files = json["latestFiles"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|&file| {
                file["releaseType"] == STABLE_RELEASE
                    && file["gameVersionFlavor"] == game_version_flavor
            })
            .max_by(|a, b| b["id"].to_string().cmp(&a["id"].to_string()));
        files
    }

    #[tokio::main]
    pub async fn download(
        url: &str,
        save_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp_dir = Builder::new().tempdir()?;
        let response = reqwest::get(url).await?;
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");
        let fname = tmp_dir.path().join(fname);

        let mut dest = File::create(fname.clone())?;
        let content = response.bytes().await?;
        dest.write_all(&content)?;
        let file = fs::File::open(fname).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut outpath = PathBuf::from(save_path.clone());
            let mut file = archive.by_index(i).unwrap();
            outpath.push(file.sanitized_name());

            if (&*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath).unwrap();
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }
        }

        Ok(())
    }

    #[tokio::main]
    pub async fn check_for_updates(
        addons: Vec<i32>,
        game_version: Version,
    ) -> Result<HashMap<String, Addon>, Box<dyn std::error::Error>> {
        let url = format!("{}/addon", BASE_URL);
        let client = reqwest::Client::new();
        let resp = client.post(&url).json(&addons).send().await?.text().await?;
        let data: serde_json::Value = serde_json::from_str(&resp)?;
        let mut items = HashMap::new();
        for addon in data.as_array().unwrap().iter() {
            match CurseForgeAPI::parse_json(addon, game_version) {
                Some(a) => {
                    items.insert(a.addon_id.clone(), a.clone());
                }
                None => {}
            }
        }

        Ok(items)
    }
}
