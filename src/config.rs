use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub accent_color: String,
    pub nav_mode: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            accent_color: "orange".to_string(),
            nav_mode: "arrow".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub favorites: HashMap<String, Vec<String>>,
    pub recents: HashMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        let mut favorites = HashMap::new();
        favorites.insert("dirs".to_string(), Vec::new());
        favorites.insert("tools".to_string(), Vec::new());
        favorites.insert("providers".to_string(), Vec::new());
        favorites.insert("models".to_string(), Vec::new());

        let mut recents = HashMap::new();
        recents.insert("dirs".to_string(), Vec::new());
        recents.insert("tools".to_string(), Vec::new());
        recents.insert("providers".to_string(), Vec::new());
        recents.insert("models".to_string(), Vec::new());

        Self {
            settings: Settings::default(),
            favorites,
            recents,
        }
    }
}

impl Config {
    /// Load configuration from ~/.config/clumsycat/config.json
    /// Returns default config if file doesn't exist or is corrupted
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if let Some(path) = config_path {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str(&content) {
                        Ok(config) => return config,
                        Err(_) => {
                            eprintln!("Warning: Failed to parse config file, using defaults");
                            return Self::default();
                        }
                    },
                    Err(_) => {
                        eprintln!("Warning: Failed to read config file, using defaults");
                        return Self::default();
                    }
                }
            }
        }

        Self::default()
    }

    /// Save configuration to ~/.config/clumsycat/config.json
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_path) = Self::config_path() {
            // Create parent directories if they don't exist
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let json = serde_json::to_string_pretty(self)?;
            fs::write(&config_path, json)?;
        }

        Ok(())
    }

    /// Get the path to the config file
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("clumsycat").join("config.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.settings.accent_color, "orange");
        assert_eq!(config.settings.nav_mode, "arrow");
        assert!(config.favorites.contains_key("dirs"));
        assert!(config.favorites.contains_key("tools"));
        assert!(config.recents.contains_key("dirs"));
        assert!(config.recents.contains_key("providers"));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: Config = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.settings.accent_color, config.settings.accent_color);
    }
}
