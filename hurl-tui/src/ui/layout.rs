//! Layout management for the UI
//!
//! Defines the panel layout structure for the application.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Layout areas for different UI panels
pub struct AppLayout {
    /// File browser panel (left side)
    pub file_browser: Rect,
    /// Editor panel (top right)
    pub editor: Rect,
    /// Variables panel (bottom left)
    pub variables: Rect,
    /// Response panel (bottom center)
    pub response: Rect,
    /// Assertions panel (bottom right)
    pub assertions: Rect,
    /// Status bar (bottom)
    pub status_bar: Rect,
}

/// Create the application layout with configurable sidebar width
pub fn create_layout(area: Rect, sidebar_width: u16) -> AppLayout {
    // Clamp sidebar width to reasonable bounds (10-50%)
    let sidebar_pct = sidebar_width.clamp(10, 50);
    let main_pct = 100 - sidebar_pct;

    // Main vertical split: content area and status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Content area
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    let content_area = main_chunks[0];
    let status_bar = main_chunks[1];

    // Horizontal split: left sidebar and main content
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(sidebar_pct), // Left sidebar
            Constraint::Percentage(main_pct),    // Main content
        ])
        .split(content_area);

    let left_sidebar = horizontal_chunks[0];
    let main_content = horizontal_chunks[1];

    // Left sidebar split: file browser and variables
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70), // File browser
            Constraint::Percentage(30), // Variables
        ])
        .split(left_sidebar);

    let file_browser = left_chunks[0];
    let variables = left_chunks[1];

    // Main content split: editor (top) and results (bottom)
    let main_vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55), // Editor
            Constraint::Percentage(45), // Results area
        ])
        .split(main_content);

    let editor = main_vertical_chunks[0];
    let results_area = main_vertical_chunks[1];

    // Results area split: response and assertions
    let results_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Response
            Constraint::Percentage(40), // Assertions
        ])
        .split(results_area);

    let response = results_chunks[0];
    let assertions = results_chunks[1];

    AppLayout {
        file_browser,
        editor,
        variables,
        response,
        assertions,
        status_bar,
    }
}

/// Create a centered popup area
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
