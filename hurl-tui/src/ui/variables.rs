//! Variables panel
//!
//! Displays environment variables with hacker terminal aesthetic.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::theme::{BoxChars, HackerTheme};
use crate::app::{ActivePanel, App};

/// Render the variables panel
pub fn render_variables(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == ActivePanel::Variables;

    let border_color = if is_active {
        HackerTheme::MATRIX_GREEN
    } else {
        HackerTheme::BORDER_DIM
    };

    let block = Block::default()
        .title(format!(" {} Variables ", BoxChars::LAMBDA))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(HackerTheme::VOID_BLACK));

    let mut lines: Vec<Line> = Vec::new();

    // Environment selector
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ENV ", BoxChars::DIAMOND),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        ),
        Span::styled(
            app.current_environment.clone(),
            Style::default()
                .fg(HackerTheme::CYBER_CYAN)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(""));

    // Variables list
    if app.variables.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("  {} No variables loaded", BoxChars::DOT),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        )));
    } else {
        for var in &app.variables {
            let value_display = if var.is_secret {
                mask_secret(&var.value)
            } else {
                truncate_value(&var.value, 15)
            };

            let value_color = if var.is_secret {
                HackerTheme::NEON_RED
            } else {
                HackerTheme::TEXT_PRIMARY
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!(
                        "  {} ",
                        if var.is_secret {
                            BoxChars::BLOCK_FULL
                        } else {
                            BoxChars::DOT
                        }
                    ),
                    Style::default().fg(if var.is_secret {
                        HackerTheme::NEON_RED
                    } else {
                        HackerTheme::TEXT_MUTED
                    }),
                ),
                Span::styled(
                    format!("{}: ", var.name),
                    Style::default().fg(HackerTheme::SYNTAX_VARIABLE),
                ),
                Span::styled(value_display, Style::default().fg(value_color)),
            ]));
        }
    }

    // Hint
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  {} [E] cycle env", BoxChars::TERMINAL_PROMPT),
        Style::default().fg(HackerTheme::TEXT_MUTED),
    )));

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

/// Mask a secret value
fn mask_secret(value: &str) -> String {
    if value.len() <= 4 {
        format!(
            "{}{}{}",
            BoxChars::BLOCK_MEDIUM,
            BoxChars::BLOCK_MEDIUM,
            BoxChars::BLOCK_MEDIUM
        )
    } else {
        format!(
            "{}{}{}...",
            &value[..1],
            BoxChars::BLOCK_MEDIUM,
            BoxChars::BLOCK_MEDIUM
        )
    }
}

/// Truncate a value to a maximum length
fn truncate_value(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        value.to_string()
    } else {
        format!("{}...", &value[..max_len])
    }
}
