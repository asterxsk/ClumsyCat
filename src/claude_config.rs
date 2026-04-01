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
}
