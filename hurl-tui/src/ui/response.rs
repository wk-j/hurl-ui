//! Response panel
//!
//! Displays HTTP response details with cyberpunk hacker aesthetic.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::theme::{BoxChars, HackerTheme};
use crate::app::{ActivePanel, App};

/// Response view tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseTab {
    Body,
    Headers,
    Raw,
}

/// Render the response panel
pub fn render_response(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == ActivePanel::Response;

    let border_color = if is_active {
        HackerTheme::MATRIX_GREEN
    } else {
        HackerTheme::BORDER_DIM
    };

    let block = Block::default()
        .title(format!(" {} Response ", BoxChars::ARROW_RIGHT))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(HackerTheme::VOID_BLACK));

    // Check if we have a response
    let Some(result) = &app.execution_result else {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {} Awaiting response data...", BoxChars::DOT),
                Style::default().fg(HackerTheme::TEXT_MUTED),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!(
                    "  {} Press [r] to execute request",
                    BoxChars::TERMINAL_PROMPT
                ),
                Style::default().fg(HackerTheme::TEXT_SECONDARY),
            )),
        ])
        .block(block);
        frame.render_widget(placeholder, area);
        return;
    };

    let Some(response) = &result.response else {
        // No response - show error from stderr
        let mut lines: Vec<Line> = Vec::new();

        if result.success {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  {} Response data unavailable", BoxChars::DOT),
                Style::default().fg(HackerTheme::TEXT_MUTED),
            )));
        } else {
            // Show error header
            lines.push(Line::from(Span::styled(
                format!(" {} ERROR", BoxChars::CROSS),
                Style::default()
                    .fg(HackerTheme::NEON_RED)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            // Show stderr content (the actual error message)
            if !result.stderr.is_empty() {
                for line in result.stderr.lines() {
                    let styled_line = if line.starts_with("error:") {
                        Line::from(Span::styled(
                            format!(" {}", line),
                            Style::default()
                                .fg(HackerTheme::NEON_RED)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else if line.contains("-->") {
                        Line::from(Span::styled(
                            format!(" {}", line),
                            Style::default().fg(HackerTheme::TEXT_COMMENT),
                        ))
                    } else if line.trim().starts_with('|') {
                        Line::from(Span::styled(
                            format!(" {}", line),
                            Style::default().fg(HackerTheme::TEXT_PRIMARY),
                        ))
                    } else if line.contains("^^^") {
                        Line::from(Span::styled(
                            format!(" {}", line),
                            Style::default().fg(HackerTheme::NEON_RED),
                        ))
                    } else {
                        Line::from(Span::styled(
                            format!(" {}", line),
                            Style::default().fg(HackerTheme::TEXT_SECONDARY),
                        ))
                    };
                    lines.push(styled_line);
                }
            } else {
                lines.push(Line::from(Span::styled(
                    "  Request failed (no error details)",
                    Style::default().fg(HackerTheme::NEON_RED),
                )));
            }
        }

        let error = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.response_scroll as u16, 0));
        frame.render_widget(error, area);
        return;
    };

    // Build response content
    let mut lines: Vec<Line> = Vec::new();

    // Status line with cyber colors
    let status_color = match response.status_code {
        200..=299 => HackerTheme::STATUS_2XX,
        300..=399 => HackerTheme::STATUS_3XX,
        400..=499 => HackerTheme::STATUS_4XX,
        500..=599 => HackerTheme::STATUS_5XX,
        _ => HackerTheme::TEXT_PRIMARY,
    };

    let status_icon = match response.status_code {
        200..=299 => BoxChars::CHECK,
        300..=399 => BoxChars::ARROW_RIGHT,
        400..=499 => BoxChars::CROSS,
        500..=599 => BoxChars::CROSS,
        _ => BoxChars::DOT,
    };

    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", status_icon),
            Style::default().fg(status_color),
        ),
        Span::styled("STATUS ", Style::default().fg(HackerTheme::TEXT_MUTED)),
        Span::styled(
            format!("{}", response.status_code),
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{} ", BoxChars::DOT),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        ),
        Span::styled(
            format!("{}ms", response.duration_ms),
            Style::default().fg(HackerTheme::CYBER_CYAN),
        ),
    ]));

    lines.push(Line::from(""));

    // Headers section
    if !response.headers.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(
                "{} HEADERS [{}]",
                BoxChars::TRIANGLE_DOWN,
                response.headers.len()
            ),
            Style::default()
                .fg(HackerTheme::SYNTAX_SECTION)
                .add_modifier(Modifier::BOLD),
        )));

        // Show first few headers
        for (name, value) in response.headers.iter().take(5) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", BoxChars::DOT),
                    Style::default().fg(HackerTheme::TEXT_MUTED),
                ),
                Span::styled(
                    format!("{}: ", name),
                    Style::default().fg(HackerTheme::SYNTAX_HEADER),
                ),
                Span::styled(
                    value.clone(),
                    Style::default().fg(HackerTheme::SYNTAX_VALUE),
                ),
            ]));
        }

        if response.headers.len() > 5 {
            lines.push(Line::from(Span::styled(
                format!(
                    "    {} +{} more...",
                    BoxChars::DOT,
                    response.headers.len() - 5
                ),
                Style::default().fg(HackerTheme::TEXT_MUTED),
            )));
        }

        lines.push(Line::from(""));
    }

    // Body section
    lines.push(Line::from(Span::styled(
        format!("{} BODY", BoxChars::TRIANGLE_DOWN),
        Style::default()
            .fg(HackerTheme::SYNTAX_SECTION)
            .add_modifier(Modifier::BOLD),
    )));

    // Try to pretty-print JSON
    let body_lines = format_body(&response.body);
    let scroll = app.response_scroll;
    let visible_height = area.height.saturating_sub(lines.len() as u16 + 3) as usize;

    for line in body_lines.iter().skip(scroll).take(visible_height) {
        lines.push(Line::from(Span::styled(
            format!("  {}", line),
            Style::default().fg(HackerTheme::SYNTAX_DATA),
        )));
    }

    // Show scroll indicator if needed
    if body_lines.len() > visible_height {
        let total = body_lines.len();
        let visible_end = (scroll + visible_height).min(total);
        lines.push(Line::from(Span::styled(
            format!(
                "  {} [{}-{}/{}]",
                BoxChars::GLITCH_1,
                scroll + 1,
                visible_end,
                total
            ),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Format the response body for display
fn format_body(body: &str) -> Vec<String> {
    // Try to parse and pretty-print JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Ok(pretty) = serde_json::to_string_pretty(&json) {
            return pretty.lines().map(String::from).collect();
        }
    }

    // Fall back to raw body
    body.lines().map(String::from).collect()
}
