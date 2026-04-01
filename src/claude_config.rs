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
}
