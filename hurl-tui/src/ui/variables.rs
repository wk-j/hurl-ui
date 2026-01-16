//! Variables panel
//!
//! Displays environment variables and allows switching environments.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{ActivePanel, App};

/// Render the variables panel
pub fn render_variables(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == ActivePanel::Variables;

    let border_color = if is_active {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(" Variables ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let mut lines: Vec<Line> = Vec::new();

    // Environment selector
    lines.push(Line::from(vec![
        Span::styled("env: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            app.current_environment.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(""));

    // Variables list
    if app.variables.is_empty() {
        lines.push(Line::from(Span::styled(
            "No variables",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for var in &app.variables {
            let value_display = if var.is_secret {
                mask_secret(&var.value)
            } else {
                truncate_value(&var.value, 15)
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", var.name),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    value_display,
                    Style::default().fg(if var.is_secret {
                        Color::DarkGray
                    } else {
                        Color::White
                    }),
                ),
            ]));
        }
    }

    // Hint
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[E] Switch env",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

/// Mask a secret value
fn mask_secret(value: &str) -> String {
    if value.len() <= 4 {
        "*".repeat(value.len())
    } else {
        format!("{}...", &value[..3])
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
