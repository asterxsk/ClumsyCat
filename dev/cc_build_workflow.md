# CC / Clumsy Cat — Agent Build Workflow

Four sequential prompts. Each prompt is a self-contained instruction for an AI coding agent. Execute them in order. Do not skip steps or combine them.

---

## Step 1 — TUI Foundation & File Browser

```
You are building a terminal UI application in Rust called "Clumsy Cat" (binary: cc).
The TUI library is ratatui (latest stable). Use crossterm as the backend.

Ratatui docs: https://ratatui.rs — read this before writing any code. Specifically understand:
- Layout system (Direction, Constraint, Layout::default())
- Block and Borders
- List and ListState
- Paragraph
- StatefulWidget trait
- event handling with crossterm (KeyCode, KeyModifiers, Event::Key)

---

LAYOUT

The application has three visible regions rendered every frame:

1. TOP BAR
   - Full width, fixed height (~5 lines)
   - Bordered (Borders::ALL)
   - Centered content: ASCII art placeholder text "CLUMSY CAT" (the user will replace this later)
   - Title on border: none (the art IS the content)

2. LEFT PANEL
   - Sits below the top bar
   - ~25% of terminal width
   - Bordered (Borders::ALL)
   - Title on border: " Navigation "
   - Contains two stacked sub-sections:
       a. FAVORITES — a List widget, title "Favorites"
       b. RECENTS   — a List widget, title "Recents"
   - Both sub-sections are separated by a horizontal rule (use a Paragraph with a line of "─" chars)
   - Placeholder items for now: 3 fake favorites, 3 fake recent paths

3. RIGHT PANEL
   - Sits below the top bar, beside the left panel
   - Remaining width (~75%)
   - Bordered (Borders::ALL)
   - Title on border: " Browser "
   - Shows the contents of the current directory
   - Each entry is shown as:   [type_icon]  name   (type_icon: "▸" for dir, "·" for file)
   - Highlight the currently selected entry using a highlighted style (bold + accent color from theme)
   - Show full current path as a sub-title on the bottom border: " /path/to/dir "

4. BOTTOM BAR
   - Full width, fixed height (~3 lines)
   - Bordered (Borders::ALL)
   - Displays keybind hints as a single line:
     [/] Search   [Space] Select   [D] Open   [A] Back   [W/S] Up/Down   [Tab] Cycle Panel   [Esc] Panel Mode   [Ctrl+D×2] Quit

---

THEMING

Define a Theme struct that holds a set of ratatui Style values:
- border_normal: default dim style
- border_focused: bold + a specific Color (default: Cyan)
- highlight: bold + Cyan background + Black fg
- text_normal
- text_dim

Implement a Default theme. All borders and highlights must use Theme values — no hardcoded colors anywhere else.

---

NAVIGATION MODEL

Define an enum ActivePanel { Left, Right } for which panel is focused.
Define an enum LeftSection { Favorites, Recents } for which sub-section is active in the left panel.

State struct fields (minimum):
- current_dir: PathBuf
- entries: Vec<DirEntry>  (sorted: dirs first, then files, both alphabetical)
- selected_index: usize   (index in entries)
- left_section: LeftSection
- active_panel: ActivePanel
- favorites: Vec<PathBuf>
- recents: Vec<PathBuf>   (max 10, push_front on visit, deduplicate)
- quit_confirm: u8        (counts Ctrl+D presses, resets after 1s)

---

KEYBINDS

When active_panel == Right (browser panel is focused):
  W / Up    → move selection up
  S / Down  → move selection down
  D / Right → if selected entry is a directory, enter it (update current_dir, reload entries, push to recents)
  A / Left  → go to parent directory (if any)
  Space     → "select" current entry — for now, just print a TODO log line
  /         → TODO stub: enter search mode (no-op for now, just log)
  Esc       → switch to panel mode (ActivePanel focus switches to Left)
  Tab       → cycle active panel Left ↔ Right

When active_panel == Left:
  W / S     → move between Favorites and Recents sub-sections
  Space     → navigate to the highlighted path in the selected sub-section
  Tab / Esc → switch to Right panel

Global:
  Ctrl+D    → increment quit_confirm; if == 2 within 1s, quit. Reset timer on each press.

---

FILE SYSTEM

- Load entries from current_dir on startup (default: user's home directory, fallback to "/")
- Re-sort and reload whenever current_dir changes
- Handle permission errors gracefully: show an inline error line in the browser panel
- Do NOT use async. Use std::fs for all file operations.

---

IMPLEMENTATION RULES

- No comments in code
- Use newest stable versions of all crates
- Entry point: src/main.rs — keep it under 30 lines, delegate everything to modules
- Suggested module layout:
    src/
      main.rs
      app.rs       (App / State struct, update logic)
      ui.rs        (all ratatui rendering)
      theme.rs     (Theme struct)
      fs.rs        (directory loading helpers)
- Cargo.toml must include: ratatui, crossterm
- The app must compile and run with `cargo run` from the project root
- Test that navigation, recents tracking, and Ctrl+D double-tap quit all work before finishing
```

---

## Step 2 — CLI Launcher Screen

```
You are continuing development of "Clumsy Cat" (cc), a ratatui TUI app in Rust.
Ratatui docs: https://ratatui.rs
Step 1 built the file browser. Now add the CLI launcher screen.

This screen appears when the user presses Space on a directory entry in the file browser.
The selected path is passed into this screen as context.

---

SCREEN STRUCTURE

The CLI screen replaces the browser layout entirely (full screen takeover).
It has the same top bar (same ASCII art, same style) and bottom keybind bar.

The main body has the same left/right split:

LEFT PANEL — " Navigation " (same as before)
  - Favorites (list of favorited CLI tools, stored as Vec<String> tool names)
  - Recents   (last 5 used CLI tools)
  - Keybind to add current selection to favorites: [F]

RIGHT PANEL — " CLI Tools "
  - A scrollable List of all supported CLI tools
  - Each item: name + short description aligned with padding
  - One entry per tool, highlight on selection
  - Pressing Enter or Space on a tool proceeds to launch (or next screen for Claude Code)

BOTTOM BAR keybind line:
  [W/S] Up/Down   [Space/Enter] Select   [F] Favorite   [A] Back   [Tab] Cycle   [Ctrl+D×2] Quit

---

CLI TOOL LIST

Include every tool in this list. Add more if you know others not listed.
Format: (display_name, launch_command, description)

("Claude Code",        "claude",         "Anthropic Claude — agentic coding")
("Kilocode",           "kilo",           "Lightweight AI code assistant")
("OpenCode",           "opencode",       "Open-source AI coding agent")
("Gemini CLI",         "gemini",         "Google Gemini in the terminal")
("GitHub Copilot CLI", "gh copilot",     "Copilot explain, suggest, alias")
("Aider",              "aider",          "AI pair programming in your terminal")
("Continue",           "continue",       "Open-source AI code assistant")
("Cody CLI",           "cody",           "Sourcegraph Cody CLI")
("Cursor",             "cursor",         "AI-first code editor")
("Amazon Q CLI",       "q",              "AWS Amazon Q developer CLI")
("Tabnine",            "tabnine",        "AI code completion")
("Codeium CLI",        "codeium",        "Free AI code assistant")
("Plandex",            "plandex",        "AI coding agent for large tasks")
("Goose",              "goose",          "Block's open-source coding agent")
("Mentat",             "mentat",         "AI coding assistant")
("Shell-AI (shai)",    "shai",           "AI shell command suggestions")
("Open Interpreter",   "interpreter",    "Natural language code execution")
("None — just open terminal", "__none__", "Drop into a shell in selected dir")

Store this list as a static/const array or lazy_static. Do not hardcode it inline in the UI.

---

LAUNCH BEHAVIOR

For every tool EXCEPT "Claude Code" and "__none__":
  - Suspend the TUI (use ratatui's terminal.show_cursor() + terminal.clear())
  - Run the tool's launch_command in the selected directory using std::process::Command
    - Set the working directory to the path selected in the file browser
    - Inherit stdin/stdout/stderr
    - Wait for it to exit
  - After the process exits, resume the TUI and return to the file browser

For "__none__":
  - Detect the OS:
      Windows → spawn: cmd /K "cd /d <path>"
      macOS   → spawn: open -a Terminal <path>   (fallback: osascript)
      Linux   → try: $TERM_PROGRAM, $TERMINAL env vars in order; fallback to xterm
  - This opens a new terminal window at the path (platform-aware)
  - Do NOT block; spawn and detach

For "Claude Code":
  - Do NOT launch yet
  - Transition to the Claude Code Provider screen (Step 3)
  - Pass the selected directory path forward

---

STATE CHANGES

Add to App state:
  screen: Screen   (enum: Browser, CliLauncher { path: PathBuf }, ClaudeProvider { path: PathBuf })
  cli_selected_index: usize
  cli_favorites: Vec<String>
  cli_recents: Vec<String>

Navigation between screens is via the screen field. The render function matches on screen to decide what to draw.

---

RULES

- No comments
- No new crates unless strictly necessary
- All platform detection via std::env::consts::OS
- Add a `launch.rs` module for all process-spawning logic
- Update Cargo.toml only if a new crate is truly needed
- The app must still compile and run with `cargo run`
```

---

## Step 3 — Claude Code Provider Screen & ENV Management

```
You are continuing development of "Clumsy Cat" (cc), a ratatui TUI app in Rust.
Ratatui docs: https://ratatui.rs
Steps 1 and 2 built the file browser and CLI launcher. Now add the Claude Code provider screen.

This screen appears when the user selects "Claude Code" from the CLI launcher.

---

SCREEN STRUCTURE

Full screen takeover. Same top bar and bottom bar.

LEFT PANEL — " Navigation "
  - A simple list of saved provider profiles (name only)
  - Pressing Enter on a profile loads it into the right panel
  - [N] to create a new profile, [Del] to delete selected

RIGHT PANEL — " Provider Setup "
  - Shows the currently selected/active provider configuration
  - Provider selector at the top: a horizontal tab row
    Active providers to support now:
      [ GitHub Copilot ]   [ OpenRouter ]   [ LM Studio ]   [ Ollama ]   [ Custom ]
    Future providers (show as dimmed, non-selectable):
      Cerebras · Groq · Together AI · Fireworks · Mistral
  - Below the tab row: a form for the selected provider

FORM FIELDS per provider:

  GitHub Copilot:
    - No API key needed
    - Info line: "Uses your GitHub Copilot subscription via MCP bridge"
    - [Launch] button only

  OpenRouter:
    - Field: OPENROUTER_API_KEY  (masked input, toggle [V] to reveal)
    - Field: Model  (editable text, default: "anthropic/claude-opus-4")
    - ANTHROPIC_BASE_URL will be set to: https://openrouter.ai/api/v1
    - ANTHROPIC_API_KEY will be set to: the value of OPENROUTER_API_KEY
    - [Save Profile] [Launch]

  LM Studio:
    - Field: LM_STUDIO_BASE_URL  (default: http://localhost:1234/v1)
    - Field: Model  (editable text)
    - ANTHROPIC_BASE_URL = LM_STUDIO_BASE_URL
    - ANTHROPIC_API_KEY  = "lm-studio" (literal placeholder)
    - [Save Profile] [Launch]

  Ollama:
    - Field: OLLAMA_BASE_URL  (default: http://localhost:11434/v1)
    - Field: Model  (editable text, default: "llama3.1")
    - ANTHROPIC_BASE_URL = OLLAMA_BASE_URL
    - ANTHROPIC_API_KEY  = "ollama" (literal placeholder)
    - [Save Profile] [Launch]

  Custom:
    - Field: ANTHROPIC_BASE_URL  (free text)
    - Field: ANTHROPIC_API_KEY   (masked, toggle [V])
    - [Save Profile] [Launch]

BOTTOM BAR:
  [Tab] Cycle Provider   [Enter] Edit Field   [V] Reveal Key   [S] Save Profile   [Launch] Go   [A] Back

---

.ENV FILE MANAGEMENT

Location: <user's home directory>/.cc/.env

On startup of this screen, read the file if it exists and populate saved keys.

Key format in the file:
  CC_OPENROUTER_API_KEY=...
  CC_LM_STUDIO_BASE_URL=...
  CC_OLLAMA_BASE_URL=...
  CC_OPENROUTER_MODEL=...
  CC_LM_STUDIO_MODEL=...
  CC_OLLAMA_MODEL=...
  CC_CUSTOM_BASE_URL=...
  CC_CUSTOM_API_KEY=...

When [Save Profile] is pressed, write/update these keys in the file.
Never overwrite keys for other providers — do a targeted line-by-line update.
Use a simple line-based parser (no external dotenv crate for writing; dotenvy for reading is fine).

Profile names are stored in: ~/.cc/profiles.toml
Each profile maps a name to a provider name + a subset of keys.
Use the `toml` crate for serialization.

---

GITHUB COPILOT LAUNCH SEQUENCE

This is the most complex case. GitHub Copilot requires an MCP server to bridge to Claude Code.
The MCP server command is:
  npx -y @github/copilot-mcp@latest

Previously this needed two terminal tabs. Now do it in one process tree:

1. Spawn the MCP server as a background child process:
     let mut server = Command::new("npx")
         .args(["-y", "@github/copilot-mcp@latest"])
         .stdin(Stdio::null())
         .stdout(Stdio::null())
         .stderr(Stdio::null())
         .spawn()?;

2. Wait 2 seconds for it to initialize (std::thread::sleep).

3. Set environment variables for the Claude Code process:
     ANTHROPIC_BASE_URL not needed for Copilot — Copilot MCP uses a different flag
   Instead, launch Claude Code with the MCP config flag:
     claude --mcp-config <generated_config_path>
   
   Generate a minimal MCP JSON config file at ~/.cc/copilot_mcp.json:
   {
     "mcpServers": {
       "github-copilot": {
         "command": "npx",
         "args": ["-y", "@github/copilot-mcp@latest"]
       }
     }
   }
   
   Write this file once on first use (skip if already exists).

4. Suspend the TUI, then run:
     Command::new("claude")
         .arg("--mcp-config")
         .arg(config_path)
         .current_dir(selected_path)
         .stdin(Stdio::inherit())
         .stdout(Stdio::inherit())
         .stderr(Stdio::inherit())
         .spawn()?
         .wait()?;

5. After Claude Code exits, kill the MCP server child:
     server.kill().ok();
     server.wait().ok();

6. Resume TUI, return to CLI launcher screen.

---

OTHER PROVIDER LAUNCH SEQUENCE (OpenRouter, LM Studio, Ollama, Custom)

1. Set env vars on the child process (do NOT set them on the current process):
     Command::new("claude")
         .current_dir(selected_path)
         .env("ANTHROPIC_BASE_URL", resolved_base_url)
         .env("ANTHROPIC_API_KEY",  resolved_api_key)
         .stdin(Stdio::inherit())
         .stdout(Stdio::inherit())
         .stderr(Stdio::inherit())
         .spawn()?
         .wait()?;

2. Suspend TUI before spawn, resume after wait. Same pattern as Step 2.

---

STATE ADDITIONS

Add to Screen enum:
  ClaudeProvider { path: PathBuf }

Add to App:
  provider_selected: ProviderKind   (enum matching the tab list)
  provider_profiles: Vec<Profile>
  provider_form_state: FormState    (tracks which field is being edited, current values)
  env_store: EnvStore               (loaded from ~/.cc/.env on startup)

Add a `provider.rs` module for all provider logic, env file read/write, and profile management.
Add a `claude_launch.rs` module for the Copilot and standard launch sequences.

---

RULES

- No comments
- Newest stable crates: dotenvy (read), toml (profiles), everything else std
- Cross-platform paths: use dirs crate (dirs::home_dir()) for ~/.cc resolution
- Masked input: replace chars with '●' in the rendered Paragraph, keep raw value in state
- The app must compile and run with `cargo run`
- Test: OpenRouter key saves to .env, persists across restarts, is loaded back into the form
```

---

## Step 4 — Theming System, Polish & Cross-Platform Hardening

```
You are completing "Clumsy Cat" (cc), a ratatui TUI app in Rust.
Ratatui docs: https://ratatui.rs
Steps 1–3 built the core app. This step adds theming, UX polish, and cross-platform hardening.

---

THEMING SYSTEM

Extend the Theme struct (theme.rs) into a full named theme system.

Add a ThemeKind enum with these variants:
  Default       — cyan accents, dark background assumption
  Nord          — nord palette (blues and frost)
  Gruvbox       — warm oranges and greens
  Catppuccin    — soft mauve/lavender/peach
  Tokyonight    — purple/blue dark
  Solarized     — teal/yellow
  Dracula       — pink/purple

Each ThemeKind maps to a Theme instance with fully specified values for:
  border_normal, border_focused, highlight, text_normal, text_dim,
  title_style, tab_active, tab_inactive, error_style, success_style, key_hint_style

Store the active theme in App state: active_theme: ThemeKind
Persist it in ~/.cc/config.toml under [ui] theme = "nord" (or whichever).

Add a theme picker accessible from any screen via [T]:
  A centered floating popup (use a Clear widget + a Block overlay)
  Arrow keys to select theme, Enter to apply immediately (live preview), Esc to cancel.

---

SEARCH MODE (stub from Step 1, now implement)

When [/] is pressed in the file browser:
  - Show a search input bar above the bottom bar (a bordered Paragraph, 1 line)
  - As the user types, filter the entries list in real time (case-insensitive substring match on filename)
  - Esc or Enter exits search mode (Enter keeps the filtered list navigable; Esc restores full list)
  - Input handling: standard printable chars append, Backspace removes last char

---

FAVORITES MANAGEMENT (both file browser and CLI screen)

File browser favorites:
  [F] on a highlighted directory → toggle it in/out of favorites
  Stored in ~/.cc/config.toml under [[favorites.dirs]]
  Max 20 entries

CLI favorites:
  [F] on a highlighted CLI tool → toggle favorite
  Stored in ~/.cc/config.toml under [[favorites.cli]]
  Max 10 entries

On both left panels, highlight the focused item within Favorites/Recents sub-sections
  with the same highlight style used in the browser.

---

RESPONSIVE LAYOUT

Handle terminal resize events (Event::Resize):
  - Re-render immediately on resize
  - If terminal width < 60 or height < 20, show a single centered message:
      "Terminal too small — resize to continue"
    Do not crash or panic.

All layout Constraints must be relative (Percentage) or Min-based — never fixed pixel widths
  except for the top bar height (5) and bottom bar height (3).

---

ERROR HANDLING UX

Replace all .unwrap() and .expect() calls throughout the codebase with proper error handling:
  - File system errors (permission denied, not found): show inline in the browser panel as a
    styled error line using theme.error_style. Do not exit.
  - Launch failures (command not found): show a centered popup with the error message and
    "Press any key to dismiss".
  - Config read/write errors: log to ~/.cc/cc.log (append mode) and continue with defaults.
    Show a subtle dim notice in the bottom bar: "Config error — see ~/.cc/cc.log"

---

CROSS-PLATFORM HARDENING

Windows-specific:
  - Use ENABLE_VIRTUAL_TERMINAL_PROCESSING for crossterm on Windows (crossterm handles this,
    but ensure it's enabled in main.rs terminal setup)
  - Path separators: always use PathBuf, never string-concatenate paths
  - "None — just open terminal" on Windows must use:
      Command::new("cmd").args(["/K", &format!("cd /d \"{}\"", path.display())])
      and spawn a new conhost window via:
      Command::new("cmd").args(["/C", "start", "cmd", "/K", ...])

macOS-specific:
  - "None — just open terminal": try $TERM_PROGRAM first, then fallback chain:
      iTerm2 → Terminal.app → osascript open
  - claude binary may be at /usr/local/bin/claude or via npm global; resolve via `which claude`

Linux-specific:
  - Terminal detection order: $TERM_PROGRAM → $TERMINAL → x-terminal-emulator → xterm → kitty → alacritty
  - Spawn detached from current process group for "none" launches

General:
  - All child processes for tool launches: set .current_dir(selected_path) without exception
  - claude and all CLI tools: check if binary exists with `which`/`where` before launching;
    if not found, show the error popup instead of crashing
  - Env var setting for providers: use .env() on Command, never std::env::set_var

---

CONFIG FILE SCHEMA (~/.cc/config.toml)

[ui]
theme = "default"
show_hidden = false

[[favorites.dirs]]
path = "/home/user/projects"

[[favorites.cli]]
name = "Claude Code"

[recents]
dirs = ["/home/user/projects/foo", ...]
cli  = ["Claude Code", ...]

Implement config load on startup and save on exit (or on each relevant mutation).
Use the `toml` crate for both read and write.
Use `serde` + `serde_derive` for the config struct.

---

FINAL CHECKLIST (agent must verify before finishing)

- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo build --release` succeeds on the current platform
- [ ] Theme switching live-previews and persists across restarts
- [ ] Search filters entries in real time
- [ ] Favorites toggle correctly in both browser and CLI screens
- [ ] Terminal resize does not panic; small terminal shows the resize notice
- [ ] All .unwrap() removed from non-test code
- [ ] ~/.cc/ directory is created automatically if it does not exist
- [ ] OpenRouter key round-trips: save → restart → key is pre-filled in form
- [ ] GitHub Copilot MCP server is killed cleanly after Claude Code exits
- [ ] "None" terminal open works on Windows (cmd), macOS, and Linux

---

RULES

- No comments
- No new crates beyond: ratatui, crossterm, dotenvy, toml, serde, serde_derive, dirs
  (and any already added in Steps 1–3)
- The final binary must be a single `cargo build --release` artifact
- Update README.md with: install instructions, keybind reference, theme list, provider setup guide
```
