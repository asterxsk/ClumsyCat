use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
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
                vars.insert(
                    "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                    "claude-opus-4.5".to_string(),
                );
                vars.insert(
                    "ANTHROPIC_MODEL".to_string(),
                    "claude-sonnet-4.5".to_string(),
                );
                vars.insert(
                    "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                    "claude-haiku-4.5".to_string(),
                );
            }
            ModelProfile::ClaudePro => {
                vars.insert(
                    "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                    "claude-opus-4.5".to_string(),
                );
                vars.insert(
                    "ANTHROPIC_MODEL".to_string(),
                    "claude-sonnet-4.5".to_string(),
                );
                vars.insert(
                    "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                    "gpt-5-mini".to_string(),
                );
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
            Ok(Self {
                raw,
                path: path.clone(),
            })
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

            let settings = Self {
                raw,
                path: path.clone(),
            };
            settings.save()?;
            Ok(settings)
        }
    }

    pub fn set_model_profile(&mut self, profile: ModelProfile) {
        if !self.raw.is_object() {
            self.raw = serde_json::json!({});
        }

        let env_vars = profile.env_vars();

        if self.raw.get("env").is_none() {
            let default_env = serde_json::json!({
                "ANTHROPIC_BASE_URL": "http://localhost:4141",
                "ANTHROPIC_AUTH_TOKEN": "sk-dummy",
                "DISABLE_NON_ESSENTIAL_MODEL_CALLS": "1",
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
                "CLAUDE_CODE_ATTRIBUTION_HEADER": "0"
            });
            self.raw["env"] = default_env;
        }

        let env_obj = if let Some(v) = self.raw.get_mut("env") {
                if let Some(obj) = v.as_object_mut() {
                    obj
                } else {
                    // replace malformed env with object
                    self.raw["env"] = serde_json::json!({});
                    self.raw["env"].as_object_mut().expect("env object just created")
                }
            } else {
                self.raw["env"] = serde_json::json!({});
                self.raw["env"].as_object_mut().expect("env object just created")
            };

        for (key, value) in env_vars {
            env_obj.insert(key, Value::String(value));
        }
    }

    pub fn save(&self) -> Result<(), io::Error> {
        let json = serde_json::to_string_pretty(&self.raw)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let temp_path = self.path.with_extension("json.tmp");
        fs::write(&temp_path, json)?;

        fs::rename(&temp_path, &self.path)?;

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
        assert_eq!(
            vars.get("ANTHROPIC_DEFAULT_OPUS_MODEL"),
            Some(&"claude-opus-4.5".to_string())
        );
        assert_eq!(
            vars.get("ANTHROPIC_MODEL"),
            Some(&"claude-sonnet-4.5".to_string())
        );
        assert_eq!(
            vars.get("ANTHROPIC_DEFAULT_HAIKU_MODEL"),
            Some(&"claude-haiku-4.5".to_string())
        );
    }

    #[test]
    fn test_claude_pro_env_vars() {
        let vars = ModelProfile::ClaudePro.env_vars();
        assert_eq!(
            vars.get("ANTHROPIC_DEFAULT_OPUS_MODEL"),
            Some(&"claude-opus-4.5".to_string())
        );
        assert_eq!(
            vars.get("ANTHROPIC_MODEL"),
            Some(&"claude-sonnet-4.5".to_string())
        );
        assert_eq!(
            vars.get("ANTHROPIC_DEFAULT_HAIKU_MODEL"),
            Some(&"gpt-5-mini".to_string())
        );
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

    #[test]
    fn test_set_model_profile_updates_env() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_settings_update.json");

        let content = r#"{"env":{"ANTHROPIC_MODEL":"old-model"},"other":"preserved"}"#;
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let mut settings = ClaudeSettings::load_from_path(&test_file).unwrap();
        settings.set_model_profile(ModelProfile::ClaudePro);

        assert_eq!(
            settings.raw["env"]["ANTHROPIC_DEFAULT_OPUS_MODEL"],
            "claude-opus-4.5"
        );
        assert_eq!(settings.raw["env"]["ANTHROPIC_MODEL"], "claude-sonnet-4.5");
        assert_eq!(
            settings.raw["env"]["ANTHROPIC_DEFAULT_HAIKU_MODEL"],
            "gpt-5-mini"
        );
        assert_eq!(settings.raw["other"], "preserved");

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_set_model_profile_creates_env_if_missing() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_settings_no_env.json");

        let content = r#"{"other":"data"}"#;
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let mut settings = ClaudeSettings::load_from_path(&test_file).unwrap();
        settings.set_model_profile(ModelProfile::ClaudeMax);

        assert!(settings.raw.get("env").is_some());
        assert_eq!(settings.raw["env"]["ANTHROPIC_MODEL"], "claude-sonnet-4.5");

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_save_writes_json() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_settings_save.json");

        fs::remove_file(&test_file).ok();

        let mut settings = ClaudeSettings::load_from_path(&test_file).unwrap();
        settings.set_model_profile(ModelProfile::ClaudeFree);
        settings.save().unwrap();

        assert!(test_file.exists());

        let content = fs::read_to_string(&test_file).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["env"]["ANTHROPIC_MODEL"], "gpt-5-mini");

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_save_is_atomic() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_settings_atomic.json");

        let initial = r#"{"env":{"key":"value"}}"#;
        fs::write(&test_file, initial).unwrap();

        let mut settings = ClaudeSettings::load_from_path(&test_file).unwrap();
        settings.set_model_profile(ModelProfile::ClaudeMax);
        settings.save().unwrap();

        let content = fs::read_to_string(&test_file).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_object());

        fs::remove_file(&test_file).ok();
    }
}
