use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelProfile {
    ClaudeMax,
    ClaudePro,
    ClaudeFree,
}

impl ModelProfile {
    pub fn env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        match self {
            ModelProfile::ClaudeMax => {
                vars.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(), "claude-opus-4.5".to_string());
                vars.insert("ANTHROPIC_MODEL".to_string(), "claude-sonnet-4.5".to_string());
                vars.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(), "claude-haiku-4.5".to_string());
            }
            ModelProfile::ClaudePro => {
                vars.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(), "claude-opus-4.5".to_string());
                vars.insert("ANTHROPIC_MODEL".to_string(), "claude-sonnet-4.5".to_string());
                vars.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(), "gpt-5-mini".to_string());
            }
            ModelProfile::ClaudeFree => {
                vars.insert("ANTHROPIC_MODEL".to_string(), "gpt-5-mini".to_string());
            }
        }
        vars
    }
}

#[derive(Debug)]
pub struct ClaudeSettings {
    raw: Value,
    path: PathBuf,
}

impl ClaudeSettings {
    fn settings_path() -> PathBuf {
        #[cfg(windows)]
        {
            std::env::var("USERPROFILE")
                .map(|p| PathBuf::from(p).join(".claude").join("settings.json"))
                .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default\\.claude\\settings.json"))
        }
        #[cfg(not(windows))]
        {
            std::env::var("HOME")
                .map(|p| PathBuf::from(p).join(".claude").join("settings.json"))
                .unwrap_or_else(|_| PathBuf::from("/home/.claude/settings.json"))
        }
    }

    pub fn load() -> Result<Self, io::Error> {
        let path = Self::settings_path();
        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &PathBuf) -> Result<Self, io::Error> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let raw: Value = serde_json::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(Self { raw, path: path.clone() })
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            let default_env = serde_json::json!({
                "ANTHROPIC_BASE_URL": "http://localhost:4141",
                "ANTHROPIC_AUTH_TOKEN": "sk-dummy",
                "ANTHROPIC_DEFAULT_OPUS_MODEL": "claude-opus-4.5",
                "ANTHROPIC_MODEL": "claude-sonnet-4.5",
                "ANTHROPIC_DEFAULT_HAIKU_MODEL": "claude-haiku-4.5",
                "DISABLE_NON_ESSENTIAL_MODEL_CALLS": "1",
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
                "CLAUDE_CODE_ATTRIBUTION_HEADER": "0"
            });

            let raw = serde_json::json!({
                "env": default_env
            });

            let settings = Self { raw, path: path.clone() };
            settings.save()?;
            Ok(settings)
        }
    }

    pub fn save(&self) -> Result<(), io::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_path_returns_claude_dir() {
        let path = ClaudeSettings::settings_path();
        assert!(path.to_string_lossy().contains(".claude"));
        assert!(path.to_string_lossy().ends_with("settings.json"));
    }

    #[test]
    fn test_claude_max_env_vars() {
        let vars = ModelProfile::ClaudeMax.env_vars();
        assert_eq!(vars.get("ANTHROPIC_DEFAULT_OPUS_MODEL"), Some(&"claude-opus-4.5".to_string()));
        assert_eq!(vars.get("ANTHROPIC_MODEL"), Some(&"claude-sonnet-4.5".to_string()));
        assert_eq!(vars.get("ANTHROPIC_DEFAULT_HAIKU_MODEL"), Some(&"claude-haiku-4.5".to_string()));
    }

    #[test]
    fn test_claude_pro_env_vars() {
        let vars = ModelProfile::ClaudePro.env_vars();
        assert_eq!(vars.get("ANTHROPIC_DEFAULT_OPUS_MODEL"), Some(&"claude-opus-4.5".to_string()));
        assert_eq!(vars.get("ANTHROPIC_MODEL"), Some(&"claude-sonnet-4.5".to_string()));
        assert_eq!(vars.get("ANTHROPIC_DEFAULT_HAIKU_MODEL"), Some(&"gpt-5-mini".to_string()));
    }

    #[test]
    fn test_claude_free_env_vars() {
        let vars = ModelProfile::ClaudeFree.env_vars();
        assert_eq!(vars.get("ANTHROPIC_MODEL"), Some(&"gpt-5-mini".to_string()));
        assert_eq!(vars.get("ANTHROPIC_DEFAULT_OPUS_MODEL"), None);
        assert_eq!(vars.get("ANTHROPIC_DEFAULT_HAIKU_MODEL"), None);
    }

    #[test]
    fn test_load_existing_settings() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_settings_load.json");

        let content = r#"{"env":{"ANTHROPIC_MODEL":"test-model"},"other":"data"}"#;
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let settings = ClaudeSettings::load_from_path(&test_file).unwrap();
        assert!(settings.raw.is_object());
        assert_eq!(settings.raw["env"]["ANTHROPIC_MODEL"], "test-model");

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_missing_file_creates_default() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_settings_missing.json");

        fs::remove_file(&test_file).ok();

        let settings = ClaudeSettings::load_from_path(&test_file).unwrap();
        assert!(settings.raw.is_object());
        assert!(settings.raw.get("env").is_some());

        fs::remove_file(&test_file).ok();
    }
}
