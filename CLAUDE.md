# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
cargo build              # debug build
cargo build --release    # release build (binary at target/release/cc)
cargo run                # run in debug mode
cargo test               # run all tests
cargo test --lib         # unit tests only
cargo test <test_name>   # run single test
cargo clippy -- -D warnings  # lint check (must pass with zero warnings)
```

Binary names are `cc`, `claudecat`, and `csc` (all defined in Cargo.toml as bin entries).

## Architecture Overview

ClaudeCat is a terminal UI launcher for AI coding tools, built with ratatui/crossterm. It follows a 4-page workflow with a unified layout.

### State Machine

```
Page::Browser → Page::ToolSelection → Page::Provider → Page::Model → Launch
     ↑              ↑ (Esc)              ↑ (Esc)          ↑ (Esc)
     └──────────────┴────────────────────┴────────────────┘
```

- **Browser**: Directory navigation with favorites/recents
- **ToolSelection**: Choose AI tool (Claude Code, Codex, Kilocode, etc.)
- **Provider**: Choose provider (only for Claude Code) - GitHub Copilot, OpenRouter, NVIDIA NIM, LM Studio
- **Model**: Choose model, then launch

Only Claude Code advances through Provider/Model pages; other tools launch directly from ToolSelection.

### Module Responsibilities

| Module | Purpose |
|--------|---------|
| `app.rs` | State machine, input handling, keybind remapping |
| `ui.rs` | All ratatui rendering (layout, panels, dialogs) |
| `tools.rs` | Tool registry, binary detection, process launching with signal forwarding |
| `config.rs` | Settings persistence to `~/.config/clumsycat/config.json` |
| `search.rs` | Search mode state, substring/fuzzy filtering |
| `theme.rs` | Accent colors (orange, red, purple, blue, etc.) |
| `fs.rs` | Directory listing (dirs first, then files, alphabetically) |

### Key Patterns

**Dialog System** (`app.rs`): Modal overlays handled via `Dialog` enum. Dialogs capture input with highest priority. Settings overlay is a separate state (`settings_open: bool`).

**Dual Navigation Mode**: WASD (default) or Vim keybinds (k/j/h/l). Keybind remapping applied in input handler before processing.

**Search Mode**: Two sub-modes - typing mode (characters append to query) and navigation mode (w/s to traverse filtered results). Press `/` to enter, Esc to exit or toggle between sub-modes.

**Tool Launch** (`tools.rs`): Suspends TUI, spawns process with `setsid()` for process group isolation (Unix), forwards signals (SIGINT/SIGTERM/SIGQUIT), waits up to 1 hour, then restores TUI.

### Layout Structure

All pages share: ASCII art box (top-left), navigation panel (bottom-left with favorites/recents sections), main content panel (right 75%), bottom keybind bar.

### Config Location

`~/.config/claudecat/config.json` - stores settings, favorites (by category), and recents (capped at 10 per category).

## Code Style

- No comments in source code
- All UI text is lowercase
- Keybind hints format: `[key] action` separated by `●`
- Borders: rounded style with accent color on focused panel
- Error handling: inline display in UI, no `.unwrap()` in non-test code
