---
name: Global Config Switcher
description: Settings overlay for switching Claude Code model configurations via settings.json
type: feature
---

# Global Config Switcher Design

## Context

Users need to configure Claude Code's default model names (ANTHROPIC_DEFAULT_OPUS_MODEL, ANTHROPIC_MODEL, ANTHROPIC_DEFAULT_HAIKU_MODEL) from within ClumsyCat. Currently, these values are hardcoded in the launch process, but they should be persisted to `~/.claude/settings.json` so they apply globally to all Claude Code sessions.

ClumsyCat already shows three GitHub Copilot profiles on the Model selection page:
- **Claude Max**: opus 4.5, sonnet 4.5, haiku 4.5
- **Claude Pro**: opus 4.5, sonnet 4.5, haiku: gpt-5-mini
- **Claude Free**: sonnet: gpt-5-mini only

This feature adds a settings overlay accessible via the command bar that lets users select one of these profiles and apply it globally to Claude Code's settings.json.

## Architecture

### Module: `claude_config.rs`

A new module for managing Claude Code's settings.json file.

**Why:** Separation of concerns - Claude-specific config logic doesn't belong in app.rs or config.rs (which manages ClumsyCat's own config).

**How to apply:** Import and use from app.rs when executing the global config command.

**Responsibilities:**
- Read settings.json from `~/.claude/settings.json` (or `%USERPROFILE%/.claude/settings.json` on Windows)
- Parse JSON, preserving existing structure
- Update/insert the `env` section with model environment variables
- Handle edge case: if `env` section doesn't exist, create it at the top level
- Write back atomically (write to temp file, then rename)
- Validate JSON structure before writing

**Structure:**
```rust
pub struct ClaudeSettings {
    // Raw JSON value to preserve unknown fields
    raw: serde_json::Value,
}

pub enum ModelProfile {
    ClaudeMax,
    ClaudePro,
    ClaudeFree,
}

impl ClaudeSettings {
    pub fn load() -> Result<Self, Error>
    pub fn set_model_profile(&mut self, profile: ModelProfile)
    pub fn save(&self) -> Result<(), Error>
}
```

### UI: Settings Overlay Pattern

Follows the existing settings overlay pattern (Ctrl+S), but triggered via command bar.

**Why:** Reuses proven UI pattern that users already understand. The settings overlay provides a full-screen modal experience suitable for important configuration changes.

**How to apply:** Add new app state fields and input handler similar to existing settings_open logic.

**State in `App`:**
- `global_config_open: bool` - whether overlay is active
- `global_config_selection: usize` - selected profile index (0-2)

**Rendering:** Similar to settings overlay in `ui.rs`, full-screen centered box with:
- Title: "global config switcher"
- Three options: "claude max", "claude pro", "claude free"
- Navigation: w/s to move, d/enter to select, esc to cancel
- Visual indicator showing current selection

### Command Bar Integration

**Command:**
- Name: `"globalconf"`
- Description: `"switch claude code model configuration"`

**Why:** Command bar provides discoverability and consistent access pattern. Users already use Shift+F for configuration tasks.

**How to apply:** Add to COMMANDS array in app.rs, implement execute_command case.

### Input Handling

When overlay is open:
- `w/s/up/down`: Navigate between profiles
- `d/right/enter`: Apply selected profile
- `esc`: Cancel without changes

**Why:** Matches existing settings overlay behavior for consistency.

**How to apply:** Add `handle_global_config_input()` method similar to `handle_settings_input()`.

### Profile Selection Flow

1. User opens command bar (Shift+F)
2. Types/selects "globalconf"
3. Overlay appears with three profiles
4. User navigates with w/s, selects with d/enter
5. App calls `ClaudeSettings::load()`, `set_model_profile()`, `save()`
6. Success: overlay closes, brief confirmation message
7. Error: overlay closes, error dialog shows what went wrong

**Why:** Simple, predictable flow with clear feedback.

**How to apply:** Execute in `execute_command()` when command index matches globalconf.

## Component Design

### ClaudeSettings Module

**Profile to Environment Variable Mapping:**

```rust
impl ModelProfile {
    pub fn env_vars(&self) -> HashMap<String, String> {
        match self {
            ClaudeMax => [
                ("ANTHROPIC_DEFAULT_OPUS_MODEL", "claude-opus-4.5"),
                ("ANTHROPIC_MODEL", "claude-sonnet-4.5"),
                ("ANTHROPIC_DEFAULT_HAIKU_MODEL", "claude-haiku-4.5"),
            ],
            ClaudePro => [
                ("ANTHROPIC_DEFAULT_OPUS_MODEL", "claude-opus-4.5"),
                ("ANTHROPIC_MODEL", "claude-sonnet-4.5"),
                ("ANTHROPIC_DEFAULT_HAIKU_MODEL", "gpt-5-mini"),
            ],
            ClaudeFree => [
                ("ANTHROPIC_MODEL", "gpt-5-mini"),
            ],
        }
    }
}
```

**Edge Case Handling:**

If settings.json doesn't have an `env` section, create it:
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://localhost:4141",
    "ANTHROPIC_AUTH_TOKEN": "sk-dummy",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "claude-opus-4.5",
    "ANTHROPIC_MODEL": "claude-sonnet-4.5",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "claude-haiku-4.5",
    "DISABLE_NON_ESSENTIAL_MODEL_CALLS": "1",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
    "CLAUDE_CODE_ATTRIBUTION_HEADER": "0"
  },
  ...existing fields...
}
```

**Why:** Preserve existing env variables (BASE_URL, AUTH_TOKEN, etc.) while only updating model-related ones. If env doesn't exist, initialize with sensible defaults.

**How to apply:** Use serde_json::Value for flexible JSON manipulation. Update only model-related keys, preserve everything else.

### UI Rendering

**Overlay Layout:**
- Centered box, 50% width, 40% height
- Rounded border with accent color
- Title bar: "global config switcher"
- Three items with highlighting on selected
- Footer: keybind hints `[w/s] navigate  [enter] apply  [esc] cancel`

**Visual States:**
- Normal item: gray text
- Selected item: accent color with `>` prefix
- Confirmation flash: brief "applied!" message on success

**Why:** Matches existing dialog patterns for immediate familiarity.

**How to apply:** Implement in `ui.rs` similar to render_settings_overlay.

## Error Handling

**File Operations:**
- Permission denied: Show error dialog "cannot write to settings.json - check permissions"
- File not found: Create `~/.claude/` directory and settings.json with defaults
- Parse error: Show error dialog with details, don't corrupt file

**JSON Manipulation:**
- Invalid JSON: Show error dialog, don't write
- Missing expected structure: Create structure with defaults

**Why:** Configuration errors should never leave the user in a broken state. Always validate before writing.

**How to apply:** Use Result types throughout claude_config.rs, convert to error dialogs in app.rs.

## Testing Strategy

**Unit Tests (claude_config.rs):**
- Load and parse valid settings.json
- Handle missing env section
- Apply each profile correctly
- Preserve unknown fields
- Atomic write operation (temp file then rename)

**Integration Tests:**
- Command bar can open overlay
- Overlay navigation works
- Profile selection updates settings.json
- Error cases show appropriate dialogs

**Manual Testing:**
- Verify settings.json changes persist
- Launch Claude Code after changing profile, confirm models are correct
- Test on Linux and Windows paths

## File Changes

**New Files:**
- `src/claude_config.rs` - Claude settings management module

**Modified Files:**
- `src/main.rs` - Add `mod claude_config;`
- `src/app.rs`:
  - Add global_config_open and global_config_selection state
  - Add command to COMMANDS array
  - Add execute_command case for globalconf
  - Add handle_global_config_input method
  - Update run() to handle overlay input
- `src/ui.rs`:
  - Add render_global_config_overlay function

**Build Changes:**
- None - uses existing dependencies (serde_json)

## Verification

After implementation:
1. Run `cargo build` - should compile without errors
2. Run `cargo test` - all tests pass
3. Run `cargo clippy -- -D warnings` - no warnings
4. Manual test:
   - Launch ClumsyCat
   - Press Shift+F, select "globalconf"
   - Select "claude pro" profile
   - Check `~/.claude/settings.json` has correct model values
   - Launch Claude Code, verify models match selection
5. Edge case test:
   - Backup settings.json
   - Remove `env` section
   - Apply profile via globalconf
   - Verify `env` section created with correct values
