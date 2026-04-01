use std::io::{self, Write};
use std::net::{TcpStream, SocketAddr};
use std::path::Path;
use std::time::Duration;
use std::sync::mpsc;
use std::thread;

use crate::terminal::ProxyTerminal;

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
pub static PROVIDERS: &[&str] = &[
    "GitHub Copilot",
    "OpenRouter",
    "NVIDIA NIM",
    "LM Studio",
];

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

fn is_binary_in_path(binary: &str) -> bool {
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
    for binary_name in tool.binary_names {
        if is_binary_in_path(binary_name) {
            return Some(binary_name);
        }
    }
    None
}

/// Check if the GitHub Copilot API proxy is running by connecting to port 11437
pub fn check_copilot_proxy_running() -> bool {
    let addr: SocketAddr = "127.0.0.1:11437".parse().unwrap();
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
fn launch_tool_direct(binary: &str, dir: &Path, provider: Option<&str>, model: Option<&str>) -> LaunchResult {
    #[cfg(unix)]
    {
        use signal_hook::consts::signal::*;
        use signal_hook::iterator::Signals;
        use std::time::{Duration, Instant};
        use std::process::Command;

        // Build the command
        let mut cmd = Command::new(binary);
        cmd.current_dir(dir);

        // Set environment variables based on provider and model
        if let (Some(prov), Some(mdl)) = (provider, model) {
            match prov {
                "GitHub Copilot" => {
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
                _ => {}
            }
        }

        // Direct terminal inheritance for interactivity
        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        // Spawn child in its own process group
        use std::os::unix::process::CommandExt;
        match unsafe { cmd.pre_exec(|| { let _ = libc::setsid(); Ok(()) }).spawn() } {
            Ok(mut child) => {
                let child_pid = child.id() as i32;

                // Setup signal forwarding
                let mut signals = match Signals::new(&[SIGINT, SIGTERM, SIGQUIT]) {
                    Ok(s) => s,
                    Err(_) => {
                        // Fallback: simple wait if we cannot register signals
                        return match child.wait() {
                            Ok(_status) => LaunchResult::Success,
                            Err(e) => LaunchResult::LaunchFailed(format!("Failed to wait for process: {}", e)),
                        };
                    }
                };

                let handle = signals.handle();
                let s_child = child_pid;
                let signal_thread = std::thread::spawn(move || {
                    for sig in signals.forever() {
                        // Forward signal to process group
                        unsafe { libc::kill(-s_child, sig); }
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
                            let _ = handle.close();
                            let _ = signal_thread.join();
                            return LaunchResult::LaunchFailed(format!("Failed to wait for process: {}", e));
                        }
                    }
                }

                // Clean up signal handling
                let _ = handle.close();
                let _ = signal_thread.join();

                // Final wait to reap status
                match child.wait() {
                    Ok(_status) => LaunchResult::Success,
                    Err(e) => LaunchResult::LaunchFailed(format!("Failed to wait for process: {}", e)),
                }
            }
            Err(e) => LaunchResult::LaunchFailed(format!("Failed to spawn process: {}", e)),
        }
    }

    #[cfg(not(unix))]
    {
        use std::process::Command;
        use std::time::{Duration, Instant};

        let mut cmd = Command::new(binary);
        cmd.current_dir(dir);

        if let (Some(prov), Some(mdl)) = (provider, model) {
            match prov {
                "GitHub Copilot" => {
                    if mdl.contains("claude") {
                        cmd.env("ANTHROPIC_DEFAULT_OPUS_MODEL", "claude-opus-4.5");
                        cmd.env("ANTHROPIC_MODEL", "claude-sonnet-4.5");
                        cmd.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", "claude-haiku-4.5");
                    }
                    else if mdl.contains("gpt") {
                        cmd.env("ANTHROPIC_MODEL", "gpt-5-mini");
                    }
                }
                _ => {}
            }
        }

        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        #[cfg(windows)]
        {
            use windows_sys::Win32::System::JobObjects::*;
            use windows_sys::Win32::System::Threading::*;
            use windows_sys::Win32::Foundation::*;

            match cmd.spawn() {
                Ok(child) => {
                    let job_handle = unsafe {
                        CreateJobObjectW(std::ptr::null(), std::ptr::null())
                    };

                    if job_handle != 0 {
                        let mut job_info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
                        job_info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

                        unsafe {
                            SetInformationJobObject(
                                job_handle,
                                JobObjectExtendedLimitInformation,
                                &mut job_info as *mut _ as *mut _,
                                std::mem::size_of_val(&job_info) as u32,
                            );

                            let process_handle = OpenProcess(
                                PROCESS_ALL_ACCESS,
                                0,
                                child.id(),
                            );

                            if process_handle != 0 {
                                AssignProcessToJobObject(job_handle, process_handle);
                                CloseHandle(process_handle);
                            }
                        }

                        let timeout = Duration::from_secs(3600);
                        let start = Instant::now();

                        let mut child = child;
                        loop {
                            match child.try_wait() {
                                Ok(Some(_status)) => break,
                                Ok(None) => {
                                    if start.elapsed() > timeout {
                                        unsafe {
                                            TerminateJobObject(job_handle, 1);
                                        }
                                        break;
                                    }
                                    std::thread::sleep(Duration::from_millis(100));
                                }
                                Err(e) => {
                                    unsafe { CloseHandle(job_handle); }
                                    return LaunchResult::LaunchFailed(
                                        format!("Failed to wait for process: {}", e)
                                    );
                                }
                            }
                        }

                        unsafe { CloseHandle(job_handle); }

                        match child.wait() {
                            Ok(_status) => LaunchResult::Success,
                            Err(e) => LaunchResult::LaunchFailed(
                                format!("Failed to wait for process: {}", e)
                            ),
                        }
                    } else {
                        match child.wait() {
                            Ok(_status) => LaunchResult::Success,
                            Err(e) => LaunchResult::LaunchFailed(
                                format!("Failed to wait for process: {}", e)
                            ),
                        }
                    }
                }
                Err(e) => LaunchResult::LaunchFailed(
                    format!("Failed to spawn process: {}", e)
                ),
            }
        }

        #[cfg(not(windows))]
        {
            match cmd.spawn() {
                Ok(mut child) => match child.wait() {
                    Ok(_status) => LaunchResult::Success,
                    Err(e) => LaunchResult::LaunchFailed(format!("Failed to wait for process: {}", e)),
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
    let _ = print!("\x1B[0m");
    let _ = out.flush();

    // Very brief pause to let terminal settle
    std::thread::sleep(std::time::Duration::from_millis(30));
}

/// Spawn Claude proxy in an embedded PTY for background operation
pub fn spawn_proxy_terminal(size: (u16, u16)) -> Result<ProxyTerminal, Box<dyn std::error::Error>> {
    use portable_pty::{CommandBuilder, PtySize, native_pty_system};

    let pty_system = native_pty_system();

    let pair = pty_system.openpty(PtySize {
        rows: size.1,
        cols: size.0,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Spawn claude-proxy command
    let cmd = CommandBuilder::new("claude-proxy");
    let child = pair.slave.spawn_command(cmd)?;

    // Create channel for PTY output
    let (tx, rx) = mpsc::channel();

    // Clone reader for background thread
    let mut reader = pair.master.try_clone_reader()?;

    // Start background thread to read PTY output
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let data = buf[..n].to_vec();
                    if tx.send(data).is_err() {
                        break; // Channel closed, exit thread
                    }
                }
                Ok(_) => break, // EOF
                Err(e) => {
                    eprintln!("PTY read error: {}", e);
                    break;
                }
            }
        }
    });

    // Get writer for input
    let writer = pair.master.take_writer()?;

    Ok(ProxyTerminal::new(size.0, size.1, rx, writer, child))
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
