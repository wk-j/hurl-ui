//! File browser panel
//!
//! Displays the file tree of .hurl files in the working directory.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::{ActivePanel, App};

/// Render the file browser panel
pub fn render_file_browser(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == ActivePanel::FileBrowser;

    let block = Block::default()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_active {
            Color::Cyan
        } else {
            Color::DarkGray
        }));

    let visible_files = app.get_visible_files();
    let items: Vec<ListItem> = visible_files
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let indent = "  ".repeat(entry.depth);
            let icon = if entry.is_dir {
                if entry.is_expanded {
                    "v "
                } else {
                    "> "
                }
            } else {
                "  "
            };

            let file_icon = if entry.is_dir {
                "+"
            } else {
                " "
            };

            let name = &entry.name;
            let display = format!("{}{}{} {}", indent, icon, file_icon, name);

            let style = if idx == app.file_tree_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(display, style)))
        })
        .collect();

    let list = List::new(items).block(block);

    frame.render_widget(list, area);
}
