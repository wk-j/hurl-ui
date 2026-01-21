//! Hurl execution runner module
//!
//! This module handles executing Hurl files and capturing results.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Hurl runner that executes .hurl files
pub struct Runner {
    /// Path to hurl binary (None = use PATH)
    hurl_path: Option<PathBuf>,
    /// Default timeout in seconds
    timeout: u64,
}

impl Runner {
    /// Create a new runner instance
    pub fn new() -> Self {
        Self {
            hurl_path: None,
            timeout: 30,
        }
    }

    /// Create a runner with a specific hurl binary path
    pub fn with_hurl_path(mut self, path: PathBuf) -> Self {
        self.hurl_path = Some(path);
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Run a hurl file and return the execution result
    pub async fn run(
        &self,
        file_path: &PathBuf,
        variables_file: Option<&PathBuf>,
    ) -> Result<ExecutionResult> {
        let hurl_cmd = self
            .hurl_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "hurl".to_string());

        // First run: with --very-verbose to get body and details
        let mut cmd = Command::new(&hurl_cmd);
        cmd.arg(file_path);
        cmd.arg("--very-verbose");
        cmd.arg("--max-time");
        cmd.arg(self.timeout.to_string());

        if let Some(vars_file) = variables_file {
            cmd.arg("--variables-file");
            cmd.arg(vars_file);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .context("Failed to execute hurl command")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        // Parse response from very-verbose output (includes body)
        let response = self.parse_response_from_very_verbose(&stderr, &stdout);
        let asserts = self.parse_asserts(&stderr);

        Ok(ExecutionResult {
            success,
            response,
            assertions: asserts,
            stdout,
            stderr,
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    /// Parse response from --very-verbose output
    /// The body is output to stdout, metadata is in stderr
    fn parse_response_from_very_verbose(&self, stderr: &str, stdout: &str) -> Option<Response> {
        let mut status_code = 0u16;
        let mut headers = Vec::new();
        let mut duration_ms = 0u64;
        let mut in_response_headers = false;
        let mut body_lines: Vec<String> = Vec::new();
        let mut in_response_body = false;

        for line in stderr.lines() {
            // Look for response status line: "< HTTP/1.1 200 OK" or "< HTTP/2 200"
            if line.starts_with("< HTTP/") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    // Status code is the second part after "< HTTP/x.x"
                    if let Ok(status) = parts[1].parse::<u16>() {
                        status_code = status;
                        in_response_headers = true;
                        in_response_body = false;
                    } else if parts.len() >= 3 {
                        // Try third part (for "< HTTP/1.1 200 OK" format)
                        if let Ok(status) = parts[2].parse::<u16>() {
                            status_code = status;
                            in_response_headers = true;
                            in_response_body = false;
                        }
                    }
                }
                continue;
            }

            // Parse response headers (lines starting with "< ")
            if in_response_headers && line.starts_with("< ") {
                let header_line = &line[2..];
                if header_line.trim().is_empty() {
                    // Empty line marks end of headers
                    in_response_headers = false;
                    in_response_body = true;
                    continue;
                }
                if let Some((name, value)) = header_line.split_once(':') {
                    headers.push((name.trim().to_string(), value.trim().to_string()));
                }
                continue;
            }

            // Capture response body lines (lines starting with "* Response body:")
            // or lines after we've seen the body marker
            if line.starts_with("* Response body:") {
                in_response_body = true;
                // The body content follows on subsequent lines with "* " prefix
                continue;
            }

            // Capture body lines (prefixed with "* " in very-verbose mode)
            if in_response_body && line.starts_with("* ") {
                let body_line = &line[2..];
                // Stop if we hit another section marker
                if body_line.starts_with("Executing entry")
                    || body_line.starts_with("Cookie store:")
                    || body_line.starts_with("Request:")
                    || body_line.contains("Timings:")
                {
                    in_response_body = false;
                    continue;
                }
                body_lines.push(body_line.to_string());
                continue;
            }

            // Look for timing information
            if line.contains("time_total:") {
                // Format: "* time_total: 0.123456 s"
                if let Some(time_part) = line.split("time_total:").nth(1) {
                    let time_str = time_part.trim().trim_end_matches(" s").trim_end_matches('s');
                    if let Ok(secs) = time_str.trim().parse::<f64>() {
                        duration_ms = (secs * 1000.0) as u64;
                    }
                }
            }
        }

        // If we didn't capture body from stderr, use stdout (hurl outputs body to stdout)
        let body = if body_lines.is_empty() {
            stdout.to_string()
        } else {
            body_lines.join("\n")
        };

        if status_code > 0 {
            Some(Response {
                status_code,
                headers,
                body,
                duration_ms,
            })
        } else {
            None
        }
    }

    /// Parse assertion results from stderr
    fn parse_asserts(&self, stderr: &str) -> Vec<AssertionResult> {
        let mut results = Vec::new();

        for line in stderr.lines() {
            // Look for assertion output patterns
            // Success: "  jsonpath "$.id" exists"
            // Failure: "error: Assert failure" followed by details

            if line.contains("Assert failure") || line.contains("assert failure") {
                // This is a failed assertion
                let text = line.trim().to_string();
                results.push(AssertionResult {
                    success: false,
                    text,
                    expected: None,
                    actual: None,
                    message: Some(line.to_string()),
                });
            } else if line.trim().starts_with("*") && line.contains("assert") {
                // Verbose output shows assertions with *
                let text = line.trim_start_matches('*').trim().to_string();
                results.push(AssertionResult {
                    success: true,
                    text,
                    expected: None,
                    actual: None,
                    message: None,
                });
            }
        }

        results
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of executing a Hurl file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution was successful (all assertions passed)
    pub success: bool,
    /// Response details (if available)
    pub response: Option<Response>,
    /// Assertion results
    pub assertions: Vec<AssertionResult>,
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
}

/// HTTP response details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// HTTP status code
    pub status_code: u16,
    /// Response headers
    pub headers: Vec<(String, String)>,
    /// Response body
    pub body: String,
    /// Response duration in milliseconds
    pub duration_ms: u64,
}

/// Result of a single assertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    /// Whether the assertion passed
    pub success: bool,
    /// Assertion text
    pub text: String,
    /// Expected value (if applicable)
    pub expected: Option<String>,
    /// Actual value (if applicable)
    pub actual: Option<String>,
    /// Error message (if failed)
    pub message: Option<String>,
}

// Structures for parsing Hurl JSON output

#[derive(Debug, Deserialize)]
struct HurlJsonOutput {
    entries: Vec<HurlJsonEntry>,
}

#[derive(Debug, Deserialize)]
struct HurlJsonEntry {
    request: Option<HurlJsonRequest>,
    response: Option<HurlJsonResponse>,
    #[serde(rename = "timings")]
    time_in_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct HurlJsonRequest {
    method: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct HurlJsonResponse {
    status: u16,
    headers: Vec<HurlJsonHeader>,
    body: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HurlJsonHeader {
    name: String,
    value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let runner = Runner::new();
        assert!(runner.hurl_path.is_none());
        assert_eq!(runner.timeout, 30);
    }

    #[test]
    fn test_runner_with_options() {
        let runner = Runner::new()
            .with_hurl_path(PathBuf::from("/usr/bin/hurl"))
            .with_timeout(60);

        assert_eq!(runner.hurl_path, Some(PathBuf::from("/usr/bin/hurl")));
        assert_eq!(runner.timeout, 60);
    }
}
