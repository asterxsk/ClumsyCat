# ClaudeCat (ClumsyCat)

A terminal UI launcher for AI coding tools — navigate projects and launch AI assistants with a modern, keyboard-first interface.

<!-- Demo

Insert gif or link to demo

-->

<!-- Documentation

[Documentation](https://linktodocumentation)

-->

<!-- ## Demo

Insert a short animated GIF or link demonstrating directory navigation and launching a tool.

-->

## License

Distributed under the Apache License 2.0. See [LICENSE](./LICENSE) for details.

<!-- ## Documentation

Project docs and extended usage will be added here. (Currently commented out above.)

-->

## Installation

Prerequisites

- Rust toolchain (rustup)
- Git

Build and run

```bash
git clone https://github.com/asterxsk/ClumsyCat.git
cd ClumsyCat
# debug build
cargo build
# run in development
cargo run
# release build
cargo build --release
```

## Usage / Examples

Run the binary using one of the provided names after building:

```bash
# during development
cargo run
# or run the built binary
target/debug/cc
# installed release binary
target/release/cc
```

Common binary names

- cc
- claudecat
- csc

Keyboard-first controls

- navigation: `w` / `up` = up, `s` / `down` = down, `a` / `left` = back, `d` / `right` = open (default)
- vim mode: `k` / `up`, `j` / `down`, `h` / `left`, `l` / `right` (enable via settings)
- `enter` — select / open
- `/` — enter search mode; `Esc` — exit search, close dialogs, or go back
- `Tab` — switch panels (favorites ↔ recents ↔ main)
- `f` — focus Favorites (left panel); `r` — focus Recents (left panel)
- `Ctrl+F` — add current directory to favorites (dialog)
- `Shift+F` — open command bar
- `Ctrl+S` — open settings
- `Ctrl+D` — double-press (twice quickly) to confirm quit

## Features

- fast filesystem browser with favorites and recents
- multi-tool detection and launches (claude code, codex, kilocode, gemini cli, opencode)
- claude code provider and model selection flow (provider → model → launch)
- configurable keybind modes (wasd or vim)
- themes and accessible terminal UI built with ratatui + crossterm
- robust process management with signal forwarding and terminal restoration

## Configuration

Per-user config is stored in JSON:

- Linux/macOS: `~/.config/claudecat/config.json`
- Windows: `%USERPROFILE%\.config\claudecat\config.json`

Example config structure

```json
{
  "keybind_mode": "WASD",
  "theme": "Orange",
  "favorites": { "projects": [] },
  "recents": { "Claude Code": [] }
}
```

## Development

Run tests and lint checks:

```bash
# run tests
cargo test
# lint (must pass zero warnings)
cargo clippy -- -D warnings
```

## Roadmap

- improve macOS support and testing
- extensible provider integrations
- richer theme and accessibility options

## Contributing

Contributions welcome — fork, create a branch, and open a pull request. Please follow project code style and run tests before submitting.

## Authors

- asterxsk (@asterxsk)

## Acknowledgements

- ratatui, crossterm, and the open-source ecosystem that made this project possible
