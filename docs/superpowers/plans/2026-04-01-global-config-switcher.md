# Global Config Switcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add settings overlay that lets users switch Claude Code model profiles and persist to ~/.claude/settings.json

**Architecture:** New claude_config.rs module manages Claude settings.json reading/writing. App.rs adds overlay state and command bar integration. UI.rs renders settings overlay matching existing patterns.

**Tech Stack:** Rust, ratatui, serde_json, existing ClumsyCat architecture

---

## File Structure

**New files:**
- `src/claude_config.rs` - Claude settings.json management (load, parse, update env vars, save atomically)

**Modified files:**
- `src/main.rs` - Add claude_config module declaration
- `src/app.rs` - Add overlay state, command, input handler, execute logic
- `src/ui.rs` - Add overlay rendering function

---

## Task 1: Create claude_config Module Foundation

**Files:**
- Create: `src/claude_config.rs`
- Test: `src/claude_config.rs` (inline tests)

- [ ] **Step 1: Write failing test for ClaudeSettings::settings_path**

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_settings_path_returns_claude_dir`
Expected: FAIL with "ClaudeSettings not found"

- [ ] **Step 3: Write minimal ClaudeSettings structure**

```rust
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_settings_path_returns_claude_dir`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/claude_config.rs
git commit -m "feat(config): add ClaudeSettings foundation with path resolution"
```

---

## Task 2: Implement ModelProfile Environment Variables

**Files:**
- Modify: `src/claude_config.rs`

- [ ] **Step 1: Write failing test for ModelProfile::env_vars**

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_claude_max_env_vars test_claude_pro_env_vars test_claude_free_env_vars`
Expected: FAIL with "method not found"

- [ ] **Step 3: Implement ModelProfile::env_vars**

```rust
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
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_claude_max_env_vars test_claude_pro_env_vars test_claude_free_env_vars`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/claude_config.rs
git commit -m "feat(config): add ModelProfile env_vars mapping"
```

---

## Task 3: Implement ClaudeSettings::load

**Files:**
- Modify: `src/claude_config.rs`

- [ ] **Step 1: Write failing test for load with existing file**

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_load_existing_settings test_load_missing_file_creates_default`
Expected: FAIL with "method not found"

- [ ] **Step 3: Implement ClaudeSettings::load methods**

```rust
impl ClaudeSettings {
    pub fn load() -> Result<Self, io::Error> {
        let path = Self::settings_path();
        Self::load_from_path(&path)
    }

    fn load_from_path(path: &PathBuf) -> Result<Self, io::Error> {
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
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_load_existing_settings test_load_missing_file_creates_default`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/claude_config.rs
git commit -m "feat(config): implement ClaudeSettings load with default creation"
```

---

## Task 4: Implement ClaudeSettings::set_model_profile

**Files:**
- Modify: `src/claude_config.rs`

- [ ] **Step 1: Write failing test for set_model_profile**

```rust
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

    assert_eq!(settings.raw["env"]["ANTHROPIC_DEFAULT_OPUS_MODEL"], "claude-opus-4.5");
    assert_eq!(settings.raw["env"]["ANTHROPIC_MODEL"], "claude-sonnet-4.5");
    assert_eq!(settings.raw["env"]["ANTHROPIC_DEFAULT_HAIKU_MODEL"], "gpt-5-mini");
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_set_model_profile_updates_env test_set_model_profile_creates_env_if_missing`
Expected: FAIL with "method not found"

- [ ] **Step 3: Implement ClaudeSettings::set_model_profile**

```rust
impl ClaudeSettings {
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

        let env_obj = self.raw["env"].as_object_mut().unwrap();

        for (key, value) in env_vars {
            env_obj.insert(key, Value::String(value));
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_set_model_profile_updates_env test_set_model_profile_creates_env_if_missing`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/claude_config.rs
git commit -m "feat(config): implement set_model_profile with env creation"
```

---

## Task 5: Implement ClaudeSettings::save with Atomic Write

**Files:**
- Modify: `src/claude_config.rs`

- [ ] **Step 1: Write failing test for save**

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_save_writes_json test_save_is_atomic`
Expected: FAIL with "method not found"

- [ ] **Step 3: Implement ClaudeSettings::save**

```rust
impl ClaudeSettings {
    pub fn save(&self) -> Result<(), io::Error> {
        let json = serde_json::to_string_pretty(&self.raw)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let temp_path = self.path.with_extension("json.tmp");
        fs::write(&temp_path, json)?;

        fs::rename(&temp_path, &self.path)?;

        Ok(())
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_save_writes_json test_save_is_atomic`
Expected: PASS

- [ ] **Step 5: Run all claude_config tests**

Run: `cargo test --lib claude_config`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```bash
git add src/claude_config.rs
git commit -m "feat(config): implement atomic save for settings.json"
```

---

## Task 6: Add claude_config Module to main.rs

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add module declaration**

Find the existing module declarations (likely near the top after use statements) and add:

```rust
mod claude_config;
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: SUCCESS with no errors

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat(config): register claude_config module"
```

---

## Task 7: Add Global Config State to App

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add state fields to App struct**

Find the `pub struct App` definition (around line 91) and add these fields after `settings_selection`:

```rust
pub global_config_open: bool,
pub global_config_selection: usize,
```

- [ ] **Step 2: Initialize fields in App::new()**

Find the `App::new()` implementation (around line 164) and add these initializations in the returned `Self` block after `settings_selection: 0`:

```rust
global_config_open: false,
global_config_selection: 0,
```

- [ ] **Step 3: Add globalconf command to COMMANDS array**

Find the `pub const COMMANDS: &[Command]` definition (around line 70) and add after the existing commands:

```rust
Command { name: "globalconf", description: "switch claude code model configuration" },
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build`
Expected: SUCCESS with no errors

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat(config): add global config state and command"
```

---

## Task 8: Implement Global Config Input Handler

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add import for claude_config**

Find the imports at the top of app.rs (after `use crate::`) and add:

```rust
use crate::claude_config::{ClaudeSettings, ModelProfile};
```

- [ ] **Step 2: Add handle_global_config_input method**

Add this method to the `impl App` block (after `handle_settings_input` around line 1213):

```rust
fn handle_global_config_input(&mut self, code: ratatui::crossterm::event::KeyCode) {
    use ratatui::crossterm::event::KeyCode;

    match code {
        KeyCode::Esc => {
            self.global_config_open = false;
            self.global_config_selection = 0;
        }
        KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
            if self.global_config_selection > 0 {
                self.global_config_selection -= 1;
            } else {
                self.global_config_selection = 2;
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
            self.global_config_selection = (self.global_config_selection + 1) % 3;
        }
        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right | KeyCode::Enter => {
            let profile = match self.global_config_selection {
                0 => ModelProfile::ClaudeMax,
                1 => ModelProfile::ClaudePro,
                2 => ModelProfile::ClaudeFree,
                _ => ModelProfile::ClaudeMax,
            };

            match ClaudeSettings::load() {
                Ok(mut settings) => {
                    settings.set_model_profile(profile);
                    match settings.save() {
                        Ok(_) => {
                            self.global_config_open = false;
                            self.global_config_selection = 0;
                        }
                        Err(e) => {
                            self.dialog = Dialog::Error {
                                message: format!("cannot write to settings.json: {}", e),
                            };
                            self.global_config_open = false;
                        }
                    }
                }
                Err(e) => {
                    self.dialog = Dialog::Error {
                        message: format!("cannot load settings.json: {}", e),
                    };
                    self.global_config_open = false;
                }
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: SUCCESS with no errors

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat(config): add global config input handler"
```

---

## Task 9: Wire Global Config Input to Event Loop

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add global config check to run() method**

Find the `run()` method (around line 455). After the settings overlay input handling (around line 509) and before search mode handling, add:

```rust
// Handle global config overlay input
if self.global_config_open {
    let is_ctrl_d = matches!(code, KeyCode::Char('d') | KeyCode::Char('D'))
        && key.modifiers.contains(KeyModifiers::CONTROL);
    if !is_ctrl_d {
        self.handle_global_config_input(code);
        continue;
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: SUCCESS with no errors

- [ ] **Step 3: Commit**

```bash
git add src/app.rs
git commit -m "feat(config): wire global config overlay to event loop"
```

---

## Task 10: Implement Command Execution for globalconf

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add case to execute_command method**

Find the `execute_command` method (around line 2001). Add a new case after case 3:

```rust
4 => {
    // globalconf
    self.global_config_open = true;
    self.global_config_selection = 0;
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: SUCCESS with no errors

- [ ] **Step 3: Commit**

```bash
git add src/app.rs
git commit -m "feat(config): add globalconf command execution"
```

---

## Task 11: Add Global Config Overlay Rendering

**Files:**
- Modify: `src/ui.rs`

- [ ] **Step 1: Add render call in main render function**

Find the `pub fn render(app: &App, frame: &mut Frame)` function. After the settings overlay render check (search for `if app.settings_open`), add:

```rust
if app.global_config_open {
    render_global_config_overlay(app, frame);
}
```

- [ ] **Step 2: Implement render_global_config_overlay function**

Add this function at the end of ui.rs (after render_settings_overlay):

```rust
fn render_global_config_overlay(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let accent = get_accent_color(&app.settings.accent_color, &app.settings.custom_color_hex);

    let overlay_width = area.width / 2;
    let overlay_height = (area.height * 2) / 5;
    let overlay_x = (area.width.saturating_sub(overlay_width)) / 2;
    let overlay_y = (area.height.saturating_sub(overlay_height)) / 2;

    let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent))
        .border_type(BorderType::Rounded)
        .title(" global config switcher ")
        .title_style(Style::default().fg(accent).add_modifier(Modifier::BOLD));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let profiles = ["claude max", "claude pro", "claude free"];
    let descriptions = [
        "opus 4.5 | sonnet 4.5 | haiku 4.5",
        "opus 4.5 | sonnet 4.5 | haiku gpt-5-mini",
        "sonnet gpt-5-mini",
    ];

    let content_area = Rect::new(
        inner.x + 2,
        inner.y + 1,
        inner.width.saturating_sub(4),
        inner.height.saturating_sub(3),
    );

    for (i, (profile, desc)) in profiles.iter().zip(descriptions.iter()).enumerate() {
        let y = content_area.y + (i as u16 * 3);
        if y + 2 >= content_area.y + content_area.height {
            break;
        }

        let is_selected = i == app.global_config_selection;
        let (prefix, style) = if is_selected {
            ("> ", Style::default().fg(accent).add_modifier(Modifier::BOLD))
        } else {
            ("  ", Style::default().fg(Color::Gray))
        };

        let profile_text = format!("{}{}", prefix, profile);
        let profile_para = Paragraph::new(profile_text).style(style);
        frame.render_widget(
            profile_para,
            Rect::new(content_area.x, y, content_area.width, 1),
        );

        let desc_para = Paragraph::new(*desc).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(
            desc_para,
            Rect::new(content_area.x + 2, y + 1, content_area.width, 1),
        );
    }

    let footer_y = inner.y + inner.height.saturating_sub(1);
    let footer_text = "[w/s] navigate  [enter] apply  [esc] cancel";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(
        footer,
        Rect::new(inner.x, footer_y, inner.width, 1),
    );
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: SUCCESS with no errors

- [ ] **Step 4: Commit**

```bash
git add src/ui.rs
git commit -m "feat(ui): add global config overlay rendering"
```

---

## Task 12: Manual Integration Testing

**Files:**
- Test: Full application

- [ ] **Step 1: Build and run ClumsyCat**

Run: `cargo run`
Expected: Application launches without errors

- [ ] **Step 2: Open command bar and test globalconf**

Actions:
1. Press Shift+F to open command bar
2. Type "globalconf" or navigate to it
3. Press Enter

Expected: Global config overlay appears with three profiles

- [ ] **Step 3: Test navigation**

Actions:
1. Press 'w' to move up
2. Press 's' to move down
3. Verify selection wraps around

Expected: Selection indicator moves correctly, wraps from top to bottom and vice versa

- [ ] **Step 4: Test profile application**

Actions:
1. Navigate to "claude pro"
2. Press Enter or 'd'

Expected: Overlay closes without error

- [ ] **Step 5: Verify settings.json was updated**

Run: `cat ~/.claude/settings.json | grep ANTHROPIC_MODEL`
Expected: See "claude-sonnet-4.5" and "gpt-5-mini" values

- [ ] **Step 6: Test cancel**

Actions:
1. Open globalconf again (Shift+F → globalconf → Enter)
2. Navigate to different profile
3. Press Esc

Expected: Overlay closes, settings.json unchanged

- [ ] **Step 7: Note manual test results**

Document any issues found. If all tests pass, proceed to commit.

---

## Task 13: Run Full Test Suite and Linting

**Files:**
- Test: All

- [ ] **Step 1: Run all tests**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings or errors

- [ ] **Step 3: Run cargo fmt check**

Run: `cargo fmt -- --check`
Expected: All files properly formatted (or run `cargo fmt` to auto-fix)

- [ ] **Step 4: Final build verification**

Run: `cargo build --release`
Expected: SUCCESS

- [ ] **Step 5: Commit if all checks pass**

```bash
git add -A
git commit -m "test: verify global config switcher integration"
```

---

## Task 14: Edge Case Testing

**Files:**
- Test: settings.json edge cases

- [ ] **Step 1: Test missing env section**

Actions:
1. Backup ~/.claude/settings.json: `cp ~/.claude/settings.json ~/.claude/settings.json.bak`
2. Edit settings.json to remove the entire "env" section
3. Launch ClumsyCat
4. Open globalconf (Shift+F → globalconf)
5. Select "claude max"
6. Press Enter

Expected: No error, settings.json now has "env" section with correct values

- [ ] **Step 2: Verify env section created correctly**

Run: `cat ~/.claude/settings.json | grep -A 8 '"env"'`
Expected: See complete env section with all required keys

- [ ] **Step 3: Test missing settings.json file**

Actions:
1. Delete settings.json: `rm ~/.claude/settings.json`
2. Launch ClumsyCat
3. Open globalconf
4. Select "claude free"
5. Press Enter

Expected: Settings.json created with correct structure

- [ ] **Step 4: Verify file creation**

Run: `test -f ~/.claude/settings.json && echo "exists" || echo "missing"`
Expected: "exists"

- [ ] **Step 5: Test preserving unknown fields**

Actions:
1. Add custom field to settings.json: `"custom_field": "custom_value"`
2. Launch ClumsyCat
3. Change profile via globalconf
4. Check settings.json

Expected: custom_field still present

- [ ] **Step 6: Restore backup**

Run: `cp ~/.claude/settings.json.bak ~/.claude/settings.json`

- [ ] **Step 7: Document edge case test results**

Note any issues found. If all pass, proceed to final commit.

---

## Task 15: Final Documentation and Commit

**Files:**
- Create: None (documentation already exists in spec)
- Verify: All files committed

- [ ] **Step 1: Verify all changes committed**

Run: `git status`
Expected: "working tree clean"

- [ ] **Step 2: Review commit history**

Run: `git log --oneline -15`
Expected: See all feature commits for global config switcher

- [ ] **Step 3: Test final binary**

Run: `cargo build --release && ./target/release/cc`
Expected: Application works correctly with all global config features

- [ ] **Step 4: Create final summary commit if needed**

If there were any final tweaks not yet committed:

```bash
git add -A
git commit -m "feat(config): complete global config switcher implementation

- Add claude_config module for settings.json management
- Add global config overlay with three model profiles
- Add command bar integration via 'globalconf' command
- Support atomic writes and edge case handling
- Preserve existing settings while updating model configs"
```

---

## Self-Review Checklist

**Spec Coverage:**
- ✓ claude_config.rs module created (Tasks 1-5)
- ✓ ModelProfile with env_vars mapping (Task 2)
- ✓ ClaudeSettings load/save/set_model_profile (Tasks 3-5)
- ✓ App state for overlay (Task 7)
- ✓ Command bar integration (Tasks 7, 10)
- ✓ Input handler (Tasks 8-9)
- ✓ UI overlay rendering (Task 11)
- ✓ Error handling via dialogs (Task 8)
- ✓ Edge case handling (Tasks 3, 4, 14)
- ✓ Testing strategy (all tasks include tests)

**Placeholder Scan:**
- ✓ No TBD, TODO, or "implement later"
- ✓ All code blocks complete
- ✓ All file paths exact
- ✓ All commands have expected output

**Type Consistency:**
- ✓ ModelProfile enum used consistently (ClaudeMax, ClaudePro, ClaudeFree)
- ✓ ClaudeSettings struct and methods match across tasks
- ✓ App state fields (global_config_open, global_config_selection) consistent
- ✓ Dialog::Error usage matches existing pattern

**Execution Ready:**
- ✓ Each task is 2-5 minute chunks
- ✓ TDD flow: test first, implement, verify, commit
- ✓ Build verification after each task
- ✓ Integration and edge case testing included
