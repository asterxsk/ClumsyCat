use crate::config::{Config, Settings};
use crate::fs::load_dir_entries;
use crate::search::{filter_entries, SearchMode};
use crate::tools::{self, find_tool_by_display_name, LaunchResult, PROVIDERS, STUB_MODELS, TOOLS};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Represents the current page in the application flow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Browser,
    ToolSelection,
    Provider,
    Model,
}

/// Represents active dialog states
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Dialog {
    #[default]
    None,
    AddToFavorites {
        path: PathBuf,
    },
    SudoPassword {
        target_path: PathBuf,
        password_input: String,
    },
    ToolNotInstalled {
        tool_name: String,
    },
    Error {
        message: String,
    },
    CustomColorInput {
        hex_input: String,
    },
    Opening {
        tool_name: String,
    },
    CommandBar {
        query: String,
        filtered_indices: Vec<(usize, i32)>, // (command index, score)
        selected_index: usize,
    },
    ProviderConfig {
        selected_index: usize,
    },
    KeybindConfig {
        selected_index: usize,
        editing_field: Option<usize>,
    },
    EnvConfig {
        entries: Vec<(String, String)>,
        selected_index: usize,
        editing_field: Option<usize>, // 0=key, 1=value
        input_buffer: String,
    },
    SettingsConfig {
        selected_index: usize,
    },
}

/// Command definition for the command bar
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
}

/// Available commands in the command bar
pub const COMMANDS: &[Command] = &[
    Command { name: "providerconf", description: "Edit provider configurations" },
    Command { name: "keybindconf", description: "Customize keybindings" },
    Command { name: "env", description: "Manage environment variables" },
    Command { name: "settings", description: "Open settings" },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LeftSection {
    #[default]
    Favorites,
    Recents,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivePanel {
    Left,
    #[default]
    Right,
}

pub struct App {
    // Global state
    pub page: Page,
    pub dialog: Dialog,
    pub search_mode: SearchMode,
    pub search_typing_mode: bool,  // true when in typing mode, false when in navigation mode
    pub settings: Settings,
    pub previous_page: Option<Page>,
    pub ascii_art: String,

    // Page 1: Browser state
    pub current_dir: PathBuf,
    pub entries: Vec<crate::fs::DirEntry>,
    pub selected_index: usize,
    pub left_section: LeftSection,
    pub active_panel: ActivePanel,
    pub favorites_dirs: Vec<PathBuf>,
    pub recents_dirs: Vec<PathBuf>,
    pub selected_dir: Option<PathBuf>,

    // Page 2: Tool selection state
    pub tools: Vec<String>,
    pub selected_tool_index: usize,
    pub favorites_tools: Vec<String>,
    pub recents_tools: Vec<String>,
    pub tool_left_section: LeftSection,
    pub selected_tool: Option<String>,

    // Page 3: Provider selection state
    pub providers: Vec<String>,
    pub selected_provider_index: usize,
    pub favorites_providers: Vec<String>,
    pub recents_providers: Vec<String>,
    pub provider_left_section: LeftSection,
    pub selected_provider: Option<String>,

    // Page 4: Model selection state
    pub models: Vec<String>,
    pub selected_model_index: usize,
    pub favorites_models: Vec<String>,
    pub recents_models: Vec<String>,
    pub model_left_section: LeftSection,
    pub models_loading: bool,
    pub models_error: Option<String>,

    // UI state
    pub quit_confirm: u8,
    pub quit_timer: Option<Instant>,
    pub error: Option<String>,

    // Dialog state
    pub dialog_selection: usize,

    // Settings overlay state
    pub settings_open: bool,
    pub settings_selection: usize,

    // Command bar state
    pub last_shift_time: Option<Instant>,

    // Config for saving
    config: Config,
}

impl App {
    pub fn new() -> Self {
        // Load configuration
        let config = Config::load();

        // Load ASCII art from file
        let ascii_art = std::fs::read_to_string("ascii.md")
            .unwrap_or_else(|_| "CLUMSY CAT".to_string());

        let current_dir = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));

        let dir_entries = load_dir_entries(&current_dir);

        // Load favorites and recents from config, with defaults
        let favorites_dirs = config
            .favorites
            .get("dirs")
            .map(|v| v.iter().map(PathBuf::from).collect())
            .unwrap_or_else(|| {
                vec![
                    PathBuf::from("/"),
                    PathBuf::from("/tmp"),
                    PathBuf::from("/home"),
                ]
            });

        let recents_dirs = config
            .recents
            .get("dirs")
            .map(|v| v.iter().map(PathBuf::from).collect())
            .unwrap_or_default();

        let favorites_tools = config
            .favorites
            .get("tools")
            .cloned()
            .unwrap_or_default();

        let recents_tools = config.recents.get("tools").cloned().unwrap_or_default();

        let favorites_providers = config
            .favorites
            .get("providers")
            .cloned()
            .unwrap_or_default();

        let recents_providers = config
            .recents
            .get("providers")
            .cloned()
            .unwrap_or_default();

        let favorites_models = config
            .favorites
            .get("models")
            .cloned()
            .unwrap_or_default();

        let recents_models = config.recents.get("models").cloned().unwrap_or_default();

        // Tool list from tools.rs
        let tools: Vec<String> = TOOLS.iter().map(|t| t.display_name.to_string()).collect();

        // Provider list from tools.rs
        let providers: Vec<String> = PROVIDERS.iter().map(|p| p.to_string()).collect();

        Self {
            // Global state
            page: Page::default(),
            dialog: Dialog::default(),
            search_mode: SearchMode::Inactive,
            search_typing_mode: false,
            settings: config.settings.clone(),
            previous_page: None,
            ascii_art,

            // Page 1: Browser state
            current_dir,
            entries: dir_entries.entries,
            selected_index: 0,
            left_section: LeftSection::default(),
            active_panel: ActivePanel::default(),
            favorites_dirs,
            recents_dirs,
            selected_dir: None,

            // Page 2: Tool selection state
            tools,
            selected_tool_index: 0,
            favorites_tools,
            recents_tools,
            tool_left_section: LeftSection::default(),
            selected_tool: None,

            // Page 3: Provider selection state
            providers,
            selected_provider_index: 0,
            favorites_providers,
            recents_providers,
            provider_left_section: LeftSection::default(),
            selected_provider: None,

            // Page 4: Model selection state
            models: Vec::new(),
            selected_model_index: 0,
            favorites_models,
            recents_models,
            model_left_section: LeftSection::default(),
            models_loading: false,
            models_error: None,

            // UI state
            quit_confirm: 0,
            quit_timer: None,
            error: dir_entries.error,

            // Dialog state
            dialog_selection: 0,

            // Settings overlay state
            settings_open: false,
            settings_selection: 0,

            // Command bar state
            last_shift_time: None,

            // Config for saving
            config,
        }
    }

    /// Advance to the next page based on current page and selections
    pub fn advance_page(&mut self) {
        match self.page {
            Page::Browser => {
                self.page = Page::ToolSelection;
            }
            Page::ToolSelection => {
                self.page = Page::Provider;
            }
            Page::Provider => {
                self.page = Page::Model;
                self.models_loading = true;
            }
            Page::Model => {
                // Final page - launch the selected tool
            }
        }
    }

    /// Go back to the previous page
    pub fn go_back(&mut self) {
        match self.page {
            Page::Browser => {
                // Already at first page, do nothing
            }
            Page::ToolSelection => {
                self.page = Page::Browser;
                self.selected_tool = None;
            }
            Page::Provider => {
                self.page = Page::ToolSelection;
                self.selected_provider = None;
            }
            Page::Model => {
                self.page = Page::Provider;
                self.models.clear();
                self.models_error = None;
            }
        }
    }

    /// Open the settings overlay, saving current page
    pub fn open_settings(&mut self) {
        self.previous_page = Some(self.page);
    }

    /// Close the settings overlay, restoring previous page
    pub fn close_settings(&mut self) {
        if let Some(prev) = self.previous_page.take() {
            self.page = prev;
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers, KeyEventKind};

        loop {
            terminal.draw(|frame| crate::ui::render(self, frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Shift+F opens command bar
                    if matches!(key.code, KeyCode::Char('f') | KeyCode::Char('F'))
                        && key.modifiers.contains(KeyModifiers::SHIFT)
                        && !key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        self.open_command_bar();
                        continue;
                    }

                    // Map vim keys if nav_mode is "vim"
                    let code = self.map_vim_key(key.code);

                    // Handle dialog input first (highest priority)
                    if self.dialog != Dialog::None {
                        self.handle_dialog_input(code, key.modifiers);
                        continue;
                    }

                    // Handle settings overlay input (but allow ctrl+d to bypass)
                    if self.settings_open {
                        let is_ctrl_d = matches!(code, KeyCode::Char('d') | KeyCode::Char('D'))
                            && key.modifiers.contains(KeyModifiers::CONTROL);
                        if !is_ctrl_d {
                            self.handle_settings_input(code);
                            continue;
                        }
                    }

                    // Handle search mode input
                    if self.search_mode.is_active() {
                        // Allow control key combinations to pass through
                        let is_ctrl_shortcut = key.modifiers.contains(KeyModifiers::CONTROL);

                        if is_ctrl_shortcut {
                            // Let control shortcuts fall through to normal handling
                            // (don't continue, so they get processed below)
                        } else if self.search_typing_mode {
                            // Typing mode: only Esc, Enter, Backspace have special meaning
                            match code {
                                KeyCode::Esc => {
                                    // Exit typing mode, enter navigation mode
                                    self.search_typing_mode = false;
                                    continue;
                                }
                                KeyCode::Enter => {
                                    // Select from search and let normal Enter handler process it
                                    if self.handle_search_selection() {
                                        self.search_mode = SearchMode::Inactive;
                                        self.search_typing_mode = false;
                                        // Don't continue - let normal mode handle Enter
                                    } else {
                                        continue;
                                    }
                                }
                                KeyCode::Backspace => {
                                    self.search_backspace();
                                    continue;
                                }
                                KeyCode::Char(c) => {
                                    // All characters go into search query (including w/s/a/d)
                                    self.update_search_query(c);
                                    continue;
                                }
                                _ => {
                                    continue;
                                }
                            }
                        } else {
                            // Navigation mode: w/s/a/d navigate filtered results
                            match code {
                                KeyCode::Char('w') | KeyCode::Char('W') => {
                                    self.search_prev_match();
                                    continue;
                                }
                                KeyCode::Char('s') | KeyCode::Char('S') => {
                                    self.search_next_match();
                                    continue;
                                }
                                KeyCode::Char('a') | KeyCode::Char('A') => {
                                    // Navigate back
                                    self.search_mode = SearchMode::Inactive;
                                    self.search_typing_mode = false;
                                    // Continue to normal mode to handle 'a'
                                }
                                KeyCode::Char('d') | KeyCode::Char('D') => {
                                    // Open the selected directory
                                    if let Some(dir_path) = self.get_current_search_directory() {
                                        self.navigate_to_dir(&dir_path);
                                        self.search_mode = SearchMode::Inactive;
                                        self.search_typing_mode = false;
                                    }
                                    continue;
                                }
                                KeyCode::Char('/') => {
                                    // Resume typing mode
                                    self.search_typing_mode = true;
                                    continue;
                                }
                                KeyCode::Enter => {
                                    // Select from search and let normal Enter handler process it
                                    if self.handle_search_selection() {
                                        self.search_mode = SearchMode::Inactive;
                                        self.search_typing_mode = false;
                                        // Don't continue - let normal mode handle Enter
                                    } else {
                                        continue;
                                    }
                                }
                                KeyCode::Esc => {
                                    // Exit search entirely
                                    self.search_mode = SearchMode::Inactive;
                                    self.search_typing_mode = false;
                                    continue;
                                }
                                _ => {
                                    continue;
                                }
                            }
                        }
                    }

                    // Normal mode input handling
                    match code {
                        KeyCode::Char('d') | KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if self.handle_ctrl_d() {
                                break; // Exit the event loop
                            }
                        }
                        KeyCode::Char('f') | KeyCode::Char('F') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.open_add_favorite_dialog();
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.settings_open = true;
                            self.settings_selection = 0;
                        }
                        // Global hotkey: R - Switch left panel to Recents and focus it
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            self.active_panel = ActivePanel::Left;
                            match self.page {
                                Page::Browser => { self.left_section = LeftSection::Recents; self.selected_index = 0; }
                                Page::ToolSelection => { self.tool_left_section = LeftSection::Recents; self.selected_tool_index = 0; }
                                Page::Provider => { self.provider_left_section = LeftSection::Recents; self.selected_provider_index = 0; }
                                Page::Model => { self.model_left_section = LeftSection::Recents; self.selected_model_index = 0; }
                            }
                        }
                        // Global hotkey: F - Switch left panel to Favorites and focus it
                        KeyCode::Char('f') | KeyCode::Char('F') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.active_panel = ActivePanel::Left;
                            match self.page {
                                Page::Browser => { self.left_section = LeftSection::Favorites; self.selected_index = 0; }
                                Page::ToolSelection => { self.tool_left_section = LeftSection::Favorites; self.selected_tool_index = 0; }
                                Page::Provider => { self.provider_left_section = LeftSection::Favorites; self.selected_provider_index = 0; }
                                Page::Model => { self.model_left_section = LeftSection::Favorites; self.selected_model_index = 0; }
                            }
                        }
                        // Global hotkey: B - Switch focus to Browser (right panel)
                        KeyCode::Char('b') | KeyCode::Char('B') => {
                            self.active_panel = ActivePanel::Right;
                            match self.page {
                                Page::Browser => { self.selected_index = 0; }
                                Page::ToolSelection => { self.selected_tool_index = 0; }
                                Page::Provider => { self.selected_provider_index = 0; }
                                Page::Model => { self.selected_model_index = 0; }
                            }
                        }
                        // Global hotkey: T - Switch focus to Tools (right panel on ToolSelection page)
                        KeyCode::Char('t') | KeyCode::Char('T') => {
                            if self.page == Page::ToolSelection {
                                self.active_panel = ActivePanel::Right;
                                self.selected_tool_index = 0;
                            }
                        }
                        // Global hotkey: P - Switch focus to Profiles (right panel on Provider page)
                        KeyCode::Char('p') | KeyCode::Char('P') => {
                            if self.page == Page::Provider {
                                self.active_panel = ActivePanel::Right;
                                self.selected_provider_index = 0;
                            }
                        }
                        // Global hotkey: M - Switch focus to Models (right panel on Model page)
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            if self.page == Page::Model {
                                self.active_panel = ActivePanel::Right;
                                self.selected_model_index = 0;
                            }
                        }
                        KeyCode::Tab => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;

                            let new_panel = match self.active_panel {
                                ActivePanel::Left => ActivePanel::Right,
                                ActivePanel::Right => ActivePanel::Left,
                            };

                            // When switching TO left panel, smart-default to section with items
                            if new_panel == ActivePanel::Left {
                                match self.page {
                                    Page::Browser => {
                                        if !self.favorites_dirs.is_empty() {
                                            self.left_section = LeftSection::Favorites;
                                        } else if !self.recents_dirs.is_empty() {
                                            self.left_section = LeftSection::Recents;
                                        }
                                        self.selected_index = 0;
                                    }
                                    Page::ToolSelection => {
                                        if !self.favorites_tools.is_empty() {
                                            self.tool_left_section = LeftSection::Favorites;
                                        } else if !self.recents_tools.is_empty() {
                                            self.tool_left_section = LeftSection::Recents;
                                        }
                                        self.selected_tool_index = 0;
                                    }
                                    Page::Provider => {
                                        if !self.favorites_providers.is_empty() {
                                            self.provider_left_section = LeftSection::Favorites;
                                        } else if !self.recents_providers.is_empty() {
                                            self.provider_left_section = LeftSection::Recents;
                                        }
                                        self.selected_provider_index = 0;
                                    }
                                    Page::Model => {
                                        if !self.favorites_models.is_empty() {
                                            self.model_left_section = LeftSection::Favorites;
                                        } else if !self.recents_models.is_empty() {
                                            self.model_left_section = LeftSection::Recents;
                                        }
                                        self.selected_model_index = 0;
                                    }
                                }
                            }

                            self.active_panel = new_panel;
                        }
                        KeyCode::Esc => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            // Navigate back to previous page, or do nothing if on first page
                            if self.page != Page::Browser {
                                self.go_back();
                            }
                        }
                        KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.handle_up();
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.handle_down();
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.handle_open(terminal);
                        }
                        KeyCode::Enter => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.handle_select(terminal);
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.handle_back();
                        }
                        KeyCode::Char('/') => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.activate_search();
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(()) // Return Ok when loop exits
    }

    /// Map vim keys (k/j/h/l) to navigation keys when in vim mode
    fn map_vim_key(&self, code: ratatui::crossterm::event::KeyCode) -> ratatui::crossterm::event::KeyCode {
        use ratatui::crossterm::event::KeyCode;

        // Use custom keybinds for mapping
        match code {
            KeyCode::Char(c) if c.to_string() == self.settings.keybinds.up => KeyCode::Char('w'),
            KeyCode::Char(c) if c.to_string() == self.settings.keybinds.down => KeyCode::Char('s'),
            KeyCode::Char(c) if c.to_string() == self.settings.keybinds.left => KeyCode::Char('a'),
            KeyCode::Char(c) if c.to_string() == self.settings.keybinds.right => KeyCode::Char('d'),
            other => other,
        }
    }

    /// Handle input when a dialog is open
    fn handle_dialog_input(&mut self, code: ratatui::crossterm::event::KeyCode, modifiers: ratatui::crossterm::event::KeyModifiers) {
        use ratatui::crossterm::event::KeyCode;
        use ratatui::crossterm::event::KeyModifiers;

        match &mut self.dialog {
            Dialog::None => {}
            Dialog::AddToFavorites { path } => {
                match code {
                    KeyCode::Esc => {
                        self.dialog = Dialog::None;
                        self.dialog_selection = 0;
                    }
                    KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                        if self.dialog_selection > 0 {
                            self.dialog_selection -= 1;
                        }
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                        if self.dialog_selection < 1 {
                            self.dialog_selection += 1;
                        }
                    }
                    KeyCode::Enter => {
                        let path_to_add = if self.dialog_selection == 0 {
                            path.clone()
                        } else {
                            path.parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or_else(|| PathBuf::from("/"))
                        };
                        self.add_to_favorites(path_to_add);
                        self.dialog = Dialog::None;
                        self.dialog_selection = 0;
                    }
                    _ => {}
                }
            }
            Dialog::SudoPassword { target_path: _, password_input } => {
                match code {
                    KeyCode::Esc => {
                        self.dialog = Dialog::None;
                    }
                    KeyCode::Enter => {
                        // Would authenticate here - for now just close
                        self.dialog = Dialog::None;
                    }
                    KeyCode::Backspace => {
                        password_input.pop();
                    }
                    KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                        password_input.push(c);
                    }
                    _ => {}
                }
            }
            Dialog::CustomColorInput { hex_input } => {
                match code {
                    KeyCode::Esc => {
                        self.dialog = Dialog::None;
                    }
                    KeyCode::Enter => {
                        // Validate and apply the hex color
                        let hex_clone = hex_input.clone();
                        if self.is_valid_hex_color(&hex_clone) {
                            self.settings.custom_color_hex = hex_clone;
                            self.settings.accent_color = "custom".to_string();
                        }
                        self.dialog = Dialog::None;
                    }
                    KeyCode::Backspace => {
                        hex_input.pop();
                    }
                    KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                        if hex_input.len() < 7 && (c.is_ascii_hexdigit() || (c == '#' && hex_input.is_empty())) {
                            hex_input.push(c.to_ascii_uppercase());
                        }
                    }
                    _ => {}
                }
            }
            Dialog::ToolNotInstalled { .. } | Dialog::Error { .. } => {
                match code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.dialog = Dialog::None;
                    }
                    _ => {}
                }
            }
            Dialog::Opening { .. } => {
                // Allow user to dismiss the opening overlay with Esc or Enter
                match code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.dialog = Dialog::None;
                    }
                    _ => {}
                }
            }
            Dialog::CommandBar { query, filtered_indices, selected_index } => {
                match code {
                    KeyCode::Esc => {
                        self.dialog = Dialog::None;
                    }
                    KeyCode::Enter => {
                        if let Some(&(cmd_idx, _)) = filtered_indices.get(*selected_index) {
                            self.execute_command(cmd_idx);
                        }
                    }
                    KeyCode::Tab => {
                        if let Some(&(cmd_idx, _)) = filtered_indices.get(*selected_index) {
                            *query = COMMANDS[cmd_idx].name.to_string();
                            self.filter_commands();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        if !filtered_indices.is_empty() {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                            } else if !filtered_indices.is_empty() {
                                *selected_index = filtered_indices.len().saturating_sub(1);
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                        if !filtered_indices.is_empty() {
                            *selected_index = (*selected_index + 1) % filtered_indices.len();
                        }
                    }
                    KeyCode::Backspace => {
                        query.pop();
                        self.filter_commands();
                    }
                    KeyCode::Char(c) if c.is_ascii_lowercase() && !modifiers.contains(KeyModifiers::CONTROL) => {
                        query.push(c);
                        self.filter_commands();
                    }
                    _ => {}
                }
            }
            Dialog::ProviderConfig { selected_index } => {
                match code {
                    KeyCode::Esc => self.dialog = Dialog::None,
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                        } else {
                            *selected_index = PROVIDERS.len() - 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                        *selected_index = (*selected_index + 1) % PROVIDERS.len();
                    }
                    _ => {}
                }
            }
            Dialog::KeybindConfig { selected_index, editing_field } => {
                match code {
                    KeyCode::Esc => {
                        if editing_field.is_some() {
                            *editing_field = None;
                        } else {
                            self.dialog = Dialog::None;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') if editing_field.is_none() => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                        } else {
                            *selected_index = 4; // 5 items: up, down, left, right, preset
                        }
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') if editing_field.is_none() => {
                        *selected_index = (*selected_index + 1) % 5;
                    }
                    KeyCode::Enter if editing_field.is_none() => {
                        if *selected_index < 4 {
                            *editing_field = Some(*selected_index);
                        }
                    }
                    KeyCode::Char(c) if editing_field.is_some() => {
                        let field_idx = editing_field.unwrap();
                        match field_idx {
                            0 => self.settings.keybinds.up = c.to_string(),
                            1 => self.settings.keybinds.down = c.to_string(),
                            2 => self.settings.keybinds.left = c.to_string(),
                            3 => self.settings.keybinds.right = c.to_string(),
                            _ => {}
                        }
                        *editing_field = None;
                        self.settings.nav_mode = "custom".to_string();
                    }
                    _ => {}
                }
            }
            Dialog::EnvConfig { entries, selected_index, editing_field, input_buffer } => {
                match code {
                    KeyCode::Esc => {
                        if editing_field.is_some() {
                            *editing_field = None;
                            input_buffer.clear();
                        } else {
                            self.dialog = Dialog::None;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') if editing_field.is_none() => {
                        let total = entries.len() + 1; // +1 for "add new"
                        if *selected_index > 0 {
                            *selected_index -= 1;
                        } else {
                            *selected_index = total - 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') if editing_field.is_none() => {
                        let total = entries.len() + 1; // +1 for "add new"
                        *selected_index = (*selected_index + 1) % total;
                    }
                    KeyCode::Tab if editing_field.is_some() => {
                        // Toggle between key (0) and value (1) field
                        let current = editing_field.unwrap();
                        *editing_field = Some(if current == 0 { 1 } else { 0 });
                    }
                    KeyCode::Enter => {
                        if let Some(field) = *editing_field {
                            // Save the current input
                            if *selected_index < entries.len() {
                                if field == 0 {
                                    entries[*selected_index].0 = input_buffer.clone();
                                } else {
                                    entries[*selected_index].1 = input_buffer.clone();
                                }
                            }
                            *editing_field = None;
                            input_buffer.clear();
                        } else if *selected_index == entries.len() {
                            // Add new entry
                            entries.push((String::new(), String::new()));
                            *editing_field = Some(0);
                        } else {
                            *editing_field = Some(0);
                            *input_buffer = entries[*selected_index].0.clone();
                        }
                    }
                    KeyCode::Backspace if editing_field.is_some() => {
                        input_buffer.pop();
                    }
                    KeyCode::Char(c) if editing_field.is_some() && !modifiers.contains(KeyModifiers::CONTROL) => {
                        input_buffer.push(c);
                    }
                    _ => {}
                }
            }
            Dialog::SettingsConfig { selected_index } => {
                match code {
                    KeyCode::Esc => self.dialog = Dialog::None,
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                        } else {
                            *selected_index = 1; // 2 settings
                        }
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                        *selected_index = (*selected_index + 1) % 2;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Handle input when settings overlay is open
    fn handle_settings_input(&mut self, code: ratatui::crossterm::event::KeyCode) {
        use ratatui::crossterm::event::KeyCode;

        let accent_colors = ["orange", "red", "purple", "blue", "light_blue", "custom"];
        let mut nav_modes = vec!["wasd", "vim"];

        // Add custom presets to nav modes
        let custom_preset_names: Vec<String> = self.settings.custom_presets.keys().cloned().collect();
        for preset_name in &custom_preset_names {
            nav_modes.push(preset_name);
        }

        match code {
            KeyCode::Esc => {
                // Save and close settings
                self.config.settings = self.settings.clone();
                let _ = self.config.save();
                self.settings_open = false;
                self.quit_confirm = 0;
                self.quit_timer = None;
            }
            KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                self.quit_confirm = 0;
                self.quit_timer = None;
                if self.settings_selection > 0 {
                    self.settings_selection -= 1;
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                self.quit_confirm = 0;
                self.quit_timer = None;
                if self.settings_selection < 1 { // 2 settings (color, nav_mode)
                    self.settings_selection += 1;
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right | KeyCode::Enter => {
                self.quit_confirm = 0;
                self.quit_timer = None;
                match self.settings_selection {
                    0 => {
                        // Accent color
                        let current_idx = accent_colors
                            .iter()
                            .position(|&c| c == self.settings.accent_color)
                            .unwrap_or(0);
                        let next_idx = (current_idx + 1) % accent_colors.len();
                        let next_color = accent_colors[next_idx];

                        if next_color == "custom" {
                            // Open custom color input dialog
                            self.dialog = Dialog::CustomColorInput {
                                hex_input: self.settings.custom_color_hex.clone(),
                            };
                        } else {
                            self.settings.accent_color = next_color.to_string();
                        }
                    }
                    1 => {
                        // Nav mode
                        let current_idx = nav_modes
                            .iter()
                            .position(|&m| m == self.settings.nav_mode)
                            .unwrap_or(0);
                        let next_idx = (current_idx + 1) % nav_modes.len();
                        let next_mode = nav_modes[next_idx].to_string();
                        self.settings.nav_mode = next_mode.clone();

                        // Update keybinds based on preset
                        self.update_keybinds_for_preset(&next_mode);
                    }
                    _ => {}
                }
            }
            KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                self.quit_confirm = 0;
                self.quit_timer = None;
                match self.settings_selection {
                    0 => {
                        // Accent color
                        let current_idx = accent_colors
                            .iter()
                            .position(|&c| c == self.settings.accent_color)
                            .unwrap_or(0);
                        let next_idx = if current_idx == 0 {
                            accent_colors.len() - 1
                        } else {
                            current_idx - 1
                        };
                        let next_color = accent_colors[next_idx];

                        if next_color == "custom" {
                            // Open custom color input dialog
                            self.dialog = Dialog::CustomColorInput {
                                hex_input: self.settings.custom_color_hex.clone(),
                            };
                        } else {
                            self.settings.accent_color = next_color.to_string();
                        }
                    }
                    1 => {
                        // Nav mode
                        let current_idx = nav_modes
                            .iter()
                            .position(|&m| m == self.settings.nav_mode)
                            .unwrap_or(0);
                        let next_idx = if current_idx == 0 {
                            nav_modes.len() - 1
                        } else {
                            current_idx - 1
                        };
                        let next_mode = nav_modes[next_idx].to_string();
                        self.settings.nav_mode = next_mode.clone();

                        // Update keybinds based on preset
                        self.update_keybinds_for_preset(&next_mode);
                    }
                    2 => {
                        // Keybind Config (no left action)
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Open the Add to Favorites dialog for the current selection
    fn open_add_favorite_dialog(&mut self) {
        let path = match self.page {
            Page::Browser => {
                if self.active_panel == ActivePanel::Right && self.selected_index < self.entries.len() {
                    self.entries[self.selected_index].path.clone()
                } else {
                    self.current_dir.clone()
                }
            }
            _ => return, // Only browser page supports favorites for now
        };

        self.dialog = Dialog::AddToFavorites { path };
        self.dialog_selection = 0;
    }

    /// Add a path to favorites
    fn add_to_favorites(&mut self, path: PathBuf) {
        match self.page {
            Page::Browser => {
                if !self.favorites_dirs.contains(&path) {
                    self.favorites_dirs.push(path.clone());
                    // Update config and save
                    let dirs: Vec<String> = self.favorites_dirs.iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    self.config.favorites.insert("dirs".to_string(), dirs);
                    let _ = self.config.save();
                }
            }
            _ => {}
        }
    }

    /// Save all recents to config and persist to disk
    fn save_recents_to_config(&mut self) {
        // Save dirs
        let dirs: Vec<String> = self.recents_dirs.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        self.config.recents.insert("dirs".to_string(), dirs);

        // Save tools
        self.config.recents.insert("tools".to_string(), self.recents_tools.clone());

        // Save providers
        self.config.recents.insert("providers".to_string(), self.recents_providers.clone());

        // Save models
        self.config.recents.insert("models".to_string(), self.recents_models.clone());

        // Save config to disk
        let _ = self.config.save();
    }

    fn handle_ctrl_d(&mut self) -> bool {
        let now = Instant::now();
        if let Some(timer) = self.quit_timer {
            if now.duration_since(timer) < Duration::from_secs(1) {
                self.quit_confirm += 1;
                if self.quit_confirm >= 2 {
                    return true; // Signal to quit
                }
            } else {
                self.quit_confirm = 1;
                self.quit_timer = Some(now);
            }
        } else {
            self.quit_confirm = 1;
            self.quit_timer = Some(now);
        }
        false
    }

    fn handle_up(&mut self) {
        match self.page {
            Page::Browser => {
                let max = if self.active_panel == ActivePanel::Left {
                    match self.left_section {
                        LeftSection::Favorites => self.favorites_dirs.len(),
                        LeftSection::Recents => self.recents_dirs.len(),
                    }
                } else if self.search_mode.is_active() {
                    self.search_mode.match_count()
                } else {
                    self.entries.len()
                };
                if max > 0 {
                    self.selected_index = if self.selected_index > 0 {
                        self.selected_index - 1
                    } else {
                        max - 1
                    };
                    // Clamp to valid range
                    if self.selected_index >= max {
                        self.selected_index = max.saturating_sub(1);
                    }
                }
            }
            Page::ToolSelection => {
                let max = if self.active_panel == ActivePanel::Left {
                    match self.tool_left_section {
                        LeftSection::Favorites => self.favorites_tools.len(),
                        LeftSection::Recents => self.recents_tools.len(),
                    }
                } else {
                    self.tools.len()
                };
                if max > 0 {
                    self.selected_tool_index = if self.selected_tool_index > 0 {
                        self.selected_tool_index - 1
                    } else {
                        max - 1
                    };
                    // Clamp to valid range
                    if self.selected_tool_index >= max {
                        self.selected_tool_index = max.saturating_sub(1);
                    }
                }
            }
            Page::Provider => {
                let max = if self.active_panel == ActivePanel::Left {
                    match self.provider_left_section {
                        LeftSection::Favorites => self.favorites_providers.len(),
                        LeftSection::Recents => self.recents_providers.len(),
                    }
                } else {
                    self.providers.len()
                };
                if max > 0 {
                    self.selected_provider_index = if self.selected_provider_index > 0 {
                        self.selected_provider_index - 1
                    } else {
                        max - 1
                    };
                    // Clamp to valid range
                    if self.selected_provider_index >= max {
                        self.selected_provider_index = max.saturating_sub(1);
                    }
                }
            }
            Page::Model => {
                let max = if self.active_panel == ActivePanel::Left {
                    match self.model_left_section {
                        LeftSection::Favorites => self.favorites_models.len(),
                        LeftSection::Recents => self.recents_models.len(),
                    }
                } else {
                    self.models.len()
                };
                if max > 0 {
                    self.selected_model_index = if self.selected_model_index > 0 {
                        self.selected_model_index - 1
                    } else {
                        max - 1
                    };
                    // Clamp to valid range
                    if self.selected_model_index >= max {
                        self.selected_model_index = max.saturating_sub(1);
                    }
                }
            }
        }
    }

    fn handle_down(&mut self) {
        match self.page {
            Page::Browser => {
                let max = if self.active_panel == ActivePanel::Left {
                    match self.left_section {
                        LeftSection::Favorites => self.favorites_dirs.len(),
                        LeftSection::Recents => self.recents_dirs.len(),
                    }
                } else if self.search_mode.is_active() {
                    self.search_mode.match_count()
                } else {
                    self.entries.len()
                };
                if max > 0 {
                    self.selected_index = (self.selected_index + 1) % max;
                }
            }
            Page::ToolSelection => {
                let max = if self.active_panel == ActivePanel::Left {
                    match self.tool_left_section {
                        LeftSection::Favorites => self.favorites_tools.len(),
                        LeftSection::Recents => self.recents_tools.len(),
                    }
                } else {
                    self.tools.len()
                };
                if max > 0 {
                    self.selected_tool_index = (self.selected_tool_index + 1) % max;
                }
            }
            Page::Provider => {
                let max = if self.active_panel == ActivePanel::Left {
                    match self.provider_left_section {
                        LeftSection::Favorites => self.favorites_providers.len(),
                        LeftSection::Recents => self.recents_providers.len(),
                    }
                } else {
                    self.providers.len()
                };
                if max > 0 {
                    self.selected_provider_index = (self.selected_provider_index + 1) % max;
                }
            }
            Page::Model => {
                let max = if self.active_panel == ActivePanel::Left {
                    match self.model_left_section {
                        LeftSection::Favorites => self.favorites_models.len(),
                        LeftSection::Recents => self.recents_models.len(),
                    }
                } else {
                    self.models.len()
                };
                if max > 0 {
                    self.selected_model_index = (self.selected_model_index + 1) % max;
                }
            }
        }
    }

    fn handle_open(&mut self, terminal: &mut ratatui::DefaultTerminal) {
        match self.page {
            Page::Browser => {
                if self.active_panel == ActivePanel::Right && self.selected_index < self.entries.len() {
                    let entry = &self.entries[self.selected_index];
                    if entry.is_dir {
                        let path = entry.path.clone();
                        self.navigate_to_dir(&path);
                    }
                }
            }
            Page::ToolSelection | Page::Provider | Page::Model => {
                self.handle_enter(terminal);
            }
        }
    }

    fn handle_back(&mut self) {
        match self.page {
            Page::Browser => {
                if self.active_panel == ActivePanel::Right {
                    if let Some(parent) = self.current_dir.parent() {
                        let parent = parent.to_path_buf();
                        self.navigate_to_dir(&parent);
                    }
                } else {
                    self.active_panel = ActivePanel::Right;
                }
            }
            Page::ToolSelection | Page::Provider | Page::Model => {
                if self.active_panel == ActivePanel::Left {
                    self.active_panel = ActivePanel::Right;
                } else {
                    self.go_back();
                }
            }
        }
    }

    fn handle_select(&mut self, terminal: &mut ratatui::DefaultTerminal) {
        match self.page {
            Page::Browser => {
                if self.active_panel == ActivePanel::Left {
                    match self.left_section {
                        LeftSection::Favorites => {
                            if self.selected_index < self.favorites_dirs.len() {
                                let path = self.favorites_dirs[self.selected_index].clone();
                                self.navigate_to_dir(&path);
                            }
                        }
                        LeftSection::Recents => {
                            if self.selected_index < self.recents_dirs.len() {
                                let path = self.recents_dirs[self.selected_index].clone();
                                self.navigate_to_dir(&path);
                            }
                        }
                    }
                } else {
                    // Right panel - check if highlighting a directory entry
                    if self.selected_index < self.entries.len() {
                        let entry = &self.entries[self.selected_index];
                        if entry.is_dir {
                            // Selected directory - set it and advance to tool selection
                            self.selected_dir = Some(entry.path.clone());
                            self.advance_page();
                        } else {
                            // Selected a file - set as current directory and advance
                            self.selected_dir = Some(self.current_dir.clone());
                            self.advance_page();
                        }
                    } else {
                        // No valid selection - use current directory and advance
                        self.selected_dir = Some(self.current_dir.clone());
                        self.advance_page();
                    }
                }
            }
            Page::ToolSelection | Page::Provider | Page::Model => {
                self.handle_enter(terminal);
            }
        }
    }

    fn handle_enter(&mut self, terminal: &mut ratatui::DefaultTerminal) {
        match self.page {
            Page::Browser => {
                self.selected_dir = Some(self.current_dir.clone());
                self.advance_page();
            }
            Page::ToolSelection => {
                if self.selected_tool_index < self.tools.len() {
                    let tool_name = self.tools[self.selected_tool_index].clone();
                    if let Some(tool_info) = find_tool_by_display_name(&tool_name) {
                        if !tools::check_tool_installed(tool_info) {
                            self.dialog = Dialog::ToolNotInstalled {
                                tool_name: tool_name.clone(),
                            };
                            return;
                        }
                        self.add_to_recents_tools(&tool_name);
                        self.selected_tool = Some(tool_name.clone());
                        if tool_info.needs_provider_selection {
                            self.advance_page();
                        } else {
                            // Force a redraw so the Opening overlay is visible
                            terminal.draw(|frame| crate::ui::render(self, frame)).ok();
                            self.launch_selected_tool(terminal);
                        }
                    }
                }
            }
            Page::Provider => {
                if self.selected_provider_index < self.providers.len() {
                    let provider = self.providers[self.selected_provider_index].clone();
                    self.add_to_recents_providers(&provider);
                    self.selected_provider = Some(provider);
                    self.advance_page();
                    self.start_model_loading();
                }
            }
            Page::Model => {
                if !self.models_loading && self.selected_model_index < self.models.len() {
                    let model = self.models[self.selected_model_index].clone();
                    self.add_to_recents_models(&model);
                    // Force a redraw so the Opening overlay is visible
                    terminal.draw(|frame| crate::ui::render(self, frame)).ok();
                    self.launch_selected_tool(terminal);
                }
            }
        }
    }

    fn add_to_recents_tools(&mut self, tool: &str) {
        self.recents_tools.retain(|t| t != tool);
        self.recents_tools.insert(0, tool.to_string());
        if self.recents_tools.len() > 10 {
            self.recents_tools.pop();
        }
        self.save_recents_to_config();
    }

    fn add_to_recents_providers(&mut self, provider: &str) {
        self.recents_providers.retain(|p| p != provider);
        self.recents_providers.insert(0, provider.to_string());
        if self.recents_providers.len() > 10 {
            self.recents_providers.pop();
        }
        self.save_recents_to_config();
    }

    fn add_to_recents_models(&mut self, model: &str) {
        self.recents_models.retain(|m| m != model);
        self.recents_models.insert(0, model.to_string());
        if self.recents_models.len() > 10 {
            self.recents_models.pop();
        }
        self.save_recents_to_config();
    }

    fn start_model_loading(&mut self) {
        self.models_loading = true;
        self.models.clear();

        // For GitHub Copilot, show profiles instead of models
        if let Some(ref provider) = self.selected_provider {
            if provider == "GitHub Copilot" {
                self.models = vec!["Claude".to_string(), "OpenAI".to_string()];
            } else {
                self.models = STUB_MODELS.iter().map(|m| m.to_string()).collect();
            }
        } else {
            self.models = STUB_MODELS.iter().map(|m| m.to_string()).collect();
        }

        self.models_loading = false;
        self.selected_model_index = 0;
    }

    pub fn launch_selected_tool(&mut self, terminal: &mut ratatui::DefaultTerminal) -> bool {
        let tool_name = match &self.selected_tool {
            Some(name) => name.clone(),
            None => return false,
        };
        let dir = match &self.selected_dir {
            Some(d) => d.clone(),
            None => self.current_dir.clone(),
        };
        let tool_info = match find_tool_by_display_name(&tool_name) {
            Some(t) => t,
            None => return false,
        };

        // Show "Opening..." dialog
        self.dialog = Dialog::Opening {
            tool_name: tool_name.clone(),
        };

        tools::prepare_for_launch();
        let result = tools::launch_tool(
            tool_info,
            &dir,
            self.selected_provider.as_deref(),
            self.models.get(self.selected_model_index).map(|s| s.as_str()),
        );
        tools::restore_after_launch();

        // Clear the dialog after launch
        self.dialog = Dialog::None;

        // Reset to fresh startup state after returning from tool
        self.reset_to_homepage();

        // CRITICAL: Clear terminal and force full redraw as recommended by ratatui docs
        // This ensures the terminal buffer is completely invalidated after child process
        let _ = terminal.clear();
        terminal.draw(|frame| crate::ui::render(self, frame)).ok();

        match result {
            LaunchResult::Success => {
                true
            }
            LaunchResult::ToolNotInstalled(name) => {
                self.dialog = Dialog::ToolNotInstalled { tool_name: name };
                false
            }
            LaunchResult::LaunchFailed(msg) => {
                self.dialog = Dialog::Error { message: msg };
                false
            }
        }
    }

    /// Reset the application to homepage/startup state
    fn reset_to_homepage(&mut self) {
        // Reset to Browser page (homepage)
        self.page = Page::Browser;

        // Clear all tool/provider/model selections
        self.selected_tool = None;
        self.selected_provider = None;
        self.models.clear();

        // Reset all indices and panels
        self.selected_index = 0;
        self.selected_tool_index = 0;
        self.selected_provider_index = 0;
        self.selected_model_index = 0;

        // Reset active panel to right (main content)
        self.active_panel = ActivePanel::Right;

        // Reset left sections to Favorites
        self.left_section = LeftSection::Favorites;
        self.tool_left_section = LeftSection::Favorites;
        self.provider_left_section = LeftSection::Favorites;
        self.model_left_section = LeftSection::Favorites;

        // Clear any loading states
        self.models_loading = false;
        self.models_error = None;

        // Clear search mode
        self.search_mode = SearchMode::Inactive;

        // Clear any errors
        self.error = None;

        // Reset quit confirmation
        self.quit_confirm = 0;
        self.quit_timer = None;

        // Ensure dialog is cleared
        self.dialog = Dialog::None;
        self.dialog_selection = 0;

        // Ensure settings are closed
        self.settings_open = false;
        self.settings_selection = 0;
    }

    fn navigate_to_dir(&mut self, path: &PathBuf) {
        if !self.recents_dirs.contains(path) {
            self.recents_dirs.insert(0, path.clone());
            if self.recents_dirs.len() > 10 {
                self.recents_dirs.pop();
            }
            self.save_recents_to_config();
        }

        self.current_dir = path.clone();
        let dir_entries = load_dir_entries(path);
        self.entries = dir_entries.entries;
        self.error = dir_entries.error;
        self.selected_index = 0;
    }

    // Search methods

    /// Activate search mode for the current page
    pub fn activate_search(&mut self) {
        // Only activate search when on the right panel
        if self.active_panel != ActivePanel::Right {
            return;
        }

        let names = self.get_current_list_names();
        self.search_mode = SearchMode::Active {
            query: String::new(),
            filtered_indices: (0..names.len()).collect(), // All items initially
            current_match_index: 0,
        };
        self.search_typing_mode = true;  // Start in typing mode
    }

    /// Update the search query with a new character
    pub fn update_search_query(&mut self, c: char) {
        if let SearchMode::Active { query, .. } = &mut self.search_mode {
            query.push(c);
            let new_query = query.clone();
            let names = self.get_current_list_names();
            let filtered_indices = filter_entries(&names, &new_query);

            self.search_mode = SearchMode::Active {
                query: new_query,
                filtered_indices,
                current_match_index: 0,
            };

            // Update selection to first match
            self.update_selection_from_search();
        }
    }

    /// Remove last character from search query
    pub fn search_backspace(&mut self) {
        if let SearchMode::Active { query, .. } = &mut self.search_mode {
            query.pop();
            let new_query = query.clone();
            let names = self.get_current_list_names();
            let filtered_indices = filter_entries(&names, &new_query);

            self.search_mode = SearchMode::Active {
                query: new_query,
                filtered_indices,
                current_match_index: 0,
            };

            // Update selection to first match
            self.update_selection_from_search();
        }
    }

    /// Move to the next search match
    pub fn search_next_match(&mut self) {
        if let SearchMode::Active {
            filtered_indices,
            current_match_index,
            query,
        } = &self.search_mode
        {
            if filtered_indices.is_empty() {
                return;
            }
            let new_index = if *current_match_index + 1 < filtered_indices.len() {
                *current_match_index + 1
            } else {
                0 // Wrap around
            };

            self.search_mode = SearchMode::Active {
                query: query.clone(),
                filtered_indices: filtered_indices.clone(),
                current_match_index: new_index,
            };

            self.update_selection_from_search();
        }
    }

    /// Move to the previous search match
    pub fn search_prev_match(&mut self) {
        if let SearchMode::Active {
            filtered_indices,
            current_match_index,
            query,
        } = &self.search_mode
        {
            if filtered_indices.is_empty() {
                return;
            }
            let new_index = if *current_match_index > 0 {
                *current_match_index - 1
            } else {
                filtered_indices.len() - 1 // Wrap around
            };

            self.search_mode = SearchMode::Active {
                query: query.clone(),
                filtered_indices: filtered_indices.clone(),
                current_match_index: new_index,
            };

            self.update_selection_from_search();
        }
    }

    /// Confirm the current search selection and exit search mode
    pub fn confirm_search(&mut self) {
        // Selection is already updated, just exit search mode
        self.search_mode = SearchMode::Inactive;
    }

    /// Exit search mode without changing selection
    pub fn exit_search(&mut self) {
        self.search_mode = SearchMode::Inactive;
        self.search_typing_mode = false;
    }

    /// Get the names of items in the current list view
    fn get_current_list_names(&self) -> Vec<String> {
        match self.page {
            Page::Browser => self.entries.iter().map(|e| e.name.clone()).collect(),
            Page::ToolSelection => self.tools.clone(),
            Page::Provider => self.providers.clone(),
            Page::Model => self.models.clone(),
        }
    }

    /// Update keybinds based on preset name
    fn update_keybinds_for_preset(&mut self, preset: &str) {
        match preset {
            "wasd" => self.settings.keybinds = crate::config::Keybinds::wasd_preset(),
            "vim" => self.settings.keybinds = crate::config::Keybinds::vim_preset(),
            _ => {
                // Check if it's a custom preset
                if let Some(keybinds) = self.settings.custom_presets.get(preset) {
                    self.settings.keybinds = keybinds.clone();
                }
            }
        }
    }

    /// Validate if a string is a valid hex color code
    fn is_valid_hex_color(&self, hex: &str) -> bool {
        if !hex.starts_with('#') || hex.len() != 7 {
            return false;
        }
        hex.chars().skip(1).all(|c| c.is_ascii_hexdigit())
    }

    /// Open the command bar dialog
    fn open_command_bar(&mut self) {
        use crate::search::filter_commands_fuzzy;

        let filtered = filter_commands_fuzzy(
            COMMANDS.iter().enumerate().map(|(i, c)| (i, c.name)),
            "",
        );
        self.dialog = Dialog::CommandBar {
            query: String::new(),
            filtered_indices: filtered,
            selected_index: 0,
        };
    }

    /// Filter commands based on current query
    fn filter_commands(&mut self) {
        use crate::search::filter_commands_fuzzy;

        if let Dialog::CommandBar { query, filtered_indices, selected_index } = &mut self.dialog {
            let filtered = filter_commands_fuzzy(
                COMMANDS.iter().enumerate().map(|(i, c)| (i, c.name)),
                query,
            );
            *filtered_indices = filtered;
            *selected_index = 0;

            // Clamp selected_index to valid range
            if !filtered_indices.is_empty() && *selected_index >= filtered_indices.len() {
                *selected_index = filtered_indices.len().saturating_sub(1);
            }
        }
    }

    /// Execute a command by index
    fn execute_command(&mut self, cmd_idx: usize) {
        self.dialog = Dialog::None;
        match cmd_idx {
            0 => {
                // providerconf
                self.dialog = Dialog::ProviderConfig { selected_index: 0 };
            }
            1 => {
                // keybindconf
                self.dialog = Dialog::KeybindConfig {
                    selected_index: 0,
                    editing_field: None,
                };
            }
            2 => {
                // env
                self.dialog = Dialog::EnvConfig {
                    entries: Vec::new(),
                    selected_index: 0,
                    editing_field: None,
                    input_buffer: String::new(),
                };
            }
            3 => {
                // settings
                self.dialog = Dialog::SettingsConfig { selected_index: 0 };
            }
            _ => {}
        }
    }

    /// Update the selected index based on the current search match
    fn update_selection_from_search(&mut self) {
        if let Some(idx) = self.search_mode.current_match() {
            match self.page {
                Page::Browser => {
                    if idx < self.entries.len() {
                        self.selected_index = idx;
                    }
                }
                Page::ToolSelection => {
                    if idx < self.tools.len() {
                        self.selected_tool_index = idx;
                    }
                }
                Page::Provider => {
                    if idx < self.providers.len() {
                        self.selected_provider_index = idx;
                    }
                }
                Page::Model => {
                    if idx < self.models.len() {
                        self.selected_model_index = idx;
                    }
                }
            }
        }
    }

    /// Get the current search target directory if search is active
    fn get_current_search_directory(&self) -> Option<PathBuf> {
        // Only on Browser page with right panel active
        if self.page != Page::Browser || self.active_panel != ActivePanel::Right {
            return None;
        }

        // Get current search match index
        let current_idx = self.search_mode.current_match()?;

        // Check if it's a valid directory entry
        if current_idx < self.entries.len() {
            let entry = &self.entries[current_idx];
            if entry.is_dir {
                Some(entry.path.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Handle directory selection during search mode
    /// Returns true if selection was performed, false if should fall back to normal behavior
    fn handle_search_selection(&mut self) -> bool {
        if let Some(dir_path) = self.get_current_search_directory() {
            // Set this as the selected directory
            if self.page == Page::Browser && self.active_panel == ActivePanel::Right {
                // Find the index in entries and set selected_index
                if let Some(idx) = self.entries.iter().position(|e| e.path == dir_path) {
                    self.selected_index = idx;
                    return true;
                }
            }
        }
        false
    }
}
