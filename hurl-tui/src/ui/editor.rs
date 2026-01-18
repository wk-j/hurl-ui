//! Editor panel
//!
//! Displays and allows editing of Hurl file content with hacker aesthetic.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{ActivePanel, App, AppMode};
use super::theme::{HackerTheme, BoxChars};

/// Render the editor panel
pub fn render_editor(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == ActivePanel::Editor;
    let is_editing = app.mode == AppMode::Editing;

    let title = match (&app.current_file_path, is_editing) {
        (Some(path), true) => format!(
            " {} {} [EDITING] ",
            BoxChars::TERMINAL_PROMPT,
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string())
        ),
        (Some(path), false) => format!(
            " {} {} ",
            BoxChars::LAMBDA,
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string())
        ),
        (None, _) => format!(" {} Editor ", BoxChars::LAMBDA),
    };

    let border_color = if is_editing {
        HackerTheme::MODE_EDIT_FG
    } else if is_active {
        HackerTheme::MATRIX_GREEN
    } else {
        HackerTheme::BORDER_DIM
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(HackerTheme::VOID_BLACK));

    if app.editor_content.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {} No file loaded", BoxChars::DOT),
                Style::default().fg(HackerTheme::TEXT_MUTED)
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {} Select a .hurl file to begin", BoxChars::ARROW_RIGHT),
                Style::default().fg(HackerTheme::TEXT_SECONDARY)
            )),
            Line::from(Span::styled(
                format!("  {} Press [Enter] to open", BoxChars::ARROW_RIGHT),
                Style::default().fg(HackerTheme::TEXT_SECONDARY)
            )),
        ])
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
            let line_number = format!("{:4} {} ", line_num + 1, BoxChars::VERTICAL);
            let styled_content = highlight_hurl_line(content);

            let mut spans = vec![Span::styled(
                line_number,
                Style::default().fg(HackerTheme::TEXT_MUTED),
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
                        .fg(HackerTheme::CURSOR_FG)
                        .bg(HackerTheme::CURSOR_BG)
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
        return vec![Span::styled(text, Style::default().fg(HackerTheme::TEXT_COMMENT))];
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
                        .fg(HackerTheme::SYNTAX_METHOD)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(rest.to_string(), Style::default().fg(HackerTheme::SYNTAX_URL)),
            ];
        }
    }

    // HTTP status line
    if trimmed.starts_with("HTTP") {
        return vec![Span::styled(
            text,
            Style::default()
                .fg(HackerTheme::SYNTAX_STATUS)
                .add_modifier(Modifier::BOLD),
        )];
    }

    // Section markers
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return vec![Span::styled(
            text,
            Style::default()
                .fg(HackerTheme::SYNTAX_SECTION)
                .add_modifier(Modifier::BOLD),
        )];
    }

    // Headers (Name: Value)
    if trimmed.contains(':') && !trimmed.starts_with('{') {
        if let Some(colon_pos) = trimmed.find(':') {
            let (name, rest) = trimmed.split_at(colon_pos);
            return vec![
                Span::styled(name.to_string(), Style::default().fg(HackerTheme::SYNTAX_HEADER)),
                Span::styled(rest.to_string(), Style::default().fg(HackerTheme::SYNTAX_VALUE)),
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
                Style::default().fg(HackerTheme::SYNTAX_KEYWORD),
            )];
        }
    }

    // Variables {{var}}
    if trimmed.contains("{{") && trimmed.contains("}}") {
        return vec![Span::styled(
            text,
            Style::default().fg(HackerTheme::SYNTAX_VARIABLE),
        )];
    }

    // JSON content
    if trimmed.starts_with('{') || trimmed.starts_with('[') || trimmed.starts_with('"') {
        return vec![Span::styled(text, Style::default().fg(HackerTheme::SYNTAX_DATA))];
    }

    // Default
    vec![Span::styled(text, Style::default().fg(HackerTheme::TEXT_PRIMARY))]
}
