//! Hurl file parser module
//!
//! This module provides parsing functionality for Hurl files.

use anyhow::Result;
use regex::Regex;

/// Represents a parsed Hurl file
#[derive(Debug, Clone)]
pub struct HurlFile {
    /// List of entries (requests) in the file
    pub entries: Vec<HurlEntry>,
    /// Raw content of the file
    pub content: String,
}

/// Represents a single entry (request/response pair) in a Hurl file
#[derive(Debug, Clone)]
pub struct HurlEntry {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request URL
    pub url: String,
    /// Request headers
    pub headers: Vec<Header>,
    /// Request body (if any)
    pub body: Option<String>,
    /// Expected response status
    pub expected_status: Option<u16>,
    /// Assertions
    pub asserts: Vec<Assert>,
    /// Captures
    pub captures: Vec<Capture>,
    /// Line number where this entry starts
    pub line_start: usize,
    /// Line number where this entry ends
    pub line_end: usize,
}

/// HTTP header
#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}

/// Assertion in Hurl file
#[derive(Debug, Clone)]
pub struct Assert {
    /// Original assertion text
    pub text: String,
    /// Query type (jsonpath, xpath, header, status, etc.)
    pub query_type: String,
    /// Query value
    pub query: String,
    /// Predicate (equals, contains, exists, etc.)
    pub predicate: String,
    /// Expected value (if applicable)
    pub expected: Option<String>,
    /// Line number
    pub line: usize,
}

/// Capture definition
#[derive(Debug, Clone)]
pub struct Capture {
    pub name: String,
    pub query_type: String,
    pub query: String,
    pub line: usize,
}

/// Parse a Hurl file content into structured representation
pub fn parse_hurl_file(content: &str) -> Result<HurlFile> {
    let mut entries = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        // Skip empty lines and comments
        let line = lines[i].trim();
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        // Try to parse a request
        if let Some(entry) = parse_entry(&lines, &mut i) {
            entries.push(entry);
        } else {
            i += 1;
        }
    }

    Ok(HurlFile {
        entries,
        content: content.to_string(),
    })
}

/// Parse a single entry starting at the given index
fn parse_entry(lines: &[&str], index: &mut usize) -> Option<HurlEntry> {
    let line_start = *index;
    let line = lines[*index].trim();

    // Parse method and URL
    let method_regex =
        Regex::new(r"^(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS|CONNECT|TRACE)\s+(.+)$").ok()?;
    let caps = method_regex.captures(line)?;

    let method = caps.get(1)?.as_str().to_string();
    let url = caps.get(2)?.as_str().to_string();

    *index += 1;

    let mut headers = Vec::new();
    let mut body = None;
    let mut expected_status = None;
    let mut asserts = Vec::new();
    let mut captures = Vec::new();
    let mut in_asserts_section = false;
    let mut in_captures_section = false;
    let mut body_lines: Vec<String> = Vec::new();
    let mut in_body = false;

    // Parse headers, body, response, asserts
    while *index < lines.len() {
        let line = lines[*index];
        let trimmed = line.trim();

        // Empty line might separate sections
        if trimmed.is_empty() {
            if in_body {
                in_body = false;
            }
            *index += 1;
            continue;
        }

        // Check if this is a new request (start of next entry)
        if method_regex.is_match(trimmed) {
            break;
        }

        // HTTP response status line
        if trimmed.starts_with("HTTP") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(status) = parts[1].parse::<u16>() {
                    expected_status = Some(status);
                }
            }
            in_asserts_section = false;
            in_captures_section = false;
            *index += 1;
            continue;
        }

        // Section markers
        if trimmed == "[Asserts]" {
            in_asserts_section = true;
            in_captures_section = false;
            in_body = false;
            *index += 1;
            continue;
        }

        if trimmed == "[Captures]" {
            in_captures_section = true;
            in_asserts_section = false;
            in_body = false;
            *index += 1;
            continue;
        }

        if trimmed == "[Options]"
            || trimmed == "[QueryStringParams]"
            || trimmed == "[FormParams]"
            || trimmed == "[MultipartFormData]"
            || trimmed == "[Cookies]"
            || trimmed == "[BasicAuth]"
        {
            in_asserts_section = false;
            in_captures_section = false;
            in_body = false;
            *index += 1;
            continue;
        }

        // Body markers
        if trimmed.starts_with("```") || trimmed.starts_with("{") || trimmed.starts_with("[") {
            in_body = true;
            body_lines.push(trimmed.to_string());
            *index += 1;
            continue;
        }

        if in_body {
            body_lines.push(trimmed.to_string());
            if trimmed.starts_with("```") || trimmed == "}" || trimmed == "]" {
                in_body = false;
                body = Some(body_lines.join("\n"));
                body_lines.clear();
            }
            *index += 1;
            continue;
        }

        // Parse assertions
        if in_asserts_section {
            if let Some(assert) = parse_assert(trimmed, *index) {
                asserts.push(assert);
            }
            *index += 1;
            continue;
        }

        // Parse captures
        if in_captures_section {
            if let Some(capture) = parse_capture(trimmed, *index) {
                captures.push(capture);
            }
            *index += 1;
            continue;
        }

        // Parse headers (before response)
        if expected_status.is_none() && trimmed.contains(':') && !trimmed.starts_with('#') {
            if let Some((name, value)) = trimmed.split_once(':') {
                headers.push(Header {
                    name: name.trim().to_string(),
                    value: value.trim().to_string(),
                });
            }
        }

        *index += 1;
    }

    let line_end = index.saturating_sub(1);

    Some(HurlEntry {
        method,
        url,
        headers,
        body,
        expected_status,
        asserts,
        captures,
        line_start,
        line_end,
    })
}

/// Parse an assertion line
fn parse_assert(line: &str, line_num: usize) -> Option<Assert> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    // Common patterns:
    // status == 200
    // jsonpath "$.id" exists
    // jsonpath "$.name" == "value"
    // header "Content-Type" contains "json"

    let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
    if parts.is_empty() {
        return None;
    }

    let query_type = parts[0].to_string();
    let rest = parts.get(1).unwrap_or(&"").to_string();

    // Try to extract predicate and expected value
    let predicates = [
        "==",
        "!=",
        ">=",
        "<=",
        ">",
        "<",
        "contains",
        "startsWith",
        "endsWith",
        "matches",
        "exists",
        "isNumber",
        "isString",
        "isBoolean",
        "isCollection",
        "count",
    ];

    let mut query = rest.clone();
    let mut predicate = String::new();
    let mut expected = None;

    for pred in predicates {
        if let Some(pos) = rest.find(pred) {
            query = rest[..pos].trim().to_string();
            predicate = pred.to_string();
            let exp = rest[pos + pred.len()..].trim();
            if !exp.is_empty() {
                expected = Some(exp.to_string());
            }
            break;
        }
    }

    Some(Assert {
        text: trimmed.to_string(),
        query_type,
        query,
        predicate,
        expected,
        line: line_num,
    })
}

/// Parse a capture line
fn parse_capture(line: &str, line_num: usize) -> Option<Capture> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    // Pattern: name: query_type query
    // e.g., token: jsonpath "$.token"
    let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim().to_string();
    let rest = parts[1].trim();

    let query_parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
    let query_type = query_parts.first().unwrap_or(&"").to_string();
    let query = query_parts.get(1).unwrap_or(&"").to_string();

    Some(Capture {
        name,
        query_type,
        query,
        line: line_num,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_request() {
        let content = r#"
GET https://api.example.com/users
Accept: application/json

HTTP 200
"#;
        let hurl_file = parse_hurl_file(content).unwrap();
        assert_eq!(hurl_file.entries.len(), 1);
        assert_eq!(hurl_file.entries[0].method, "GET");
        assert_eq!(hurl_file.entries[0].url, "https://api.example.com/users");
        assert_eq!(hurl_file.entries[0].expected_status, Some(200));
    }

    #[test]
    fn test_parse_request_with_asserts() {
        let content = r#"
GET https://api.example.com/users
Accept: application/json

HTTP 200
[Asserts]
jsonpath "$.users" count > 0
jsonpath "$.users[0].id" exists
"#;
        let hurl_file = parse_hurl_file(content).unwrap();
        assert_eq!(hurl_file.entries.len(), 1);
        assert_eq!(hurl_file.entries[0].asserts.len(), 2);
    }
}
