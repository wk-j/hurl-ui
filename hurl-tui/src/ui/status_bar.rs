//! Status bar
//!
//! Displays the bottom status bar with mode, shortcuts, and messages.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, AppMode, StatusLevel};

/// Render the status bar
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    // Mode indicator
    match app.mode {
        AppMode::Normal => {
            spans.push(Span::styled(
                " NORMAL ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        AppMode::Editing => {
            spans.push(Span::styled(
                " EDIT ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        AppMode::Search => {
            spans.push(Span::styled(
                " SEARCH ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(" ", Style::default()));
            spans.push(Span::styled(
                format!("/{}", app.search_query),
                Style::default().fg(Color::White),
            ));
        }
        AppMode::Command => {
            spans.push(Span::styled(
                " COMMAND ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(" ", Style::default()));
            spans.push(Span::styled(
                format!(":{}", app.command_input),
                Style::default().fg(Color::White),
            ));
        }
    }

    spans.push(Span::styled(" ", Style::default()));

    // Status message
    if let Some((message, level)) = &app.status_message {
        let color = match level {
            StatusLevel::Info => Color::White,
            StatusLevel::Success => Color::Green,
            StatusLevel::Warning => Color::Yellow,
            StatusLevel::Error => Color::Red,
        };
        spans.push(Span::styled(message.clone(), Style::default().fg(color)));
    }

    // Running indicator
    if app.is_running {
        spans.push(Span::styled(
            " [Running...] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Right-aligned shortcuts (calculate available space)
    let shortcuts = if app.mode == AppMode::Normal {
        " [r]un [e]dit [v]ars [?]help [q]uit "
    } else {
        " [Esc] back "
    };

    // Calculate padding
    let left_width: usize = spans.iter().map(|s| s.content.len()).sum();
    let right_width = shortcuts.len();
    let total_width = area.width as usize;
    let padding = total_width.saturating_sub(left_width + right_width);

    spans.push(Span::styled(
        " ".repeat(padding),
        Style::default(),
    ));

    spans.push(Span::styled(
        shortcuts,
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}
