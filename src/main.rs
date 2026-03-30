mod app;
mod config;
mod fs;
mod search;
mod theme;
mod ui;

use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();

    let mut app = App::new();
    let result = app.run(&mut terminal);

    ratatui::restore();
    result
}
