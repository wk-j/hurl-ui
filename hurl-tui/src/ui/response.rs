//! Response panel
//!
//! Displays HTTP response details including status, headers, and body.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

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
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(" Response ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    // Check if we have a response
    let Some(result) = &app.execution_result else {
        let placeholder = Paragraph::new("No response yet. Press 'r' to run the request.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(placeholder, area);
        return;
    };

    let Some(response) = &result.response else {
        let error_msg = if result.success {
            "Response data not available"
        } else {
            "Request failed. Check stderr for details."
        };
        let error = Paragraph::new(error_msg)
            .style(Style::default().fg(Color::Red))
            .block(block);
        frame.render_widget(error, area);
        return;
    };

    // Build response content
    let mut lines: Vec<Line> = Vec::new();

    // Status line
    let status_color = match response.status_code {
        200..=299 => Color::Green,
        300..=399 => Color::Yellow,
        400..=499 => Color::Red,
        500..=599 => Color::Magenta,
        _ => Color::White,
    };

    lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", response.status_code),
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("Time: {}ms", response.duration_ms),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    lines.push(Line::from(""));

    // Headers section (collapsed)
    if !response.headers.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("Headers ({})", response.headers.len()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));

        // Show first few headers
        for (name, value) in response.headers.iter().take(5) {
            lines.push(Line::from(vec![
                Span::styled(format!("  {}: ", name), Style::default().fg(Color::Blue)),
                Span::styled(value.clone(), Style::default().fg(Color::White)),
            ]));
        }

        if response.headers.len() > 5 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", response.headers.len() - 5),
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines.push(Line::from(""));
    }

    // Body section
    lines.push(Line::from(Span::styled(
        "Body:",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));

    // Try to pretty-print JSON
    let body_lines = format_body(&response.body);
    let scroll = app.response_scroll;
    let visible_height = area.height.saturating_sub(lines.len() as u16 + 3) as usize;

    for line in body_lines.iter().skip(scroll).take(visible_height) {
        lines.push(Line::from(Span::styled(
            format!("  {}", line),
            Style::default().fg(Color::White),
        )));
    }

    // Show scroll indicator if needed
    if body_lines.len() > visible_height {
        let total = body_lines.len();
        let visible_end = (scroll + visible_height).min(total);
        lines.push(Line::from(Span::styled(
            format!("  [{}-{}/{}]", scroll + 1, visible_end, total),
            Style::default().fg(Color::DarkGray),
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
