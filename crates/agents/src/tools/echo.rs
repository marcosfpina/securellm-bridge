//! Echo tool - Simple test tool for development

use crate::{Result, Tool, ToolParams, ToolResult};
use async_trait::async_trait;
use serde_json::json;

/// Echo tool - returns the input message
///
/// Useful for testing and debugging the agent system.
pub struct EchoTool;

impl EchoTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo back the provided message. Useful for testing."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo back"
                }
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, params: ToolParams) -> Result<ToolResult> {
        let message = params.get_str("message")?;

        Ok(ToolResult::success(format!("Echo: {}", message)))
    }

    fn estimated_latency_ms(&self) -> u64 {
        1 // < 1ms
    }
}

impl Default for EchoTool {
    fn default() -> Self {
        Self::new()
    }
}
