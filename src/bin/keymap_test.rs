use claude_cat::app::App; // crate name may be claude-cat; adjust as needed
use std::env;
use ratatui::crossterm::event::KeyCode;

fn main() {
    // Create app with non-default mode
    let mut app = App::new(false);

    let tests = vec![
        KeyCode::Char('w'),
        KeyCode::Char('s'),
        KeyCode::Char('a'),
        KeyCode::Char('d'),
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Enter,
    ];

    println!("settings.nav_mode={} keybinds: up={} down={} left={} right={}",
        app.settings.nav_mode, app.settings.keybinds.up, app.settings.keybinds.down, app.settings.keybinds.left, app.settings.keybinds.right);

    for k in tests {
        let mapped = app.map_vim_key(k.clone());
        // Determine which handler the mapped key would match in normal mode
        let handler = match mapped {
            KeyCode::Char('p') | KeyCode::Char('P') => "proxy_ctrl",
            KeyCode::Enter => "enter",
            KeyCode::Char('d') | KeyCode::Char('D') => "ctrl_d_or_d",
            KeyCode::Char('f') | KeyCode::Char('F') => "open_fav",
            KeyCode::Char('s') | KeyCode::Char('S') => "ctrl_s_or_s",
            KeyCode::Char('r') | KeyCode::Char('R') => "hot_r",
            KeyCode::Tab => "tab",
            KeyCode::Esc => "esc",
            KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => "up",
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => "down",
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => "open/right",
            KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => "back/left",
            KeyCode::Char('/') => "search",
            _ => "other",
        };
        println!("raw={:?} mapped={:?} -> {}", k, mapped, handler);
    }
}
