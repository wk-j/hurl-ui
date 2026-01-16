//! Assertions panel
//!
//! Displays assertion results from Hurl execution.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{ActivePanel, App};

/// Render the assertions panel
pub fn render_assertions(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == ActivePanel::Assertions;

    let border_color = if is_active {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(" Assertions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    // Check if we have execution results
    let Some(result) = &app.execution_result else {
        let placeholder = Paragraph::new("No assertions to display.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(placeholder, area);
        return;
    };

    // Check if we have any assertions from the parsed file
    let file_assertions = app.current_file.as_ref().map(|f| {
        f.entries
            .iter()
            .flat_map(|e| e.asserts.iter())
            .collect::<Vec<_>>()
    });

    let mut lines: Vec<Line> = Vec::new();

    // Show assertion results from execution
    if !result.assertions.is_empty() {
        let passed = result.assertions.iter().filter(|a| a.success).count();
        let total = result.assertions.len();

        // Summary line
        let summary_color = if passed == total {
            Color::Green
        } else if passed > 0 {
            Color::Yellow
        } else {
            Color::Red
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{}/{} passed", passed, total),
                Style::default()
                    .fg(summary_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Individual assertions
        let scroll = app.assertions_scroll;
        let visible_height = area.height.saturating_sub(5) as usize;

        for assertion in result.assertions.iter().skip(scroll).take(visible_height) {
            let (icon, color) = if assertion.success {
                ("v", Color::Green)
            } else {
                ("x", Color::Red)
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} ", icon),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    assertion.text.clone(),
                    Style::default().fg(if assertion.success {
                        Color::White
                    } else {
                        Color::Red
                    }),
                ),
            ]));

            // Show expected/actual on failure
            if !assertion.success {
                if let Some(expected) = &assertion.expected {
                    lines.push(Line::from(Span::styled(
                        format!("     expected: {}", expected),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                if let Some(actual) = &assertion.actual {
                    lines.push(Line::from(Span::styled(
                        format!("     actual:   {}", actual),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                if let Some(message) = &assertion.message {
                    lines.push(Line::from(Span::styled(
                        format!("     {}", message),
                        Style::default().fg(Color::Red),
                    )));
                }
            }
        }
    } else if let Some(asserts) = &file_assertions {
        // Show assertions from file if no execution results
        if asserts.is_empty() {
            lines.push(Line::from(Span::styled(
                "No assertions defined",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("{} assertions defined", asserts.len()),
                Style::default().fg(Color::Yellow),
            )));
            lines.push(Line::from(""));

            for assert in asserts.iter().take(10) {
                lines.push(Line::from(vec![
                    Span::styled(" - ", Style::default().fg(Color::DarkGray)),
                    Span::styled(assert.text.clone(), Style::default().fg(Color::White)),
                ]));
            }

            if asserts.len() > 10 {
                lines.push(Line::from(Span::styled(
                    format!("   ... and {} more", asserts.len() - 10),
                    Style::default().fg(Color::DarkGray),
                )));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Press 'r' to run",
                Style::default().fg(Color::Cyan),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "Open a .hurl file to see assertions",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}
