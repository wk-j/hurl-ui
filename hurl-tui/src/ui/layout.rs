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

/// Panel visibility options
#[derive(Default)]
pub struct PanelVisibility {
    /// Whether to show the assertions panel
    pub show_assertions: bool,
    /// Whether to show the editor panel
    pub show_editor: bool,
    /// Whether to show the response panel
    pub show_response: bool,
}

/// Create the application layout with configurable sidebar width and panel visibility
pub fn create_layout(area: Rect, sidebar_width: u16, visibility: &PanelVisibility) -> AppLayout {
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
    // Adapt based on which panels are visible
    let (editor, results_area) = match (visibility.show_editor, visibility.show_response) {
        (true, true) => {
            // Both visible: standard 55/45 split
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(55), // Editor
                    Constraint::Percentage(45), // Results area
                ])
                .split(main_content);
            (chunks[0], chunks[1])
        }
        (true, false) => {
            // Editor only: editor takes full height
            (main_content, Rect::default())
        }
        (false, true) => {
            // Response only: results area takes full height
            (Rect::default(), main_content)
        }
        (false, false) => {
            // Neither visible (shouldn't happen due to guard, but handle gracefully)
            // Default to showing both
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(main_content);
            (chunks[0], chunks[1])
        }
    };

    // Results area split: response and assertions (if visible)
    let (response, assertions) = if !visibility.show_response {
        // Response hidden - both response and assertions get zero rects
        (Rect::default(), Rect::default())
    } else if visibility.show_assertions {
        let results_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Response
                Constraint::Percentage(40), // Assertions
            ])
            .split(results_area);
        (results_chunks[0], results_chunks[1])
    } else {
        // Assertions hidden - response takes full width
        (results_area, Rect::default())
    };

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
