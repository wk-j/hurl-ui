//! UI module
//!
//! This module contains all UI rendering components for the Hurl TUI.

mod assertions;
mod editor;
mod file_browser;
mod help;
mod layout;
mod response;
mod status_bar;
pub mod theme;
mod variables;

use ratatui::Frame;

use crate::app::App;

pub use assertions::render_assertions;
pub use editor::render_editor;
pub use file_browser::render_file_browser;
pub use help::render_help;
pub use layout::create_layout;
pub use response::render_response;
pub use status_bar::render_status_bar;
pub use variables::render_variables;

/// Main draw function that renders the entire UI
pub fn draw(frame: &mut Frame, app: &App) {
    let layout = create_layout(frame.area());

    // Render file browser (left panel)
    render_file_browser(frame, app, layout.file_browser);

    // Render editor (top right)
    render_editor(frame, app, layout.editor);

    // Render variables (bottom left)
    render_variables(frame, app, layout.variables);

    // Render response (bottom center)
    render_response(frame, app, layout.response);

    // Render assertions (bottom right)
    render_assertions(frame, app, layout.assertions);

    // Render status bar (bottom)
    render_status_bar(frame, app, layout.status_bar);

    // Render help overlay if active
    if app.show_help {
        render_help(frame, app);
    }
}
