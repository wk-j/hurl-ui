//! Response panel
//!
//! Displays HTTP response details with cyberpunk hacker aesthetic.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

use super::theme::{BoxChars, HackerTheme};
use crate::app::{ActivePanel, App};

/// Response view tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResponseTab {
    #[default]
    Body,
    Headers,
    Raw,
}

impl ResponseTab {
    pub fn index(&self) -> usize {
        match self {
            ResponseTab::Body => 0,
            ResponseTab::Headers => 1,
            ResponseTab::Raw => 2,
        }
    }
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

    // Render the outer block first
    frame.render_widget(block, area);

    // Calculate inner area (inside the border)
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Split inner area into: status bar, tabs, content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Status line
            Constraint::Length(1), // Tabs
            Constraint::Min(1),    // Content
        ])
        .split(inner_area);

    // Render status line
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

    let status_line = Line::from(vec![
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
        Span::styled("    ", Style::default()),
        Span::styled(
            format!("{}", BoxChars::DOT),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        ),
        Span::styled(
            format!(" {}ms", response.duration_ms),
            Style::default().fg(HackerTheme::CYBER_CYAN),
        ),
    ]);

    let status_paragraph = Paragraph::new(status_line);
    frame.render_widget(status_paragraph, chunks[0]);

    // Render tabs
    let tab_titles = vec![
        format!("[1] Body"),
        format!("[2] Headers ({})", response.headers.len()),
        format!("[3] Raw"),
    ];
    let tabs = Tabs::new(tab_titles)
        .select(app.response_tab.index())
        .style(Style::default().fg(HackerTheme::TEXT_MUTED))
        .highlight_style(
            Style::default()
                .fg(HackerTheme::MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled(" | ", Style::default().fg(HackerTheme::TEXT_MUTED)));

    frame.render_widget(tabs, chunks[1]);

    // Render content based on selected tab
    let content_area = chunks[2];
    let visible_height = content_area.height as usize;
    let scroll = app.response_scroll;

    match app.response_tab {
        ResponseTab::Body => {
            render_body_tab(frame, response, content_area, scroll, visible_height);
        }
        ResponseTab::Headers => {
            render_headers_tab(frame, response, content_area, scroll, visible_height);
        }
        ResponseTab::Raw => {
            render_raw_tab(frame, result, content_area, scroll, visible_height);
        }
    }
}

/// Render the Body tab content
fn render_body_tab(
    frame: &mut Frame,
    response: &crate::runner::Response,
    area: Rect,
    scroll: usize,
    visible_height: usize,
) {
    let mut lines: Vec<Line> = Vec::new();

    if response.body.trim().is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {} No response body", BoxChars::DOT),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        )));
    } else {
        // Try to pretty-print and syntax highlight JSON
        let body_lines = format_body_with_highlighting(&response.body);

        for line in body_lines.iter().skip(scroll).take(visible_height.saturating_sub(1)) {
            lines.push(line.clone());
        }

        // Show scroll indicator if needed
        if body_lines.len() > visible_height {
            let total = body_lines.len();
            let visible_end = (scroll + visible_height).min(total);
            lines.push(Line::from(Span::styled(
                format!(
                    " {} [{}-{}/{}]",
                    BoxChars::GLITCH_1,
                    scroll + 1,
                    visible_end,
                    total
                ),
                Style::default().fg(HackerTheme::TEXT_MUTED),
            )));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

/// Render the Headers tab content
fn render_headers_tab(
    frame: &mut Frame,
    response: &crate::runner::Response,
    area: Rect,
    scroll: usize,
    visible_height: usize,
) {
    let mut lines: Vec<Line> = Vec::new();

    if response.headers.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {} No headers", BoxChars::DOT),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        )));
    } else {
        // Build all header lines first
        let mut header_lines: Vec<Line> = Vec::new();
        for (name, value) in response.headers.iter() {
            header_lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} ", BoxChars::DOT),
                    Style::default().fg(HackerTheme::TEXT_MUTED),
                ),
                Span::styled(
                    format!("{}: ", name),
                    Style::default()
                        .fg(HackerTheme::SYNTAX_HEADER)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(value.clone(), Style::default().fg(HackerTheme::SYNTAX_VALUE)),
            ]));
        }

        // Apply scroll and visible height
        for line in header_lines.iter().skip(scroll).take(visible_height.saturating_sub(1)) {
            lines.push(line.clone());
        }

        // Show scroll indicator if needed
        if header_lines.len() > visible_height {
            let total = header_lines.len();
            let visible_end = (scroll + visible_height).min(total);
            lines.push(Line::from(Span::styled(
                format!(
                    " {} [{}-{}/{}]",
                    BoxChars::GLITCH_1,
                    scroll + 1,
                    visible_end,
                    total
                ),
                Style::default().fg(HackerTheme::TEXT_MUTED),
            )));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

/// Render the Raw tab content (full verbose output)
fn render_raw_tab(
    frame: &mut Frame,
    result: &crate::runner::ExecutionResult,
    area: Rect,
    scroll: usize,
    visible_height: usize,
) {
    let mut lines: Vec<Line> = Vec::new();

    // Show the raw stderr (verbose output from hurl)
    if result.stderr.is_empty() && result.stdout.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {} No raw output available", BoxChars::DOT),
            Style::default().fg(HackerTheme::TEXT_MUTED),
        )));
    } else {
        // Build all raw lines
        let mut raw_lines: Vec<Line> = Vec::new();

        // Add stderr (verbose output)
        if !result.stderr.is_empty() {
            for line in result.stderr.lines() {
                let color = if line.starts_with('*') {
                    HackerTheme::CYBER_CYAN
                } else if line.starts_with('>') {
                    HackerTheme::SYNTAX_HEADER
                } else if line.starts_with('<') {
                    HackerTheme::SYNTAX_VALUE
                } else if line.starts_with("error:") {
                    HackerTheme::NEON_RED
                } else {
                    HackerTheme::TEXT_SECONDARY
                };
                raw_lines.push(Line::from(Span::styled(
                    format!(" {}", line),
                    Style::default().fg(color),
                )));
            }
        }

        // Add stdout if present
        if !result.stdout.is_empty() {
            if !result.stderr.is_empty() {
                raw_lines.push(Line::from(""));
                raw_lines.push(Line::from(Span::styled(
                    format!(" {} STDOUT", BoxChars::TRIANGLE_DOWN),
                    Style::default()
                        .fg(HackerTheme::SYNTAX_SECTION)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            for line in result.stdout.lines() {
                raw_lines.push(Line::from(Span::styled(
                    format!(" {}", line),
                    Style::default().fg(HackerTheme::TEXT_PRIMARY),
                )));
            }
        }

        // Apply scroll and visible height
        for line in raw_lines.iter().skip(scroll).take(visible_height.saturating_sub(1)) {
            lines.push(line.clone());
        }

        // Show scroll indicator if needed
        if raw_lines.len() > visible_height {
            let total = raw_lines.len();
            let visible_end = (scroll + visible_height).min(total);
            lines.push(Line::from(Span::styled(
                format!(
                    " {} [{}-{}/{}]",
                    BoxChars::GLITCH_1,
                    scroll + 1,
                    visible_end,
                    total
                ),
                Style::default().fg(HackerTheme::TEXT_MUTED),
            )));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

/// Format the response body with syntax highlighting for JSON
fn format_body_with_highlighting(body: &str) -> Vec<Line<'static>> {
    let trimmed = body.trim();
    
    // Try to parse as JSON and pretty-print with highlighting
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Ok(pretty) = serde_json::to_string_pretty(&json) {
            return pretty
                .lines()
                .map(|line| highlight_json_line(line))
                .collect();
        }
    }

    // Try to find and format JSON within the body (might be prefixed with other content)
    // Look for first { or [ that starts a JSON structure
    if let Some(json_start) = trimmed.find(|c| c == '{' || c == '[') {
        let potential_json = &trimmed[json_start..];
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(potential_json) {
            if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                let mut lines: Vec<Line<'static>> = Vec::new();
                
                // Add any prefix content before JSON
                if json_start > 0 {
                    for line in trimmed[..json_start].lines() {
                        lines.push(Line::from(Span::styled(
                            format!(" {}", line),
                            Style::default().fg(HackerTheme::TEXT_SECONDARY),
                        )));
                    }
                }
                
                // Add formatted JSON
                for line in pretty.lines() {
                    lines.push(highlight_json_line(line));
                }
                
                return lines;
            }
        }
    }

    // Try to handle multiple JSON objects (one per line - NDJSON format)
    let mut all_lines: Vec<Line<'static>> = Vec::new();
    let mut found_any_json = false;
    
    for line in body.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            all_lines.push(Line::from(""));
            continue;
        }
        
        // Try to parse each line as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line_trimmed) {
            found_any_json = true;
            if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                for pretty_line in pretty.lines() {
                    all_lines.push(highlight_json_line(pretty_line));
                }
                // Add separator between JSON objects
                all_lines.push(Line::from(""));
            }
        } else {
            // Not JSON, show as plain text
            all_lines.push(Line::from(Span::styled(
                format!(" {}", line),
                Style::default().fg(HackerTheme::TEXT_PRIMARY),
            )));
        }
    }
    
    if found_any_json {
        return all_lines;
    }

    // Fall back to raw body (plain text)
    body.lines()
        .map(|line| {
            Line::from(Span::styled(
                format!(" {}", line),
                Style::default().fg(HackerTheme::TEXT_PRIMARY),
            ))
        })
        .collect()
}

/// Highlight a single line of JSON
fn highlight_json_line(line: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled(" ", Style::default())); // Leading space

    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();

    // Add indentation
    if indent > 0 {
        spans.push(Span::styled(
            " ".repeat(indent),
            Style::default(),
        ));
    }

    // Simple JSON syntax highlighting
    let mut chars = trimmed.chars().peekable();
    let mut current = String::new();
    let mut in_string = false;
    let mut is_key = true; // Track if we're parsing a key or value

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_string {
                    // End of string
                    current.push(ch);
                    let color = if is_key {
                        HackerTheme::SYNTAX_HEADER // Keys in cyan-ish
                    } else {
                        HackerTheme::SYNTAX_VALUE // String values in green
                    };
                    spans.push(Span::styled(current.clone(), Style::default().fg(color)));
                    current.clear();
                    in_string = false;
                } else {
                    // Start of string
                    if !current.is_empty() {
                        spans.push(Span::styled(
                            current.clone(),
                            Style::default().fg(HackerTheme::TEXT_PRIMARY),
                        ));
                        current.clear();
                    }
                    current.push(ch);
                    in_string = true;
                }
            }
            ':' if !in_string => {
                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(HackerTheme::TEXT_PRIMARY),
                    ));
                    current.clear();
                }
                spans.push(Span::styled(
                    ":".to_string(),
                    Style::default().fg(HackerTheme::TEXT_MUTED),
                ));
                is_key = false; // Next string will be a value
            }
            ',' if !in_string => {
                if !current.is_empty() {
                    // This could be a number or boolean
                    let color = if current.trim().parse::<f64>().is_ok() {
                        HackerTheme::CYBER_CYAN // Numbers
                    } else if current.trim() == "true" || current.trim() == "false" {
                        HackerTheme::ELECTRIC_PURPLE // Booleans
                    } else if current.trim() == "null" {
                        HackerTheme::TEXT_MUTED // Null
                    } else {
                        HackerTheme::TEXT_PRIMARY
                    };
                    spans.push(Span::styled(current.clone(), Style::default().fg(color)));
                    current.clear();
                }
                spans.push(Span::styled(
                    ",".to_string(),
                    Style::default().fg(HackerTheme::TEXT_MUTED),
                ));
                is_key = true; // Next string will be a key
            }
            '{' | '}' | '[' | ']' if !in_string => {
                if !current.is_empty() {
                    let color = if current.trim().parse::<f64>().is_ok() {
                        HackerTheme::CYBER_CYAN
                    } else if current.trim() == "true" || current.trim() == "false" {
                        HackerTheme::ELECTRIC_PURPLE
                    } else if current.trim() == "null" {
                        HackerTheme::TEXT_MUTED
                    } else {
                        HackerTheme::TEXT_PRIMARY
                    };
                    spans.push(Span::styled(current.clone(), Style::default().fg(color)));
                    current.clear();
                }
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default()
                        .fg(HackerTheme::MATRIX_GREEN)
                        .add_modifier(Modifier::BOLD),
                ));
                if ch == '{' || ch == '[' {
                    is_key = ch == '{'; // After {, expect key; after [, expect value
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Handle remaining content
    if !current.is_empty() {
        let color = if current.trim().parse::<f64>().is_ok() {
            HackerTheme::CYBER_CYAN
        } else if current.trim() == "true" || current.trim() == "false" {
            HackerTheme::ELECTRIC_PURPLE
        } else if current.trim() == "null" {
            HackerTheme::TEXT_MUTED
        } else {
            HackerTheme::TEXT_PRIMARY
        };
        spans.push(Span::styled(current, Style::default().fg(color)));
    }

    Line::from(spans)
}

/// Format the response body for display (plain text version)
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
