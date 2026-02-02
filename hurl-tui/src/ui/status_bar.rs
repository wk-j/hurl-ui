//! Status bar
//!
//! Displays the bottom status bar with mode, shortcuts, and messages.
//! Clean modern aesthetic.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::theme::HackerTheme;
use crate::app::{App, AppMode, StatusLevel, VimMode};

/// Render the status bar
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    // Mode indicator - clean minimal style
    match app.mode {
        AppMode::Normal => {
            spans.push(Span::styled(
                " NORMAL ",
                Style::default()
                    .fg(HackerTheme::MODE_NORMAL_FG)
                    .bg(HackerTheme::MODE_NORMAL_BG),
            ));
        }
        AppMode::Editing => {
            let (vim_label, fg, bg) = match app.vim_mode {
                VimMode::Normal => (
                    " NORMAL ",
                    HackerTheme::MODE_NORMAL_FG,
                    HackerTheme::MODE_NORMAL_BG,
                ),
                VimMode::Insert => (
                    " INSERT ",
                    HackerTheme::MODE_EDIT_FG,
                    HackerTheme::MODE_EDIT_BG,
                ),
            };
            spans.push(Span::styled(vim_label, Style::default().fg(fg).bg(bg)));
        }
        AppMode::Search => {
            spans.push(Span::styled(
                " SEARCH ",
                Style::default()
                    .fg(HackerTheme::MODE_SEARCH_FG)
                    .bg(HackerTheme::MODE_SEARCH_BG),
            ));
            spans.push(Span::styled(
                "  ",
                Style::default().bg(HackerTheme::DARK_BG),
            ));
            spans.push(Span::styled(
                format!("/{}_", app.search_query),
                Style::default()
                    .fg(HackerTheme::CYBER_CYAN)
                    .bg(HackerTheme::DARK_BG),
            ));
        }
        AppMode::Command => {
            spans.push(Span::styled(
                " COMMAND ",
                Style::default()
                    .fg(HackerTheme::MODE_COMMAND_FG)
                    .bg(HackerTheme::MODE_COMMAND_BG),
            ));
            spans.push(Span::styled(
                "  ",
                Style::default().bg(HackerTheme::DARK_BG),
            ));
            spans.push(Span::styled(
                format!(":{}_", app.command_input),
                Style::default()
                    .fg(HackerTheme::ELECTRIC_PURPLE)
                    .bg(HackerTheme::DARK_BG),
            ));
        }
        AppMode::Filter => {
            spans.push(Span::styled(
                " FILTER ",
                Style::default()
                    .fg(HackerTheme::MODE_FILTER_FG)
                    .bg(HackerTheme::MODE_FILTER_BG),
            ));
            spans.push(Span::styled(
                "  ",
                Style::default().bg(HackerTheme::DARK_BG),
            ));
            spans.push(Span::styled(
                format!("{}_", app.filter_query),
                Style::default()
                    .fg(HackerTheme::AMBER_WARNING)
                    .bg(HackerTheme::DARK_BG),
            ));
        }
        AppMode::Rename => {
            spans.push(Span::styled(
                " RENAME ",
                Style::default()
                    .fg(HackerTheme::MODE_EDIT_FG)
                    .bg(HackerTheme::MODE_EDIT_BG),
            ));
            spans.push(Span::styled(
                "  ",
                Style::default().bg(HackerTheme::DARK_BG),
            ));
            spans.push(Span::styled(
                format!("{}_", app.rename_input),
                Style::default()
                    .fg(HackerTheme::CYBER_CYAN)
                    .bg(HackerTheme::DARK_BG),
            ));
        }
    }

    spans.push(Span::styled(
        "  ",
        Style::default().bg(HackerTheme::DARK_BG),
    ));

    // Status message
    if let Some((message, level)) = &app.status_message {
        let color = match level {
            StatusLevel::Info => HackerTheme::TEXT_SECONDARY,
            StatusLevel::Success => HackerTheme::NEON_GREEN,
            StatusLevel::Warning => HackerTheme::AMBER_WARNING,
            StatusLevel::Error => HackerTheme::NEON_RED,
        };
        spans.push(Span::styled(
            message.clone(),
            Style::default().fg(color).bg(HackerTheme::DARK_BG),
        ));
    }

    // Running indicator
    if app.is_running {
        let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let spinner = spinner_chars[app.spinner_frame % spinner_chars.len()];

        spans.push(Span::styled(
            format!("  {} Running...", spinner),
            Style::default()
                .fg(HackerTheme::RUNNING)
                .bg(HackerTheme::DARK_BG),
        ));
    }

    // Right-aligned hints - minimal
    let shortcuts = match app.mode {
        AppMode::Normal => " r:run  e:edit  ?:help  q:quit ",
        AppMode::Editing => match app.vim_mode {
            VimMode::Normal => " i:insert  q:quit ",
            VimMode::Insert => " Esc:normal ",
        },
        _ => " Esc:back ",
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
        Style::default()
            .fg(HackerTheme::TEXT_MUTED)
            .bg(HackerTheme::DARK_BG),
    ));

    let paragraph =
        Paragraph::new(Line::from(spans)).style(Style::default().bg(HackerTheme::DARK_BG));

    frame.render_widget(paragraph, area);
}
