# ClumsyCat TUI Design Specification

**Date**: 2026-03-30
**Project**: ClumsyCat - CLI Tool Launcher TUI
**Status**: Design Approved

## Overview

ClumsyCat is a terminal user interface (TUI) tool for navigating directories and launching AI-powered CLI coding tools with configurable providers and models. The application provides a consistent 4-panel layout across all pages with favorites/recents management, search functionality, and a unified navigation system.

## Core Goals

1. **Fast Navigation**: WASD or Vim keybinds for quick directory/tool/provider/model selection
2. **Persistent State**: Remember favorites and recents across sessions
3. **Unified Experience**: Consistent layout and navigation across all pages
4. **Flexible Configuration**: Support multiple CLI tools, providers, and models
5. **Graceful Handling**: Permission prompts, error recovery, tool validation

## Application State

### State Machine

The application uses a page-based state machine with 5 main pages:

```rust
enum Page {
    Browser,       // Page 1: Directory navigation with favorites/recents
    ToolSelection, // Page 2: Choose CLI tool (Claude Code, Codex, etc.)
    Provider,      // Page 3: Choose provider (only for Claude Code)
    Model,         // Page 4: Choose model from selected provider
    Settings,      // Settings overlay (accessible from any page via Ctrl+S)
}
```

### Dialog System

```rust
enum Dialog {
    None,
    AddToFavorites { path: PathBuf },
    SudoPassword { target_path: PathBuf, password_input: String },
    ToolNotInstalled { tool_name: String },
    Error { message: String },
}
```

### Core App State

```rust
struct App {
    // Navigation state
    page: Page,
    previous_page: Option<Page>,  // For returning from settings
    dialog: Dialog,

    // Page 1: Browser state
    current_dir: PathBuf,
    entries: Vec<DirEntry>,
    selected_index: usize,
    active_panel: ActivePanel,  // Left or Right
    left_section: LeftSection,  // Favorites or Recents
    favorites_dirs: Vec<PathBuf>,
    recents_dirs: Vec<PathBuf>,
    search_mode: SearchMode,

    // Page 2: Tool selection
    tools: Vec<ToolInfo>,
    selected_tool_index: usize,
    favorites_tools: Vec<String>,
    recents_tools: Vec<String>,
    tool_left_section: LeftSection,

    // Page 3: Provider selection (Claude Code only)
    providers: Vec<String>,
    selected_provider_index: usize,
    favorites_providers: Vec<String>,
    recents_providers: Vec<String>,
    provider_left_section: LeftSection,

    // Page 4: Model selection
    models: Vec<String>,
    selected_model_index: usize,
    models_loading: bool,
    models_error: Option<String>,
    favorites_models: Vec<String>,
    recents_models: Vec<String>,
    model_left_section: LeftSection,

    // Settings
    settings: Settings,

    // UI state
    quit_confirm: u8,
    quit_timer: Option<Instant>,
}

struct Settings {
    accent_color: Color,
    nav_mode: NavMode,
}

enum NavMode {
    WASD,
    Vim,  // k/j/h/l
}

enum SearchMode {
    Inactive,
    Active {
        query: String,
        filtered_indices: Vec<usize>,
        current_match_index: usize,
    },
}

enum ActivePanel {
    Left,
    Right,
}

enum LeftSection {
    Favorites,
    Recents,
}
```

## Page Flow

### Navigation Flow

1. **Page 1 (Browser) → Page 2 (Tool Selection)**
   - User presses Enter on a directory in right panel
   - Selected directory stored, advance to tool selection

2. **Page 2 (Tool Selection) → Page 3 (Provider) OR Launch**
   - If "Claude Code" selected → advance to provider selection
   - If other tool selected → validate installation → launch tool (suspend TUI)

3. **Page 3 (Provider) → Page 4 (Model)**
   - User selects provider → advance to model selection
   - Start fetching models in background

4. **Page 4 (Model) → Launch Claude Code**
   - User selects model → set environment variables → launch Claude Code → exit TUI

5. **Any Page → Settings**
   - Press Ctrl+S → open settings overlay
   - Press Esc → return to previous page

6. **Any Page → Previous Page**
   - Press Esc (when no dialog/search active) → go back one page
   - From Page 1 → Ctrl+D×2 to quit

## UI Layout

### Unified 4-Panel Layout (Pages 1-4)

All pages share the same layout structure:

```
┌─────────────────────┬──────────────────────────────────────────┐
│  ╭─────────────╮    │  ╭─ [context title] ─────────────────╮  │
│  │             │    │  │                                    │  │
│  │  ASCII ART  │    │  │  main content area                 │  │
│  │  (orange)   │    │  │  (list of items)                   │  │
│  │             │    │  │                                    │  │
│  ╰─────────────╯    │  │                                    │  │
│                     │  │                                    │  │
│  ╭─ navigation ─╮  │  │                                    │  │
│  │ ★ favorites   │  │  │                                    │  │
│  │   item 1      │  │  │                                    │  │
│  │   item 2      │  │  ╰─ [context info] ─────────────────╯  │
│  │ ─────────────│  │                                          │
│  │ ◷ recents     │  │                                          │
│  │   item a      │  │                                          │
│  │   item b      │  │                                          │
│  ╰───────────────╯  │                                          │
└─────────────────────┴──────────────────────────────────────────┘
[contextual keybinds shown here] ● [separated] ● [by bullets]
```

**Layout proportions**:
- Left column: 25% width
- Right column: 75% width
- ASCII art box: Top of left column
- Navigation panel: Bottom of left column (fills remaining space)
- All borders: Rounded style
- Active panel: Orange accent color
- Inactive panel: Default border color

### Page-Specific Content

#### Page 1: Browser
- **Right panel title**: "browser"
- **Right panel content**: Directory entries (folders first, then files)
- **Right panel footer**: Current directory path
- **Left navigation**: "favorites" / "recents" directories

#### Page 2: Tool Selection
- **Right panel title**: "select tool"
- **Right panel content**: List of CLI tools (claude code, codex, cli kilocode, cli gemini, opencode)
- **Left navigation**: "favorites" / "recents" tools

#### Page 3: Provider Selection
- **Right panel title**: "select provider"
- **Right panel content**: List of providers (github copilot, openrouter, nvidia nim, lm studio)
- **Left navigation**: "favorites" / "recents" providers

#### Page 4: Model Selection
- **Right panel title**: "select model"
- **Right panel content**: Fetched models from provider API (or loading spinner)
- **Left navigation**: "favorites" / "recents" models

### Settings Overlay

Settings appears as a centered overlay on top of the current page:

```
[Current page dimmed in background]

              ╭─────────────────────────────╮
              │                             │
              │        ASCII ART            │
              │                             │
              │        settings             │
              │                             │
              │  accent color: [orange ▼]  │
              │                             │
              │  navigation mode: [wasd ▼] │
              │                             │
              ╰─────────────────────────────╯

[w/s] navigate ● [d/enter] change ● [esc] back
```

**Settings options**:
- **Accent color**: orange, blue, green, purple, red, cyan, yellow
- **Navigation mode**: wasd, vim

Changes apply immediately (live preview).

### Dialog Overlays

Dialogs appear centered over the current page with dimmed background.

#### Add to Favorites Dialog

```
╭─ add to favorites ──────────────────╮
│                                      │
│  add which directory to favorites?   │
│                                      │
│  ● current directory                 │
│    /home/user/projects               │
│                                      │
│  ○ selected directory                │
│    /home/user/projects/myapp         │
│                                      │
│  [enter] confirm  [esc] cancel       │
╰──────────────────────────────────────╯
```

**Note**: Only directories can be added to favorites.

#### Tool Not Installed Dialog

```
╭─ error ─────────────────────────╮
│                                  │
│  please install aider            │
│                                  │
│  [enter] close                   │
╰──────────────────────────────────╯
```

#### Sudo Password Dialog

```
╭─ sudo access required ──────────╮
│                                  │
│  enter password for /root:       │
│                                  │
│  [••••••••]                      │
│                                  │
│  [enter] confirm  [esc] cancel   │
╰──────────────────────────────────╯
```

Password input shown as dots. If incorrect, show "incorrect password" error and return to previous directory.

## Navigation & Keybinds

### Global Keybinds (All Pages)

- **Ctrl+S**: Open settings overlay
- **Ctrl+D×2**: Quit application (requires two presses within 1 second)
- **Esc**: Go back / close dialog / exit search / return from settings

### Page 1 (Browser) - Right Panel Active

- **w / s / ↑ / ↓**: Navigate up/down through entries
- **a / ←**: Go to parent directory
- **d / →**: Enter selected directory (if directory)
- **Enter**: Confirm directory selection, advance to Page 2
- **Tab / Esc**: Switch to left panel
- **/**: Activate search mode
- **Ctrl+F**: Add to favorites (shows dialog)

### Page 1 (Browser) - Left Panel Active

- **w / s / ↑ / ↓**: Navigate up/down through favorites/recents list
- **f**: Toggle between favorites and recents view
- **d**: Load selected directory into right panel (browser)
- **Enter**: Load selected directory and advance to Page 2
- **a / ←**: (does nothing)
- **Tab / Esc**: Switch to right panel
- **Ctrl+F**: Add to favorites (shows dialog)

### Page 1 (Browser) - Search Mode Active

- **Type**: Filter entries to matches, show only matching items
- **w / s / ↑ / ↓**: Navigate through filtered matches
- **Enter**: Confirm selected match (opens directory or advances)
- **Esc**: Exit search mode, return to full unfiltered list

### Pages 2-4 (Tool/Provider/Model Selection) - Right Panel Active

- **w / s / ↑ / ↓**: Navigate up/down through items
- **d**: Mark item as selected/ready (visual feedback)
- **Enter**: Confirm selection, advance to next page or launch
- **Tab / Esc**: Switch to left panel
- **/**: Activate search mode (filters list)
- **Ctrl+F**: Add current selection to favorites

### Pages 2-4 - Left Panel Active

- **w / s / ↑ / ↓**: Navigate up/down through favorites/recents
- **f**: Toggle between favorites and recents view
- **d**: Load selected item into right panel
- **Enter**: Load selected item and advance to next page
- **Tab / Esc**: Switch to right panel
- **Ctrl+F**: Add current selection to favorites

### Settings Overlay

- **w / s / ↑ / ↓**: Navigate between settings
- **d / Enter**: Cycle through options or open dropdown
- **Esc**: Save and close settings, return to previous page

### Dialog Navigation

- **w / s / ↑ / ↓**: Navigate options in dialog
- **Enter**: Confirm dialog action
- **Esc**: Cancel dialog

### Vim Mode Keybinds

When `nav_mode` is set to "vim", replace WASD with:
- **k / ↑**: Up
- **j / ↓**: Down
- **h / ←**: Left/back
- **l / →**: Right/open

All other keybinds remain the same.

## Search Functionality

### Behavior

1. Press `/` to activate search (works in right panel of any page 1-4)
2. Start typing → filters list to show only matching items
3. **w/s** navigates through filtered matches
4. Continue typing to refine filter
5. **Enter** confirms selected match and proceeds (opens/advances)
6. **Esc** exits search, returns to full unfiltered list

### Matching

- Case-insensitive substring match
- Matches anywhere in item name
- Shows "(N matches)" at bottom of filtered list

### Visual Feedback

```
╭─ browser ─────────────────────────╮
│ search: doc_                      │
│                                   │
│  ▸ documents          ← selected  │
│  · docker-compose.yml             │
│  · docs.md                        │
│                                   │
│ (3 matches)                       │
```

## Tool Detection & Launch

### Tool List (Page 2)

The following tools are available:

| Display Name    | Binary Name(s)       | Behavior              |
|-----------------|----------------------|-----------------------|
| claude code     | `claude`             | Go to provider page   |
| codex           | `codex`              | Launch directly       |
| cli kilocode    | `kilo` or `kilocode` | Launch directly       |
| cli gemini      | `gemini`             | Launch directly       |
| opencode        | `opencode`           | Launch directly       |

### Tool Validation

When user selects a tool and presses Enter:

1. Check if binary exists using `which <binary>`
2. For tools with multiple variants (kilo/kilocode), try both
3. If not found → show "please install {tool}" dialog
4. If found:
   - **Claude Code**: Advance to provider selection (Page 3)
   - **Other tools**: Launch tool and suspend TUI

### Non-Claude Code Launch

1. Suspend TUI: `crossterm::terminal::disable_raw_mode()`
2. Spawn process: `std::process::Command::new(tool).current_dir(selected_dir).spawn()`
3. Wait for process to complete
4. Restore TUI: `crossterm::terminal::enable_raw_mode()`
5. Return to Page 1 (Browser)

### Claude Code Launch (After Page 4)

1. Set environment variables based on selected provider (TBD - provider-specific)
2. Execute `claude` in selected directory
3. Exit TUI completely

**Note**: Environment variable mapping will be specified during implementation.

## Provider & Model Selection

### Provider List (Page 3)

When Claude Code is selected, the following providers are available:

- github copilot
- openrouter
- nvidia nim
- lm studio

### Model Fetching (Page 4)

**Behavior**:
1. When entering Page 4, show loading spinner
2. Fetch models from provider API in background
3. On success: Display model list
4. On error: Show error message in red, offer retry
5. On empty result: Show "no models available"

**Loading state**:
```
╭─ select model ────────────────╮
│                               │
│         loading...            │
│         (spinner)             │
│                               │
```

**Loaded state**:
```
╭─ select model ────────────────╮
│                               │
│  claude-opus-4    ← selected  │
│  claude-sonnet-4              │
│  claude-haiku-4               │
│                               │
```

**Error state**:
```
╭─ select model ────────────────╮
│                               │
│  failed to fetch models       │
│  (error message in red)       │
│                               │
│  [enter] retry                │
```

## Data Persistence

### Config File Location

**Path**: `~/.config/clumsycat/config.json`

### Config Structure

```json
{
  "settings": {
    "accent_color": "orange",
    "nav_mode": "wasd"
  },
  "favorites": {
    "directories": ["/home/user", "/projects"],
    "tools": ["claude-code", "codex"],
    "providers": ["openrouter", "github-copilot"],
    "models": ["claude-opus-4", "gpt-4"]
  },
  "recents": {
    "directories": ["/tmp", "/var"],
    "tools": ["cursor"],
    "providers": ["nvidia-nim"],
    "models": ["claude-sonnet-4"]
  }
}
```

### Persistence Behavior

- **On startup**: Load config, use defaults if missing/corrupted
- **Settings changes**: Save immediately
- **Favorites**: Save when added via Ctrl+F
- **Recents**: Save on app exit
- **Recents limit**: 10 items per category (FIFO)
- **Invalid paths**: Remove silently from favorites/recents

### Default Values

```rust
Settings {
    accent_color: Color::from_str("orange"),
    nav_mode: NavMode::WASD,
}
```

Default favorites/recents are empty.

## Error Handling

### File System Errors

**Permission Denied**:
1. Show sudo password dialog
2. If password correct → access directory
3. If password incorrect → show "incorrect password" error, return to previous directory
4. If user cancels (Esc) → return to previous directory

**Directory Deleted/Moved**:
- Show error in browser panel
- Navigate to parent directory
- Remove from recents/favorites

**Empty Directory**:
- Show "empty directory" message in dim text

### API/Network Errors (Page 4)

**Model fetch timeout**:
- Show "failed to fetch models" in red
- Offer retry with Enter

**API error**:
- Display error message in model list area
- Offer retry

**No models returned**:
- Show "no models available" message

### Invalid State Recovery

**Corrupted config**:
- Use defaults
- Show warning dialog once

**Favorite path no longer exists**:
- Remove from list silently

**Tool executable removed**:
- Show install dialog when selected

### User Input Errors

**Invalid search query**:
- Show "no matches" in dim text
- Allow user to continue typing or Esc

**No selection when pressing Enter**:
- Ignore key press (do nothing)

### Graceful Degradation

**Missing config**:
- Create with defaults
- No warning needed

**Failed to save config**:
- Continue with in-memory state
- Warn on exit (dialog)

**Terminal too small**:
- Show "terminal too small" overlay
- Minimum size: 80x24

## ASCII Art

The ASCII art displayed in the top-left box is loaded from `ascii.md`:

```
      ▄▄
      ██                                           ██
▄████ ██ ██ ██ ███▄███▄ ▄█▀▀▀ ██ ██   ▄████  ▀▀█▄ ▀██▀▀
██    ██ ██ ██ ██ ██ ██ ▀███▄ ██▄██   ██    ▄█▀██  ██
▀████ ██ ▀██▀█ ██ ██ ██ ▄▄▄█▀  ▀██▀   ▀████ ▀█▄██  ██
                                ██
                              ▀▀▀
```

Rendered in the accent color (default: orange).

## Visual Style

### Typography

- **All text**: Lowercase (labels, buttons, titles, content)
- **Keybinds**: Lowercase in brackets, e.g. `[enter]`, `[ctrl+s]`

### Colors

**Theme**:
- **Accent color** (default: orange): Active borders, selected items, ASCII art
- **Normal text**: Default terminal foreground
- **Dim text**: Dimmed for secondary info (unselected items, hints)
- **Error text**: Red
- **Border normal**: Default terminal color
- **Border focused**: Accent color

### Borders

- **Style**: Rounded (`╭─╮│╰─╯`)
- **Active panel**: Accent color
- **Inactive panel**: Normal color
- **Dialogs/overlays**: Accent color

### Bottom Bar

Format: `[key] action ● [key] action ● [key] action`

Context-sensitive keybinds shown, separated by `●` (bullet) symbol.

**Page 1 examples**:
- Right panel active: `[w/s] up/down ● [a] back ● [d] open ● [enter] confirm ● [tab] switch panel ● [/] search ● [ctrl+f] add favorite ● [ctrl+s] settings ● [ctrl+d×2] quit`
- Left panel active: `[w/s] navigate ● [d] load dir ● [enter] select ● [f] toggle view ● [tab] switch panel ● [ctrl+f] add favorite ● [ctrl+s] settings`
- Search active: `[type] filter ● [w/s] navigate ● [enter] confirm ● [esc] exit search`

**Page 2-4 examples**:
- `[w/s] navigate ● [d] select ● [enter] confirm ● [tab] switch panel ● [/] search ● [ctrl+f] add favorite ● [esc] back ● [ctrl+s] settings`

**Settings**:
- `[w/s] navigate ● [d/enter] change ● [esc] back`

**Dialogs**:
- `[w/s] navigate ● [enter] confirm ● [esc] cancel`

## Implementation Notes

### Module Structure

```
src/
├── main.rs          # Entry point, terminal init/restore
├── app.rs           # App state machine, event handling
├── ui.rs            # Rendering logic for all pages/dialogs
├── theme.rs         # Color scheme, accent colors
├── fs.rs            # File system operations, directory loading
├── config.rs        # Config loading/saving, persistence
├── tools.rs         # Tool detection, validation, launch
├── api.rs           # Provider API clients for model fetching
└── search.rs        # Search filtering logic
```

### Key Dependencies

- `ratatui = "0.29"` - TUI framework
- `crossterm = "0.28"` - Terminal manipulation, event handling
- `serde = { version = "1.0", features = ["derive"] }` - Config serialization
- `serde_json = "1.0"` - JSON config format
- `tokio` or `async-std` - Async runtime for API calls

### State Transitions

State transitions are explicit and unidirectional:

```
Page 1 (Browser)
    ├─ Enter → Page 2 (Tool Selection)
    └─ Esc → Quit (with confirmation)

Page 2 (Tool Selection)
    ├─ Enter (Claude Code) → Page 3 (Provider)
    ├─ Enter (Other tool) → Launch & suspend
    └─ Esc → Page 1

Page 3 (Provider)
    ├─ Enter → Page 4 (Model)
    └─ Esc → Page 2

Page 4 (Model)
    ├─ Enter → Launch Claude Code & exit
    └─ Esc → Page 3

Any Page
    ├─ Ctrl+S → Settings overlay
    └─ Settings Esc → Previous page
```

### Testing Strategy

**Unit tests**:
- Search filtering logic
- Config loading/saving with missing/corrupted files
- Tool detection with various PATH configurations
- State transitions

**Integration tests**:
- Full page flow: Browser → Tool → Provider → Model → Launch
- Favorites/recents persistence across restarts
- Error recovery (permission denied, missing tools, API failures)

**Manual testing**:
- Keyboard navigation on all pages
- Panel switching, search, dialogs
- Terminal resize handling
- Color theme variations

## Future Enhancements

Potential features for future versions (not in scope for initial implementation):

- **Bookmarks**: Named bookmarks beyond favorites
- **History**: Full navigation history with back/forward
- **Themes**: Full theme support beyond accent color
- **Plugins**: Custom tool definitions via config
- **Multiplexing**: Keep TUI active while tool runs in split pane
- **Remote**: SSH into remote machines and launch tools there
- **Git integration**: Show git status in browser, quick commit actions

## Open Questions

- **Provider environment variables**: Mapping of provider → env vars (TBD during implementation)
- **API authentication**: How to handle API keys for model fetching (config file? env vars?)
- **Model caching**: Should fetched models be cached to disk for offline use?

---

**Design Status**: ✅ Approved
**Next Step**: Write implementation plan using writing-plans skill
