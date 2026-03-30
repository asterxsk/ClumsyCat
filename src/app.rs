use crate::config::{Config, Settings};
use crate::fs::load_dir_entries;
use crate::search::SearchMode;
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
}

impl App {
    pub fn new() -> Self {
        // Load configuration
        let config = Config::load();

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

        // Default tool list
        let tools = vec![
            "Claude Code".to_string(),
            "Aider".to_string(),
            "Codex".to_string(),
            "Goose".to_string(),
        ];

        // Default provider list
        let providers = vec![
            "Anthropic".to_string(),
            "OpenAI".to_string(),
            "Google".to_string(),
            "OpenRouter".to_string(),
            "Ollama".to_string(),
        ];

        Self {
            // Global state
            page: Page::default(),
            dialog: Dialog::default(),
            search_mode: SearchMode::Inactive,
            settings: config.settings,
            previous_page: None,

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
                    match key.code {
                        KeyCode::Char('d') | KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.handle_ctrl_d();
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
                            eprintln!("TODO: Search mode");
                        }
                        _ => {}
                    }
                }
            }
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
}
