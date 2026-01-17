//! Application state management
//!
//! This module contains the core application state and logic for the Hurl TUI.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::Config;
use crate::parser::HurlFile;
use crate::runner::{ExecutionResult, Runner};

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
}

/// Active panel in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

    /// File tree entries
    pub file_tree: Vec<FileEntry>,

    /// Index of selected file in the flattened tree
    pub file_tree_index: usize,

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

    /// Variables for the current environment
    pub variables: Vec<Variable>,

    /// Current environment name
    pub current_environment: String,

    /// Available environments
    pub environments: Vec<String>,

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
            file_tree: Vec::new(),
            file_tree_index: 0,
            current_file: None,
            current_file_path: None,
            editor_content: Vec::new(),
            editor_cursor: (0, 0),
            editor_scroll: 0,
            execution_result: None,
            file_execution_states: HashMap::new(),
            is_running: false,
            variables: Vec::new(),
            current_environment: String::from("default"),
            environments: vec![String::from("default")],
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
        };

        // Load file tree
        app.refresh_file_tree()?;

        // Load environments
        app.load_environments()?;

        // Restore last opened file
        app.restore_last_opened_file();

        Ok(app)
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

    /// Save the current state (last opened file and execution states)
    fn save_state(&self) {
        let state = PersistedState {
            last_opened_file: self.current_file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            file_tree_index: self.file_tree_index,
            file_execution_states: self.file_execution_states.clone(),
        };
        
        if let Ok(content) = serde_json::to_string_pretty(&state) {
            let _ = std::fs::write(self.get_state_file_path(), content);
        }
    }

    /// Restore the last opened file and execution states from persisted state
    fn restore_last_opened_file(&mut self) {
        let state_path = self.get_state_file_path();
        
        if let Ok(content) = std::fs::read_to_string(&state_path) {
            if let Ok(state) = serde_json::from_str::<PersistedState>(&content) {
                // Restore file execution states
                self.file_execution_states = state.file_execution_states;
                
                // Restore last opened file
                if let Some(file_path) = state.last_opened_file {
                    let path = PathBuf::from(&file_path);
                    if path.exists() {
                        let _ = self.preview_file(&path);
                        
                        // Try to restore file tree index
                        let max = self.get_visible_file_count().saturating_sub(1);
                        self.file_tree_index = state.file_tree_index.min(max);
                        
                        self.set_status(
                            &format!("Restored: {}", path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default()),
                            StatusLevel::Info
                        );
                    }
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
        // Clear status message after some time (could track time)
        // For now, status messages persist until next action
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

            // Output AI context to stdout and quit (o = output for pipe/Helix)
            KeyCode::Char('o') => {
                self.output_ai_context_and_quit();
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle key events in editing mode
    fn handle_editing_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                self.editor_insert_newline();
            }
            KeyCode::Backspace => {
                self.editor_backspace();
            }
            KeyCode::Delete => {
                self.editor_delete();
            }
            KeyCode::Left => {
                self.editor_move_cursor_left();
            }
            KeyCode::Right => {
                self.editor_move_cursor_right();
            }
            KeyCode::Up => {
                self.editor_move_cursor_up();
            }
            KeyCode::Down => {
                self.editor_move_cursor_down();
            }
            KeyCode::Home => {
                self.editor_cursor.1 = 0;
            }
            KeyCode::End => {
                if let Some(line) = self.editor_content.get(self.editor_cursor.0) {
                    self.editor_cursor.1 = line.len();
                }
            }
            KeyCode::Char(c) => {
                self.editor_insert_char(c);
            }
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
                    self.set_status(&format!("Filter: {} ({} files)", self.filter_query, count), StatusLevel::Info);
                }
            }
            KeyCode::Backspace => {
                self.filter_query.pop();
                self.file_tree_index = 0;
            }
            KeyCode::Char(c) => {
                self.filter_query.push(c);
                self.file_tree_index = 0;
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
        self.active_panel = match self.active_panel {
            ActivePanel::FileBrowser => ActivePanel::Editor,
            ActivePanel::Editor => ActivePanel::Response,
            ActivePanel::Response => ActivePanel::Assertions,
            ActivePanel::Assertions => ActivePanel::Variables,
            ActivePanel::Variables => ActivePanel::FileBrowser,
        };
    }

    /// Navigate to previous panel
    fn previous_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::FileBrowser => ActivePanel::Variables,
            ActivePanel::Editor => ActivePanel::FileBrowser,
            ActivePanel::Response => ActivePanel::Editor,
            ActivePanel::Assertions => ActivePanel::Response,
            ActivePanel::Variables => ActivePanel::Assertions,
        };
    }

    /// Navigate down in current panel
    fn navigate_down(&mut self) {
        match self.active_panel {
            ActivePanel::FileBrowser => {
                let max = self.get_visible_file_count().saturating_sub(1);
                if self.file_tree_index < max {
                    self.file_tree_index += 1;
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
            ActivePanel::FileBrowser => self.file_tree_index = 0,
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
                    if let Ok(children) = Self::load_directory_children(&entry.path, entry.depth + 1)
                    {
                        entry.children = children;
                    }
                }
            }
        }
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

    /// Refresh the file tree
    pub fn refresh_file_tree(&mut self) -> Result<()> {
        self.file_tree = Self::load_directory_children(&self.working_dir, 0)?;
        // Auto-expand directories containing .hurl files
        self.auto_expand_hurl_directories();
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
                    if let Ok(children) = App::load_directory_children(&entry.path, entry.depth + 1) {
                        entry.children = children;
                    }
                }
                
                // Check if this directory or its children contain .hurl files
                let has_hurl = entry.children.iter().any(|c| {
                    !c.is_dir || Self::dir_contains_hurl(&c.path)
                });
                
                if has_hurl {
                    entry.is_expanded = true;
                    // Recursively expand children
                    Self::expand_entries_with_hurl(&mut entry.children);
                }
            }
        }
    }

    /// Check if a directory contains any .hurl files (recursively)
    fn dir_contains_hurl(path: &PathBuf) -> bool {
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

                // Skip hidden files and directories
                if path
                    .file_name()
                    .map_or(false, |n| n.to_string_lossy().starts_with('.'))
                {
                    continue;
                }

                // Only include directories and .hurl files
                if path.is_dir() || path.extension().map_or(false, |e| e == "hurl") {
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
        let hurl_file = crate::parser::parse_hurl_file(&content)?;

        self.current_file = Some(hurl_file);
        self.current_file_path = Some(path.clone());
        self.editor_content = content.lines().map(String::from).collect();
        self.editor_cursor = (0, 0);
        self.editor_scroll = 0;
        
        // Restore execution state for this file if available
        let relative_path = self.get_relative_path(path);
        self.execution_result = self.file_execution_states.get(&relative_path).cloned();
        
        self.response_scroll = 0;
        self.assertions_scroll = 0;

        if switch_panel {
            self.active_panel = ActivePanel::Editor;
        }
        
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        
        // Show status with execution state info
        if self.execution_result.is_some() {
            self.set_status(&format!("Preview: {} (with cached result)", file_name), StatusLevel::Info);
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
            if !entry.is_dir && entry.path.extension().map_or(false, |e| e == "hurl") {
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

        // Build variables map
        let variables: HashMap<String, String> = self
            .variables
            .iter()
            .map(|v| (v.name.clone(), v.value.clone()))
            .collect();

        // Run the request
        let start = std::time::Instant::now();
        let result = self.runner.run(&path, &variables).await;
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
                self.file_execution_states.insert(relative_path, exec_result.clone());

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
                } else {
                    self.set_status("Request completed with failures", StatusLevel::Warning);
                }
            }
            Err(e) => {
                self.set_status(&format!("Error: {e}"), StatusLevel::Error);
            }
        }

        Ok(())
    }

    /// Load environments from config
    fn load_environments(&mut self) -> Result<()> {
        // Look for environment files in the working directory
        let env_dir = self.working_dir.join("environments");
        if env_dir.exists() {
            if let Ok(read_dir) = std::fs::read_dir(&env_dir) {
                for entry in read_dir.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .map_or(false, |e| e == "env" || e == "toml" || e == "json")
                    {
                        if let Some(name) = path.file_stem() {
                            let name = name.to_string_lossy().to_string();
                            if !self.environments.contains(&name) {
                                self.environments.push(name);
                            }
                        }
                    }
                }
            }
        }

        self.load_current_environment_variables()?;

        Ok(())
    }

    /// Load variables for the current environment
    fn load_current_environment_variables(&mut self) -> Result<()> {
        self.variables.clear();

        // Try to load from environment file
        let env_file = self
            .working_dir
            .join("environments")
            .join(format!("{}.env", self.current_environment));

        if env_file.exists() {
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
        }

        Ok(())
    }

    /// Cycle through environments
    fn cycle_environment(&mut self) {
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
        } else {
            self.editor_content.push(String::new());
            self.editor_cursor = (self.editor_content.len() - 1, 0);
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
        }
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
                let matches_filter = filter.is_empty()
                    || entry.name.to_lowercase().contains(&filter_lower);
                
                // For directories, also check if any children match
                let has_matching_children = entry.is_dir && entry.is_expanded
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
