//! Configuration module
//!
//! This module handles application configuration loading and management.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// UI settings
    #[serde(default)]
    pub ui: UiConfig,

    /// Editor settings
    #[serde(default)]
    pub editor: EditorConfig,

    /// Keybindings
    #[serde(default)]
    pub keys: KeyConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ui: UiConfig::default(),
            editor: EditorConfig::default(),
            keys: KeyConfig::default(),
        }
    }
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Default working directory
    pub default_directory: Option<PathBuf>,

    /// Hurl binary path (if not in PATH)
    pub hurl_path: Option<PathBuf>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum history entries
    #[serde(default = "default_max_history")]
    pub max_history: usize,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_directory: None,
            hurl_path: None,
            timeout: default_timeout(),
            max_history: default_max_history(),
        }
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_max_history() -> usize {
    100
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Show line numbers in editor
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,

    /// Show file icons
    #[serde(default = "default_true")]
    pub show_icons: bool,

    /// Color theme
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Panel border style
    #[serde(default = "default_border_style")]
    pub border_style: String,

    /// File browser width percentage
    #[serde(default = "default_file_browser_width")]
    pub file_browser_width: u16,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            show_icons: true,
            theme: default_theme(),
            border_style: default_border_style(),
            file_browser_width: default_file_browser_width(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_border_style() -> String {
    "rounded".to_string()
}

fn default_file_browser_width() -> u16 {
    20
}

/// Editor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Tab size
    #[serde(default = "default_tab_size")]
    pub tab_size: usize,

    /// Use spaces for tabs
    #[serde(default = "default_true")]
    pub use_spaces: bool,

    /// Auto-save on run
    #[serde(default)]
    pub auto_save: bool,

    /// Syntax highlighting enabled
    #[serde(default = "default_true")]
    pub syntax_highlighting: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: default_tab_size(),
            use_spaces: true,
            auto_save: false,
            syntax_highlighting: true,
        }
    }
}

fn default_tab_size() -> usize {
    2
}

/// Keybinding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    /// Quit key
    #[serde(default = "default_quit_key")]
    pub quit: String,

    /// Run request key
    #[serde(default = "default_run_key")]
    pub run: String,

    /// Edit mode key
    #[serde(default = "default_edit_key")]
    pub edit: String,

    /// Save key
    #[serde(default = "default_save_key")]
    pub save: String,

    /// Help key
    #[serde(default = "default_help_key")]
    pub help: String,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            quit: default_quit_key(),
            run: default_run_key(),
            edit: default_edit_key(),
            save: default_save_key(),
            help: default_help_key(),
        }
    }
}

fn default_quit_key() -> String {
    "q".to_string()
}

fn default_run_key() -> String {
    "r".to_string()
}

fn default_edit_key() -> String {
    "e".to_string()
}

fn default_save_key() -> String {
    "ctrl+s".to_string()
}

fn default_help_key() -> String {
    "?".to_string()
}

impl Config {
    /// Load configuration from file or use defaults
    pub fn load() -> Result<Self> {
        // Try to load from standard config locations
        let config_paths = vec![
            dirs::config_dir().map(|p| p.join("hurl-tui").join("config.toml")),
            Some(PathBuf::from(".hurl-tui.toml")),
            Some(PathBuf::from("hurl-tui.toml")),
        ];

        for path in config_paths.into_iter().flatten() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str(&content) {
                        return Ok(config);
                    }
                }
            }
        }

        // Return default config
        Ok(Config::default())
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
