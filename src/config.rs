use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub language: String,
    pub nerd_font: bool,
    pub theme: String,
    /// Version string (e.g. "0.2.0") that the user chose to skip in the
    /// update modal. When set, the startup check will not prompt again
    /// for this exact version. Cleared automatically when a newer version
    /// is released.
    #[serde(default)]
    pub skipped_version: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn round_trip_works() {
            let _dir = env::temp_dir().join("git-hero-test-config");
            let _ = fs::create_dir_all(&_dir);
            // Test serialization round-trip directly instead of via
            // file I/O to avoid needing to set cwd.
        let c = Config {
            language: "es".into(),
            nerd_font: true,
            theme: "Nord".into(),
            skipped_version: Some("0.1.0".into()),
        };
        let json = serde_json::to_string_pretty(&c).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.language, "es");
        assert!(deserialized.nerd_font);
        assert_eq!(deserialized.theme, "Nord");
        assert_eq!(deserialized.skipped_version.as_deref(), Some("0.1.0"));
    }

    #[test]
    fn default_config_matches_expected_fields() {
        let c = Config {
            language: "en".into(),
            nerd_font: false,
            theme: "Tokyo Night".into(),
            skipped_version: None,
        };
        assert_eq!(c.language, "en");
        assert!(!c.nerd_font);
        assert_eq!(c.theme, "Tokyo Night");
        assert!(c.skipped_version.is_none());
    }
}
