//! Assertions panel
//!
//! Displays assertion results with hacker terminal aesthetic.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::theme::{BoxChars, HackerTheme};
use crate::app::{ActivePanel, App};

/// Render the assertions panel
pub fn render_assertions(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == ActivePanel::Assertions;

    let border_color = if is_active {
        HackerTheme::MATRIX_GREEN
    } else {
        HackerTheme::BORDER_DIM
    };

    let block = Block::default()
        .title(format!(" {} Assertions ", BoxChars::CHECK))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(HackerTheme::VOID_BLACK));

    // Check if we have execution results
    let Some(result) = &app.execution_result else {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {} No test results", BoxChars::DOT),
                Style::default().fg(HackerTheme::TEXT_MUTED),
            )),
        ])
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

        // Summary line with hacker styling
        let (summary_color, summary_icon) = if passed == total {
            (HackerTheme::ASSERT_PASS, BoxChars::CHECK)
        } else if passed > 0 {
            (HackerTheme::AMBER_WARNING, BoxChars::DIAMOND)
        } else {
            (HackerTheme::ASSERT_FAIL, BoxChars::CROSS)
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} ", summary_icon),
                Style::default().fg(summary_color),
            ),
            Span::styled(
                format!("{}/{} PASSED", passed, total),
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
                (BoxChars::CHECK, HackerTheme::ASSERT_PASS)
            } else {
                (BoxChars::CROSS, HackerTheme::ASSERT_FAIL)
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", icon),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    assertion.text.clone(),
                    Style::default().fg(if assertion.success {
                        HackerTheme::TEXT_PRIMARY
                    } else {
                        HackerTheme::ASSERT_FAIL
                    }),
                ),
            ]));

            // Show expected/actual on failure
            if !assertion.success {
                if let Some(expected) = &assertion.expected {
                    lines.push(Line::from(Span::styled(
                        format!("      {} expected: {}", BoxChars::DOT, expected),
                        Style::default().fg(HackerTheme::TEXT_MUTED),
                    )));
                }
                if let Some(actual) = &assertion.actual {
                    lines.push(Line::from(Span::styled(
                        format!("      {} actual:   {}", BoxChars::DOT, actual),
                        Style::default().fg(HackerTheme::TEXT_MUTED),
                    )));
                }
                if let Some(message) = &assertion.message {
                    lines.push(Line::from(Span::styled(
                        format!("      {} {}", BoxChars::CROSS, message),
                        Style::default().fg(HackerTheme::NEON_RED),
                    )));
                }
            }
        }
    } else if let Some(asserts) = &file_assertions {
        // Show assertions from file if no execution results
        if asserts.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("  {} No assertions defined", BoxChars::DOT),
                Style::default().fg(HackerTheme::TEXT_MUTED),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("  {} {} tests pending", BoxChars::BULLET, asserts.len()),
                Style::default().fg(HackerTheme::ASSERT_PENDING),
            )));
            lines.push(Line::from(""));

            for assert in asserts.iter().take(10) {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {} ", BoxChars::DOT),
                        Style::default().fg(HackerTheme::TEXT_MUTED),
                    ),
                    Span::styled(
                        assert.text.clone(),
                        Style::default().fg(HackerTheme::TEXT_SECONDARY),
                    ),
                ]));
            }

            if asserts.len() > 10 {
                lines.push(Line::from(Span::styled(
                    format!("    {} +{} more...", BoxChars::DOT, asserts.len() - 10),
                    Style::default().fg(HackerTheme::TEXT_MUTED),
                )));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  {} Press [r] to execute", BoxChars::TERMINAL_PROMPT),
                Style::default().fg(HackerTheme::MATRIX_GREEN_DIM),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            format!("  {} Load a .hurl file", BoxChars::DOT),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        )));
    }

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}
