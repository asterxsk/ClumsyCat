use crate::app::{ActivePanel, App, Dialog, LeftSection, Page, COMMANDS};
use crate::search::SearchMode;
use crate::theme::Theme;
use crate::tools::PROVIDERS;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    block::BorderType, Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};
use ratatui::Frame;

pub fn render(app: &mut App, frame: &mut Frame) {
    let theme = if app.settings.accent_color == "custom" {
        Theme::with_custom_hex(&app.settings.custom_color_hex)
    } else {
        Theme::with_accent(&app.settings.accent_color)
    };
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

    // Render global config overlay if open
    if app.global_config_open {
        render_global_config_overlay(app, frame);
    }

    // Render dialogs on top of everything
    match &app.dialog {
        Dialog::None => {}
        Dialog::AddToFavorites { path } => {
            render_add_favorite_dialog(frame, area, path, app.dialog_selection, &theme);
        }
        Dialog::ToolNotInstalled { tool_name } => {
            render_tool_not_installed_dialog(frame, area, tool_name, &theme);
        }
        Dialog::Error { message } => {
            render_error_dialog(frame, area, message, &theme);
        }
        Dialog::CustomColorInput { hex_input } => {
            render_custom_color_dialog(frame, area, hex_input, &theme);
        }
        Dialog::Opening { tool_name } => {
            render_opening_dialog(frame, area, tool_name, &theme);
        }
        Dialog::CommandBar {
            query,
            filtered_indices,
            selected_index,
        } => {
            render_command_bar(
                frame,
                area,
                query,
                filtered_indices,
                *selected_index,
                &theme,
            );
        }
        Dialog::ProviderConfig { selected_index } => {
            render_provider_config(frame, area, *selected_index, &theme);
        }
        Dialog::KeybindConfig {
            selected_index,
            editing_field,
        } => {
            render_keybind_config(frame, area, app, *selected_index, *editing_field, &theme);
        }
        Dialog::EnvConfig {
            entries,
            selected_index,
            editing_field,
            input_buffer,
        } => {
            render_env_config(
                frame,
                area,
                entries,
                *selected_index,
                *editing_field,
                input_buffer,
                &theme,
            );
        }
        Dialog::SettingsConfig { selected_index } => {
            render_settings_config(frame, area, app, *selected_index, &theme);
        }
    }
}

/// Render the left column containing ASCII art box and navigation box
fn render_left_column(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    // Calculate ASCII art height (lines + border)
    // Calculate ASCII art height (lines + border). If it's too tall for the area, fall back to cat-only ASCII
    let ascii_lines = app.ascii_art.lines().count() as u16;
    let max_allowed = area.height.saturating_sub(6); // leave space for navigation and bottom bar
    let ascii_height = if ascii_lines + 2 > max_allowed {
        // Try loading small cat-only ASCII
        let small =
            std::fs::read_to_string("ascii_cat.md").unwrap_or_else(|_| app.ascii_art.clone());
        let small_lines = small.lines().count() as u16;
        if small_lines + 2 > max_allowed {
            // Still too big, clamp to max_allowed
            max_allowed
        } else {
            // Replace app.ascii_art for rendering only
            // We'll temporarily render the small art by creating a new App-like string
            // (cheap and safe) — set ascii_lines to small_lines
            small_lines + 2
        }
    } else {
        ascii_lines + 2
    };

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
fn render_ascii_box(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    let accent_color = Theme::from_name(&app.settings.accent_color);

    let version_label = format!(" ClumsyCat v{} ", env!("CARGO_PKG_VERSION"));
    let mut block = Block::default()
        .title(version_label)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_normal));

    if app.proxy_terminal.is_some() {
        block = block.title_top(Line::from(" [●] proxy ").right_aligned());
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Try to load the small cat ASCII art first, then fallback to full art, then fallback to text
    let ascii_to_render = std::fs::read_to_string("ascii_cat.md")
        .or_else(|_| std::fs::read_to_string("ascii.md"))
        .unwrap_or_else(|_| {
            if app.ascii_art.trim().is_empty() {
                "   CLUMSY\n     CAT".to_string()
            } else {
                app.ascii_art.clone()
            }
        });

    if let Some(ref mut terminal) = app.proxy_terminal {
        if terminal.visible {
            // Render terminal instead of ASCII
            let border_color = if terminal.focused {
                theme.highlight
            } else {
                theme.border_normal
            };

            let terminal_block = Block::default()
                .title(" [terminal] enter:focus space:close ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color));

            let terminal_inner = terminal_block.inner(area);
            frame.render_widget(terminal_block, area);

            // Resize terminal buffer if needed
            terminal.resize(terminal_inner.width, terminal_inner.height);

            // Render terminal content
            frame.render_widget(&terminal.buffer, terminal_inner);
            return;
        }
    }

    if app.copilot_proxy_active {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(inner);

        let text = Paragraph::new(ascii_to_render.as_str())
            .style(Style::default().fg(accent_color))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(text, chunks[0]);

        let status_text = vec![
            Line::from(Span::styled(
                "Copilot proxy:",
                Style::default().fg(accent_color),
            )),
            Line::from(Span::styled("  Active", Style::default().fg(accent_color))),
            Line::from(""),
            Line::from(Span::styled(
                "[c] toggle",
                Style::default().fg(theme.text_dim),
            )),
        ];
        let status = Paragraph::new(status_text).alignment(Alignment::Left);
        frame.render_widget(status, chunks[1]);
    } else {
        let text = Paragraph::new(ascii_to_render.as_str())
            .style(Style::default().fg(accent_color))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(text, inner);
    }
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
        .title_top(Line::from(" [F]av [R]ec ").right_aligned())
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
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim))
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
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim))
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
        .title_top(Line::from(" [F]av [R]ec ").right_aligned())
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
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim))
        ])));
    } else {
        for (idx, tool) in app.favorites_tools.iter().enumerate() {
            let color = if app.tool_left_section == LeftSection::Favorites
                && app.selected_tool_index == idx
            {
                theme.highlight
            } else {
                theme.text_dim
            };
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
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim))
        ])));
    } else {
        for (idx, tool) in app.recents_tools.iter().enumerate() {
            let color = if app.tool_left_section == LeftSection::Recents
                && app.selected_tool_index == idx
            {
                theme.highlight
            } else {
                theme.text_dim
            };
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
        .title_top(Line::from(" [F]av [R]ec ").right_aligned())
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
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim))
        ])));
    } else {
        for (idx, provider) in app.favorites_providers.iter().enumerate() {
            let color = if app.provider_left_section == LeftSection::Favorites
                && app.selected_provider_index == idx
            {
                theme.highlight
            } else {
                theme.text_dim
            };
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
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim))
        ])));
    } else {
        for (idx, provider) in app.recents_providers.iter().enumerate() {
            let color = if app.provider_left_section == LeftSection::Recents
                && app.selected_provider_index == idx
            {
                theme.highlight
            } else {
                theme.text_dim
            };
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
        .title_top(Line::from(" [F]av [R]ec ").right_aligned())
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
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim))
        ])));
    } else {
        for (idx, model) in app.favorites_models.iter().enumerate() {
            let color = if app.model_left_section == LeftSection::Favorites
                && app.selected_model_index == idx
            {
                theme.highlight
            } else {
                theme.text_dim
            };
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
            Span::raw("  (none)").style(Style::default().fg(theme.text_dim))
        ])));
    } else {
        for (idx, model) in app.recents_models.iter().enumerate() {
            let color = if app.model_left_section == LeftSection::Recents
                && app.selected_model_index == idx
            {
                theme.highlight
            } else {
                theme.text_dim
            };
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
        .title_top(Line::from(" [B]rowser ").right_aligned())
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

    // Find the position of selected_index in the filtered list for scrolling
    let selected_position = indices_to_show
        .iter()
        .position(|&i| i == app.selected_index)
        .unwrap_or(0);

    let mut list_state = ListState::default();
    list_state.select(Some(selected_position));

    let list = List::new(items);
    frame.render_stateful_widget(list, content_area, &mut list_state);
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
        .title_top(Line::from(" [T]ools ").right_aligned())
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
        .title_top(Line::from(" [P]rofile ").right_aligned())
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

    // Determine title based on provider
    let title = if let Some(ref provider) = app.selected_provider {
        if provider == "GitHub Copilot" {
            " Select Profile "
        } else {
            " Select Model "
        }
    } else {
        " Select Model "
    };

    let block = Block::default()
        .title(title)
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

    let is_github_profiles = app.selected_provider.as_deref() == Some("GitHub Copilot");

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

            if is_github_profiles {
                let description = match model.as_str() {
                    "Claude Max" => "highest token usage, best result",
                    "Claude Pro" => "lower token usage, good results",
                    "Claude Free" => "no token usage, usable results",
                    _ => "",
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw("> "),
                        Span::raw(model).style(Style::default().fg(color)),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::raw(description).style(Style::default().fg(theme.text_dim)),
                    ]),
                ])
            } else {
                ListItem::new(Line::from(vec![
                    Span::raw("> "),
                    Span::raw(model).style(Style::default().fg(color)),
                ]))
            }
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
    // Handle quit confirmation first (but context-aware)
    if app.quit_confirm >= 1 {
        if app.settings_open {
            return "ctrl+d again to quit".to_string();
        }
        return "ctrl+d x2 confirm quit".to_string();
    }

    // Handle dialog state
    if app.dialog != Dialog::None {
        return match &app.dialog {
            Dialog::None => String::new(),
            Dialog::AddToFavorites { .. } => {
                "w/s navigate  \u{25CF}  enter confirm  \u{25CF}  esc cancel".to_string()
            }
            Dialog::ToolNotInstalled { .. } => "enter dismiss".to_string(),
            Dialog::Error { .. } => "enter dismiss".to_string(),
            Dialog::CustomColorInput { .. } => "type hex code  \u{25CF}  enter confirm  \u{25CF}  esc cancel".to_string(),
            Dialog::Opening { .. } => "opening...".to_string(),
            Dialog::CommandBar { .. } => "type to search  \u{25CF}  tab autocomplete  \u{25CF}  enter select  \u{25CF}  esc close".to_string(),
            Dialog::ProviderConfig { .. } => "w/s navigate  \u{25CF}  enter start/stop  \u{25CF}  esc close".to_string(),
            Dialog::KeybindConfig { .. } => "w/s navigate  \u{25CF}  enter edit  \u{25CF}  esc close".to_string(),
            Dialog::EnvConfig { .. } => "w/s navigate  \u{25CF}  tab switch field  \u{25CF}  enter edit  \u{25CF}  esc close".to_string(),
            Dialog::SettingsConfig { .. } => "w/s navigate  \u{25CF}  esc close".to_string(),
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

    // Handle focused proxy terminal
    if let Some(ref terminal) = app.proxy_terminal {
        if terminal.focused {
            return "space unfocus  \u{25CF}  type input to proxy  \u{25CF}  ctrl+c/d to proxy"
                .to_string();
        } else if terminal.visible {
            return "enter focus  \u{25CF}  ctrl+p stop  \u{25CF}  normal navigation".to_string();
        }
    }

    // Page-specific keybinds (all lowercase, separated by bullet)
    match app.page {
        Page::Browser => {
            if app.proxy_terminal.is_some() {
                "/ search  \u{25CF}  enter select  \u{25CF}  ctrl+f favorite  \u{25CF}  ctrl+s settings  \u{25CF}  d open  \u{25CF}  a back  \u{25CF}  w/s up/down  \u{25CF}  ctrl+p proxy  \u{25CF}  ctrl+d x2 quit".to_string()
            } else {
                "/ search  \u{25CF}  enter select  \u{25CF}  ctrl+f favorite  \u{25CF}  ctrl+s settings  \u{25CF}  d open  \u{25CF}  a back  \u{25CF}  w/s up/down  \u{25CF}  ctrl+p proxy  \u{25CF}  ctrl+d x2 quit".to_string()
            }
        }
        Page::ToolSelection => {
            "w/s navigate  \u{25CF}  enter select  \u{25CF}  a back  \u{25CF}  tab cycle panel  \u{25CF}  ctrl+p proxy  \u{25CF}  ctrl+d x2 quit".to_string()
        }
        Page::Provider => {
            "w/s navigate  \u{25CF}  enter select  \u{25CF}  a back  \u{25CF}  tab cycle panel  \u{25CF}  ctrl+p proxy  \u{25CF}  ctrl+d x2 quit".to_string()
        }
        Page::Model => {
            if app.models_loading {
                "loading...  \u{25CF}  a back  \u{25CF}  ctrl+p proxy  \u{25CF}  ctrl+d x2 quit".to_string()
            } else {
                "w/s navigate  \u{25CF}  enter select  \u{25CF}  a back  \u{25CF}  tab cycle panel  \u{25CF}  ctrl+p proxy  \u{25CF}  ctrl+d x2 quit".to_string()
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
    let dim_block = Block::default().style(Style::default().bg(ratatui::style::Color::Black));
    // Removed Clear widget to allow background to show through for semi-transparent effect
    frame.render_widget(dim_block, area);
}

/// Render the Add to Favorites dialog
fn render_add_favorite_dialog(
    frame: &mut Frame,
    area: Rect,
    path: &std::path::Path,
    selection: usize,
    theme: &Theme,
) {
    render_dialog_background(frame, area);

    let dialog_area = centered_rect(50, 10, area);

    // Clear only the dialog area to remove text behind
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Add to Favorites ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight))
        .style(Style::default().bg(ratatui::style::Color::Black));

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
        Line::from(Span::styled(
            "Select path to add:",
            Style::default().fg(theme.text_normal),
        )),
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

/// Render the Tool Not Installed dialog
fn render_tool_not_installed_dialog(frame: &mut Frame, area: Rect, tool_name: &str, theme: &Theme) {
    render_dialog_background(frame, area);

    let dialog_area = centered_rect(50, 9, area);

    // Clear only the dialog area to remove text behind
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Tool Not Installed ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ratatui::style::Color::Red))
        .style(Style::default().bg(ratatui::style::Color::Black));

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

    // Clear only the dialog area to remove text behind
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ratatui::style::Color::Red))
        .style(Style::default().bg(ratatui::style::Color::Black));

    frame.render_widget(&block, dialog_area);

    let inner = block.inner(dialog_area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            message,
            Style::default().fg(theme.text_normal),
        )),
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
    // Dim the background (keeps content visible behind)
    render_dialog_background(frame, area);

    // Centered settings dialog box - increased size to prevent clipping
    let dialog_area = centered_rect(70, 14, area);

    // Clear only the dialog area to remove text behind, then render solid background
    frame.render_widget(Clear, dialog_area);

    let settings_block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight))
        .style(Style::default().bg(ratatui::style::Color::Black));

    frame.render_widget(&settings_block, dialog_area);
    let inner = settings_block.inner(dialog_area);

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
    let accent_display = format!("< {} >", app.settings.accent_color.to_lowercase());

    // Build nav mode display with arrows
    let nav_display = format!("< {} >", app.settings.nav_mode.to_lowercase());

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    // Center the settings items with padding
    let width = inner.width as usize;
    let accent_color_line = format!("Accent Color:  {}", accent_display);
    let accent_padding = (width.saturating_sub(accent_color_line.len())) / 2;
    lines.push(Line::from(vec![
        Span::raw(" ".repeat(accent_padding)),
        Span::styled("Accent Color:  ", accent_style),
        Span::styled(accent_display.clone(), accent_style),
    ]));

    lines.push(Line::from(""));

    let nav_line = format!("Navigation:    {}", nav_display);
    let nav_padding = (width.saturating_sub(nav_line.len())) / 2;
    lines.push(Line::from(vec![
        Span::raw(" ".repeat(nav_padding)),
        Span::styled("Navigation:    ", nav_style),
        Span::styled(nav_display.clone(), nav_style),
    ]));

    let text = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(text, inner);

    // Render controls at the very bottom of the dialog
    let controls_text = "w/s navigate  \u{25CF}  d/enter change  \u{25CF}  esc save & close";
    let controls_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };
    let controls = Paragraph::new(Line::from(Span::styled(
        controls_text,
        Style::default().fg(theme.text_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(controls, controls_area);
}

/// Truncate a path string for display
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - max_len + 3..])
    }
}

/// Render the custom color input dialog
fn render_custom_color_dialog(frame: &mut Frame, area: Rect, hex_input: &str, theme: &Theme) {
    render_dialog_background(frame, area);

    let popup_area = centered_rect(50, 30, area);

    // Clear only the dialog area to remove text behind
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Enter HEX Color ")
        .title_style(Style::default().fg(theme.highlight).bold())
        .border_style(Style::default().fg(theme.highlight))
        .style(Style::default().bg(ratatui::style::Color::Black));
    let inner = block.inner(popup_area);

    frame.render_widget(block, popup_area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Enter HEX color code (e.g., #FF6600):",
            Style::default().fg(theme.text_normal),
        )),
        Line::from(""),
        Line::from(Span::styled(
            hex_input,
            Style::default().fg(theme.highlight).bg(theme.border_normal),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render the keybind configuration dialog
/// Render the opening dialog as a simple overlay with a spinner (no dimming)
fn render_opening_dialog(frame: &mut Frame, area: Rect, tool_name: &str, theme: &Theme) {
    // Render a small centered box on top without clearing or dimming the background
    let dialog_area = centered_rect(40, 7, area);

    let block = Block::default()
        .title(" Opening ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight));

    // Draw the dialog box directly on top of existing content
    frame.render_widget(&block, dialog_area);

    let inner = block.inner(dialog_area);

    // Simple time-based spinner
    let frames = ["|", "/", "-", "\\"];
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let idx = ((ms / 150) % (frames.len() as u128)) as usize;
    let spinner = frames[idx];

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{} Opening {}...", spinner, tool_name),
            Style::default().fg(theme.text_normal),
        )),
        Line::from(""),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render the command bar overlay (centered in top half of screen)
fn render_command_bar(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    filtered: &[(usize, i32)],
    selected: usize,
    theme: &Theme,
) {
    // Position: centered, top portion of screen
    let width = (area.width * 60 / 100).clamp(40, 80);
    // Scale height: 1 line per command, max 6 commands visible, +4 for header/footer
    let max_visible_commands = 6;
    let content_height = (filtered.len() as u16).min(max_visible_commands as u16);
    let height = content_height + 4; // +4 for search, separator, and footer
    let x = (area.width.saturating_sub(width)) / 2;
    let y = area.height / 6;
    let dialog_area = Rect::new(x, y, width, height);

    // Dim background
    render_dialog_background(frame, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Command Bar ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Search field with cursor
    let cursor = if query.is_empty() {
        "type to search..."
    } else {
        "_"
    };
    let search_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(theme.highlight)),
        Span::styled(query, Style::default().fg(theme.text_normal)),
        Span::styled(
            cursor,
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(search_line),
        Rect::new(inner.x + 1, inner.y, inner.width.saturating_sub(2), 1),
    );

    // Separator
    let sep = "─".repeat(inner.width.saturating_sub(2) as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(sep, Style::default().fg(theme.border_normal))),
        Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1),
    );

    // Command list with scrolling
    let list_area = Rect::new(
        inner.x + 1,
        inner.y + 2,
        inner.width.saturating_sub(2),
        inner.height.saturating_sub(3),
    );
    let max_items = (list_area.height as usize).min(filtered.len());

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = if selected > max_visible_commands - 1 {
        selected - max_visible_commands + 1
    } else {
        0
    };

    for (display_idx, (actual_idx, &(cmd_idx, _))) in filtered.iter().enumerate().skip(scroll_offset).take(max_items).enumerate() {
        let cmd = &COMMANDS[cmd_idx];
        let is_selected = actual_idx == selected;
        let style = if is_selected {
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_normal)
        };
        let prefix = if is_selected { "▸ " } else { "  " };
        let line = Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(cmd.name, style),
            Span::styled(" - ", Style::default().fg(theme.text_dim)),
            Span::styled(cmd.description, Style::default().fg(theme.text_dim)),
        ]);
        frame.render_widget(
            Paragraph::new(line),
            Rect::new(list_area.x, list_area.y + display_idx as u16, list_area.width, 1),
        );
    }

    // Hint at bottom
    if inner.height > 4 {
        let hint = Line::from(vec![
            Span::styled("Tab", Style::default().fg(theme.highlight)),
            Span::styled(" autocomplete  ", Style::default().fg(theme.text_dim)),
            Span::styled("Enter", Style::default().fg(theme.highlight)),
            Span::styled(" select  ", Style::default().fg(theme.text_dim)),
            Span::styled("Esc", Style::default().fg(theme.highlight)),
            Span::styled(" close", Style::default().fg(theme.text_dim)),
        ]);
        frame.render_widget(
            Paragraph::new(hint).alignment(Alignment::Center),
            Rect::new(
                inner.x,
                inner.y + inner.height.saturating_sub(1),
                inner.width,
                1,
            ),
        );
    }
}

/// Render the provider configuration dialog
fn render_provider_config(frame: &mut Frame, area: Rect, selected_index: usize, theme: &Theme) {
    let dialog_area = centered_rect(60, 50, area);
    render_dialog_background(frame, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Provider Configuration ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let items: Vec<ListItem> = PROVIDERS
        .iter()
        .enumerate()
        .map(|(i, provider)| {
            let style = if i == selected_index {
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_normal)
            };
            let prefix = if i == selected_index { "▸ " } else { "  " };
            let auth_type = if *provider == "GitHub Copilot" {
                "(proxy login)"
            } else {
                "(env key)"
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(*provider, style),
                Span::styled(
                    format!(" {}", auth_type),
                    Style::default().fg(theme.text_dim),
                ),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(
        list,
        Rect::new(
            inner.x + 1,
            inner.y + 1,
            inner.width.saturating_sub(2),
            inner.height.saturating_sub(2),
        ),
    );
}

/// Render the keybind configuration dialog
fn render_keybind_config(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    selected_index: usize,
    editing_field: Option<usize>,
    theme: &Theme,
) {
    let dialog_area = centered_rect(50, 40, area);
    render_dialog_background(frame, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Keybind Configuration ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let keybinds = &app.settings.keybinds;
    let items = [
        ("Up", &keybinds.up),
        ("Down", &keybinds.down),
        ("Left", &keybinds.left),
        ("Right", &keybinds.right),
    ];

    for (i, (label, value)) in items.iter().enumerate() {
        let is_selected = i == selected_index;
        let is_editing = editing_field == Some(i);
        let style = if is_selected {
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_normal)
        };
        let prefix = if is_selected { "▸ " } else { "  " };
        let value_display = if is_editing {
            "[_]".to_string()
        } else {
            format!("[{}]", value)
        };
        let line = Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("{}: ", label), style),
            Span::styled(
                value_display,
                if is_editing {
                    Style::default()
                        .fg(theme.highlight)
                        .add_modifier(Modifier::SLOW_BLINK)
                } else {
                    Style::default().fg(theme.text_dim)
                },
            ),
        ]);
        frame.render_widget(
            Paragraph::new(line),
            Rect::new(
                inner.x + 1,
                inner.y + 1 + i as u16,
                inner.width.saturating_sub(2),
                1,
            ),
        );
    }

    // Preset display
    let preset_line = Line::from(vec![
        Span::styled("  Preset: ", Style::default().fg(theme.text_normal)),
        Span::styled(&app.settings.nav_mode, Style::default().fg(theme.highlight)),
    ]);
    frame.render_widget(
        Paragraph::new(preset_line),
        Rect::new(inner.x + 1, inner.y + 6, inner.width.saturating_sub(2), 1),
    );

    // Hint
    let hint = Line::from(vec![
        Span::styled("Enter", Style::default().fg(theme.highlight)),
        Span::styled(" edit  ", Style::default().fg(theme.text_dim)),
        Span::styled("Esc", Style::default().fg(theme.highlight)),
        Span::styled(" close", Style::default().fg(theme.text_dim)),
    ]);
    frame.render_widget(
        Paragraph::new(hint).alignment(Alignment::Center),
        Rect::new(
            inner.x,
            inner.y + inner.height.saturating_sub(2),
            inner.width,
            1,
        ),
    );
}

/// Render the environment variables configuration dialog
fn render_env_config(
    frame: &mut Frame,
    area: Rect,
    entries: &[(String, String)],
    selected_index: usize,
    editing_field: Option<usize>,
    input_buffer: &str,
    theme: &Theme,
) {
    let dialog_area = centered_rect(70, 60, area);
    render_dialog_background(frame, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Environment Variables ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // List entries
    for (i, (key, value)) in entries.iter().enumerate() {
        let is_selected = i == selected_index;
        let style = if is_selected {
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_normal)
        };
        let prefix = if is_selected { "▸ " } else { "  " };

        let key_display = if is_selected && editing_field == Some(0) {
            format!("{}_", input_buffer)
        } else {
            key.clone()
        };

        let value_display = if is_selected && editing_field == Some(1) {
            format!("{}_", input_buffer)
        } else {
            "*".repeat(value.len().min(20))
        };

        let line = Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(key_display, style),
            Span::styled(" = ", Style::default().fg(theme.text_dim)),
            Span::styled(value_display, Style::default().fg(theme.text_dim)),
        ]);
        frame.render_widget(
            Paragraph::new(line),
            Rect::new(
                inner.x + 1,
                inner.y + 1 + i as u16,
                inner.width.saturating_sub(2),
                1,
            ),
        );
    }

    // Add new entry option
    let add_idx = entries.len();
    let is_add_selected = selected_index == add_idx;
    let add_style = if is_add_selected {
        Style::default()
            .fg(theme.highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_dim)
    };
    let add_prefix = if is_add_selected { "▸ " } else { "  " };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(add_prefix, add_style),
            Span::styled("+ Add new variable", add_style),
        ])),
        Rect::new(
            inner.x + 1,
            inner.y + 1 + add_idx as u16,
            inner.width.saturating_sub(2),
            1,
        ),
    );

    // Hint
    let hint = Line::from(vec![
        Span::styled("Tab", Style::default().fg(theme.highlight)),
        Span::styled(" switch field  ", Style::default().fg(theme.text_dim)),
        Span::styled("Enter", Style::default().fg(theme.highlight)),
        Span::styled(" edit  ", Style::default().fg(theme.text_dim)),
        Span::styled("Esc", Style::default().fg(theme.highlight)),
        Span::styled(" close", Style::default().fg(theme.text_dim)),
    ]);
    frame.render_widget(
        Paragraph::new(hint).alignment(Alignment::Center),
        Rect::new(
            inner.x,
            inner.y + inner.height.saturating_sub(2),
            inner.width,
            1,
        ),
    );
}

/// Render the settings configuration dialog
fn render_settings_config(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    selected_index: usize,
    theme: &Theme,
) {
    let dialog_area = centered_rect(50, 30, area);
    render_dialog_background(frame, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.highlight));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let settings_items = [
        ("Accent Color", &app.settings.accent_color),
        ("Navigation Mode", &app.settings.nav_mode),
    ];

    for (i, (label, value)) in settings_items.iter().enumerate() {
        let is_selected = i == selected_index;
        let style = if is_selected {
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_normal)
        };
        let prefix = if is_selected { "▸ " } else { "  " };
        let line = Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("{}: ", label), style),
            Span::styled(*value, Style::default().fg(theme.text_dim)),
        ]);
        frame.render_widget(
            Paragraph::new(line),
            Rect::new(
                inner.x + 1,
                inner.y + 1 + i as u16,
                inner.width.saturating_sub(2),
                1,
            ),
        );
    }

    // Hint
    let hint = Line::from(vec![Span::styled(
        "Use existing settings overlay (Ctrl+S) for full options",
        Style::default().fg(theme.text_dim),
    )]);
    frame.render_widget(
        Paragraph::new(hint).alignment(Alignment::Center),
        Rect::new(
            inner.x,
            inner.y + inner.height.saturating_sub(2),
            inner.width,
            1,
        ),
    );
}

fn render_global_config_overlay(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let accent = Theme::from_name(&app.settings.accent_color);

    let overlay_width = area.width / 2;
    let overlay_height = (area.height * 2) / 5;
    let overlay_x = (area.width.saturating_sub(overlay_width)) / 2;
    let overlay_y = (area.height.saturating_sub(overlay_height)) / 2;

    let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent))
        .border_type(BorderType::Rounded)
        .title(" global config switcher ")
        .title_style(Style::default().fg(accent).add_modifier(Modifier::BOLD));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let profiles = ["claude max", "claude pro", "claude free"];
    let descriptions = [
        "opus 4.5 | sonnet 4.5 | haiku 4.5",
        "opus 4.5 | sonnet 4.5 | haiku gpt-5-mini",
        "sonnet gpt-5-mini",
    ];

    let content_area = Rect::new(
        inner.x + 2,
        inner.y + 1,
        inner.width.saturating_sub(4),
        inner.height.saturating_sub(3),
    );

    for (i, (profile, desc)) in profiles.iter().zip(descriptions.iter()).enumerate() {
        let y = content_area.y + (i as u16 * 3);
        if y + 2 >= content_area.y + content_area.height {
            break;
        }

        let is_selected = i == app.global_config_selection;
        let (prefix, style) = if is_selected {
            (
                "> ",
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )
        } else {
            ("  ", Style::default().fg(Color::Gray))
        };

        let profile_text = format!("{}{}", prefix, profile);
        let profile_para = Paragraph::new(profile_text).style(style);
        frame.render_widget(
            profile_para,
            Rect::new(content_area.x, y, content_area.width, 1),
        );

        let desc_para = Paragraph::new(*desc).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(
            desc_para,
            Rect::new(content_area.x + 2, y + 1, content_area.width, 1),
        );
    }

    let footer_y = inner.y + inner.height.saturating_sub(1);
    let footer_text = "[w/s] navigate  [enter] apply  [esc] cancel";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(footer, Rect::new(inner.x, footer_y, inner.width, 1));
}
