use crate::app::{ActivePanel, App, Dialog, LeftSection};
use crate::search::SearchMode;
use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

const ASCII_ART: &str = "CLUMSY CAT";

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

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_top_bar(frame, main_chunks[0], &theme);

    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ])
        .split(main_chunks[1]);

    render_left_panel(frame, middle_chunks[0], app, &theme);
    render_right_panel(frame, middle_chunks[1], app, &theme);
    render_bottom_bar(frame, main_chunks[2], app, &theme);

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

fn render_top_bar(frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_normal));
    
    let inner = block.inner(area);
    frame.render_widget(&block, area);
    
    let text = Paragraph::new(ASCII_ART)
        .style(Style::default().fg(theme.text_normal))
        .centered();
    frame.render_widget(text, inner);
}

fn render_left_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_color = if app.active_panel == ActivePanel::Left {
        theme.border_focused
    } else {
        theme.border_normal
    };
    
    let block = Block::default()
        .title(" Navigation ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    
    let inner = block.inner(area);
    frame.render_widget(&block, area);
    
    let mut items: Vec<ListItem> = Vec::new();
    
    let fav_color = if app.left_section == LeftSection::Favorites {
        theme.highlight
    } else {
        theme.text_normal
    };
    items.push(ListItem::new(
        Line::from(vec![
            Span::raw("★ "),
            Span::raw("Favorites").style(Style::default().fg(fav_color)),
        ])
    ));
    
    if app.favorites_dirs.is_empty() {
        items.push(ListItem::new(
            Line::from(vec![
                Span::raw("  (none)").style(Style::default().fg(theme.text_dim)),
            ])
        ));
    } else {
        for fav in &app.favorites_dirs {
            let name = fav.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| fav.to_string_lossy().to_string());
            let color = if app.left_section == LeftSection::Favorites && app.selected_index < app.favorites_dirs.len() && &app.favorites_dirs[app.selected_index] == fav {
                theme.highlight
            } else {
                theme.text_dim
            };
            items.push(ListItem::new(
                Line::from(vec![
                    Span::raw("  "),
                    Span::raw(name).style(color),
                ])
            ));
        }
    }
    
    let sep_width = (inner.width as usize).saturating_sub(2);
    items.push(ListItem::new(Line::from("─".repeat(sep_width))));
    
    let rec_color = if app.left_section == LeftSection::Recents {
        theme.highlight
    } else {
        theme.text_normal
    };
    items.push(ListItem::new(
        Line::from(vec![
            Span::raw("◷ "),
            Span::raw("Recents").style(Style::default().fg(rec_color)),
        ])
    ));
    
    if app.recents_dirs.is_empty() {
        items.push(ListItem::new(
            Line::from(vec![
                Span::raw("  (none)").style(Style::default().fg(theme.text_dim)),
            ])
        ));
    } else {
        for recent in &app.recents_dirs {
            let name = recent.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| recent.to_string_lossy().to_string());
            let color = if app.left_section == LeftSection::Recents && app.selected_index < app.recents_dirs.len() && &app.recents_dirs[app.selected_index] == recent {
                theme.highlight
            } else {
                theme.text_dim
            };
            items.push(ListItem::new(
                Line::from(vec![
                    Span::raw("  "),
                    Span::raw(name).style(color),
                ])
            ));
        }
    }
    
    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_right_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
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
            .centered();
        frame.render_widget(error_msg, content_area);
        return;
    }

    if app.entries.is_empty() {
        let empty_msg = Paragraph::new("Empty directory")
            .style(theme.text_dim)
            .centered();
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
            let icon = if entry.is_dir { "▸" } else { "·" };
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

fn render_bottom_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_normal));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    let keybinds = if app.quit_confirm >= 1 {
        " [Ctrl+D×2] Confirm quit "
    } else if app.search_mode.is_active() {
        " [Type] Search   [W/S] Navigate matches   [Enter] Confirm   [Esc] Cancel   [Backspace] Delete char "
    } else if app.settings_open {
        " [W/S] Navigate   [D/Enter] Change   [Esc] Save & Close "
    } else {
        " [/] Search   [Space] Select   [Ctrl+F] Favorite   [Ctrl+S] Settings   [D] Open   [A] Back   [W/S] Up/Down   [Ctrl+D×2] Quit "
    };

    let text = Paragraph::new(keybinds)
        .style(Style::default().fg(theme.text_dim))
        .centered();

    frame.render_widget(text, inner);
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
