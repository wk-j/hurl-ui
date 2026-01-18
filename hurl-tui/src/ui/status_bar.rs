//! Status bar
//!
//! Displays the bottom status bar with mode, shortcuts, and messages.
//! Hacker terminal aesthetic with matrix-inspired colors.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, AppMode, StatusLevel};
use super::theme::{HackerTheme, BoxChars};

/// Render the status bar
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    // Mode indicator with hacker styling
    match app.mode {
        AppMode::Normal => {
            spans.push(Span::styled(
                format!(" {} NORMAL ", BoxChars::TERMINAL_PROMPT),
                Style::default()
                    .fg(HackerTheme::MODE_NORMAL_FG)
                    .bg(HackerTheme::MODE_NORMAL_BG)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        AppMode::Editing => {
            spans.push(Span::styled(
                format!(" {} EDIT ", BoxChars::SCANNER),
                Style::default()
                    .fg(HackerTheme::MODE_EDIT_FG)
                    .bg(HackerTheme::MODE_EDIT_BG)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        AppMode::Search => {
            spans.push(Span::styled(
                format!(" {} SEARCH ", BoxChars::BULLET),
                Style::default()
                    .fg(HackerTheme::MODE_SEARCH_FG)
                    .bg(HackerTheme::MODE_SEARCH_BG)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(" ", Style::default().bg(HackerTheme::DARK_BG)));
            spans.push(Span::styled(
                format!("/{}_", app.search_query),
                Style::default().fg(HackerTheme::CYBER_CYAN).bg(HackerTheme::DARK_BG),
            ));
        }
        AppMode::Command => {
            spans.push(Span::styled(
                format!(" {} CMD ", BoxChars::LAMBDA),
                Style::default()
                    .fg(HackerTheme::MODE_COMMAND_FG)
                    .bg(HackerTheme::MODE_COMMAND_BG)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(" ", Style::default().bg(HackerTheme::DARK_BG)));
            spans.push(Span::styled(
                format!(":{}_", app.command_input),
                Style::default().fg(HackerTheme::ELECTRIC_PURPLE).bg(HackerTheme::DARK_BG),
            ));
        }
        AppMode::Filter => {
            spans.push(Span::styled(
                format!(" {} FILTER ", BoxChars::GLITCH_2),
                Style::default()
                    .fg(HackerTheme::MODE_FILTER_FG)
                    .bg(HackerTheme::MODE_FILTER_BG)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(" ", Style::default().bg(HackerTheme::DARK_BG)));
            spans.push(Span::styled(
                format!(">{}_", app.filter_query),
                Style::default().fg(HackerTheme::AMBER_WARNING).bg(HackerTheme::DARK_BG),
            ));
        }
    }

    spans.push(Span::styled(" ", Style::default().bg(HackerTheme::DARK_BG)));

    // Status message with cyber colors
    if let Some((message, level)) = &app.status_message {
        let color = match level {
            StatusLevel::Info => HackerTheme::TEXT_PRIMARY,
            StatusLevel::Success => HackerTheme::NEON_GREEN,
            StatusLevel::Warning => HackerTheme::AMBER_WARNING,
            StatusLevel::Error => HackerTheme::NEON_RED,
        };
        let prefix = match level {
            StatusLevel::Info => format!("{} ", BoxChars::DOT),
            StatusLevel::Success => format!("{} ", BoxChars::CHECK),
            StatusLevel::Warning => format!("{} ", BoxChars::DIAMOND),
            StatusLevel::Error => format!("{} ", BoxChars::CROSS),
        };
        spans.push(Span::styled(
            format!("{}{}", prefix, message),
            Style::default().fg(color).bg(HackerTheme::DARK_BG),
        ));
    }

    // Running indicator with animated spinner
    if app.is_running {
        let spinner = BoxChars::spinner(app.spinner_frame);
        let matrix_char = BoxChars::spinner_matrix(app.spinner_frame);
        
        spans.push(Span::styled(
            format!(" {} ", spinner),
            Style::default()
                .fg(HackerTheme::MATRIX_GREEN_BRIGHT)
                .bg(HackerTheme::DARK_BG)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "EXECUTING",
            Style::default()
                .fg(HackerTheme::RUNNING)
                .bg(HackerTheme::DARK_BG)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {} ", matrix_char),
            Style::default()
                .fg(HackerTheme::CYBER_CYAN)
                .bg(HackerTheme::DARK_BG)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Right-aligned shortcuts with hacker style
    let shortcuts = if app.mode == AppMode::Normal {
        format!(
            " [r]un {} [e]dit {} [v]ars {} [?]help {} [q]uit ",
            BoxChars::DOT, BoxChars::DOT, BoxChars::DOT, BoxChars::DOT
        )
    } else {
        format!(" [Esc] {} back ", BoxChars::ARROW_RIGHT)
    };

    // Calculate padding
    let left_width: usize = spans.iter().map(|s| s.content.len()).sum();
    let right_width = shortcuts.len();
    let total_width = area.width as usize;
    let padding = total_width.saturating_sub(left_width + right_width);

    spans.push(Span::styled(
        " ".repeat(padding),
        Style::default().bg(HackerTheme::DARK_BG),
    ));

    spans.push(Span::styled(
        shortcuts,
        Style::default().fg(HackerTheme::TEXT_MUTED).bg(HackerTheme::DARK_BG),
    ));

    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(HackerTheme::DARK_BG));

    frame.render_widget(paragraph, area);
}
