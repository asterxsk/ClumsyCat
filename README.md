# ClaudeCat (ClumsyCat) 🐱

<div align="center">

  ![ClaudeCat Banner](ascii_cat.md)

  ### The Ultimate AI Coding Tool Launcher
  *A beautiful terminal UI for launching AI coding assistants with style.*

  [![GitHub release (latest by date)](https://img.shields.io/github/v/release/asterxsk/ClumsyCat?style=for-the-badge&color=blue&label=v0.1.0)](https://github.com/asterxsk/ClumsyCat/releases)
  [![MIT License](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](https://github.com/asterxsk/ClumsyCat/blob/main/LICENSE)
  [![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
  
  [**Report Bug**](https://github.com/asterxsk/ClumsyCat/issues) • [**Request Feature**](https://github.com/asterxsk/ClumsyCat/issues)

</div>

---

## 🚀 Overview

**ClaudeCat** (also known as ClumsyCat) is a blazingly fast terminal UI launcher for AI coding tools. Built with Rust and ratatui, it provides a beautiful, intuitive interface for navigating your projects and launching your favorite AI coding assistants with the right configuration.

Whether you're using Claude Code, Codex, Kilocode, Gemini CLI, or OpenCode, ClaudeCat streamlines your workflow with:
- **Smart directory navigation** with favorites and recents
- **Multi-tool support** with automatic detection
- **Provider and model selection** for Claude Code
- **GitHub Copilot integration** with automatic proxy management
- **Beautiful terminal UI** with glassmorphism design and smooth animations

## ✨ Key Features

### 🎯 Smart Project Navigation
*   **Directory Browser:** Fast filesystem navigation with alphabetical sorting (directories first)
*   **Favorites System:** Bookmark your most-used projects by category
*   **Recent Projects:** Quick access to your 10 most recently launched projects per tool
*   **Search Mode:** Substring and fuzzy search to quickly find directories

### 🤖 Multi-Tool Support
*   **Claude Code:** Full support with provider/model selection workflow
*   **Codex:** Direct launch with current directory context
*   **Kilocode (CLI):** Instant project-aware launches
*   **Gemini (CLI):** Google's AI coding assistant integration
*   **OpenCode:** Community AI coding tool support

### ⚡ Claude Code Integration
*   **Provider Selection:** GitHub Copilot, OpenRouter, NVIDIA NIM, LM Studio
*   **Model Tiers:** Claude Max, Claude Pro, Claude Free with automatic environment setup
*   **Copilot Proxy:** Automatic `copilot-api` proxy detection and management
*   **Smart Defaults:** Pre-configured model mappings (Opus 4.5, Sonnet 4.5, Haiku 4.5)

### 🎨 Beautiful UI/UX
*   **Dual Navigation Modes:** WASD (default) or Vim keybinds (hjkl)
*   **Accent Themes:** Orange, red, purple, blue, green, and more
*   **Glassmorphism Design:** Modern, clean interface with rounded borders
*   **Responsive Layout:** ASCII art, navigation panel, content panel, keybind hints
*   **Modal Dialogs:** Settings overlay, help screens, and confirmation prompts

### 🛡️ Robust Process Management
*   **Signal Forwarding:** Proper SIGINT/SIGTERM/SIGQUIT handling (Unix)
*   **Process Groups:** Isolated process groups via `setsid()` for clean spawning
*   **Extended Timeout:** 1-hour timeout for long-running interactive sessions
*   **Terminal Restoration:** Seamless TUI suspend/resume with full state preservation

---

## 🛠️ Technology Stack

Built with modern Rust for maximum performance and reliability.

| Category | Technology |
|----------|------------|
| **Language** | [Rust](https://www.rust-lang.org/) |
| **TUI Framework** | [Ratatui](https://ratatui.rs/), [Crossterm](https://github.com/crossterm-rs/crossterm) |
| **Terminal** | [Portable PTY](https://github.com/wez/wezterm/tree/main/pty) |
| **Signal Handling** | [signal-hook](https://github.com/vorner/signal-hook) (Unix) |
| **Config Storage** | JSON (`~/.config/claudecat/config.json`) |
| **Architecture** | State machine with 4-page workflow |

---

## 📋 How It Works

ClaudeCat follows a simple 4-page state machine workflow:

```
┌─────────────┐      ┌──────────────────┐      ┌──────────┐      ┌───────┐
│   Browser   │ ───> │ Tool Selection   │ ───> │ Provider │ ───> │ Model │ ───> Launch!
│  (Pick Dir) │      │ (Pick AI Tool)   │      │  (Claude)│      │(Pick) │
└─────────────┘      └──────────────────┘      └──────────┘      └───────┘
       ↑                      ↑ (Esc)                ↑ (Esc)         ↑ (Esc)
       └──────────────────────┴─────────────────────┴────────────────┘
```

1. **Browser Page:** Navigate your filesystem, use favorites/recents, or search for a directory
2. **Tool Selection Page:** Choose from installed AI coding tools (auto-detected from PATH)
3. **Provider Page:** *(Claude Code only)* Select your AI provider
4. **Model Page:** *(Claude Code only)* Choose model tier and launch

Other tools (Codex, Kilocode, etc.) skip Provider/Model pages and launch directly from Tool Selection.

### Architecture Highlights

- **State Machine:** Clean page-based navigation with Esc to go back
- **Module Separation:** `app.rs` (state), `ui.rs` (rendering), `tools.rs` (launching), `config.rs` (persistence)
- **Search System:** Two sub-modes (typing mode + navigation mode) with substring filtering
- **Keybind Remapping:** WASD ↔ Vim keybinds applied before input processing
- **Config Persistence:** Settings, favorites by category, recents capped at 10 per category

---

## 🎮 Commands & Keybindings

### Navigation (Default: WASD Mode)

| Key | Action |
|-----|--------|
| `w` / `↑` | Move up in lists |
| `s` / `↓` | Move down in lists |
| `a` / `←` | Go back to previous page |
| `d` / `→` / `Enter` | Select item / advance to next page |
| `Tab` | Switch between panels (favorites ↔ recents ↔ main) |

### Navigation (Vim Mode)

| Key | Action |
|-----|--------|
| `k` / `↑` | Move up in lists |
| `j` / `↓` | Move down in lists |
| `h` / `←` | Go back to previous page |
| `l` / `→` / `Enter` | Select item / advance to next page |
| `Tab` | Switch between panels |

### Global Actions

| Key | Action |
|-----|--------|
| `/` | Enter search mode |
| `Esc` | Exit search / close dialogs / go back |
| `?` | Open help dialog |
| `,` | Open settings overlay |
| `f` | Add current directory to favorites |
| `F` | Remove directory from favorites |
| `q` / `Ctrl+C` | Quit application |

### Search Mode

| Key | Action |
|-----|--------|
| *Any character* | Append to search query (typing mode) |
| `Backspace` | Delete last character |
| `w` / `s` | Navigate filtered results (navigation mode) |
| `Enter` | Select filtered result |
| `Esc` | Exit search mode / toggle mode |

### Settings Overlay

- **Toggle Keybind Mode:** Switch between WASD and Vim
- **Change Theme:** Cycle through accent colors (orange, red, purple, blue, green, yellow, pink)
- **Auto-save:** All settings persist to `~/.config/claudecat/config.json`

---

## ⚡ Installation

> **Coming Soon**

### Prerequisites

- Rust toolchain (1.70 or higher)
- Git
- At least one AI coding tool installed (Claude Code, Codex, etc.)

### Windows

**Step 1 — Install Rust:**
```bash
# Download and run rustup-init.exe from https://rustup.rs/
# Or use winget:
winget install Rustlang.Rustup
```

**Step 2 — Clone and Build:**
```bash
git clone https://github.com/asterxsk/ClumsyCat.git
cd ClumsyCat
cargo build --release
```

**Step 3 — Add to PATH:**
```bash
# Copy binary to a directory in your PATH
copy target\release\cc.exe C:\Windows\System32\

# Or add the target\release directory to your PATH environment variable
```

**Step 4 — Run:**
```bash
cc
# Or use alternative names:
claudecat
csc
```

### Linux

**Step 1 — Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**Step 2 — Clone and Build:**
```bash
git clone https://github.com/asterxsk/ClumsyCat.git
cd ClumsyCat
cargo build --release
```

**Step 3 — Install Binary:**
```bash
# Option 1: Install to ~/.local/bin (recommended)
mkdir -p ~/.local/bin
cp target/release/cc ~/.local/bin/
cp target/release/claudecat ~/.local/bin/
cp target/release/csc ~/.local/bin/

# Option 2: Install system-wide
sudo cp target/release/cc /usr/local/bin/
sudo cp target/release/claudecat /usr/local/bin/
sudo cp target/release/csc /usr/local/bin/
```

**Step 4 — Run:**
```bash
cc
# Or use alternative names:
claudecat
csc
```

### macOS

**Feel free to add macOS support and send a pull request and I will review it. I don't have a Mac and therefore can't test it on a Mac.**

The codebase should work on macOS with minimal changes (Unix signal handling is already implemented). If you're on macOS:

1. Install Rust via `brew install rust` or rustup
2. Clone and build following the Linux instructions
3. Test the binary and submit a PR with any macOS-specific fixes needed

---

## 🔧 Configuration

ClaudeCat stores all configuration in `~/.config/claudecat/config.json` (Linux/macOS) or `%USERPROFILE%\.config\claudecat\config.json` (Windows).

### Config Structure

```json
{
  "keybind_mode": "WASD",
  "theme": "Orange",
  "favorites": {
    "projects": ["/home/user/project1", "/home/user/project2"],
    "work": ["/home/user/work/repo1"]
  },
  "recents": {
    "Claude Code": ["/home/user/recent1", "/home/user/recent2"],
    "Codex": ["/home/user/codex-project"]
  }
}
```

### Available Themes

- **Orange** (default)
- **Red**
- **Purple**
- **Blue**
- **Green**
- **Yellow**
- **Pink**

### Keybind Modes

- **WASD** (default): `w/a/s/d` for navigation
- **Vim**: `k/h/j/l` for navigation

All settings can be changed via the settings overlay (press `,` in the app).

---

## 🎯 Supported Tools

ClaudeCat automatically detects installed AI coding tools from your PATH:

| Tool | Binary Names | Provider/Model Selection | Auto-detected |
|------|-------------|-------------------------|---------------|
| **Claude Code** | `claude` | ✅ Yes (4-page flow) | ✅ |
| **Codex** | `codex` | ❌ Direct launch | ✅ |
| **Kilocode (CLI)** | `kilo`, `kilocode` | ❌ Direct launch | ✅ |
| **Gemini (CLI)** | `gemini` | ❌ Direct launch | ✅ |
| **OpenCode** | `opencode` | ❌ Direct launch | ✅ |

### Claude Code Providers

When launching Claude Code, you can select from:

- **GitHub Copilot** (requires `copilot-api` installed)
- **OpenRouter**
- **NVIDIA NIM**
- **LM Studio**

### Model Tiers (GitHub Copilot)

| Tier | Opus Model | Sonnet Model | Haiku Model |
|------|-----------|--------------|-------------|
| **Claude Max** | claude-opus-4.5 | claude-sonnet-4.5 | claude-haiku-4.5 |
| **Claude Pro** | claude-opus-4.5 | claude-sonnet-4.5 | gpt-5-mini |
| **Claude Free** | - | gpt-5-mini | - |

Environment variables are automatically set based on your selection.

---

## 🤝 Contributing

Contributions are welcome! ClaudeCat is open source and thrives on community input.

1.  **Fork** the repository
2.  Create a new **Branch** (`git checkout -b feature/AmazingFeature`)
3.  **Commit** your changes (`git commit -m 'Add some AmazingFeature'`)
4.  **Push** to the branch (`git push origin feature/AmazingFeature`)
5.  Open a **Pull Request**

### Development Setup

```bash
# Clone the repo
git clone https://github.com/asterxsk/ClumsyCat.git
cd ClumsyCat

# Run in debug mode
cargo run

# Run tests
cargo test

# Lint check (must pass with zero warnings)
cargo clippy -- -D warnings

# Build release binary
cargo build --release
```

### Code Style Guidelines

- No comments in source code (keep code self-documenting)
- All UI text must be lowercase
- Keybind hints format: `[key] action` separated by `●`
- Borders: rounded style with accent color on focused panel
- No `.unwrap()` in non-test code (proper error handling)

---

## 📜 License

Distributed under the **MIT License**. See `LICENSE` for more information.

---

## 🙏 Acknowledgments

- **[Ratatui](https://ratatui.rs/)** - Fantastic terminal UI framework
- **[Crossterm](https://github.com/crossterm-rs/crossterm)** - Cross-platform terminal manipulation
- **Claude Code, Codex, Kilocode, Gemini CLI, OpenCode** - The amazing AI coding tools this launcher supports

---

<div align="center">
  <p>Made with ❤️ by <a href="https://github.com/asterxsk">Asterxsk</a></p>
  <p><i>Star this repo if you find it useful!</i> ⭐</p>
</div>
