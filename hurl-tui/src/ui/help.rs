//! Help overlay
//!
//! Displays a help popup with keyboard shortcuts in hacker terminal style.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use super::layout::centered_rect;
use super::theme::{HackerTheme, BoxChars};

/// Render the help overlay
pub fn render_help(frame: &mut Frame, _app: &App) {
    let area = centered_rect(60, 70, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} SYSTEM MANUAL ", BoxChars::TERMINAL_PROMPT))
        .title_style(Style::default().fg(HackerTheme::MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(HackerTheme::MATRIX_GREEN))
        .style(Style::default().bg(HackerTheme::VOID_BLACK));

    let help_text = vec![
        Line::from(Span::styled(
            format!("{} HURL-TUI :: COMMAND REFERENCE {}", BoxChars::GLITCH_2, BoxChars::GLITCH_2),
            Style::default()
                .fg(HackerTheme::MATRIX_GREEN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} NAVIGATION", BoxChars::TRIANGLE_RIGHT),
            Style::default()
                .fg(HackerTheme::SYNTAX_SECTION)
                .add_modifier(Modifier::BOLD),
        )),
        help_line("j/k Up/Down", "Move cursor"),
        help_line("h/l", "Switch panels"),
        help_line("Tab/S-Tab", "Cycle panels"),
        help_line("g/G", "Jump to start/end"),
        help_line("^d/^u", "Page down/up"),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} FILE BROWSER", BoxChars::TRIANGLE_RIGHT),
            Style::default()
                .fg(HackerTheme::SYNTAX_SECTION)
                .add_modifier(Modifier::BOLD),
        )),
        help_line("Enter", "Open file / Toggle dir"),
        help_line("Space", "Expand/collapse dir"),
        help_line("R", "Refresh tree"),
        help_line("/", "Search files"),
        help_line("f / F", "Filter / Clear filter"),
        help_line("p / P", "Copy / Paste file"),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} ACTIONS", BoxChars::TRIANGLE_RIGHT),
            Style::default()
                .fg(HackerTheme::SYNTAX_SECTION)
                .add_modifier(Modifier::BOLD),
        )),
        help_line("r", "Execute request"),
        help_line("e", "Edit mode (vim)"),
        help_line("v", "Variables panel"),
        help_line("E", "Cycle environment"),
        help_line("y / Y", "Copy path / response"),
        help_line("c", "Copy AI context"),
        help_line("o", "Output context & quit"),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} VIM EDIT MODE", BoxChars::TRIANGLE_RIGHT),
            Style::default()
                .fg(HackerTheme::SYNTAX_SECTION)
                .add_modifier(Modifier::BOLD),
        )),
        help_line("h/j/k/l", "Move cursor"),
        help_line("w/b/e", "Word motions"),
        help_line("0/$/^", "Line start/end"),
        help_line("i/a/I/A", "Insert mode"),
        help_line("o/O", "Open line below/above"),
        help_line("x/d/D", "Delete char/line/to-end"),
        help_line("Esc", "Back to normal mode"),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} COMMANDS", BoxChars::TRIANGLE_RIGHT),
            Style::default()
                .fg(HackerTheme::SYNTAX_SECTION)
                .add_modifier(Modifier::BOLD),
        )),
        help_line(":w", "Save file"),
        help_line(":q", "Quit"),
        help_line(":wq", "Save and quit"),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} GENERAL", BoxChars::TRIANGLE_RIGHT),
            Style::default()
                .fg(HackerTheme::SYNTAX_SECTION)
                .add_modifier(Modifier::BOLD),
        )),
        help_line("?", "Toggle help"),
        help_line("q", "Quit / Close"),
        help_line("Esc", "Cancel / Exit mode"),
        help_line("^c", "Force quit"),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} Press [q] or [?] to close {}", BoxChars::DOT, BoxChars::DOT),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        )),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Create a help line with key and description
fn help_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {} ", BoxChars::DOT),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        ),
        Span::styled(
            format!("{:14}", key),
            Style::default().fg(HackerTheme::MATRIX_GREEN),
        ),
        Span::styled(
            desc.to_string(),
            Style::default().fg(HackerTheme::TEXT_PRIMARY),
        ),
    ])
}
