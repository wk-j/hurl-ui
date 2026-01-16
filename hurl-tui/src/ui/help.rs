//! Help overlay
//!
//! Displays a help popup with keyboard shortcuts.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use super::layout::centered_rect;

/// Render the help overlay
pub fn render_help(frame: &mut Frame, _app: &App) {
    let area = centered_rect(60, 70, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(Span::styled(
            "Hurl TUI - Keyboard Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  j/k or Up/Down  ", Style::default().fg(Color::Green)),
            Span::styled("Move up/down", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  h/l            ", Style::default().fg(Color::Green)),
            Span::styled("Switch panels left/right", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Tab/Shift+Tab   ", Style::default().fg(Color::Green)),
            Span::styled("Cycle through panels", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  g/G             ", Style::default().fg(Color::Green)),
            Span::styled("Go to top/bottom", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+d/Ctrl+u   ", Style::default().fg(Color::Green)),
            Span::styled("Page down/up", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "File Browser",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  Enter           ", Style::default().fg(Color::Green)),
            Span::styled("Open file / Toggle folder", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Space           ", Style::default().fg(Color::Green)),
            Span::styled("Expand/collapse folder", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  R               ", Style::default().fg(Color::Green)),
            Span::styled("Refresh file tree", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  /               ", Style::default().fg(Color::Green)),
            Span::styled("Search files", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Actions",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  r               ", Style::default().fg(Color::Green)),
            Span::styled("Run current request", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  e               ", Style::default().fg(Color::Green)),
            Span::styled("Enter edit mode", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  v               ", Style::default().fg(Color::Green)),
            Span::styled("Toggle variables panel", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  E               ", Style::default().fg(Color::Green)),
            Span::styled("Cycle environment", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  y               ", Style::default().fg(Color::Green)),
            Span::styled("Copy file path to clipboard", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Y               ", Style::default().fg(Color::Green)),
            Span::styled("Copy response to clipboard", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  c               ", Style::default().fg(Color::Green)),
            Span::styled("Copy AI context (request+response)", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Commands (press : first)",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  :w              ", Style::default().fg(Color::Green)),
            Span::styled("Save file", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  :q              ", Style::default().fg(Color::Green)),
            Span::styled("Quit", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  :wq             ", Style::default().fg(Color::Green)),
            Span::styled("Save and quit", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "General",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  ?               ", Style::default().fg(Color::Green)),
            Span::styled("Toggle this help", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  q               ", Style::default().fg(Color::Green)),
            Span::styled("Quit / Close help", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Esc             ", Style::default().fg(Color::Green)),
            Span::styled("Cancel / Exit mode", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+c          ", Style::default().fg(Color::Green)),
            Span::styled("Force quit", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'q' or '?' to close this help",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
