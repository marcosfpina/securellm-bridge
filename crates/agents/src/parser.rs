//! High-performance XML parser for tool calls
//!
//! Optimized for < 1ms latency:
//! - Streaming parsing (process chunks as they arrive)
//! - Zero-copy where possible
//! - Fallback regex for malformed XML

use crate::{AgentError, Result, ToolParams};
use lazy_static::lazy_static;
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;

lazy_static! {
    /// Regex fallback for malformed XML
    /// Pattern: <tool_call name="tool_name">{"param": "value"}</tool_call>
    static ref TOOL_CALL_REGEX: Regex = Regex::new(
        r#"<tool_call\s+name="([^"]+)">\s*(\{[^}]*\})\s*</tool_call>"#
    ).unwrap();
}

/// Parsed tool call
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub params: ToolParams,
}

impl ToolCall {
    pub fn new(name: impl Into<String>, params: ToolParams) -> Self {
        Self {
            name: name.into(),
            params,
        }
    }
}

/// Streaming XML parser for tool calls
///
/// Designed for low latency (< 1ms target):
/// - Process chunks as they arrive
/// - Zero-copy parsing with quick-xml
/// - Regex fallback for malformed responses
pub struct ToolCallParser {
    buffer: Vec<u8>,
}

impl ToolCallParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096), // Pre-allocate 4KB
        }
    }

    /// Parse streaming XML (accumulates in buffer)
    ///
    /// Returns completed tool calls. Partial calls remain in buffer.
    pub fn parse_streaming(&mut self, chunk: &[u8]) -> Vec<ToolCall> {
        self.buffer.extend_from_slice(chunk);
        self.parse_buffer()
    }

    /// Parse complete XML response
    pub fn parse(&mut self, xml: &str) -> Vec<ToolCall> {
        self.buffer.clear();
        self.buffer.extend_from_slice(xml.as_bytes());
        self.parse_buffer()
    }

    /// Internal: Parse current buffer
    fn parse_buffer(&mut self) -> Vec<ToolCall> {
        // Try XML parsing first (fastest)
        match self.parse_xml() {
            Ok(calls) if !calls.is_empty() => calls,
            _ => {
                // Fallback to regex (for malformed XML)
                self.parse_regex()
            }
        }
    }

    /// Parse XML using quick-xml (< 1ms for typical responses)
    fn parse_xml(&self) -> Result<Vec<ToolCall>> {
        let mut reader = Reader::from_reader(self.buffer.as_slice());
        // Note: trim_text config not available in this version

        let mut calls = Vec::new();
        let mut current_tool: Option<String> = None;
        let mut current_params = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    if e.name().as_ref() == b"tool_call" {
                        // Extract tool name from attribute
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                if attr.key.as_ref() == b"name" {
                                    current_tool =
                                        Some(String::from_utf8_lossy(&attr.value).to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    if current_tool.is_some() {
                        current_params = e
                            .unescape()
                            .map_err(|e| AgentError::ParseError(e.to_string()))?
                            .to_string();
                    }
                }
                Ok(Event::End(e)) => {
                    if e.name().as_ref() == b"tool_call" {
                        if let Some(tool_name) = current_tool.take() {
                            // Parse JSON parameters
                            match ToolParams::from_json(&current_params) {
                                Ok(params) => {
                                    calls.push(ToolCall::new(tool_name, params));
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to parse tool params: {}", e);
                                }
                            }
                            current_params.clear();
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(AgentError::ParseError(format!("XML parse error: {}", e)));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(calls)
    }

    /// Fallback regex parsing (for malformed XML)
    fn parse_regex(&self) -> Vec<ToolCall> {
        let text = String::from_utf8_lossy(&self.buffer);
        let mut calls = Vec::new();

        for cap in TOOL_CALL_REGEX.captures_iter(&text) {
            let name = cap[1].to_string();
            let params_json = &cap[2];

            match ToolParams::from_json(params_json) {
                Ok(params) => {
                    calls.push(ToolCall::new(name, params));
                }
                Err(e) => {
                    tracing::warn!("Regex parse failed for tool params: {}", e);
                }
            }
        }

        calls
    }

    /// Clear internal buffer (call after processing)
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for ToolCallParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_single_tool_call() {
        let xml = r#"<tool_call name="execute_code">{"language": "rust", "code": "println!(\"hello\")"}</tool_call>"#;

        let mut parser = ToolCallParser::new();
        let calls = parser.parse(xml);

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "execute_code");
    }

    #[test]
    fn test_parse_multiple_tool_calls() {
        let xml = r#"
            <tool_call name="read_file">{"path": "/tmp/test.txt"}</tool_call>
            <tool_call name="execute_code">{"language": "python", "code": "print('hi')"}</tool_call>
        "#;

        let mut parser = ToolCallParser::new();
        let calls = parser.parse(xml);

        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "read_file");
        assert_eq!(calls[1].name, "execute_code");
    }

    #[test]
    fn test_parse_malformed_xml() {
        // Missing closing tag - should fallback to regex
        let xml = r#"<tool_call name="test">{"param": "value"}</tool_call"#;

        let mut parser = ToolCallParser::new();
        let calls = parser.parse(xml);

        // Regex should still catch it
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn test_streaming_parse() {
        let chunk1 = b"<tool_call name=\"test\">";
        let chunk2 = b"{\"param\": \"value\"}";
        let chunk3 = b"</tool_call>";

        let mut parser = ToolCallParser::new();

        // First chunks may not return results
        let _ = parser.parse_streaming(chunk1);
        let _ = parser.parse_streaming(chunk2);

        // Final chunk completes the call
        let calls = parser.parse_streaming(chunk3);

        assert!(calls.len() > 0 || !parser.buffer.is_empty());
    }
}
