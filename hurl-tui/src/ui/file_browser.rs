//! File browser panel
//!
//! Displays the file tree of .hurl files with hacker terminal aesthetic.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use super::theme::{BoxChars, HackerTheme};
use crate::app::{ActivePanel, App, AppMode};

/// Render the file browser panel
pub fn render_file_browser(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_active = app.active_panel == ActivePanel::FileBrowser;
    let is_filtering = app.mode == AppMode::Filter;

    // Build title with filter indicator
    let title = if is_filtering {
        format!(
            " {} Files [>{}|] ",
            BoxChars::TRIANGLE_RIGHT,
            app.filter_query
        )
    } else if !app.filter_query.is_empty() {
        format!(
            " {} Files [>{}] ",
            BoxChars::TRIANGLE_RIGHT,
            app.filter_query
        )
    } else {
        format!(" {} Files ", BoxChars::TRIANGLE_RIGHT)
    };

    let border_color = if is_filtering {
        HackerTheme::MODE_FILTER_FG
    } else if is_active {
        HackerTheme::MATRIX_GREEN
    } else {
        HackerTheme::BORDER_DIM
    };

    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(HackerTheme::VOID_BLACK));

    let visible_files = app.get_visible_files();
    let items: Vec<ListItem> = visible_files
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let indent = "  ".repeat(entry.depth);
            let icon = if entry.is_dir {
                if entry.is_expanded {
                    format!("{} ", BoxChars::TRIANGLE_DOWN)
                } else {
                    format!("{} ", BoxChars::TRIANGLE_RIGHT)
                }
            } else {
                format!("{} ", BoxChars::DOT)
            };

            let dir_icon = if entry.is_dir {
                format!("{} ", BoxChars::DIAMOND)
            } else {
                "".to_string()
            };

            let name = &entry.name;
            let display = format!("{}{}{}{}", indent, icon, dir_icon, name);

            let style = if idx == app.file_tree_index {
                Style::default()
                    .fg(HackerTheme::SELECTED_FG)
                    .bg(HackerTheme::SELECTED_BG)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(HackerTheme::CYBER_CYAN_DIM)
            } else {
                Style::default().fg(HackerTheme::TEXT_PRIMARY)
            };

            ListItem::new(Line::from(Span::styled(display, style)))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().bg(HackerTheme::VOID_BLACK))
        .highlight_style(
            Style::default()
                .fg(HackerTheme::SELECTED_FG)
                .bg(HackerTheme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut app.file_tree_state);
}
