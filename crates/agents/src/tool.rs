//! Core Tool trait and types
//!
//! Optimized for low-latency execution:
//! - Async by default
//! - Supports parallel execution
//! - Estimated latency hints for scheduling

use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Core trait for agent tools
///
/// All tools must implement this trait to be registered
/// in the ToolRegistry and executed by the AgentExecutor.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique tool name (used in tool calls)
    fn name(&self) -> &str;

    /// Human-readable description for LLM
    fn description(&self) -> &str;

    /// JSON schema for parameters (for validation)
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with given parameters
    async fn execute(&self, params: ToolParams) -> Result<ToolResult>;

    /// Estimated execution latency in milliseconds (for scheduling)
    ///
    /// Default: 50ms. Override for tools with known latency.
    fn estimated_latency_ms(&self) -> u64 {
        50
    }

    /// Can this tool run in parallel with others?
    ///
    /// Default: true. Set to false for tools that require exclusive access.
    fn supports_parallel(&self) -> bool {
        true
    }

    /// Maximum timeout in milliseconds
    ///
    /// Default: 30000ms (30s). Override for long-running tools.
    fn max_timeout_ms(&self) -> u64 {
        30000
    }
}

/// Tool parameters wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    /// Raw JSON value
    pub raw: Value,
}

impl ToolParams {
    pub fn new(value: Value) -> Self {
        Self { raw: value }
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let value: Value =
            serde_json::from_str(json).map_err(|e| crate::AgentError::ParseError(e.to_string()))?;
        Ok(Self { raw: value })
    }

    /// Get string parameter
    pub fn get_str(&self, key: &str) -> Result<String> {
        self.raw
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                crate::AgentError::InvalidParams(format!("Missing or invalid '{}'", key))
            })
    }

    /// Get optional string parameter
    pub fn get_str_opt(&self, key: &str) -> Option<String> {
        self.raw
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Get u64 parameter
    pub fn get_u64(&self, key: &str) -> Result<u64> {
        self.raw.get(key).and_then(|v| v.as_u64()).ok_or_else(|| {
            crate::AgentError::InvalidParams(format!("Missing or invalid '{}'", key))
        })
    }

    /// Get optional u64 parameter
    pub fn get_u64_opt(&self, key: &str) -> Option<u64> {
        self.raw.get(key).and_then(|v| v.as_u64())
    }

    /// Get boolean parameter
    pub fn get_bool(&self, key: &str) -> Result<bool> {
        self.raw.get(key).and_then(|v| v.as_bool()).ok_or_else(|| {
            crate::AgentError::InvalidParams(format!("Missing or invalid '{}'", key))
        })
    }
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool executed successfully
    pub success: bool,

    /// Tool output (text)
    pub output: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, Value>,

    /// Actual execution time in milliseconds
    pub latency_ms: u64,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            metadata: HashMap::new(),
            latency_ms: 0,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: message.into(),
            metadata: HashMap::new(),
            latency_ms: 0,
        }
    }

    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Helper type for Arc<dyn Tool>
pub type DynTool = Arc<dyn Tool>;
