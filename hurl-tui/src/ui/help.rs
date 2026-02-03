//! Help overlay
//!
//! Displays a help popup with keyboard shortcuts in clean modern style.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::layout::centered_rect;
use super::theme::HackerTheme;
use crate::app::App;

/// Render the help overlay
pub fn render_help(frame: &mut Frame, _app: &App) {
    let area = centered_rect(55, 70, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help ")
        .title_style(
            Style::default()
                .fg(HackerTheme::MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(HackerTheme::BORDER_DIM))
        .style(Style::default().bg(HackerTheme::VOID_BLACK));

    let help_text = vec![
        Line::from(""),
        section_header("Navigation"),
        help_line("j/k", "Move down/up"),
        help_line("h/l", "Switch panels"),
        help_line("Tab", "Cycle panels"),
        help_line("g/G", "Go to start/end"),
        help_line("Ctrl+d/u", "Page down/up"),
        Line::from(""),
        section_header("Files"),
        help_line("Enter", "Open file"),
        help_line("Space", "Expand/collapse"),
        help_line("R", "Refresh"),
        help_line("f/F", "Filter / Clear"),
        help_line("p/P", "Copy / Paste file"),
        help_line("n", "Rename"),
        help_line("[/]", "Resize sidebar"),
        help_line("A", "Toggle assertions"),
        Line::from(""),
        section_header("Actions"),
        help_line("r", "Run request"),
        help_line("W", "Run & write output"),
        help_line("e", "Edit mode"),
        help_line("v", "Variables"),
        help_line("E", "Cycle environment"),
        help_line("1/2", "Editor tabs"),
        help_line("1/2/3", "Response tabs"),
        Line::from(""),
        section_header("Clipboard"),
        help_line("y", "Copy path"),
        help_line("Y", "Copy response"),
        help_line("c", "Copy AI context"),
        help_line("C", "Copy hurl command"),
        Line::from(""),
        section_header("Commands"),
        help_line(":w", "Save"),
        help_line(":q", "Quit"),
        help_line(":wq", "Save & quit"),
        Line::from(""),
        Line::from(Span::styled(
            "  Press q or ? to close",
            Style::default().fg(HackerTheme::TEXT_MUTED),
        )),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Section header
fn section_header(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {}", title),
        Style::default()
            .fg(HackerTheme::SYNTAX_SECTION)
            .add_modifier(Modifier::BOLD),
    ))
}

/// Create a help line with key and description
fn help_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("    ", Style::default()),
        Span::styled(
            format!("{:12}", key),
            Style::default().fg(HackerTheme::MATRIX_GREEN),
        ),
        Span::styled(
            desc.to_string(),
            Style::default().fg(HackerTheme::TEXT_SECONDARY),
        ),
    ])
}
