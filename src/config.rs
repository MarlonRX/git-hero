use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub language: String,
    pub nerd_font: bool,
    pub theme: String,
}

pub fn get_config_path() -> PathBuf {
    if let Some(mut path) = dirs::config_dir() {
        path.push("git-hero");
        path.push("config.json");
        path
    } else {
        PathBuf::from(".config/git-hero/config.json")
    }
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = get_config_path();
    let data = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&data)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(config)?;
    fs::write(path, data)?;
    Ok(())
}
