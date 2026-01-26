//! Low-latency code agent system with efficient tool calling
//!
//! This crate provides a high-performance agent execution framework with:
//! - < 1ms XML parsing (streaming, zero-copy)
//! - O(1) tool lookup via HashMap
//! - Connection pooling for LLM providers
//! - LRU caching for tool results
//! - Parallel tool execution

pub mod tool;
pub mod registry;
pub mod parser;
pub mod executor;
pub mod cache;
pub mod tools;

pub use tool::{Tool, ToolParams, ToolResult, DynTool};
pub use registry::ToolRegistry;
pub use parser::{ToolCallParser, ToolCall};
pub use executor::AgentExecutor;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    
    #[error(transparent)]
    Core(#[from] securellm_core::Error),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AgentError>;
