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
pub use layout::{create_layout, AppLayout};
pub use response::render_response;
pub use status_bar::render_status_bar;
pub use variables::render_variables;

/// Main draw function that renders the entire UI
pub fn draw(frame: &mut Frame, app: &mut App) {
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

    // Process and render effects
    render_effects(frame, app, &layout);
}

/// Render all active effects
fn render_effects(frame: &mut Frame, app: &mut App, layout: &AppLayout) {
    use crate::app::ActivePanel;
    use crate::effects::EffectId;
    use tachyonfx::Shader;

    // Get the time delta and convert to tachyonfx Duration
    let delta = app.effect_manager.tick();
    let delta_ms = delta.as_millis() as u32;
    // Ensure minimum 1ms to avoid zero-duration issues
    let delta_ms = delta_ms.max(1);
    let fx_duration = tachyonfx::Duration::from_millis(delta_ms);

    // Helper to get area for a panel
    let get_panel_area = |panel: ActivePanel| -> ratatui::layout::Rect {
        match panel {
            ActivePanel::FileBrowser => layout.file_browser,
            ActivePanel::Editor => layout.editor,
            ActivePanel::Response => layout.response,
            ActivePanel::Assertions => layout.assertions,
            ActivePanel::Variables => layout.variables,
        }
    };

    // Calculate help overlay area (same as in render_help: 60% width, 70% height, centered)
    let help_area = layout::centered_rect(60, 70, frame.area());

    // Render all effects with appropriate areas
    let buf = frame.buffer_mut();

    for (effect_id, effect, stored_area) in app.effect_manager.effects_iter_mut() {
        // Determine the correct area based on EffectId
        let area = if stored_area.width > 0 && stored_area.height > 0 {
            // Use explicitly stored area if valid
            stored_area
        } else {
            // Map EffectId to the appropriate panel/area
            match effect_id {
                EffectId::PanelFocus(panel) => get_panel_area(*panel),
                EffectId::ExecutionStart => layout.response,
                EffectId::ExecutionComplete => layout.response,
                EffectId::ResponseUpdate => layout.response,
                EffectId::HelpOverlay => help_area,
                EffectId::StatusNotification => layout.status_bar,
            }
        };

        // Process the effect directly on the buffer
        effect.process(fx_duration, buf, area);
    }

    // Clean up completed effects
    app.effect_manager.process_effects();
}
