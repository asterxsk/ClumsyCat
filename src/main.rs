mod app;
mod claude_config;
mod config;
mod fs;
mod search;
mod terminal;
mod theme;
mod tools;
mod ui;

use app::App;
use clap::Parser;
use ratatui::crossterm::{execute, terminal as crossterm_terminal, cursor};
use std::io::{stdout, Write};

#[derive(Parser)]
#[command(name = "cc", version, about = "terminal ui launcher for ai coding tools")]
struct Cli {
    #[arg(short = 'V', long)]
    version: bool,

    #[arg(long)]
    default: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.version {
        println!("cc {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut terminal = ratatui::init();

    let mut app = App::new(cli.default);
    let result = app.run(&mut terminal);

    // Clean up app resources (proxy, etc.) before terminal cleanup
    app.cleanup();

    // Manual terminal cleanup (don't use ratatui::restore() - do it manually)
    drop(terminal); // Drop terminal first to trigger its cleanup

    // Now explicitly restore terminal state
    let mut stdout = stdout();
    let _ = crossterm_terminal::disable_raw_mode();
    let _ = execute!(
        stdout,
        crossterm_terminal::LeaveAlternateScreen,
        cursor::Show,
    );

    // Full reset sequence
    let _ = stdout.flush();
    print!("\x1B[0m");     // Reset all attributes
    print!("\x1B[?25h");   // Show cursor
    let _ = stdout.flush();

    // Clear terminal and show ASCII art on exit
    print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top-left

    // Show ASCII art
    let ascii_art = std::fs::read_to_string("ascii.md")
        .unwrap_or_else(|_| "CLUMSY CAT".to_string());
    println!("{}", ascii_art);

    // Show version number under ASCII art
    println!("                           Version {}\n", env!("CARGO_PKG_VERSION"));

    let _ = stdout.flush();

    result
}
