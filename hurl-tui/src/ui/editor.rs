//! Editor panel
//!
//! Displays and allows editing of Hurl file content.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{ActivePanel, App, AppMode};

/// Render the editor panel
pub fn render_editor(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == ActivePanel::Editor;
    let is_editing = app.mode == AppMode::Editing;

    let title = match (&app.current_file_path, is_editing) {
        (Some(path), true) => format!(
            " {} [EDITING] ",
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string())
        ),
        (Some(path), false) => format!(
            " {} ",
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string())
        ),
        (None, _) => " Editor ".to_string(),
    };

    let border_color = if is_editing {
        Color::Yellow
    } else if is_active {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    if app.editor_content.is_empty() {
        let placeholder = Paragraph::new("No file selected. Press Enter on a .hurl file to open it.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(placeholder, area);
        return;
    }

    // Calculate visible area
    let inner_height = area.height.saturating_sub(2) as usize;
    let scroll = app.editor_scroll;

    // Build styled lines with line numbers
    let lines: Vec<Line> = app
        .editor_content
        .iter()
        .enumerate()
        .skip(scroll)
        .take(inner_height)
        .map(|(line_num, content)| {
            let line_number = format!("{:4} ", line_num + 1);
            let styled_content = highlight_hurl_line(content);

            let mut spans = vec![Span::styled(
                line_number,
                Style::default().fg(Color::DarkGray),
            )];

            // If editing and cursor is on this line, show cursor
            if is_editing && line_num == app.editor_cursor.0 {
                let col = app.editor_cursor.1.min(content.len());
                let before = &content[..col];
                let cursor_char = content.chars().nth(col).unwrap_or(' ');
                let after = if col < content.len() {
                    &content[col + 1..]
                } else {
                    ""
                };

                spans.extend(highlight_hurl_spans(before));
                spans.push(Span::styled(
                    cursor_char.to_string(),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
                spans.extend(highlight_hurl_spans(after));
            } else {
                spans.extend(styled_content);
            }

            Line::from(spans)
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Highlight a Hurl line and return styled spans
fn highlight_hurl_line(line: &str) -> Vec<Span<'static>> {
    highlight_hurl_spans(line)
}

/// Convert a string to highlighted spans for Hurl syntax
fn highlight_hurl_spans(text: &str) -> Vec<Span<'static>> {
    let text = text.to_string();
    let trimmed = text.trim();

    // Comments
    if trimmed.starts_with('#') {
        return vec![Span::styled(text, Style::default().fg(Color::DarkGray))];
    }

    // HTTP methods
    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
    for method in methods {
        if trimmed.starts_with(method) {
            let method_end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
            let (method_part, rest) = trimmed.split_at(method_end);
            return vec![
                Span::styled(
                    method_part.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(rest.to_string(), Style::default().fg(Color::Cyan)),
            ];
        }
    }

    // HTTP status line
    if trimmed.starts_with("HTTP") {
        return vec![Span::styled(
            text,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )];
    }

    // Section markers
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return vec![Span::styled(
            text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )];
    }

    // Headers (Name: Value)
    if trimmed.contains(':') && !trimmed.starts_with('{') {
        if let Some(colon_pos) = trimmed.find(':') {
            let (name, rest) = trimmed.split_at(colon_pos);
            return vec![
                Span::styled(name.to_string(), Style::default().fg(Color::Blue)),
                Span::styled(rest.to_string(), Style::default().fg(Color::White)),
            ];
        }
    }

    // Assertions with predicates
    let assertion_keywords = [
        "jsonpath", "xpath", "header", "status", "body", "bytes", "sha256", "md5", "duration",
        "certificate",
    ];
    for keyword in assertion_keywords {
        if trimmed.starts_with(keyword) {
            return vec![Span::styled(
                text,
                Style::default().fg(Color::Cyan),
            )];
        }
    }

    // Variables {{var}}
    if trimmed.contains("{{") && trimmed.contains("}}") {
        return vec![Span::styled(
            text,
            Style::default().fg(Color::Yellow),
        )];
    }

    // JSON content
    if trimmed.starts_with('{') || trimmed.starts_with('[') || trimmed.starts_with('"') {
        return vec![Span::styled(text, Style::default().fg(Color::White))];
    }

    // Default
    vec![Span::styled(text, Style::default().fg(Color::White))]
}
