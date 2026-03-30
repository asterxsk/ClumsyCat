use crate::app::{ActivePanel, App, Dialog, LeftSection, Page};
use crate::search::SearchMode;
use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{block::BorderType, Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

const SETTINGS_ASCII: &str = r#"
  ____       _   _   _
 / ___|  ___| |_| |_(_)_ __   __ _ ___
 \___ \ / _ \ __| __| | '_ \ / _` / __|
  ___) |  __/ |_| |_| | | | | (_| \__ \
 |____/ \___|\__|\__|_|_| |_|\__, |___/
                             |___/
"#;

pub fn render(app: &App, frame: &mut Frame) {
    let theme = Theme::with_accent(&app.settings.accent_color);
    let area = frame.area();

    // Main layout: left/right split with bottom bar (no top bar - ASCII moves to left column)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(main_chunks[0]);

    // Render left column (ASCII art + Navigation)
    render_left_column(frame, content_chunks[0], app, &theme);

    // Render right panel based on current page
    match app.page {
        Page::Browser => render_browser_panel(frame, content_chunks[1], app, &theme),
        Page::ToolSelection => render_tool_selection_panel(frame, content_chunks[1], app, &theme),
        Page::Provider => render_provider_selection_panel(frame, content_chunks[1], app, &theme),
        Page::Model => render_model_selection_panel(frame, content_chunks[1], app, &theme),
    }

    // Render bottom bar
    render_bottom_bar(frame, main_chunks[1], app, &theme);

    // Render settings overlay if open
    if app.settings_open {
        render_settings_overlay(frame, area, app, &theme);
    }

    // Render dialogs on top of everything
    match &app.dialog {
        Dialog::None => {}
        Dialog::AddToFavorites { path } => {
            render_add_favorite_dialog(frame, area, path, app.dialog_selection, &theme);
        }
        Dialog::SudoPassword { password_input, .. } => {
            render_sudo_password_dialog(frame, area, password_input, &theme);
        }
        Dialog::ToolNotInstalled { tool_name } => {
            render_tool_not_installed_dialog(frame, area, tool_name, &theme);
        }
        Dialog::Error { message } => {
            render_error_dialog(frame, area, message, &theme);
        }
    }
}

/// Render the left column containing ASCII art box and navigation box
fn render_left_column(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    // Calculate ASCII art height (lines + border)
    let ascii_lines = app.ascii_art.lines().count() as u16;
    let ascii_height = ascii_lines + 2; // +2 for borders

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(ascii_height), Constraint::Min(0)])
        .split(area);

    // Render ASCII art box at top
    render_ascii_box(frame, left_chunks[0], app, theme);

    // Render navigation box below based on current page
    match app.page {
        Page::Browser => render_browser_navigation(frame, left_chunks[1], app, theme),
        Page::ToolSelection => render_tool_navigation(frame, left_chunks[1], app, theme),
        Page::Provider => render_provider_navigation(frame, left_chunks[1], app, theme),
        Page::Model => render_model_navigation(frame, left_chunks[1], app, theme),
    }
}

/// Render the ASCII art box
fn render_ascii_box(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let accent_color = Theme::from_name(&app.settings.accent_color);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_normal));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = Paragraph::new(app.ascii_art.as_str())
        .style(Style::default().fg(accent_color))
        .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render browser page navigation (favorites/recents for directories)
fn render_browser_navigation(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_color = if app.active_panel == ActivePanel::Left {
        theme.border_focused
    } else {
        theme.border_normal
    };

    let block = Block::default()
        .title(" Navigation ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    let mut items: Vec<ListItem> = Vec::new();

    let fav_color = if app.left_section == LeftSection::Favorites {
        theme.highlight
    } else {
        theme.text_normal
    };
    items.push(ListItem::new(Line::from(vec![
        Span::raw("* "),
        Span::raw("Favorites").style(Style::default().fg(fav_color)),
    ])));

    if app.favorites_dirs.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim)),
        ])));
    } else {
        for fav in &app.favorites_dirs {
            let name = fav
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| fav.to_string_lossy().to_string());
            let color = if app.left_section == LeftSection::Favorites
                && app.selected_index < app.favorites_dirs.len()
                && &app.favorites_dirs[app.selected_index] == fav
            {
                theme.highlight
            } else {
                theme.text_dim
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::raw(name).style(color),
            ])));
        }
    }

    let sep_width = (inner.width as usize).saturating_sub(2);
    items.push(ListItem::new(Line::from("-".repeat(sep_width))));

    let rec_color = if app.left_section == LeftSection::Recents {
        theme.highlight
    } else {
        theme.text_normal
    };
    items.push(ListItem::new(Line::from(vec![
        Span::raw("~ "),
        Span::raw("Recents").style(Style::default().fg(rec_color)),
    ])));

    if app.recents_dirs.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim)),
        ])));
    } else {
        for recent in &app.recents_dirs {
            let name = recent
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| recent.to_string_lossy().to_string());
            let color = if app.left_section == LeftSection::Recents
                && app.selected_index < app.recents_dirs.len()
                && &app.recents_dirs[app.selected_index] == recent
            {
                theme.highlight
            } else {
                theme.text_dim
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::raw(name).style(color),
            ])));
        }
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render tool selection page navigation
fn render_tool_navigation(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_color = if app.active_panel == ActivePanel::Left {
        theme.border_focused
    } else {
        theme.border_normal
    };

    let block = Block::default()
        .title(" Navigation ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    let mut items: Vec<ListItem> = Vec::new();

    let fav_color = if app.tool_left_section == LeftSection::Favorites {
        theme.highlight
    } else {
        theme.text_normal
    };
    items.push(ListItem::new(Line::from(vec![
        Span::raw("* "),
        Span::raw("Favorites").style(Style::default().fg(fav_color)),
    ])));

    if app.favorites_tools.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim)),
        ])));
    } else {
        for tool in &app.favorites_tools {
            let color = theme.text_dim;
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::raw(tool).style(color),
            ])));
        }
    }

    let sep_width = (inner.width as usize).saturating_sub(2);
    items.push(ListItem::new(Line::from("-".repeat(sep_width))));

    let rec_color = if app.tool_left_section == LeftSection::Recents {
        theme.highlight
    } else {
        theme.text_normal
    };
    items.push(ListItem::new(Line::from(vec![
        Span::raw("~ "),
        Span::raw("Recents").style(Style::default().fg(rec_color)),
    ])));

    if app.recents_tools.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim)),
        ])));
    } else {
        for tool in &app.recents_tools {
            let color = theme.text_dim;
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::raw(tool).style(color),
            ])));
        }
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render provider selection page navigation
fn render_provider_navigation(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_color = if app.active_panel == ActivePanel::Left {
        theme.border_focused
    } else {
        theme.border_normal
    };

    let block = Block::default()
        .title(" Navigation ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    let mut items: Vec<ListItem> = Vec::new();

    let fav_color = if app.provider_left_section == LeftSection::Favorites {
        theme.highlight
    } else {
        theme.text_normal
    };
    items.push(ListItem::new(Line::from(vec![
        Span::raw("* "),
        Span::raw("Favorites").style(Style::default().fg(fav_color)),
    ])));

    if app.favorites_providers.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim)),
        ])));
    } else {
        for provider in &app.favorites_providers {
            let color = theme.text_dim;
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::raw(provider).style(color),
            ])));
        }
    }

    let sep_width = (inner.width as usize).saturating_sub(2);
    items.push(ListItem::new(Line::from("-".repeat(sep_width))));

    let rec_color = if app.provider_left_section == LeftSection::Recents {
        theme.highlight
    } else {
        theme.text_normal
    };
    items.push(ListItem::new(Line::from(vec![
        Span::raw("~ "),
        Span::raw("Recents").style(Style::default().fg(rec_color)),
    ])));

    if app.recents_providers.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim)),
        ])));
    } else {
        for provider in &app.recents_providers {
            let color = theme.text_dim;
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::raw(provider).style(color),
            ])));
        }
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render model selection page navigation
fn render_model_navigation(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_color = if app.active_panel == ActivePanel::Left {
        theme.border_focused
    } else {
        theme.border_normal
    };

    let block = Block::default()
        .title(" Navigation ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    let mut items: Vec<ListItem> = Vec::new();

    let fav_color = if app.model_left_section == LeftSection::Favorites {
        theme.highlight
    } else {
        theme.text_normal
    };
    items.push(ListItem::new(Line::from(vec![
        Span::raw("* "),
        Span::raw("Favorites").style(Style::default().fg(fav_color)),
    ])));

    if app.favorites_models.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim)),
        ])));
    } else {
        for model in &app.favorites_models {
            let color = theme.text_dim;
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::raw(model).style(color),
            ])));
        }
    }

    let sep_width = (inner.width as usize).saturating_sub(2);
    items.push(ListItem::new(Line::from("-".repeat(sep_width))));

    let rec_color = if app.model_left_section == LeftSection::Recents {
        theme.highlight
    } else {
        theme.text_normal
    };
    items.push(ListItem::new(Line::from(vec![
        Span::raw("~ "),
        Span::raw("Recents").style(Style::default().fg(rec_color)),
    ])));

    if app.recents_models.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim)),
        ])));
    } else {
        for model in &app.recents_models {
            let color = theme.text_dim;
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::raw(model).style(color),
            ])));
        }
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render the browser panel (file list)
fn render_browser_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_color = if app.active_panel == ActivePanel::Right {
        theme.border_focused
    } else {
        theme.border_normal
    };

    let title_bottom = app.current_dir.to_string_lossy().to_string();
    let block = Block::default()
        .title(" Browser ")
        .title_bottom(Span::raw(title_bottom))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    // Calculate content area (reserve space for search bar if active)
    let (search_area, content_area) = if app.search_mode.is_active() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, inner)
    };

    // Render search bar if active
    if let SearchMode::Active {
        query,
        filtered_indices,
        current_match_index,
    } = &app.search_mode
    {
        if let Some(search_rect) = search_area {
            let match_info = format!(
                " ({}/{})",
                if filtered_indices.is_empty() {
                    0
                } else {
                    current_match_index + 1
                },
                filtered_indices.len()
            );

            let search_line = Line::from(vec![
                Span::styled("/", Style::default().fg(theme.highlight)),
                Span::styled(query, Style::default().fg(theme.text_normal)),
                Span::styled("_", Style::default().fg(theme.highlight)), // Cursor
                Span::styled(match_info, Style::default().fg(theme.text_dim)),
            ]);

            let search_bar = Paragraph::new(search_line);
            frame.render_widget(search_bar, search_rect);
        }
    }

    if let Some(ref err) = app.error {
        let error_msg = Paragraph::new(format!("Error: {}", err))
            .style(Style::default().fg(ratatui::style::Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(error_msg, content_area);
        return;
    }

    if app.entries.is_empty() {
        let empty_msg = Paragraph::new("Empty directory")
            .style(theme.text_dim)
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, content_area);
        return;
    }

    // Determine which indices to show
    let indices_to_show: Vec<usize> = if let SearchMode::Active {
        filtered_indices, ..
    } = &app.search_mode
    {
        filtered_indices.clone()
    } else {
        (0..app.entries.len()).collect()
    };

    let items: Vec<ListItem> = indices_to_show
        .iter()
        .map(|&i| {
            let entry = &app.entries[i];
            let icon = if entry.is_dir { ">" } else { "." };
            let is_selected = i == app.selected_index && app.active_panel == ActivePanel::Right;
            let color = if is_selected {
                theme.highlight
            } else if entry.is_dir {
                theme.text_normal
            } else {
                theme.text_dim
            };

            ListItem::new(Line::from(vec![
                Span::raw(icon),
                Span::raw(" "),
                Span::raw(&entry.name).style(Style::default().fg(color)),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, content_area);
}

/// Render the tool selection panel
fn render_tool_selection_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_color = if app.active_panel == ActivePanel::Right {
        theme.border_focused
    } else {
        theme.border_normal
    };

    let block = Block::default()
        .title(" Select Tool ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    if app.tools.is_empty() {
        let empty_msg = Paragraph::new("No tools available")
            .style(theme.text_dim)
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .tools
        .iter()
        .enumerate()
        .map(|(i, tool)| {
            let is_selected =
                i == app.selected_tool_index && app.active_panel == ActivePanel::Right;
            let color = if is_selected {
                theme.highlight
            } else {
                theme.text_normal
            };

            ListItem::new(Line::from(vec![
                Span::raw("> "),
                Span::raw(tool).style(Style::default().fg(color)),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render the provider selection panel
fn render_provider_selection_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_color = if app.active_panel == ActivePanel::Right {
        theme.border_focused
    } else {
        theme.border_normal
    };

    let block = Block::default()
        .title(" Select Provider ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    if app.providers.is_empty() {
        let empty_msg = Paragraph::new("No providers available")
            .style(theme.text_dim)
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .providers
        .iter()
        .enumerate()
        .map(|(i, provider)| {
            let is_selected =
                i == app.selected_provider_index && app.active_panel == ActivePanel::Right;
            let color = if is_selected {
                theme.highlight
            } else {
                theme.text_normal
            };

            ListItem::new(Line::from(vec![
                Span::raw("> "),
                Span::raw(provider).style(Style::default().fg(color)),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render the model selection panel with loading state
fn render_model_selection_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_color = if app.active_panel == ActivePanel::Right {
        theme.border_focused
    } else {
        theme.border_normal
    };

    let block = Block::default()
        .title(" Select Model ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    // Handle loading state
    if app.models_loading {
        let loading_msg = Paragraph::new("Loading models...")
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Center);
        frame.render_widget(loading_msg, inner);
        return;
    }

    // Handle error state
    if let Some(ref err) = app.models_error {
        let error_msg = Paragraph::new(format!("Error: {}", err))
            .style(Style::default().fg(ratatui::style::Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(error_msg, inner);
        return;
    }

    if app.models.is_empty() {
        let empty_msg = Paragraph::new("No models available")
            .style(theme.text_dim)
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .models
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let is_selected =
                i == app.selected_model_index && app.active_panel == ActivePanel::Right;
            let color = if is_selected {
                theme.highlight
            } else {
                theme.text_normal
            };

            ListItem::new(Line::from(vec![
                Span::raw("> "),
                Span::raw(model).style(Style::default().fg(color)),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render the bottom bar with context-sensitive keybinds
fn render_bottom_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_normal));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    let keybinds = get_context_keybinds(app);

    let text = Paragraph::new(keybinds)
        .style(Style::default().fg(theme.text_dim))
        .alignment(Alignment::Center);

    frame.render_widget(text, inner);
}

/// Get context-sensitive keybind text based on current state (all lowercase, separated by bullet)
fn get_context_keybinds(app: &App) -> String {
    // Handle quit confirmation first
    if app.quit_confirm >= 1 {
        return "ctrl+d x2 confirm quit".to_string();
    }

    // Handle dialog state
    if app.dialog != Dialog::None {
        return match &app.dialog {
            Dialog::None => String::new(),
            Dialog::AddToFavorites { .. } => {
                "w/s navigate  \u{25CF}  enter confirm  \u{25CF}  esc cancel".to_string()
            }
            Dialog::SudoPassword { .. } => "enter submit  \u{25CF}  esc cancel".to_string(),
            Dialog::ToolNotInstalled { .. } => "enter dismiss".to_string(),
            Dialog::Error { .. } => "enter dismiss".to_string(),
        };
    }

    // Handle settings overlay
    if app.settings_open {
        return "w/s navigate  \u{25CF}  d/enter change  \u{25CF}  esc save & close".to_string();
    }

    // Handle search mode
    if app.search_mode.is_active() {
        return "type to search  \u{25CF}  w/s prev/next  \u{25CF}  enter confirm  \u{25CF}  esc cancel".to_string();
    }

    // Page-specific keybinds (all lowercase, separated by bullet)
    match app.page {
        Page::Browser => {
            "/ search  \u{25CF}  space select  \u{25CF}  ctrl+f favorite  \u{25CF}  ctrl+s settings  \u{25CF}  d open  \u{25CF}  a back  \u{25CF}  w/s up/down  \u{25CF}  ctrl+d x2 quit".to_string()
        }
        Page::ToolSelection => {
            "w/s navigate  \u{25CF}  space select  \u{25CF}  a back  \u{25CF}  tab cycle panel  \u{25CF}  ctrl+d x2 quit".to_string()
        }
        Page::Provider => {
            "w/s navigate  \u{25CF}  space select  \u{25CF}  a back  \u{25CF}  tab cycle panel  \u{25CF}  ctrl+d x2 quit".to_string()
        }
        Page::Model => {
            if app.models_loading {
                "loading...  \u{25CF}  a back  \u{25CF}  ctrl+d x2 quit".to_string()
            } else {
                "w/s navigate  \u{25CF}  space select  \u{25CF}  a back  \u{25CF}  tab cycle panel  \u{25CF}  ctrl+d x2 quit".to_string()
            }
        }
    }
}

/// Helper to create a centered dialog area
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Render a dimmed background for dialogs
fn render_dialog_background(frame: &mut Frame, area: Rect) {
    let dim_block = Block::default()
        .style(Style::default().bg(ratatui::style::Color::Black));
    frame.render_widget(Clear, area);
    frame.render_widget(dim_block, area);
}

/// Render the Add to Favorites dialog
fn render_add_favorite_dialog(
    frame: &mut Frame,
    area: Rect,
    path: &std::path::PathBuf,
    selection: usize,
    theme: &Theme,
) {
    render_dialog_background(frame, area);

    let dialog_area = centered_rect(50, 10, area);

    let block = Block::default()
        .title(" Add to Favorites ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight));

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(&block, dialog_area);

    let inner = block.inner(dialog_area);

    let path_display = path.to_string_lossy();
    let parent_display = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());

    let option1_style = if selection == 0 {
        Style::default()
            .fg(theme.highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_dim)
    };

    let option2_style = if selection == 1 {
        Style::default()
            .fg(theme.highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_dim)
    };

    let radio1 = if selection == 0 { "(o)" } else { "( )" };
    let radio2 = if selection == 1 { "(o)" } else { "( )" };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("Select path to add:", Style::default().fg(theme.text_normal))),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", radio1), option1_style),
            Span::styled(truncate_path(&path_display, 40), option1_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", radio2), option2_style),
            Span::styled(truncate_path(&parent_display, 40), option2_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[W/S] Navigate  [Enter] Confirm  [Esc] Cancel",
            Style::default().fg(theme.text_dim),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render the Sudo Password dialog
fn render_sudo_password_dialog(frame: &mut Frame, area: Rect, password: &str, theme: &Theme) {
    render_dialog_background(frame, area);

    let dialog_area = centered_rect(50, 9, area);

    let block = Block::default()
        .title(" Authentication Required ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight));

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(&block, dialog_area);

    let inner = block.inner(dialog_area);

    // Show dots for password
    let dots = "*".repeat(password.len());

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Enter sudo password:",
            Style::default().fg(theme.text_normal),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("[{}]", if dots.is_empty() { " " } else { &dots }),
            Style::default().fg(theme.highlight),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Submit  [Esc] Cancel",
            Style::default().fg(theme.text_dim),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render the Tool Not Installed dialog
fn render_tool_not_installed_dialog(
    frame: &mut Frame,
    area: Rect,
    tool_name: &str,
    theme: &Theme,
) {
    render_dialog_background(frame, area);

    let dialog_area = centered_rect(50, 9, area);

    let block = Block::default()
        .title(" Tool Not Installed ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ratatui::style::Color::Red));

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(&block, dialog_area);

    let inner = block.inner(dialog_area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("'{}' is not installed.", tool_name),
            Style::default().fg(theme.text_normal),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Please install it to continue.",
            Style::default().fg(theme.text_dim),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter/Esc] OK",
            Style::default().fg(theme.text_dim),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render a generic error dialog
fn render_error_dialog(frame: &mut Frame, area: Rect, message: &str, theme: &Theme) {
    render_dialog_background(frame, area);

    let dialog_area = centered_rect(60, 10, area);

    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ratatui::style::Color::Red));

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(&block, dialog_area);

    let inner = block.inner(dialog_area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(theme.text_normal))),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter/Esc] OK",
            Style::default().fg(theme.text_dim),
        )),
    ];

    let text = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(text, inner);
}

/// Render the settings overlay
fn render_settings_overlay(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    render_dialog_background(frame, area);

    let dialog_area = centered_rect(50, 18, area);

    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight));

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(&block, dialog_area);

    let inner = block.inner(dialog_area);

    let accent_style = if app.settings_selection == 0 {
        Style::default()
            .fg(theme.highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_normal)
    };

    let nav_style = if app.settings_selection == 1 {
        Style::default()
            .fg(theme.highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_normal)
    };

    // Build accent color display with arrows
    let accent_display = format!(
        "< {} >",
        app.settings.accent_color.to_uppercase()
    );

    // Build nav mode display with arrows
    let nav_display = format!("< {} >", app.settings.nav_mode.to_uppercase());

    let lines = vec![
        Line::from(Span::styled(SETTINGS_ASCII, Style::default().fg(theme.text_dim))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Accent Color:  ", accent_style),
            Span::styled(accent_display, accent_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:    ", nav_style),
            Span::styled(nav_display, nav_style),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "[W/S] Navigate  [D/Enter] Change  [Esc] Save & Close",
            Style::default().fg(theme.text_dim),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Truncate a path string for display
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - max_len + 3..])
    }
}
