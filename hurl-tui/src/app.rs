//! Application state management
//!
//! This module contains the core application state and logic for the Hurl TUI.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::widgets::ListState;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::Config;
use crate::effects::{presets, EffectId, EffectManager};
use crate::parser::HurlFile;
use crate::runner::{ExecutionResult, Runner};
use crate::ui::ResponseTab;

/// Directories to skip when scanning for .hurl files
/// These are common build output, dependency, and cache directories
const IGNORED_DIRECTORIES: &[&str] = &[
    // Node.js / JavaScript
    "node_modules",
    "bower_components",
    ".npm",
    ".yarn",
    ".pnpm-store",
    // Rust
    "target",
    // .NET / C#
    "bin",
    "obj",
    "packages",
    // Python
    "__pycache__",
    ".venv",
    "venv",
    ".env",
    "env",
    ".tox",
    ".pytest_cache",
    ".mypy_cache",
    // Java / Kotlin / Gradle / Maven
    "build",
    "out",
    ".gradle",
    ".mvn",
    // Go
    "vendor",
    // PHP
    "vendor",
    // Ruby
    ".bundle",
    // General IDE / Editor
    ".idea",
    ".vscode",
    ".vs",
    // Version control
    ".git",
    ".svn",
    ".hg",
    // OS generated
    ".DS_Store",
    "Thumbs.db",
    // Coverage / Test output
    "coverage",
    ".nyc_output",
    "htmlcov",
    // Docker
    ".docker",
    // Misc caches
    ".cache",
    ".parcel-cache",
    ".next",
    ".nuxt",
    "dist",
];

/// Serializable state for persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct PersistedState {
    /// Last opened file path
    last_opened_file: Option<String>,
    /// File tree index
    file_tree_index: usize,
    /// Execution results per file (keyed by relative path)
    #[serde(default)]
    file_execution_states: HashMap<String, ExecutionResult>,
    /// Last selected environment
    #[serde(default)]
    selected_environment: Option<String>,
    /// Expanded folder paths (relative to working directory)
    #[serde(default)]
    expanded_folders: Vec<String>,
    /// Sidebar width percentage (default 20)
    #[serde(default = "default_sidebar_width")]
    sidebar_width: u16,
}

/// Default sidebar width percentage
fn default_sidebar_width() -> u16 {
    20
}

/// Active panel in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ActivePanel {
    #[default]
    FileBrowser,
    Editor,
    Response,
    Assertions,
    Variables,
}

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    Normal,
    Editing,
    Search,
    Command,
    Filter,
    Rename,
}

/// Vim sub-mode when in Editing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimMode {
    #[default]
    Normal, // Navigation and commands (hjkl, etc.)
    Insert, // Text input mode
}

/// File tree entry
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub depth: usize,
    pub children: Vec<FileEntry>,
}

impl FileEntry {
    pub fn new(path: PathBuf, depth: usize) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let is_dir = path.is_dir();

        Self {
            path,
            name,
            is_dir,
            is_expanded: false,
            depth,
            children: Vec::new(),
        }
    }
}

/// Variable entry
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub is_secret: bool,
}

/// History entry for executed requests
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: uuid::Uuid,
    pub file_path: PathBuf,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub status_code: Option<u16>,
    pub success: bool,
}

/// Status message level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Main application state
pub struct App {
    /// Application configuration
    pub config: Config,

    /// Working directory
    pub working_dir: PathBuf,

    /// Whether the application should quit
    quit: bool,

    /// Currently active panel
    pub active_panel: ActivePanel,

    /// Current application mode
    pub mode: AppMode,

    /// Vim sub-mode (when in Editing mode)
    pub vim_mode: VimMode,

    /// File tree entries
    pub file_tree: Vec<FileEntry>,

    /// Index of selected file in the flattened tree
    pub file_tree_index: usize,

    /// List state for file browser scrolling
    pub file_tree_state: ListState,

    /// Currently open hurl file
    pub current_file: Option<HurlFile>,

    /// Current file path
    pub current_file_path: Option<PathBuf>,

    /// Editor content (lines)
    pub editor_content: Vec<String>,

    /// Editor cursor position (line, column)
    pub editor_cursor: (usize, usize),

    /// Editor scroll offset
    pub editor_scroll: usize,

    /// Last execution result
    pub execution_result: Option<ExecutionResult>,

    /// Execution results per file (keyed by relative path from working_dir)
    file_execution_states: HashMap<String, ExecutionResult>,

    /// Whether a request is currently running
    pub is_running: bool,

    /// Spinner animation frame (for progress indicator)
    pub spinner_frame: usize,

    /// Variables for the current environment
    pub variables: Vec<Variable>,

    /// Current environment name
    pub current_environment: String,

    /// Available environments
    pub environments: Vec<String>,

    /// Current environment file path (for --variables-file)
    pub current_env_file: Option<PathBuf>,

    /// Request history
    pub history: Vec<HistoryEntry>,

    /// History selection index
    pub history_index: usize,

    /// Search query
    pub search_query: String,

    /// Filter query for file browser
    pub filter_query: String,

    /// Command input
    pub command_input: String,

    /// Status message to display
    pub status_message: Option<(String, StatusLevel)>,

    /// Runner instance
    runner: Runner,

    /// Response scroll offset
    pub response_scroll: usize,

    /// Assertions scroll offset
    pub assertions_scroll: usize,

    /// Show help overlay
    pub show_help: bool,

    /// Output to print to stdout after quitting (for pipe support)
    output: Option<String>,

    /// Internal clipboard for file copy/paste operations in the file browser.
    /// Stores the absolute path of the source file when user presses 'p' to copy.
    /// Used by paste operation ('P') to duplicate the file to target location.
    /// Note: This is separate from the system clipboard used for text copying.
    pub clipboard_file: Option<PathBuf>,

    /// Input buffer for rename operation.
    /// Contains the new filename being typed by the user during rename mode.
    pub rename_input: String,

    /// The file path being renamed.
    /// Stores the original path of the file when user initiates rename with 'n'.
    rename_target: Option<PathBuf>,

    /// Effect manager for animations
    pub effect_manager: EffectManager,

    /// Previous active panel (for detecting panel changes)
    previous_panel: ActivePanel,

    /// Previous help state (for detecting help toggle)
    previous_show_help: bool,

    /// Current response tab (Body, Headers, Raw)
    pub response_tab: ResponseTab,

    /// Sidebar width percentage (10-50)
    pub sidebar_width: u16,
}

impl App {
    /// Create a new application instance
    pub fn new(config: Config, working_dir: PathBuf) -> Result<Self> {
        let mut app = Self {
            config,
            working_dir: working_dir.clone(),
            quit: false,
            active_panel: ActivePanel::FileBrowser,
            mode: AppMode::Normal,
            vim_mode: VimMode::Normal,
            file_tree: Vec::new(),
            file_tree_index: 0,
            file_tree_state: ListState::default().with_selected(Some(0)),
            current_file: None,
            current_file_path: None,
            editor_content: Vec::new(),
            editor_cursor: (0, 0),
            editor_scroll: 0,
            execution_result: None,
            file_execution_states: HashMap::new(),
            is_running: false,
            spinner_frame: 0,
            variables: Vec::new(),
            current_environment: String::new(),
            environments: Vec::new(),
            current_env_file: None,
            history: Vec::new(),
            history_index: 0,
            search_query: String::new(),
            filter_query: String::new(),
            command_input: String::new(),
            status_message: None,
            runner: Runner::new(),
            response_scroll: 0,
            assertions_scroll: 0,
            show_help: false,
            output: None,
            clipboard_file: None,
            rename_input: String::new(),
            rename_target: None,
            effect_manager: EffectManager::new(),
            previous_panel: ActivePanel::FileBrowser,
            previous_show_help: false,
            response_tab: ResponseTab::Body,
            sidebar_width: default_sidebar_width(),
        };

        // Load file tree and restore state (including expanded folders and sidebar width)
        app.load_file_tree_and_restore_state()?;

        // Load environments
        app.load_environments()?;

        // Restore selected environment from persisted state (must happen after load_environments)
        app.restore_selected_environment();

        Ok(app)
    }

    /// Restore selected environment from persisted state
    fn restore_selected_environment(&mut self) {
        let state_path = self.get_state_file_path();
        let persisted_state = std::fs::read_to_string(&state_path)
            .ok()
            .and_then(|content| serde_json::from_str::<PersistedState>(&content).ok());

        if let Some(state) = persisted_state {
            if let Some(ref env) = state.selected_environment {
                if self.environments.contains(env) {
                    self.current_environment = env.clone();
                    let _ = self.load_current_environment_variables();
                }
            }
        }
    }

    /// Get the state file path for the current working directory
    fn get_state_file_path(&self) -> PathBuf {
        self.working_dir.join(".hurl-tui-state.json")
    }

    /// Get the relative path for a file (used as key for file_execution_states)
    fn get_relative_path(&self, path: &PathBuf) -> String {
        path.strip_prefix(&self.working_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    /// Collect all expanded folder paths from the file tree
    fn collect_expanded_folders(&self) -> Vec<String> {
        let mut expanded = Vec::new();
        Self::collect_expanded_recursive(&self.file_tree, &self.working_dir, &mut expanded);
        expanded
    }

    /// Recursively collect expanded folder paths
    fn collect_expanded_recursive(
        entries: &[FileEntry],
        working_dir: &PathBuf,
        expanded: &mut Vec<String>,
    ) {
        for entry in entries {
            if entry.is_dir && entry.is_expanded {
                let relative = entry
                    .path
                    .strip_prefix(working_dir)
                    .unwrap_or(&entry.path)
                    .to_string_lossy()
                    .to_string();
                expanded.push(relative);
                Self::collect_expanded_recursive(&entry.children, working_dir, expanded);
            }
        }
    }

    /// Save the current state (last opened file and execution states)
    fn save_state(&self) {
        let state = PersistedState {
            last_opened_file: self
                .current_file_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            file_tree_index: self.file_tree_index,
            file_execution_states: self.file_execution_states.clone(),
            selected_environment: if self.current_environment.is_empty() {
                None
            } else {
                Some(self.current_environment.clone())
            },
            expanded_folders: self.collect_expanded_folders(),
            sidebar_width: self.sidebar_width,
        };

        if let Ok(content) = serde_json::to_string_pretty(&state) {
            let _ = std::fs::write(self.get_state_file_path(), content);
        }
    }

    /// Load file tree and restore all persisted state
    fn load_file_tree_and_restore_state(&mut self) -> Result<()> {
        let state_path = self.get_state_file_path();
        let persisted_state = std::fs::read_to_string(&state_path)
            .ok()
            .and_then(|content| serde_json::from_str::<PersistedState>(&content).ok());

        // Load file tree
        self.file_tree = Self::load_directory_children(&self.working_dir, 0)?;

        // Restore expanded folders if we have persisted state, otherwise auto-expand
        if let Some(ref state) = persisted_state {
            if !state.expanded_folders.is_empty() {
                // Restore saved expanded state
                self.restore_expanded_folders(&state.expanded_folders);
            } else {
                // No saved state, auto-expand directories with .hurl files
                self.auto_expand_hurl_directories();
            }

            // Restore file execution states
            self.file_execution_states = state.file_execution_states.clone();

            // Restore sidebar width
            self.sidebar_width = state.sidebar_width.clamp(10, 50);

            // Note: Environment restoration happens in restore_selected_environment()
            // after load_environments() populates the environments list

            // Restore last opened file
            if let Some(ref file_path) = state.last_opened_file {
                let path = PathBuf::from(file_path);
                if path.exists() {
                    let _ = self.preview_file(&path);

                    // Try to restore file tree index
                    let max = self.get_visible_file_count().saturating_sub(1);
                    self.file_tree_index = state.file_tree_index.min(max);
                    self.file_tree_state.select(Some(self.file_tree_index));

                    self.set_status(
                        &format!(
                            "Restored: {}",
                            path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default()
                        ),
                        StatusLevel::Info,
                    );
                }
            }
        } else {
            // No persisted state, auto-expand directories with .hurl files
            self.auto_expand_hurl_directories();
        }

        Ok(())
    }

    /// Restore expanded folders from persisted state
    fn restore_expanded_folders(&mut self, expanded_paths: &[String]) {
        let working_dir = self.working_dir.clone();
        Self::restore_expanded_recursive(&mut self.file_tree, expanded_paths, &working_dir);
    }

    /// Recursively restore expanded state for folders
    fn restore_expanded_recursive(
        entries: &mut [FileEntry],
        expanded_paths: &[String],
        working_dir: &PathBuf,
    ) {
        for entry in entries.iter_mut() {
            if entry.is_dir {
                let relative = entry
                    .path
                    .strip_prefix(working_dir)
                    .unwrap_or(&entry.path)
                    .to_string_lossy()
                    .to_string();

                if expanded_paths.contains(&relative) {
                    entry.is_expanded = true;

                    // Load children if not already loaded
                    if entry.children.is_empty() {
                        if let Ok(children) =
                            App::load_directory_children(&entry.path, entry.depth + 1)
                        {
                            entry.children = children;
                        }
                    }

                    // Recursively restore children
                    Self::restore_expanded_recursive(&mut entry.children, expanded_paths, working_dir);
                }
            }
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// Get the output to print to stdout after quitting
    pub fn get_output(&self) -> Option<String> {
        self.output.clone()
    }

    /// Handle tick event (called periodically)
    pub fn on_tick(&mut self) {
        // Advance spinner animation when running
        if self.is_running {
            self.spinner_frame = self.spinner_frame.wrapping_add(1);
        }

        // Note: Effect timing is handled in the render loop via effect_manager.tick()
    }

    /// Handle key events
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Global shortcuts (work in any mode)
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.quit = true;
                return Ok(());
            }
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                self.quit = true;
                return Ok(());
            }
            _ => {}
        }

        // Mode-specific handling
        match self.mode {
            AppMode::Normal => self.handle_normal_mode_key(key).await?,
            AppMode::Editing => self.handle_editing_mode_key(key)?,
            AppMode::Search => self.handle_search_mode_key(key)?,
            AppMode::Command => self.handle_command_mode_key(key)?,
            AppMode::Filter => self.handle_filter_mode_key(key)?,
            AppMode::Rename => self.handle_rename_mode_key(key)?,
        }

        Ok(())
    }

    /// Handle key events in normal mode
    async fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Quit
            KeyCode::Char('q') => {
                if self.show_help {
                    self.show_help = false;
                } else {
                    self.quit = true;
                }
            }

            // Help
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                if self.show_help {
                    self.trigger_help_overlay_effect();
                }
            }

            // Panel navigation
            KeyCode::Tab => {
                self.next_panel();
            }
            KeyCode::BackTab => {
                self.previous_panel();
            }

            // Vim-style navigation
            KeyCode::Char('h') => {
                self.active_panel = ActivePanel::FileBrowser;
            }
            KeyCode::Char('l') => {
                if self.active_panel == ActivePanel::FileBrowser {
                    self.active_panel = ActivePanel::Editor;
                } else if self.active_panel == ActivePanel::Editor {
                    self.active_panel = ActivePanel::Response;
                }
            }

            // Vertical navigation
            KeyCode::Char('j') | KeyCode::Down => {
                self.navigate_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.navigate_up();
            }

            // Page navigation
            KeyCode::PageDown => {
                self.page_down();
            }
            KeyCode::PageUp => {
                self.page_up();
            }
            KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                self.page_down();
            }
            KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                self.page_up();
            }

            // Selection / Expand
            KeyCode::Enter => {
                self.handle_enter().await?;
            }
            KeyCode::Char(' ') => {
                self.toggle_expand();
            }

            // Run request
            KeyCode::Char('r') => {
                self.run_current_request().await?;
            }

            // Edit mode
            KeyCode::Char('e') => {
                if self.current_file.is_some() {
                    self.mode = AppMode::Editing;
                    self.active_panel = ActivePanel::Editor;
                }
            }

            // Search
            KeyCode::Char('/') => {
                self.mode = AppMode::Search;
                self.search_query.clear();
            }

            // Filter files
            KeyCode::Char('f') => {
                self.mode = AppMode::Filter;
                self.active_panel = ActivePanel::FileBrowser;
            }

            // Clear filter
            KeyCode::Char('F') => {
                self.filter_query.clear();
                self.file_tree_index = 0;
                self.file_tree_state.select(Some(0));
                self.set_status("Filter cleared", StatusLevel::Info);
            }

            // Command mode
            KeyCode::Char(':') => {
                self.mode = AppMode::Command;
                self.command_input.clear();
            }

            // Toggle variables panel
            KeyCode::Char('v') => {
                if self.active_panel == ActivePanel::Variables {
                    self.active_panel = ActivePanel::FileBrowser;
                } else {
                    self.active_panel = ActivePanel::Variables;
                }
            }

            // Refresh file tree
            KeyCode::Char('R') => {
                self.refresh_file_tree()?;
                self.set_status("File tree refreshed", StatusLevel::Info);
            }

            // Go to top
            KeyCode::Char('g') => {
                self.go_to_top();
            }

            // Go to bottom
            KeyCode::Char('G') => {
                self.go_to_bottom();
            }

            // Environment switching
            KeyCode::Char('E') => {
                self.cycle_environment();
            }

            // Copy file path (y = yank path)
            KeyCode::Char('y') => {
                self.copy_current_file_path();
            }

            // Copy response (Y = yank response)
            KeyCode::Char('Y') => {
                self.copy_response();
            }

            // Copy AI context (c = copy context for AI)
            KeyCode::Char('c') => {
                self.copy_ai_context();
            }

            // Copy hurl command to clipboard (C = copy command)
            KeyCode::Char('C') => {
                self.copy_hurl_command();
            }

            // Output AI context to stdout and quit (o = output for pipe/Helix)
            KeyCode::Char('o') => {
                self.output_ai_context_and_quit();
            }

            // Copy file to clipboard (p = put/copy file)
            KeyCode::Char('p') => {
                self.copy_file_to_clipboard();
            }

            // Paste file from clipboard (P = paste file)
            KeyCode::Char('P') => {
                self.paste_file_from_clipboard();
            }

            // Rename file (n = name/rename file)
            KeyCode::Char('n') => {
                self.start_rename();
            }

            // Response tab switching (when in Response panel)
            KeyCode::Char('1') => {
                if self.active_panel == ActivePanel::Response {
                    self.response_tab = ResponseTab::Body;
                    self.response_scroll = 0;
                }
            }
            KeyCode::Char('2') => {
                if self.active_panel == ActivePanel::Response {
                    self.response_tab = ResponseTab::Headers;
                    self.response_scroll = 0;
                }
            }
            KeyCode::Char('3') => {
                if self.active_panel == ActivePanel::Response {
                    self.response_tab = ResponseTab::Raw;
                    self.response_scroll = 0;
                }
            }

            // Sidebar resize
            KeyCode::Char('[') => {
                self.resize_sidebar(-2);
            }
            KeyCode::Char(']') => {
                self.resize_sidebar(2);
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle key events in editing mode (vim-style)
    fn handle_editing_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.vim_mode {
            VimMode::Normal => self.handle_vim_normal_mode(key),
            VimMode::Insert => self.handle_vim_insert_mode(key),
        }
    }

    /// Handle vim normal mode keys (navigation and commands)
    fn handle_vim_normal_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Exit editing mode
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.vim_mode = VimMode::Normal;
            }
            KeyCode::Char('q') => {
                self.mode = AppMode::Normal;
                self.vim_mode = VimMode::Normal;
            }

            // Basic navigation (hjkl)
            KeyCode::Char('h') | KeyCode::Left => self.editor_move_cursor_left(),
            KeyCode::Char('j') | KeyCode::Down => self.editor_move_cursor_down(),
            KeyCode::Char('k') | KeyCode::Up => self.editor_move_cursor_up(),
            KeyCode::Char('l') | KeyCode::Right => self.editor_move_cursor_right(),

            // Line navigation
            KeyCode::Char('0') | KeyCode::Home => self.editor_move_to_line_start(),
            KeyCode::Char('$') | KeyCode::End => self.editor_move_to_line_end(),
            KeyCode::Char('^') => self.editor_move_to_first_non_whitespace(),

            // File navigation
            KeyCode::Char('g') => self.editor_move_to_first_line(),
            KeyCode::Char('G') => self.editor_move_to_last_line(),

            // Word navigation
            KeyCode::Char('w') => self.editor_move_word_forward(),
            KeyCode::Char('b') => self.editor_move_word_backward(),
            KeyCode::Char('e') => self.editor_move_word_end(),

            // Enter insert mode
            KeyCode::Char('i') => {
                self.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('a') => {
                self.editor_move_cursor_right();
                self.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('I') => {
                self.editor_move_to_first_non_whitespace();
                self.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('A') => {
                self.editor_move_to_line_end();
                self.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('o') => {
                self.editor_insert_line_below();
                self.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('O') => {
                self.editor_insert_line_above();
                self.vim_mode = VimMode::Insert;
            }

            // Page navigation (must be before delete commands to check modifiers first)
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor_page_up();
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor_page_down();
            }

            // Delete commands
            KeyCode::Char('x') => self.editor_delete_char(),
            KeyCode::Char('d') => self.editor_delete_line(),
            KeyCode::Char('D') => self.editor_delete_to_end(),

            _ => {}
        }

        Ok(())
    }

    /// Handle vim insert mode keys (text input)
    fn handle_vim_insert_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.vim_mode = VimMode::Normal;
                // Move cursor back one (vim behavior)
                if self.editor_cursor.1 > 0 {
                    self.editor_cursor.1 -= 1;
                }
            }
            KeyCode::Enter => self.editor_insert_newline(),
            KeyCode::Backspace => self.editor_backspace(),
            KeyCode::Delete => self.editor_delete(),
            KeyCode::Left => self.editor_move_cursor_left(),
            KeyCode::Right => self.editor_move_cursor_right(),
            KeyCode::Up => self.editor_move_cursor_up(),
            KeyCode::Down => self.editor_move_cursor_down(),
            KeyCode::Home => self.editor_move_to_line_start(),
            KeyCode::End => self.editor_move_to_line_end(),
            KeyCode::Tab => self.editor_insert_char('\t'),
            KeyCode::Char(c) => self.editor_insert_char(c),
            _ => {}
        }

        Ok(())
    }

    /// Handle key events in search mode
    fn handle_search_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.execute_search();
                self.mode = AppMode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle key events in command mode
    fn handle_command_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.command_input.clear();
            }
            KeyCode::Enter => {
                self.execute_command()?;
                self.mode = AppMode::Normal;
            }
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            KeyCode::Char(c) => {
                self.command_input.push(c);
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle key events in filter mode
    fn handle_filter_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                // Keep the filter active, don't clear it
            }
            KeyCode::Enter => {
                self.mode = AppMode::Normal;
                if self.filter_query.is_empty() {
                    self.set_status("Filter cleared", StatusLevel::Info);
                } else {
                    let count = self.get_visible_file_count();
                    self.set_status(
                        &format!("Filter: {} ({} files)", self.filter_query, count),
                        StatusLevel::Info,
                    );
                }
            }
            KeyCode::Backspace => {
                self.filter_query.pop();
                self.file_tree_index = 0;
                self.file_tree_state.select(Some(0));
            }
            KeyCode::Char(c) => {
                self.filter_query.push(c);
                self.file_tree_index = 0;
                self.file_tree_state.select(Some(0));
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle key events in rename mode.
    ///
    /// Allows user to type a new filename for the selected file.
    /// - Enter: Execute the rename operation
    /// - Esc: Cancel and return to normal mode
    /// - Backspace: Delete last character
    /// - Any char: Append to rename input
    fn handle_rename_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Cancel rename operation
                self.mode = AppMode::Normal;
                self.rename_input.clear();
                self.rename_target = None;
                self.set_status("Rename cancelled", StatusLevel::Info);
            }
            KeyCode::Enter => {
                // Execute the rename
                self.execute_rename();
                self.mode = AppMode::Normal;
            }
            KeyCode::Backspace => {
                self.rename_input.pop();
            }
            KeyCode::Char(c) => {
                // Only allow valid filename characters
                if c != '/' && c != '\\' && c != '\0' {
                    self.rename_input.push(c);
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle mouse events
    pub fn handle_mouse_event(&mut self, _mouse: MouseEvent) {
        // Mouse support can be implemented here
        // For now, we focus on keyboard navigation
    }

    /// Handle terminal resize
    pub fn handle_resize(&mut self, _width: u16, _height: u16) {
        // Recalculate layout if needed
    }

    /// Navigate to next panel
    fn next_panel(&mut self) {
        let old_panel = self.active_panel;
        self.active_panel = match self.active_panel {
            ActivePanel::FileBrowser => ActivePanel::Editor,
            ActivePanel::Editor => ActivePanel::Response,
            ActivePanel::Response => ActivePanel::Assertions,
            ActivePanel::Assertions => ActivePanel::Variables,
            ActivePanel::Variables => ActivePanel::FileBrowser,
        };
        if old_panel != self.active_panel {
            self.trigger_panel_focus_effect();
        }
    }

    /// Navigate to previous panel
    fn previous_panel(&mut self) {
        let old_panel = self.active_panel;
        self.active_panel = match self.active_panel {
            ActivePanel::FileBrowser => ActivePanel::Variables,
            ActivePanel::Editor => ActivePanel::FileBrowser,
            ActivePanel::Response => ActivePanel::Editor,
            ActivePanel::Assertions => ActivePanel::Response,
            ActivePanel::Variables => ActivePanel::Assertions,
        };
        if old_panel != self.active_panel {
            self.trigger_panel_focus_effect();
        }
    }

    /// Trigger panel focus effect - stored area will be updated during render
    fn trigger_panel_focus_effect(&mut self) {
        // The effect area will be set during render based on current layout
        // For now we use a placeholder area - the UI module will update it
        let effect = presets::panel_focus();
        self.effect_manager.add_effect(
            EffectId::PanelFocus(self.active_panel),
            effect,
            ratatui::layout::Rect::default(), // Will be updated during render
        );
    }

    /// Trigger execution start effect (pulse on response panel)
    fn trigger_execution_start_effect(&mut self) {
        let effect = presets::execution_pulse();
        self.effect_manager.add_effect(
            EffectId::ExecutionStart,
            effect,
            ratatui::layout::Rect::default(), // Will be set to response panel during render
        );
    }

    /// Trigger execution complete effect (success or error flash)
    fn trigger_execution_complete_effect(&mut self, success: bool) {
        let effect = if success {
            presets::success_flash()
        } else {
            presets::error_flash()
        };
        self.effect_manager.add_effect(
            EffectId::ExecutionComplete,
            effect,
            ratatui::layout::Rect::default(), // Will be set to response panel during render
        );
    }

    /// Trigger help overlay animation
    fn trigger_help_overlay_effect(&mut self) {
        let effect = presets::dissolve_in();
        self.effect_manager.add_effect(
            EffectId::HelpOverlay,
            effect,
            ratatui::layout::Rect::default(), // Will be set to help area during render
        );
    }

    /// Navigate down in current panel
    fn navigate_down(&mut self) {
        match self.active_panel {
            ActivePanel::FileBrowser => {
                let max = self.get_visible_file_count().saturating_sub(1);
                if self.file_tree_index < max {
                    self.file_tree_index += 1;
                    self.file_tree_state.select(Some(self.file_tree_index));
                    self.auto_preview_selected_file();
                }
            }
            ActivePanel::Editor => {
                if self.editor_scroll < self.editor_content.len().saturating_sub(1) {
                    self.editor_scroll += 1;
                }
            }
            ActivePanel::Response => {
                self.response_scroll += 1;
            }
            ActivePanel::Assertions => {
                self.assertions_scroll += 1;
            }
            _ => {}
        }
    }

    /// Navigate up in current panel
    fn navigate_up(&mut self) {
        match self.active_panel {
            ActivePanel::FileBrowser => {
                let old_index = self.file_tree_index;
                self.file_tree_index = self.file_tree_index.saturating_sub(1);
                self.file_tree_state.select(Some(self.file_tree_index));
                if self.file_tree_index != old_index {
                    self.auto_preview_selected_file();
                }
            }
            ActivePanel::Editor => {
                self.editor_scroll = self.editor_scroll.saturating_sub(1);
            }
            ActivePanel::Response => {
                self.response_scroll = self.response_scroll.saturating_sub(1);
            }
            ActivePanel::Assertions => {
                self.assertions_scroll = self.assertions_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Page down
    fn page_down(&mut self) {
        for _ in 0..10 {
            self.navigate_down();
        }
    }

    /// Page up
    fn page_up(&mut self) {
        for _ in 0..10 {
            self.navigate_up();
        }
    }

    /// Go to top
    fn go_to_top(&mut self) {
        match self.active_panel {
            ActivePanel::FileBrowser => {
                self.file_tree_index = 0;
                self.file_tree_state.select(Some(0));
            }
            ActivePanel::Editor => self.editor_scroll = 0,
            ActivePanel::Response => self.response_scroll = 0,
            ActivePanel::Assertions => self.assertions_scroll = 0,
            _ => {}
        }
    }

    /// Go to bottom
    fn go_to_bottom(&mut self) {
        match self.active_panel {
            ActivePanel::FileBrowser => {
                self.file_tree_index = self.get_visible_file_count().saturating_sub(1);
                self.file_tree_state.select(Some(self.file_tree_index));
            }
            ActivePanel::Editor => {
                self.editor_scroll = self.editor_content.len().saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Handle enter key
    async fn handle_enter(&mut self) -> Result<()> {
        match self.active_panel {
            ActivePanel::FileBrowser => {
                let selected = self.get_selected_file_entry().cloned();
                if let Some(entry) = selected {
                    if entry.is_dir {
                        self.toggle_expand();
                    } else if entry.path.extension().map_or(false, |e| e == "hurl") {
                        self.open_file(&entry.path)?;
                    }
                }
            }
            ActivePanel::Editor => {
                // Run the current request
                self.run_current_request().await?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Toggle directory expansion
    fn toggle_expand(&mut self) {
        let index = self.file_tree_index;
        if let Some(entry) = self.get_selected_file_entry_mut(index) {
            if entry.is_dir {
                entry.is_expanded = !entry.is_expanded;
                if entry.is_expanded && entry.children.is_empty() {
                    // Load children
                    if let Ok(children) =
                        Self::load_directory_children(&entry.path, entry.depth + 1)
                    {
                        entry.children = children;
                    }
                }
            }
        }
        // Save expanded state
        self.save_state();
    }

    /// Get the count of visible files in the tree (respects filter)
    fn get_visible_file_count(&self) -> usize {
        self.get_visible_files().len()
    }

    /// Get selected file entry
    fn get_selected_file_entry(&self) -> Option<&FileEntry> {
        let target = self.file_tree_index;
        let mut index = 0;
        Self::find_entry(&self.file_tree, &mut index, target)
    }

    fn find_entry<'a>(
        entries: &'a [FileEntry],
        index: &mut usize,
        target: usize,
    ) -> Option<&'a FileEntry> {
        for entry in entries {
            if *index == target {
                return Some(entry);
            }
            *index += 1;
            if entry.is_expanded {
                if let Some(found) = Self::find_entry(&entry.children, index, target) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Get selected file entry (mutable)
    fn get_selected_file_entry_mut(&mut self, target: usize) -> Option<&mut FileEntry> {
        let mut index = 0;
        Self::find_entry_mut(&mut self.file_tree, &mut index, target)
    }

    fn find_entry_mut<'a>(
        entries: &'a mut [FileEntry],
        index: &mut usize,
        target: usize,
    ) -> Option<&'a mut FileEntry> {
        for entry in entries {
            if *index == target {
                return Some(entry);
            }
            *index += 1;
            if entry.is_expanded {
                if let Some(found) = Self::find_entry_mut(&mut entry.children, index, target) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Refresh the file tree (preserves expanded state)
    pub fn refresh_file_tree(&mut self) -> Result<()> {
        // Collect currently expanded folders before refreshing
        let expanded = self.collect_expanded_folders();

        // Reload the file tree
        self.file_tree = Self::load_directory_children(&self.working_dir, 0)?;

        // Restore expanded state
        if !expanded.is_empty() {
            self.restore_expanded_folders(&expanded);
        } else {
            // No folders were expanded, auto-expand directories with .hurl files
            self.auto_expand_hurl_directories();
        }

        Ok(())
    }

    /// Auto-expand directories that contain .hurl files
    fn auto_expand_hurl_directories(&mut self) {
        Self::expand_entries_with_hurl(&mut self.file_tree);
    }

    /// Recursively expand directory entries that contain .hurl files
    fn expand_entries_with_hurl(entries: &mut [FileEntry]) {
        for entry in entries.iter_mut() {
            if entry.is_dir {
                // Load children if not already loaded
                if entry.children.is_empty() {
                    if let Ok(children) = App::load_directory_children(&entry.path, entry.depth + 1)
                    {
                        entry.children = children;
                    }
                }

                // Check if this directory or its children contain .hurl files
                let has_hurl = entry
                    .children
                    .iter()
                    .any(|c| !c.is_dir || Self::dir_contains_hurl(&c.path));

                if has_hurl {
                    entry.is_expanded = true;
                    // Recursively expand children
                    Self::expand_entries_with_hurl(&mut entry.children);
                }
            }
        }
    }

    /// Check if a directory should be skipped during scanning
    /// Returns true for common build output, dependency, and cache directories
    fn should_skip_directory(path: &PathBuf) -> bool {
        path.file_name()
            .map(|name| {
                let name = name.to_string_lossy();
                // Skip hidden directories
                if name.starts_with('.') {
                    return true;
                }
                // Skip known unnecessary directories (case-insensitive for cross-platform)
                let name_lower = name.to_lowercase();
                IGNORED_DIRECTORIES
                    .iter()
                    .any(|&ignored| ignored.to_lowercase() == name_lower)
            })
            .unwrap_or(false)
    }

    /// Check if a directory contains any .hurl files (recursively)
    fn dir_contains_hurl(path: &PathBuf) -> bool {
        // Skip unnecessary directories
        if Self::should_skip_directory(path) {
            return false;
        }

        if let Ok(read_dir) = std::fs::read_dir(path) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "hurl") {
                    return true;
                }
                if path.is_dir() && Self::dir_contains_hurl(&path) {
                    return true;
                }
            }
        }
        false
    }

    /// Load children of a directory
    fn load_directory_children(path: &PathBuf, depth: usize) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();

        if let Ok(read_dir) = std::fs::read_dir(path) {
            for entry in read_dir.flatten() {
                let path = entry.path();

                // Skip hidden files, build outputs, dependencies, and other unnecessary directories
                if Self::should_skip_directory(&path) {
                    continue;
                }

                // Only include .hurl/.env files and directories that contain .hurl files
                if path.is_dir() {
                    // Only include directories that contain .hurl files (recursively)
                    if Self::dir_contains_hurl(&path) {
                        entries.push(FileEntry::new(path, depth));
                    }
                } else if path.extension().map_or(false, |e| e == "hurl" || e == "env") {
                    entries.push(FileEntry::new(path, depth));
                }
            }
        }

        // Sort: directories first, then alphabetically
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(entries)
    }

    /// Open a hurl file
    fn open_file(&mut self, path: &PathBuf) -> Result<()> {
        self.open_file_internal(path, true)
    }

    /// Preview a hurl file (open without switching panel)
    fn preview_file(&mut self, path: &PathBuf) -> Result<()> {
        self.open_file_internal(path, false)
    }

    /// Internal file opening logic
    fn open_file_internal(&mut self, path: &PathBuf, switch_panel: bool) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let is_hurl_file = path.extension().map_or(false, |e| e == "hurl");

        if is_hurl_file {
            let hurl_file = crate::parser::parse_hurl_file(&content)?;
            self.current_file = Some(hurl_file);

            // Restore execution state for this file if available
            let relative_path = self.get_relative_path(path);
            self.execution_result = self.file_execution_states.get(&relative_path).cloned();
        } else {
            // For non-hurl files (like .env), just show content without parsing
            self.current_file = None;
            self.execution_result = None;
        }

        self.current_file_path = Some(path.clone());
        self.editor_content = content.lines().map(String::from).collect();
        self.editor_cursor = (0, 0);
        self.editor_scroll = 0;

        self.response_scroll = 0;
        self.assertions_scroll = 0;

        if switch_panel {
            self.active_panel = ActivePanel::Editor;
        }

        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        // Show status with execution state info
        if self.execution_result.is_some() {
            self.set_status(
                &format!("Preview: {} (with cached result)", file_name),
                StatusLevel::Info,
            );
        } else {
            self.set_status(&format!("Preview: {}", file_name), StatusLevel::Info);
        }

        // Save state for next session
        self.save_state();

        Ok(())
    }

    /// Auto-preview the currently selected file in the file browser
    fn auto_preview_selected_file(&mut self) {
        if let Some(entry) = self.get_selected_file_entry() {
            if !entry.is_dir && entry.path.extension().map_or(false, |e| e == "hurl" || e == "env") {
                let path = entry.path.clone();
                let _ = self.preview_file(&path);
            }
        }
    }

    /// Save the current file
    pub fn save_current_file(&mut self) -> Result<()> {
        if let Some(path) = &self.current_file_path {
            let content = self.editor_content.join("\n");
            std::fs::write(path, &content)?;

            // Re-parse the file
            if let Ok(hurl_file) = crate::parser::parse_hurl_file(&content) {
                self.current_file = Some(hurl_file);
            }

            self.set_status("File saved", StatusLevel::Success);
        }

        Ok(())
    }

    /// Run the current request
    pub async fn run_current_request(&mut self) -> Result<()> {
        let Some(path) = self.current_file_path.clone() else {
            self.set_status("No file selected", StatusLevel::Warning);
            return Ok(());
        };

        self.is_running = true;
        self.set_status("Running request...", StatusLevel::Info);
        self.trigger_execution_start_effect();

        // Run the request with variables file
        let start = std::time::Instant::now();
        let result = self.runner.run(&path, self.current_env_file.as_ref()).await;
        let duration = start.elapsed();

        self.is_running = false;

        match result {
            Ok(exec_result) => {
                let success = exec_result.success;
                let status_code = exec_result.response.as_ref().map(|r| r.status_code);

                // Add to history
                self.history.insert(
                    0,
                    HistoryEntry {
                        id: uuid::Uuid::new_v4(),
                        file_path: path.clone(),
                        timestamp: chrono::Utc::now(),
                        duration_ms: duration.as_millis() as u64,
                        status_code,
                        success,
                    },
                );

                // Store execution result in the per-file cache
                let relative_path = self.get_relative_path(&path);
                self.file_execution_states
                    .insert(relative_path, exec_result.clone());

                self.execution_result = Some(exec_result);
                self.response_scroll = 0;
                self.assertions_scroll = 0;

                // Persist state to disk
                self.save_state();

                if success {
                    self.set_status(
                        &format!("Request completed in {}ms", duration.as_millis()),
                        StatusLevel::Success,
                    );
                    self.trigger_execution_complete_effect(true);
                } else {
                    self.set_status("Request completed with failures", StatusLevel::Warning);
                    self.trigger_execution_complete_effect(false);
                }
            }
            Err(e) => {
                self.set_status(&format!("Error: {e}"), StatusLevel::Error);
            }
        }

        Ok(())
    }

    /// Recursively find all .env files in a directory
    fn find_env_files(dir: &PathBuf) -> Vec<PathBuf> {
        let mut env_files = Vec::new();

        if let Ok(read_dir) = std::fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();

                // Skip hidden and ignored directories
                if Self::should_skip_directory(&path) {
                    continue;
                }

                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "env" {
                            // Use absolute path
                            if let Ok(abs_path) = path.canonicalize() {
                                env_files.push(abs_path);
                            } else {
                                env_files.push(path);
                            }
                        }
                    }
                } else if path.is_dir() {
                    // Recursively scan subdirectories
                    env_files.extend(Self::find_env_files(&path));
                }
            }
        }

        env_files
    }

    /// Load environments by scanning for .env files recursively in the working directory
    fn load_environments(&mut self) -> Result<()> {
        self.environments.clear();

        // Find all .env files recursively
        let env_files = Self::find_env_files(&self.working_dir);

        for path in env_files {
            if let Some(name) = path.file_stem() {
                let name = name.to_string_lossy().to_string();
                if !self.environments.contains(&name) {
                    self.environments.push(name);
                }
            }
        }

        // Sort environments alphabetically
        self.environments.sort();

        // Set current environment to first one if available
        if !self.environments.is_empty() {
            self.current_environment = self.environments[0].clone();
            self.load_current_environment_variables()?;
        }

        Ok(())
    }

    /// Load variables for the current environment
    fn load_current_environment_variables(&mut self) -> Result<()> {
        self.variables.clear();
        self.current_env_file = None;

        if self.current_environment.is_empty() {
            return Ok(());
        }

        // Find the env file recursively
        let env_files = Self::find_env_files(&self.working_dir);
        let env_file = env_files.into_iter().find(|p| {
            p.file_stem()
                .map(|s| s.to_string_lossy() == self.current_environment)
                .unwrap_or(false)
        });

        let Some(env_file) = env_file else {
            return Ok(());
        };

        // Store the file path for --variables-file
        self.current_env_file = Some(env_file.clone());

        // Also parse variables for UI display
        if let Ok(content) = std::fs::read_to_string(&env_file) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    self.variables.push(Variable {
                        name: key.trim().to_string(),
                        value: value.trim().to_string(),
                        is_secret: key.to_lowercase().contains("secret")
                            || key.to_lowercase().contains("password")
                            || key.to_lowercase().contains("token"),
                    });
                }
            }
        }

        Ok(())
    }

    /// Cycle through environments
    fn cycle_environment(&mut self) {
        if self.environments.is_empty() {
            self.set_status("No .env files found", StatusLevel::Warning);
            return;
        }

        if let Some(idx) = self
            .environments
            .iter()
            .position(|e| e == &self.current_environment)
        {
            let next_idx = (idx + 1) % self.environments.len();
            self.current_environment = self.environments[next_idx].clone();
            let _ = self.load_current_environment_variables();
            self.set_status(
                &format!("Environment: {}", self.current_environment),
                StatusLevel::Info,
            );
            // Save state to persist selected environment
            self.save_state();
        }
    }

    /// Resize sidebar by delta (positive = wider, negative = narrower)
    fn resize_sidebar(&mut self, delta: i16) {
        let new_width = (self.sidebar_width as i16 + delta).clamp(10, 50) as u16;
        if new_width != self.sidebar_width {
            self.sidebar_width = new_width;
            self.save_state();
        }
    }

    /// Copy current file's relative path to clipboard
    fn copy_current_file_path(&mut self) {
        if let Some(path) = &self.current_file_path {
            // Get relative path from working directory
            let relative_path = path
                .strip_prefix(&self.working_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            match self.copy_to_clipboard(&relative_path) {
                Ok(_) => {
                    self.set_status(
                        &format!("Copied path: {}", relative_path),
                        StatusLevel::Success,
                    );
                }
                Err(e) => {
                    self.set_status(&format!("Copy failed: {}", e), StatusLevel::Error);
                }
            }
        } else {
            self.set_status("No file selected", StatusLevel::Warning);
        }
    }

    /// Copy response body to clipboard
    fn copy_response(&mut self) {
        if let Some(result) = &self.execution_result {
            if let Some(response) = &result.response {
                let body = if response.body.is_empty() {
                    format!("HTTP {}\n(empty body)", response.status_code)
                } else {
                    // Try to pretty print JSON
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response.body) {
                        serde_json::to_string_pretty(&json).unwrap_or(response.body.clone())
                    } else {
                        response.body.clone()
                    }
                };

                match self.copy_to_clipboard(&body) {
                    Ok(_) => {
                        let preview = if body.len() > 50 {
                            format!("{}...", &body[..50])
                        } else {
                            body.clone()
                        };
                        self.set_status(
                            &format!("Copied response ({} bytes)", body.len()),
                            StatusLevel::Success,
                        );
                    }
                    Err(e) => {
                        self.set_status(&format!("Copy failed: {}", e), StatusLevel::Error);
                    }
                }
            } else {
                self.set_status("No response body available", StatusLevel::Warning);
            }
        } else {
            self.set_status("No response yet. Run request first.", StatusLevel::Warning);
        }
    }

    /// Copy text to system clipboard
    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        use arboard::Clipboard;
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }

    /// Copy the hurl command to clipboard
    fn copy_hurl_command(&mut self) {
        let Some(file_path) = &self.current_file_path else {
            self.set_status("No file selected", StatusLevel::Warning);
            return;
        };

        // Only for .hurl files
        if !file_path.extension().map_or(false, |e| e == "hurl") {
            self.set_status("Not a .hurl file", StatusLevel::Warning);
            return;
        }

        // Build the command
        let mut cmd_parts = vec!["hurl".to_string()];

        // Add variables file if available
        if let Some(env_file) = &self.current_env_file {
            cmd_parts.push("--variables-file".to_string());
            cmd_parts.push(env_file.to_string_lossy().to_string());
        }

        // Add the hurl file path
        cmd_parts.push(file_path.to_string_lossy().to_string());

        let command = cmd_parts.join(" ");

        match self.copy_to_clipboard(&command) {
            Ok(_) => {
                self.set_status(&format!("Copied: {}", command), StatusLevel::Success);
            }
            Err(e) => {
                self.set_status(&format!("Copy failed: {}", e), StatusLevel::Error);
            }
        }
    }

    /// Copy the selected file to the internal clipboard for paste operation.
    ///
    /// This function stores the path of the currently selected file in `clipboard_file`.
    /// Only works when the file browser panel is active and a file (not directory) is selected.
    ///
    /// # Behavior
    /// - Only files can be copied, not directories
    /// - The file path is stored in memory (not system clipboard)
    /// - Shows status message indicating success or failure
    ///
    /// # Keyboard Shortcut
    /// `p` - Copy selected file
    fn copy_file_to_clipboard(&mut self) {
        // Ensure we're in the file browser panel
        if self.active_panel != ActivePanel::FileBrowser {
            self.set_status("Copy only works in file browser", StatusLevel::Warning);
            return;
        }

        if let Some(entry) = self.get_selected_file_entry() {
            // Directories cannot be copied (only files)
            if entry.is_dir {
                self.set_status("Cannot copy directories", StatusLevel::Warning);
                return;
            }

            // Store the file path in the clipboard
            let path = entry.path.clone();
            let file_name = entry.name.clone();
            self.clipboard_file = Some(path);
            self.set_status(&format!("Copied: {}", file_name), StatusLevel::Success);
        } else {
            self.set_status("No file selected", StatusLevel::Warning);
        }
    }

    /// Paste the previously copied file to the current location.
    ///
    /// This function duplicates the file stored in `clipboard_file` to the target directory.
    /// The target directory is determined by the current selection in the file browser.
    ///
    /// # Target Directory Logic
    /// - If a directory is selected: paste into that directory
    /// - If a file is selected: paste into the parent directory of that file
    /// - If nothing is selected: paste into the working directory
    ///
    /// # Name Conflict Handling
    /// If a file with the same name already exists, a suffix is appended:
    /// - `file.hurl` -> `file_copy1.hurl`
    /// - `file_copy1.hurl` -> `file_copy2.hurl`
    ///
    /// # Keyboard Shortcut
    /// `P` (Shift+p) - Paste copied file
    fn paste_file_from_clipboard(&mut self) {
        // Ensure we're in the file browser panel
        if self.active_panel != ActivePanel::FileBrowser {
            self.set_status("Paste only works in file browser", StatusLevel::Warning);
            return;
        }

        // Check if there's a file in the clipboard
        let Some(source_path) = self.clipboard_file.clone() else {
            self.set_status(
                "No file in clipboard. Use 'p' to copy first.",
                StatusLevel::Warning,
            );
            return;
        };

        // Verify the source file still exists
        if !source_path.exists() {
            self.set_status("Source file no longer exists", StatusLevel::Error);
            self.clipboard_file = None;
            return;
        }

        // Determine the target directory based on current selection
        let target_dir = if let Some(entry) = self.get_selected_file_entry() {
            if entry.is_dir {
                // Selected item is a directory - paste into it
                entry.path.clone()
            } else {
                // Selected item is a file - paste into its parent directory
                entry
                    .path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| self.working_dir.clone())
            }
        } else {
            // No selection - paste into the working directory
            self.working_dir.clone()
        };

        // Extract the source file name
        let Some(file_name) = source_path.file_name() else {
            self.set_status("Invalid source file", StatusLevel::Error);
            return;
        };

        // Build the target path, handling name conflicts by appending _copyN suffix
        let mut target_path = target_dir.join(file_name);
        let mut counter = 1;

        // Generate unique filename if target already exists
        while target_path.exists() {
            let stem = source_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let ext = source_path
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            let new_name = format!("{}_copy{}{}", stem, counter, ext);
            target_path = target_dir.join(new_name);
            counter += 1;
        }

        // Perform the file copy operation
        match std::fs::copy(&source_path, &target_path) {
            Ok(_) => {
                let target_name = target_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                self.set_status(&format!("Pasted: {}", target_name), StatusLevel::Success);

                // Refresh file tree to show the newly pasted file
                if let Err(e) = self.refresh_file_tree() {
                    self.set_status(
                        &format!("Pasted but refresh failed: {}", e),
                        StatusLevel::Warning,
                    );
                }
            }
            Err(e) => {
                self.set_status(&format!("Paste failed: {}", e), StatusLevel::Error);
            }
        }
    }

    /// Start the rename operation for the currently selected file.
    ///
    /// This function initiates rename mode and pre-fills the input with the current filename.
    /// Only works when the file browser panel is active and a file (not directory) is selected.
    ///
    /// # Behavior
    /// - Only files can be renamed, not directories
    /// - The current filename is pre-filled in the rename input
    /// - User can edit the name and press Enter to confirm or Esc to cancel
    ///
    /// # Keyboard Shortcut
    /// `n` - Start rename for selected file
    fn start_rename(&mut self) {
        // Ensure we're in the file browser panel
        if self.active_panel != ActivePanel::FileBrowser {
            self.set_status("Rename only works in file browser", StatusLevel::Warning);
            return;
        }

        // Get the selected entry info (clone to avoid borrow issues)
        let entry_info = self
            .get_selected_file_entry()
            .map(|e| (e.path.clone(), e.name.clone(), e.is_dir));

        if let Some((path, name, is_dir)) = entry_info {
            // Directories cannot be renamed (for now)
            if is_dir {
                self.set_status("Cannot rename directories", StatusLevel::Warning);
                return;
            }

            // Store the target path and pre-fill with current name
            self.rename_target = Some(path);
            self.rename_input = name;
            self.mode = AppMode::Rename;
            self.set_status(
                "Enter new name (Enter to confirm, Esc to cancel)",
                StatusLevel::Info,
            );
        } else {
            self.set_status("No file selected", StatusLevel::Warning);
        }
    }

    /// Execute the rename operation with the current input.
    ///
    /// Renames the file stored in `rename_target` to the new name in `rename_input`.
    /// Handles validation and error cases.
    ///
    /// # Validation
    /// - New name cannot be empty
    /// - New name cannot already exist in the same directory
    /// - Must preserve .hurl extension for hurl files
    fn execute_rename(&mut self) {
        let Some(source_path) = self.rename_target.take() else {
            self.set_status("No file to rename", StatusLevel::Error);
            return;
        };

        let new_name = self.rename_input.trim().to_string();
        self.rename_input.clear();

        // Validate new name
        if new_name.is_empty() {
            self.set_status("Filename cannot be empty", StatusLevel::Error);
            return;
        }

        // Check for invalid characters (additional safety)
        if new_name.contains('/') || new_name.contains('\\') {
            self.set_status(
                "Filename cannot contain path separators",
                StatusLevel::Error,
            );
            return;
        }

        // Ensure .hurl extension is preserved for hurl files
        let new_name = if source_path.extension().map_or(false, |e| e == "hurl")
            && !new_name.ends_with(".hurl")
        {
            format!("{}.hurl", new_name)
        } else {
            new_name
        };

        // Build target path in the same directory
        let Some(parent_dir) = source_path.parent() else {
            self.set_status("Cannot determine parent directory", StatusLevel::Error);
            return;
        };
        let target_path = parent_dir.join(&new_name);

        // Check if target already exists
        if target_path.exists() && target_path != source_path {
            self.set_status(
                &format!("File '{}' already exists", new_name),
                StatusLevel::Error,
            );
            return;
        }

        // If name unchanged, just return
        if target_path == source_path {
            self.set_status("Name unchanged", StatusLevel::Info);
            return;
        }

        // Perform the rename
        match std::fs::rename(&source_path, &target_path) {
            Ok(_) => {
                let old_name = source_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                self.set_status(
                    &format!("Renamed: {} -> {}", old_name, new_name),
                    StatusLevel::Success,
                );

                // Update current_file_path if the renamed file was open
                if self.current_file_path.as_ref() == Some(&source_path) {
                    self.current_file_path = Some(target_path.clone());
                }

                // Clear clipboard if it referenced the renamed file
                if self.clipboard_file.as_ref() == Some(&source_path) {
                    self.clipboard_file = Some(target_path);
                }

                // Refresh file tree to show the renamed file
                if let Err(e) = self.refresh_file_tree() {
                    self.set_status(
                        &format!("Renamed but refresh failed: {}", e),
                        StatusLevel::Warning,
                    );
                }
            }
            Err(e) => {
                self.set_status(&format!("Rename failed: {}", e), StatusLevel::Error);
            }
        }
    }

    /// Copy full test context for AI (request + response + assertions)
    fn copy_ai_context(&mut self) {
        let mut context = String::new();

        // Add file path
        if let Some(path) = &self.current_file_path {
            let relative_path = path
                .strip_prefix(&self.working_dir)
                .unwrap_or(path)
                .to_string_lossy();
            context.push_str(&format!("## Hurl Test: {}\n\n", relative_path));
        }

        // Add request (hurl file content)
        if !self.editor_content.is_empty() {
            context.push_str("### Request (Hurl file)\n\n```hurl\n");
            context.push_str(&self.editor_content.join("\n"));
            context.push_str("\n```\n\n");
        }

        // Add response
        if let Some(result) = &self.execution_result {
            if let Some(response) = &result.response {
                context.push_str(&format!(
                    "### Response\n\n**Status:** {}\n**Duration:** {}ms\n\n",
                    response.status_code, response.duration_ms
                ));

                // Headers
                if !response.headers.is_empty() {
                    context.push_str("**Headers:**\n```\n");
                    for (name, value) in &response.headers {
                        context.push_str(&format!("{}: {}\n", name, value));
                    }
                    context.push_str("```\n\n");
                }

                // Body
                if !response.body.is_empty() {
                    context.push_str("**Body:**\n```json\n");
                    // Try to pretty print JSON
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response.body) {
                        if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                            context.push_str(&pretty);
                        } else {
                            context.push_str(&response.body);
                        }
                    } else {
                        context.push_str(&response.body);
                    }
                    context.push_str("\n```\n\n");
                }
            }

            // Assertions
            if !result.assertions.is_empty() {
                context.push_str("### Assertion Results\n\n");
                let passed = result.assertions.iter().filter(|a| a.success).count();
                let total = result.assertions.len();
                context.push_str(&format!("**Summary:** {}/{} passed\n\n", passed, total));

                context.push_str("| Status | Assertion |\n");
                context.push_str("|--------|----------|\n");
                for assertion in &result.assertions {
                    let status = if assertion.success { "PASS" } else { "FAIL" };
                    context.push_str(&format!("| {} | {} |\n", status, assertion.text));
                }
                context.push_str("\n");

                // Add failure details
                let failures: Vec<_> = result.assertions.iter().filter(|a| !a.success).collect();
                if !failures.is_empty() {
                    context.push_str("**Failures:**\n\n");
                    for failure in failures {
                        context.push_str(&format!("- `{}`\n", failure.text));
                        if let Some(expected) = &failure.expected {
                            context.push_str(&format!("  - Expected: {}\n", expected));
                        }
                        if let Some(actual) = &failure.actual {
                            context.push_str(&format!("  - Actual: {}\n", actual));
                        }
                        if let Some(message) = &failure.message {
                            context.push_str(&format!("  - Error: {}\n", message));
                        }
                    }
                    context.push_str("\n");
                }
            }

            // Overall result
            context.push_str(&format!(
                "### Result: {}\n",
                if result.success { "SUCCESS" } else { "FAILED" }
            ));
        } else {
            context.push_str("### Response\n\n*No response yet - request not executed*\n");
        }

        if context.is_empty() {
            self.set_status("No context to copy", StatusLevel::Warning);
            return;
        }

        match self.copy_to_clipboard(&context) {
            Ok(_) => {
                self.set_status(
                    &format!("Copied AI context ({} bytes)", context.len()),
                    StatusLevel::Success,
                );
            }
            Err(e) => {
                self.set_status(&format!("Copy failed: {}", e), StatusLevel::Error);
            }
        }
    }

    /// Build AI context string (shared between copy and output)
    fn build_ai_context(&self) -> String {
        let mut context = String::new();

        // Add file path
        if let Some(path) = &self.current_file_path {
            let relative_path = path
                .strip_prefix(&self.working_dir)
                .unwrap_or(path)
                .to_string_lossy();
            context.push_str(&format!("## Hurl Test: {}\n\n", relative_path));
        }

        // Add request (hurl file content)
        if !self.editor_content.is_empty() {
            context.push_str("### Request (Hurl file)\n\n```hurl\n");
            context.push_str(&self.editor_content.join("\n"));
            context.push_str("\n```\n\n");
        }

        // Add response
        if let Some(result) = &self.execution_result {
            if let Some(response) = &result.response {
                context.push_str(&format!(
                    "### Response\n\n**Status:** {}\n**Duration:** {}ms\n\n",
                    response.status_code, response.duration_ms
                ));

                // Headers
                if !response.headers.is_empty() {
                    context.push_str("**Headers:**\n```\n");
                    for (name, value) in &response.headers {
                        context.push_str(&format!("{}: {}\n", name, value));
                    }
                    context.push_str("```\n\n");
                }

                // Body
                if !response.body.is_empty() {
                    context.push_str("**Body:**\n```json\n");
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response.body) {
                        if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                            context.push_str(&pretty);
                        } else {
                            context.push_str(&response.body);
                        }
                    } else {
                        context.push_str(&response.body);
                    }
                    context.push_str("\n```\n\n");
                }
            }

            // Assertions
            if !result.assertions.is_empty() {
                context.push_str("### Assertion Results\n\n");
                let passed = result.assertions.iter().filter(|a| a.success).count();
                let total = result.assertions.len();
                context.push_str(&format!("**Summary:** {}/{} passed\n\n", passed, total));

                context.push_str("| Status | Assertion |\n");
                context.push_str("|--------|----------|\n");
                for assertion in &result.assertions {
                    let status = if assertion.success { "PASS" } else { "FAIL" };
                    context.push_str(&format!("| {} | {} |\n", status, assertion.text));
                }
                context.push_str("\n");

                // Add failure details
                let failures: Vec<_> = result.assertions.iter().filter(|a| !a.success).collect();
                if !failures.is_empty() {
                    context.push_str("**Failures:**\n\n");
                    for failure in failures {
                        context.push_str(&format!("- `{}`\n", failure.text));
                        if let Some(expected) = &failure.expected {
                            context.push_str(&format!("  - Expected: {}\n", expected));
                        }
                        if let Some(actual) = &failure.actual {
                            context.push_str(&format!("  - Actual: {}\n", actual));
                        }
                        if let Some(message) = &failure.message {
                            context.push_str(&format!("  - Error: {}\n", message));
                        }
                    }
                    context.push_str("\n");
                }
            }

            // Overall result
            context.push_str(&format!(
                "### Result: {}\n",
                if result.success { "SUCCESS" } else { "FAILED" }
            ));
        } else {
            context.push_str("### Response\n\n*No response yet - request not executed*\n");
        }

        context
    }

    /// Output AI context to stdout and quit (for Helix/pipe integration)
    fn output_ai_context_and_quit(&mut self) {
        let context = self.build_ai_context();

        if context.is_empty() {
            self.set_status("No context to output", StatusLevel::Warning);
            return;
        }

        self.output = Some(context);
        self.quit = true;
    }

    /// Execute search
    fn execute_search(&mut self) {
        // Simple search in file names
        let query = self.search_query.to_lowercase();

        fn find_matching(entries: &[FileEntry], query: &str, index: &mut usize) -> Option<usize> {
            for entry in entries {
                if entry.name.to_lowercase().contains(query) {
                    return Some(*index);
                }
                *index += 1;
                if entry.is_expanded {
                    if let Some(found) = find_matching(&entry.children, query, index) {
                        return Some(found);
                    }
                }
            }
            None
        }

        let mut index = 0;
        if let Some(found_index) = find_matching(&self.file_tree, &query, &mut index) {
            self.file_tree_index = found_index;
            self.file_tree_state.select(Some(found_index));
            self.set_status(&format!("Found: {}", self.search_query), StatusLevel::Info);
        } else {
            self.set_status("No matches found", StatusLevel::Warning);
        }
    }

    /// Execute command
    fn execute_command(&mut self) -> Result<()> {
        let cmd = self.command_input.trim().to_lowercase();

        match cmd.as_str() {
            "q" | "quit" => {
                self.quit = true;
            }
            "w" | "write" | "save" => {
                self.save_current_file()?;
            }
            "wq" => {
                self.save_current_file()?;
                self.quit = true;
            }
            "refresh" | "r" => {
                self.refresh_file_tree()?;
            }
            "help" | "h" => {
                self.show_help = true;
            }
            _ => {
                self.set_status(&format!("Unknown command: {}", cmd), StatusLevel::Error);
            }
        }

        self.command_input.clear();
        Ok(())
    }

    /// Set status message
    pub fn set_status(&mut self, message: &str, level: StatusLevel) {
        self.status_message = Some((message.to_string(), level));
    }

    // Editor operations
    fn editor_insert_char(&mut self, c: char) {
        if self.editor_content.is_empty() {
            self.editor_content.push(String::new());
        }

        let (line, col) = self.editor_cursor;
        if let Some(content) = self.editor_content.get_mut(line) {
            let col = col.min(content.len());
            content.insert(col, c);
            self.editor_cursor.1 = col + 1;
        }
    }

    fn editor_insert_newline(&mut self) {
        let (line, col) = self.editor_cursor;
        if let Some(content) = self.editor_content.get_mut(line) {
            let new_line = content.split_off(col.min(content.len()));
            self.editor_content.insert(line + 1, new_line);
            self.editor_cursor = (line + 1, 0);
            self.ensure_cursor_visible();
        } else {
            self.editor_content.push(String::new());
            self.editor_cursor = (self.editor_content.len() - 1, 0);
            self.ensure_cursor_visible();
        }
    }

    fn editor_backspace(&mut self) {
        let (line, col) = self.editor_cursor;

        if col > 0 {
            if let Some(content) = self.editor_content.get_mut(line) {
                let col = col.min(content.len());
                if col > 0 {
                    content.remove(col - 1);
                    self.editor_cursor.1 = col - 1;
                }
            }
        } else if line > 0 {
            // Merge with previous line
            let current_line = self.editor_content.remove(line);
            if let Some(prev_line) = self.editor_content.get_mut(line - 1) {
                let new_col = prev_line.len();
                prev_line.push_str(&current_line);
                self.editor_cursor = (line - 1, new_col);
                self.ensure_cursor_visible();
            }
        }
    }

    fn editor_delete(&mut self) {
        let (line, col) = self.editor_cursor;

        let content_len = self.editor_content.get(line).map(|c| c.len()).unwrap_or(0);
        let total_lines = self.editor_content.len();

        if col < content_len {
            if let Some(content) = self.editor_content.get_mut(line) {
                content.remove(col);
            }
        } else if line + 1 < total_lines {
            // Merge with next line
            let next_line = self.editor_content.remove(line + 1);
            if let Some(content) = self.editor_content.get_mut(line) {
                content.push_str(&next_line);
            }
        }
    }

    fn editor_move_cursor_left(&mut self) {
        if self.editor_cursor.1 > 0 {
            self.editor_cursor.1 -= 1;
        } else if self.editor_cursor.0 > 0 {
            self.editor_cursor.0 -= 1;
            self.editor_cursor.1 = self
                .editor_content
                .get(self.editor_cursor.0)
                .map_or(0, |l| l.len());
            self.ensure_cursor_visible();
        }
    }

    fn editor_move_cursor_right(&mut self) {
        let line_len = self
            .editor_content
            .get(self.editor_cursor.0)
            .map_or(0, |l| l.len());

        if self.editor_cursor.1 < line_len {
            self.editor_cursor.1 += 1;
        } else if self.editor_cursor.0 + 1 < self.editor_content.len() {
            self.editor_cursor.0 += 1;
            self.editor_cursor.1 = 0;
            self.ensure_cursor_visible();
        }
    }

    fn editor_move_cursor_up(&mut self) {
        if self.editor_cursor.0 > 0 {
            self.editor_cursor.0 -= 1;
            let line_len = self
                .editor_content
                .get(self.editor_cursor.0)
                .map_or(0, |l| l.len());
            self.editor_cursor.1 = self.editor_cursor.1.min(line_len);
            self.ensure_cursor_visible();
        }
    }

    fn editor_move_cursor_down(&mut self) {
        if self.editor_cursor.0 + 1 < self.editor_content.len() {
            self.editor_cursor.0 += 1;
            let line_len = self
                .editor_content
                .get(self.editor_cursor.0)
                .map_or(0, |l| l.len());
            self.editor_cursor.1 = self.editor_cursor.1.min(line_len);
            self.ensure_cursor_visible();
        }
    }

    /// Ensure the cursor is visible by adjusting editor_scroll.
    /// Uses a default visible height of 20 lines (approximate terminal height minus borders).
    fn ensure_cursor_visible(&mut self) {
        // Use a reasonable default for visible height
        // This will be approximately correct for most terminal sizes
        let visible_height = 20_usize;

        // If cursor is above the visible area, scroll up
        if self.editor_cursor.0 < self.editor_scroll {
            self.editor_scroll = self.editor_cursor.0;
        }

        // If cursor is below the visible area, scroll down
        if self.editor_cursor.0 >= self.editor_scroll + visible_height {
            self.editor_scroll = self.editor_cursor.0 - visible_height + 1;
        }
    }

    // Vim-style editor navigation methods

    fn editor_move_to_line_start(&mut self) {
        self.editor_cursor.1 = 0;
    }

    fn editor_move_to_line_end(&mut self) {
        if let Some(line) = self.editor_content.get(self.editor_cursor.0) {
            self.editor_cursor.1 = line.len();
        }
    }

    fn editor_move_to_first_non_whitespace(&mut self) {
        if let Some(line) = self.editor_content.get(self.editor_cursor.0) {
            let first_non_ws = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);
            self.editor_cursor.1 = first_non_ws;
        }
    }

    fn editor_move_to_first_line(&mut self) {
        self.editor_cursor = (0, 0);
        self.editor_scroll = 0;
    }

    fn editor_move_to_last_line(&mut self) {
        if !self.editor_content.is_empty() {
            self.editor_cursor.0 = self.editor_content.len() - 1;
            self.editor_cursor.1 = 0;
            // Adjust scroll to show last line
            let visible_height = 20; // approximate
            if self.editor_cursor.0 >= visible_height {
                self.editor_scroll = self.editor_cursor.0 - visible_height + 1;
            }
        }
    }

    fn editor_move_word_forward(&mut self) {
        let (line, col) = self.editor_cursor;
        if let Some(content) = self.editor_content.get(line) {
            let chars: Vec<char> = content.chars().collect();
            let mut new_col = col;

            // Skip current word (non-whitespace)
            while new_col < chars.len() && !chars[new_col].is_whitespace() {
                new_col += 1;
            }
            // Skip whitespace
            while new_col < chars.len() && chars[new_col].is_whitespace() {
                new_col += 1;
            }

            if new_col < chars.len() {
                self.editor_cursor.1 = new_col;
            } else if line + 1 < self.editor_content.len() {
                // Move to start of next line
                self.editor_cursor = (line + 1, 0);
                // Skip leading whitespace on next line
                self.editor_move_to_first_non_whitespace();
                self.ensure_cursor_visible();
            } else {
                // End of file, go to end of line
                self.editor_cursor.1 = chars.len();
            }
        }
    }

    fn editor_move_word_backward(&mut self) {
        let (line, col) = self.editor_cursor;
        if col > 0 {
            if let Some(content) = self.editor_content.get(line) {
                let chars: Vec<char> = content.chars().collect();
                let mut new_col = col.saturating_sub(1);

                // Skip whitespace backwards
                while new_col > 0 && chars[new_col].is_whitespace() {
                    new_col -= 1;
                }
                // Skip to start of word
                while new_col > 0 && !chars[new_col - 1].is_whitespace() {
                    new_col -= 1;
                }

                self.editor_cursor.1 = new_col;
            }
        } else if line > 0 {
            // Move to end of previous line
            self.editor_cursor.0 = line - 1;
            if let Some(prev_line) = self.editor_content.get(line - 1) {
                self.editor_cursor.1 = prev_line.len();
            }
            self.ensure_cursor_visible();
        }
    }

    fn editor_move_word_end(&mut self) {
        let (line, col) = self.editor_cursor;
        if let Some(content) = self.editor_content.get(line) {
            let chars: Vec<char> = content.chars().collect();
            let mut new_col = col;

            // Move at least one character
            if new_col < chars.len() {
                new_col += 1;
            }

            // Skip whitespace
            while new_col < chars.len() && chars[new_col].is_whitespace() {
                new_col += 1;
            }
            // Go to end of word
            while new_col < chars.len() && !chars[new_col].is_whitespace() {
                new_col += 1;
            }

            if new_col > 0 && new_col <= chars.len() {
                self.editor_cursor.1 = new_col.saturating_sub(1);
            } else if line + 1 < self.editor_content.len() {
                // Move to next line
                self.editor_cursor = (line + 1, 0);
                self.ensure_cursor_visible();
            }
        }
    }

    fn editor_insert_line_below(&mut self) {
        let line = self.editor_cursor.0;
        self.editor_content.insert(line + 1, String::new());
        self.editor_cursor = (line + 1, 0);
        self.ensure_cursor_visible();
    }

    fn editor_insert_line_above(&mut self) {
        let line = self.editor_cursor.0;
        self.editor_content.insert(line, String::new());
        self.editor_cursor = (line, 0);
        self.ensure_cursor_visible();
    }

    fn editor_delete_char(&mut self) {
        // Delete character under cursor (vim 'x')
        let (line, col) = self.editor_cursor;
        if let Some(content) = self.editor_content.get_mut(line) {
            if col < content.len() {
                content.remove(col);
                // Adjust cursor if at end of line
                if col >= content.len() && col > 0 {
                    self.editor_cursor.1 = content.len().saturating_sub(1);
                }
            }
        }
    }

    fn editor_delete_line(&mut self) {
        // Delete entire line (vim 'dd')
        if !self.editor_content.is_empty() {
            let line = self.editor_cursor.0;
            self.editor_content.remove(line);

            // Ensure at least one empty line
            if self.editor_content.is_empty() {
                self.editor_content.push(String::new());
            }

            // Adjust cursor position
            if self.editor_cursor.0 >= self.editor_content.len() {
                self.editor_cursor.0 = self.editor_content.len().saturating_sub(1);
            }
            self.editor_cursor.1 = 0;
        }
    }

    fn editor_delete_to_end(&mut self) {
        // Delete from cursor to end of line (vim 'D')
        let (line, col) = self.editor_cursor;
        if let Some(content) = self.editor_content.get_mut(line) {
            content.truncate(col);
            // Adjust cursor if needed
            if self.editor_cursor.1 > 0 {
                self.editor_cursor.1 = col.saturating_sub(1);
            }
        }
    }

    fn editor_page_up(&mut self) {
        let page_size = 20; // Could be calculated from visible area
        self.editor_cursor.0 = self.editor_cursor.0.saturating_sub(page_size);
        self.editor_scroll = self.editor_scroll.saturating_sub(page_size);
        // Adjust column to line length
        let line_len = self
            .editor_content
            .get(self.editor_cursor.0)
            .map_or(0, |l| l.len());
        self.editor_cursor.1 = self.editor_cursor.1.min(line_len);
    }

    fn editor_page_down(&mut self) {
        let page_size = 20; // Could be calculated from visible area
        let max_line = self.editor_content.len().saturating_sub(1);
        self.editor_cursor.0 = (self.editor_cursor.0 + page_size).min(max_line);
        self.editor_scroll = (self.editor_scroll + page_size).min(max_line);
        // Adjust column to line length
        let line_len = self
            .editor_content
            .get(self.editor_cursor.0)
            .map_or(0, |l| l.len());
        self.editor_cursor.1 = self.editor_cursor.1.min(line_len);
    }

    /// Get flattened visible file entries for rendering (with filter applied)
    pub fn get_visible_files(&self) -> Vec<&FileEntry> {
        fn collect_visible<'a>(
            entries: &'a [FileEntry],
            result: &mut Vec<&'a FileEntry>,
            filter: &str,
        ) {
            let filter_lower = filter.to_lowercase();
            for entry in entries {
                // Check if this entry matches the filter
                let matches_filter =
                    filter.is_empty() || entry.name.to_lowercase().contains(&filter_lower);

                // For directories, also check if any children match
                let has_matching_children = entry.is_dir
                    && entry.is_expanded
                    && App::has_matching_descendants(&entry.children, &filter_lower);

                if matches_filter || has_matching_children || entry.is_dir {
                    // Include directories to maintain tree structure, but only if
                    // they match or have matching descendants (when filter is active)
                    if filter.is_empty() || matches_filter || has_matching_children {
                        result.push(entry);
                    }
                }

                if entry.is_expanded {
                    collect_visible(&entry.children, result, filter);
                }
            }
        }

        let mut result = Vec::new();
        collect_visible(&self.file_tree, &mut result, &self.filter_query);
        result
    }

    /// Check if any descendants match the filter
    fn has_matching_descendants(entries: &[FileEntry], filter_lower: &str) -> bool {
        for entry in entries {
            if entry.name.to_lowercase().contains(filter_lower) {
                return true;
            }
            if entry.is_expanded && Self::has_matching_descendants(&entry.children, filter_lower) {
                return true;
            }
        }
        false
    }
}
