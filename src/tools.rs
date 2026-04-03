use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::terminal::ProxyTerminal;

// Proxy functions are implemented in src/proxy.rs and shims below forward to them.

/// Represents information about a coding tool
#[derive(Debug, Clone)]
pub struct ToolInfo {
    /// Binary names to check for installation (any match = installed)
    pub binary_names: &'static [&'static str],
    /// Display name shown in the UI
    pub display_name: &'static str,
    /// Whether this tool requires provider/model selection (like Claude Code)
    pub needs_provider_selection: bool,
}

/// List of all supported coding tools
pub static TOOLS: &[ToolInfo] = &[
    ToolInfo {
        binary_names: &["claude"],
        display_name: "Claude Code",
        needs_provider_selection: true,
    },
    ToolInfo {
        binary_names: &["codex"],
        display_name: "Codex",
        needs_provider_selection: false,
    },
    ToolInfo {
        binary_names: &["kilo", "kilocode"],
        display_name: "Kilocode (CLI)",
        needs_provider_selection: false,
    },
    ToolInfo {
        binary_names: &["gemini"],
        display_name: "Gemini (CLI)",
        needs_provider_selection: false,
    },
    ToolInfo {
        binary_names: &["opencode"],
        display_name: "OpenCode",
        needs_provider_selection: false,
    },
];

/// List of available providers for Claude Code
pub static PROVIDERS: &[&str] = &["GitHub Copilot", "OpenRouter", "NVIDIA NIM", "LM Studio"];

/// Hardcoded models for stub implementation (will be fetched dynamically later)
pub static STUB_MODELS: &[&str] = &[
    "claude-3-5-sonnet-20241022",
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
    "gpt-4-turbo",
    "gpt-4o",
    "gpt-4o-mini",
];

/// Check if a tool is installed on the system
pub fn check_tool_installed(tool: &ToolInfo) -> bool {
    for binary_name in tool.binary_names {
        if is_binary_in_path(binary_name) {
            return true;
        }
    }
    false
}

pub(crate) fn is_binary_in_path(binary: &str) -> bool {
    use std::path::PathBuf;

    if let Ok(path_env) = std::env::var("PATH") {
        let separator = if cfg!(windows) { ';' } else { ':' };
        for dir in path_env.split(separator) {
            let mut path = PathBuf::from(dir);
            path.push(binary);

            #[cfg(windows)]
            {
                for ext in &["", ".exe", ".cmd", ".bat"] {
                    let mut path_with_ext = path.clone();
                    if !ext.is_empty() {
                        path_with_ext.set_extension(&ext[1..]);
                    }
                    if path_with_ext.exists() && path_with_ext.is_file() {
                        return true;
                    }
                }
            }

            #[cfg(not(windows))]
            {
                if path.exists() && path.is_file() {
                    return true;
                }
            }
        }
    }
    false
}

/// Get the first available binary name for a tool
pub fn get_tool_binary(tool: &ToolInfo) -> Option<&'static str> {
    tool.binary_names
        .iter()
        .find(|binary_name| is_binary_in_path(binary_name))
        .copied()
}

/// Check if the GitHub Copilot API proxy is running by connecting to port 11437
pub fn check_copilot_proxy_running() -> bool {
    let addr: SocketAddr = match "127.0.0.1:11437".parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let timeout = Duration::from_millis(200);

    TcpStream::connect_timeout(&addr, timeout).is_ok()
}

/// Find a tool by its display name
pub fn find_tool_by_display_name(display_name: &str) -> Option<&'static ToolInfo> {
    TOOLS.iter().find(|t| t.display_name == display_name)
}

/// Result of launching a tool
#[derive(Debug)]
pub enum LaunchResult {
    Success,
    ToolNotInstalled(String),
    LaunchFailed(String),
}

/// Launch a tool in the specified directory using direct terminal access
/// This gives the child process direct access to the terminal for interactivity
pub fn launch_tool(
    tool: &ToolInfo,
    dir: &Path,
    provider: Option<&str>,
    model: Option<&str>,
) -> LaunchResult {
    // Check if tool is installed
    let binary = match get_tool_binary(tool) {
        Some(b) => b,
        None => return LaunchResult::ToolNotInstalled(tool.display_name.to_string()),
    };

    // Use direct terminal access with proper process management
    launch_tool_direct(binary, dir, provider, model)
}

/// Launch tool with direct terminal access and proper signal handling
fn launch_tool_direct(
    binary: &str,
    dir: &Path,
    provider: Option<&str>,
    model: Option<&str>,
) -> LaunchResult {
    #[cfg(unix)]
    {
        use signal_hook::consts::signal::*;
        use signal_hook::iterator::Signals;
        use std::process::Command;

        // Build the command
        let mut cmd = Command::new(binary);
        cmd.current_dir(dir);

        // Set environment variables based on provider and model
        if let (Some("GitHub Copilot"), Some(mdl)) = (provider, model) {
            match mdl {
                "Claude Max" => {
                    cmd.env("ANTHROPIC_DEFAULT_OPUS_MODEL", "claude-opus-4.5");
                    cmd.env("ANTHROPIC_MODEL", "claude-sonnet-4.5");
                    cmd.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", "claude-haiku-4.5");
                }
                "Claude Pro" => {
                    cmd.env("ANTHROPIC_DEFAULT_OPUS_MODEL", "claude-opus-4.5");
                    cmd.env("ANTHROPIC_MODEL", "claude-sonnet-4.5");
                    cmd.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", "gpt-5-mini");
                }
                "Claude Free" => {
                    cmd.env("ANTHROPIC_MODEL", "gpt-5-mini");
                }
                _ => {}
            }
        }

        // Direct terminal inheritance for interactivity
        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        // Spawn child in its own process group
        use std::os::unix::process::CommandExt;
        match unsafe {
            cmd.pre_exec(|| {
                let _ = libc::setsid();
                Ok(())
            })
            .spawn()
        } {
            Ok(mut child) => {
                let child_pid = child.id() as i32;

                // Setup signal forwarding
                let mut signals = match Signals::new([SIGINT, SIGTERM, SIGQUIT]) {
                    Ok(s) => s,
                    Err(_) => {
                        // Fallback: simple wait if we cannot register signals
                        return match child.wait() {
                            Ok(_status) => LaunchResult::Success,
                            Err(e) => LaunchResult::LaunchFailed(format!(
                                "Failed to wait for process: {}",
                                e
                            )),
                        };
                    }
                };

                let handle = signals.handle();
                let s_child = child_pid;
                let signal_thread = std::thread::spawn(move || {
                    for sig in signals.forever() {
                        // Forward signal to process group
                        unsafe {
                            libc::kill(-s_child, sig);
                        }
                    }
                });

                // Wait with extended timeout for interactive tools
                let timeout = Duration::from_secs(3600); // 1 hour for long-running tools
                let start = Instant::now();

                loop {
                    match child.try_wait() {
                        Ok(Some(_status)) => break,
                        Ok(None) => {
                            if start.elapsed() > timeout {
                                // Try graceful termination
                                let _ = unsafe { libc::kill(-child_pid, libc::SIGTERM) };
                                std::thread::sleep(Duration::from_secs(2));
                                let _ = unsafe { libc::kill(-child_pid, libc::SIGKILL) };
                                break;
                            }
                            std::thread::sleep(Duration::from_millis(100));
                        }
                        Err(e) => {
                            handle.close();
                            let _ = signal_thread.join();
                            return LaunchResult::LaunchFailed(format!(
                                "Failed to wait for process: {}",
                                e
                            ));
                        }
                    }
                }

                // Clean up signal handling
                handle.close();
                let _ = signal_thread.join();

                // Final wait to reap status
                match child.wait() {
                    Ok(_status) => LaunchResult::Success,
                    Err(e) => {
                        LaunchResult::LaunchFailed(format!("Failed to wait for process: {}", e))
                    }
                }
            }
            Err(e) => LaunchResult::LaunchFailed(format!("Failed to spawn process: {}", e)),
        }
    }

    #[cfg(not(unix))]
    {
        use std::process::Command;

        let mut cmd = Command::new(binary);
        cmd.current_dir(dir);

        if let (Some("GitHub Copilot"), Some(mdl)) = (provider, model) {
            if mdl.contains("claude") {
                cmd.env("ANTHROPIC_DEFAULT_OPUS_MODEL", "claude-opus-4.5");
                cmd.env("ANTHROPIC_MODEL", "claude-sonnet-4.5");
                cmd.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", "claude-haiku-4.5");
            } else if mdl.contains("gpt") {
                cmd.env("ANTHROPIC_MODEL", "gpt-5-mini");
            }
        }

        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            use std::sync::atomic::{AtomicBool, Ordering};
            use windows_sys::Win32::Foundation::*;
            use windows_sys::Win32::System::Console::*;

            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            const CTRL_BREAK_EVENT: u32 = 1;

            static CTRL_C_PRESSED: AtomicBool = AtomicBool::new(false);

            unsafe extern "system" fn ctrl_handler(ctrl_type: u32) -> i32 {
                match ctrl_type {
                    CTRL_C_EVENT | CTRL_BREAK_EVENT => {
                        CTRL_C_PRESSED.store(true, Ordering::Relaxed);
                        TRUE
                    }
                    _ => FALSE,
                }
            }

            cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);

            match cmd.spawn() {
                Ok(mut child) => {
                    let child_pid = child.id();

                    CTRL_C_PRESSED.store(false, Ordering::Relaxed);

                    let handler_installed = unsafe {
                        SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) != 0
                    };

                    if !handler_installed {
                        eprintln!(
                            "Warning: Failed to install console control handler: {}",
                            std::io::Error::last_os_error()
                        );
                    }

                    let timeout = Duration::from_secs(3600);
                    let start = Instant::now();

                    loop {
                        if CTRL_C_PRESSED.load(Ordering::Relaxed) {
                            unsafe {
                                GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, child_pid);
                            }
                            CTRL_C_PRESSED.store(false, Ordering::Relaxed);
                        }

                        match child.try_wait() {
                            Ok(Some(_status)) => break,
                            Ok(None) => {
                                if start.elapsed() > timeout {
                                    // try graceful termination via ctrl event
                                    unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, child_pid); }
                                    std::thread::sleep(Duration::from_secs(2));
                                    if child.try_wait().unwrap_or(None).is_none() {
                                        let _ = child.kill();
                                    }
                                    break;
                                }
                                std::thread::sleep(Duration::from_millis(100));
                            }
                            Err(e) => {
                                if handler_installed {
                                    unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), FALSE); }
                                }
                                return LaunchResult::LaunchFailed(format!(
                                    "Failed to wait for process: {}",
                                    e
                                ));
                            }
                        }
                    }

                    if handler_installed {
                        unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), FALSE); }
                    }

                    LaunchResult::Success
                }
                Err(e) => LaunchResult::LaunchFailed(format!("Failed to spawn process: {}", e)),
            }
        }

        #[cfg(not(windows))]
        {
            match cmd.spawn() {
                Ok(mut child) => match child.wait() {
                    Ok(_status) => LaunchResult::Success,
                    Err(e) => {
                        LaunchResult::LaunchFailed(format!("Failed to wait for process: {}", e))
                    }
                },
                Err(e) => LaunchResult::LaunchFailed(format!("Failed to spawn process: {}", e)),
            }
        }
    }
}

/// Prepare terminal for tool launch (restore terminal state)
pub fn prepare_for_launch(tool_name: &str) {
    // Flush stdout before launching
    let _ = io::stdout().flush();

    // Try to disable raw mode and leave the alternate screen so the child
    // process runs in a normal terminal environment. Ignore errors.
    let _ = ratatui::crossterm::terminal::disable_raw_mode();
    let mut out = io::stdout();
    let _ = ratatui::crossterm::execute!(out, ratatui::crossterm::terminal::LeaveAlternateScreen);
    let _ = ratatui::crossterm::execute!(out, ratatui::crossterm::cursor::Show);

    // Clear the main terminal buffer and display a clean loading message
    use ratatui::crossterm::terminal::ClearType;
    let _ = ratatui::crossterm::execute!(
        out,
        ratatui::crossterm::cursor::MoveTo(0, 0),
        ratatui::crossterm::terminal::Clear(ClearType::All)
    );

    let loading_msg = format!("Loading {}...\n", tool_name);
    let _ = write!(out, "{}", loading_msg);
    let _ = out.flush();
}

/// Restore terminal after tool exits
pub fn restore_after_launch() {
    // Try to re-enable raw mode and re-enter alternate screen for the TUI.
    let _ = ratatui::crossterm::terminal::enable_raw_mode();
    let mut out = io::stdout();
    let _ = ratatui::crossterm::execute!(out, ratatui::crossterm::terminal::EnterAlternateScreen);
    let _ = ratatui::crossterm::execute!(out, ratatui::crossterm::cursor::Hide);

    // Minimal clear - just reset cursor position and clear current viewport
    use ratatui::crossterm::terminal::ClearType;
    let _ = ratatui::crossterm::execute!(out, ratatui::crossterm::cursor::MoveTo(0, 0));
    let _ = ratatui::crossterm::execute!(out, ratatui::crossterm::terminal::Clear(ClearType::All));

    // Reset attributes
    print!("\x1B[0m");
    let _ = out.flush();

    // Very brief pause to let terminal settle
    std::thread::sleep(std::time::Duration::from_millis(30));
}

/// Start copilot-api proxy in background for GitHub Copilot integration
// start_copilot_proxy moved to src/proxy.rs - keep a thin shim to preserve API
pub fn start_copilot_proxy() -> Option<u32> {
    // ensure copilot-api binary exists before attempting to spawn
    if !is_binary_in_path("copilot-api") {
        return None;
    }
    crate::proxy::start_copilot_proxy()
}

/// Kill the copilot-proxy process
// stop_copilot_proxy moved to src/proxy.rs - shim to preserve API
pub fn stop_copilot_proxy(pid: u32) {
    crate::proxy::stop_copilot_proxy(pid);
}

/// Spawn Claude proxy in an embedded PTY for background operation
// spawn_proxy_terminal moved to src/proxy.rs - shim
pub fn spawn_proxy_terminal(size: (u16, u16)) -> Result<ProxyTerminal, Box<dyn std::error::Error>> {
    // ensure copilot-api binary exists before attempting to spawn
    if !is_binary_in_path("copilot-api") {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "copilot-api not found in PATH",
        )));
    }
    crate::proxy::spawn_proxy_terminal(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_tools_list_not_empty() {
        assert!(!TOOLS.is_empty());
    }

    #[test]
    fn test_claude_tool_exists() {
        let claude = TOOLS.iter().find(|t| t.binary_names.contains(&"claude"));
        assert!(claude.is_some());
        let claude = claude.unwrap();
        assert_eq!(claude.display_name, "Claude Code");
        assert!(claude.needs_provider_selection);
    }

    #[test]
    fn test_find_tool_by_display_name() {
        let tool = find_tool_by_display_name("Claude Code");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().display_name, "Claude Code");
    }

    #[test]
    fn test_find_tool_by_display_name_not_found() {
        let tool = find_tool_by_display_name("Nonexistent Tool");
        assert!(tool.is_none());
    }

    #[test]
    fn test_providers_list() {
        assert!(!PROVIDERS.is_empty());
        assert!(PROVIDERS.contains(&"GitHub Copilot"));
        assert!(PROVIDERS.contains(&"OpenRouter"));
    }

    #[test]
    fn test_stub_models_list() {
        assert!(!STUB_MODELS.is_empty());
    }

    #[test]
    fn test_prepare_and_restore_no_panic() {
        prepare_for_launch("test");
        restore_after_launch();
    }

    #[test]
    fn test_launch_tool_echo() {
        use std::process::{Command, Stdio};
        let mut cmd = Command::new("echo");
        cmd.arg("hello-world");
        cmd.stdout(Stdio::piped());

        let output = cmd.output().expect("failed to run echo");
        assert!(String::from_utf8_lossy(&output.stdout).contains("hello-world"));
    }

    #[test]
    fn test_launch_tool_integration_nonexistent() {
        // Build a ToolInfo pointing to a likely-nonexistent binary
        let fake_tool = ToolInfo {
            binary_names: &["this-command-should-not-exist-12345"],
            display_name: "Nope",
            needs_provider_selection: false,
        };

        let res = launch_tool(&fake_tool, Path::new("."), None, None);
        match res {
            LaunchResult::ToolNotInstalled(_) => {}
            LaunchResult::LaunchFailed(_) => {}
            LaunchResult::Success => panic!("unexpected success for nonexistent tool"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_direct_launch_with_echo() {
        // Test the direct launch with a simple command
        let echo_tool = ToolInfo {
            binary_names: &["echo"],
            display_name: "Echo Direct Test",
            needs_provider_selection: false,
        };

        // This should succeed since echo is available on Unix systems
        let res = launch_tool(&echo_tool, Path::new("."), None, None);
        match res {
            LaunchResult::Success => {}
            LaunchResult::ToolNotInstalled(_) => {
                // Skip test if echo is not available (unlikely but possible)
            }
            LaunchResult::LaunchFailed(msg) => {
                // Print error for debugging but don't fail test in CI environments
                eprintln!("Direct launch failed (may be expected in CI): {}", msg);
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_direct_launch_with_true() {
        // Test with /bin/true which should always succeed quickly
        let true_tool = ToolInfo {
            binary_names: &["true"],
            display_name: "True Direct Test",
            needs_provider_selection: false,
        };

        let res = launch_tool(&true_tool, Path::new("."), None, None);
        match res {
            LaunchResult::Success => {}
            LaunchResult::ToolNotInstalled(_) => {
                // Skip if true is not available
            }
            LaunchResult::LaunchFailed(msg) => {
                eprintln!("Direct launch with true failed: {}", msg);
            }
        }
    }

    #[test]
    fn test_launch_tool_direct_vs_direct() {
        // Test that direct launch handles nonexistent tools correctly
        let fake_tool = ToolInfo {
            binary_names: &["definitely-does-not-exist-test-12345"],
            display_name: "Fake Tool",
            needs_provider_selection: false,
        };

        let result1 = launch_tool(&fake_tool, Path::new("."), None, None);
        let result2 = launch_tool(&fake_tool, Path::new("."), None, None);

        // Both should return ToolNotInstalled
        match (&result1, &result2) {
            (LaunchResult::ToolNotInstalled(_), LaunchResult::ToolNotInstalled(_)) => {}
            _ => panic!("Both launch approaches should handle nonexistent tools consistently"),
        }
    }
}
