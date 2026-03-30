use crate::app::{ActivePanel, App, LeftSection};
use crate::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

const ASCII_ART: &str = "CLUMSY CAT";

pub fn render(app: &App, frame: &mut Frame) {
    let theme = Theme::default();
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
    
    if let Some(ref err) = app.error {
        let error_msg = Paragraph::new(format!("Error: {}", err))
            .style(Style::default().fg(ratatui::style::Color::Red))
            .centered();
        frame.render_widget(error_msg, inner);
        return;
    }
    
    if app.entries.is_empty() {
        let empty_msg = Paragraph::new("Empty directory")
            .style(theme.text_dim)
            .centered();
        frame.render_widget(empty_msg, inner);
        return;
    }
    
    let items: Vec<ListItem> = app.entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let icon = if entry.is_dir { "▸" } else { "·" };
            let is_selected = i == app.selected_index && app.active_panel == ActivePanel::Right;
            let color = if is_selected {
                theme.highlight
            } else if entry.is_dir {
                theme.text_normal
            } else {
                theme.text_dim
            };
            
            ListItem::new(
                Line::from(vec![
                    Span::raw(icon),
                    Span::raw(" "),
                    Span::raw(&entry.name).style(Style::default().fg(color)),
                ])
            )
        })
        .collect();
    
    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_bottom_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_normal));
    
    let inner = block.inner(area);
    frame.render_widget(&block, area);
    
    let keybinds = if app.quit_confirm >= 1 {
        " [Ctrl+D×2] Confirm quit "
    } else {
        " [/] Search   [Space] Select   [D] Open   [A] Back   [W/S] Up/Down   [Tab] Cycle Panel   [Esc] Panel Mode   [Ctrl+D×2] Quit "
    };
    
    let text = Paragraph::new(keybinds)
        .style(Style::default().fg(theme.text_dim))
        .centered();
    
    frame.render_widget(text, inner);
}
