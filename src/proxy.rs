use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::terminal::ProxyTerminal;

/// Start copilot-api proxy in background for GitHub Copilot integration
pub fn start_copilot_proxy() -> Option<u32> {
    // ensure copilot-api binary exists before attempting to spawn
    if !crate::tools::is_binary_in_path("copilot-api") {
        return None;
    }

    #[cfg(unix)]
    {
        use std::process::Command;

        let mut cmd = Command::new("copilot-api");
        cmd.args(&["start", "--proxy-env"]);
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());

        use std::os::unix::process::CommandExt;
        match unsafe {
            cmd.pre_exec(|| {
                let _ = libc::setsid();
                Ok(())
            })
            .spawn()
        } {
            Ok(child) => Some(child.id()),
            Err(_) => None,
        }
    }

    #[cfg(not(unix))]
    {
        use std::process::Command;

        match Command::new("copilot-api")
            .args(["start", "--proxy-env"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(child) => Some(child.id()),
            Err(_) => None,
        }
    }
}

/// Kill the copilot-proxy process
pub fn stop_copilot_proxy(pid: u32) {
    #[cfg(unix)]
    {
        unsafe {
            let _ = libc::kill(pid as i32, libc::SIGTERM);
        }
        thread::sleep(Duration::from_millis(100));
        unsafe {
            let _ = libc::kill(pid as i32, libc::SIGKILL);
        }
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
    }
}

/// Spawn Claude proxy in an embedded PTY for background operation
pub fn spawn_proxy_terminal(size: (u16, u16)) -> Result<ProxyTerminal, Box<dyn std::error::Error>> {
    use portable_pty::{native_pty_system, CommandBuilder, PtySize};

    // ensure copilot-api binary exists before attempting to spawn
    if !crate::tools::is_binary_in_path("copilot-api") {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "copilot-api not found in PATH",
        )));
    }

    let pty_system = native_pty_system();

    let pair = pty_system.openpty(PtySize {
        rows: size.1,
        cols: size.0,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Spawn copilot-api proxy command
    let mut cmd = CommandBuilder::new("copilot-api");
    cmd.arg("start");
    cmd.arg("--proxy-env");
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
