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
}

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
                    // Map vim keys if nav_mode is "vim"
                    let code = self.map_vim_key(key.code);

                    // Handle dialog input first (highest priority)
                    if self.dialog != Dialog::None {
                        self.handle_dialog_input(code, key.modifiers);
                        continue;
                    }

                    // Handle settings overlay input
                    if self.settings_open {
                        self.handle_settings_input(code);
                        continue;
                    }

                    // Handle search mode input
                    if self.search_mode.is_active() {
                        match code {
                            KeyCode::Esc => {
                                self.exit_search();
                            }
                            KeyCode::Enter => {
                                self.confirm_search();
                            }
                            KeyCode::Backspace => {
                                self.search_backspace();
                            }
                            KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                                self.search_prev_match();
                            }
                            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                                self.search_next_match();
                            }
                            KeyCode::Char(c) => {
                                self.update_search_query(c);
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // Normal mode input handling
                    match code {
                        KeyCode::Char('d') | KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.handle_ctrl_d();
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
                        KeyCode::Tab => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.active_panel = match self.active_panel {
                                ActivePanel::Left => ActivePanel::Right,
                                ActivePanel::Right => ActivePanel::Left,
                            };
                        }
                        KeyCode::Esc => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.active_panel = match self.active_panel {
                                ActivePanel::Left => ActivePanel::Right,
                                ActivePanel::Right => ActivePanel::Left,
                            };
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
                            self.handle_open();
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.handle_back();
                        }
                        KeyCode::Char(' ') => {
                            self.quit_confirm = 0;
                            self.quit_timer = None;
                            self.handle_select();
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
    }

    /// Map vim keys (k/j/h/l) to navigation keys when in vim mode
    fn map_vim_key(&self, code: KeyCode) -> KeyCode {
        use ratatui::crossterm::event::KeyCode;
        if self.settings.nav_mode == "vim" {
            match code {
                KeyCode::Char('k') | KeyCode::Char('K') => KeyCode::Char('w'),
                KeyCode::Char('j') | KeyCode::Char('J') => KeyCode::Char('s'),
                KeyCode::Char('h') | KeyCode::Char('H') => KeyCode::Char('a'),
                KeyCode::Char('l') | KeyCode::Char('L') => KeyCode::Char('d'),
                other => other,
            }
        } else {
            code
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
            Dialog::ToolNotInstalled { .. } | Dialog::Error { .. } => {
                match code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.dialog = Dialog::None;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Handle input when settings overlay is open
    fn handle_settings_input(&mut self, code: ratatui::crossterm::event::KeyCode) {
        use ratatui::crossterm::event::KeyCode;

        let accent_colors = ["orange", "blue", "green", "red", "yellow", "magenta", "cyan"];
        let nav_modes = ["arrow", "vim"];

        match code {
            KeyCode::Esc => {
                // Save and close settings
                self.config.settings = self.settings.clone();
                let _ = self.config.save();
                self.settings_open = false;
            }
            KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                if self.settings_selection > 0 {
                    self.settings_selection -= 1;
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                if self.settings_selection < 1 {
                    self.settings_selection += 1;
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right | KeyCode::Enter => {
                // Cycle through options
                if self.settings_selection == 0 {
                    // Accent color
                    let current_idx = accent_colors
                        .iter()
                        .position(|&c| c == self.settings.accent_color)
                        .unwrap_or(0);
                    let next_idx = (current_idx + 1) % accent_colors.len();
                    self.settings.accent_color = accent_colors[next_idx].to_string();
                } else {
                    // Nav mode
                    let current_idx = nav_modes
                        .iter()
                        .position(|&m| m == self.settings.nav_mode)
                        .unwrap_or(0);
                    let next_idx = (current_idx + 1) % nav_modes.len();
                    self.settings.nav_mode = nav_modes[next_idx].to_string();
                }
            }
            KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                // Cycle backwards through options
                if self.settings_selection == 0 {
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
                    self.settings.accent_color = accent_colors[next_idx].to_string();
                } else {
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
                    self.settings.nav_mode = nav_modes[next_idx].to_string();
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

    fn handle_ctrl_d(&mut self) {
        let now = Instant::now();
        if let Some(timer) = self.quit_timer {
            if now.duration_since(timer) < Duration::from_secs(1) {
                self.quit_confirm += 1;
                if self.quit_confirm >= 2 {
                    std::process::exit(0);
                }
            } else {
                self.quit_confirm = 1;
                self.quit_timer = Some(now);
            }
        } else {
            self.quit_confirm = 1;
            self.quit_timer = Some(now);
        }
    }

    fn handle_up(&mut self) {
        // Only handle browser page navigation for now
        if self.page != Page::Browser {
            return;
        }

        if self.active_panel == ActivePanel::Left {
            self.left_section = match self.left_section {
                LeftSection::Favorites => LeftSection::Recents,
                LeftSection::Recents => LeftSection::Favorites,
            };
        } else if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn handle_down(&mut self) {
        // Only handle browser page navigation for now
        if self.page != Page::Browser {
            return;
        }

        if self.active_panel == ActivePanel::Left {
            self.left_section = match self.left_section {
                LeftSection::Favorites => LeftSection::Recents,
                LeftSection::Recents => LeftSection::Favorites,
            };
        } else {
            let max = self.entries.len();
            if self.selected_index + 1 < max {
                self.selected_index += 1;
            }
        }
    }

    fn handle_open(&mut self) {
        // Only handle browser page navigation for now
        if self.page != Page::Browser {
            return;
        }

        if self.active_panel == ActivePanel::Right && self.selected_index < self.entries.len() {
            let entry = &self.entries[self.selected_index];
            if entry.is_dir {
                let path = entry.path.clone();
                self.navigate_to_dir(&path);
            }
        }
    }

    fn handle_back(&mut self) {
        // Only handle browser page navigation for now
        if self.page != Page::Browser {
            return;
        }

        if self.active_panel == ActivePanel::Right {
            if let Some(parent) = self.current_dir.parent() {
                let parent = parent.to_path_buf();
                self.navigate_to_dir(&parent);
            }
        } else {
            self.active_panel = ActivePanel::Right;
        }
    }

    fn handle_select(&mut self) {
        // Only handle browser page navigation for now
        if self.page != Page::Browser {
            return;
        }

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
            eprintln!("TODO: Select current entry");
        }
    }

    fn navigate_to_dir(&mut self, path: &PathBuf) {
        if !self.recents_dirs.contains(path) {
            self.recents_dirs.insert(0, path.clone());
            if self.recents_dirs.len() > 10 {
                self.recents_dirs.pop();
            }
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
}
