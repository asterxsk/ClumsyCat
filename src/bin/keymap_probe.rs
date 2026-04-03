use crate::config::Config;
use ratatui::crossterm::event::KeyCode;

fn map_vim_key(code: KeyCode, keybinds: &crate::config::Keybinds) -> KeyCode {
    use ratatui::crossterm::event::KeyCode;
    match code {
        KeyCode::Char(c) if c.to_string() == keybinds.up => KeyCode::Char('w'),
        KeyCode::Char(c) if c.to_string() == keybinds.down => KeyCode::Char('s'),
        KeyCode::Char(c) if c.to_string() == keybinds.left => KeyCode::Char('a'),
        KeyCode::Char(c) if c.to_string() == keybinds.right => KeyCode::Char('d'),
        other => other,
    }
}

fn main() {
    let cfg = Config::load();
    let keybinds = cfg.settings.keybinds;
    println!("Loaded keybinds: up={} down={} left={} right={}", keybinds.up, keybinds.down, keybinds.left, keybinds.right);

    let tests = vec![
        KeyCode::Char('w'),
        KeyCode::Char('s'),
        KeyCode::Char('a'),
        KeyCode::Char('d'),
        KeyCode::Char('k'),
        KeyCode::Char('j'),
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Enter,
    ];

    for k in tests {
        let mapped = map_vim_key(k.clone(), &keybinds);
        let handler = match mapped {
            KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => "up",
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => "down",
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => "open/right",
            KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => "back/left",
            KeyCode::Enter => "enter",
            _ => "other",
        };
        println!("raw={:?} mapped={:?} -> {}", k, mapped, handler);
    }
}
