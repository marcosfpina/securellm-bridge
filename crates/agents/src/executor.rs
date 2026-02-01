//! Agent executor - orchestrates LLM and tool execution
//!
//! Optimized for low latency with:
//! - Connection pooling
//! - Result caching
//! - Parallel tool execution

use crate::{
    cache::ResultCache, AgentError, Result, ToolCall, ToolCallParser, ToolRegistry, ToolResult,
};
use securellm_core::{LLMProvider, Message, MessageContent, MessageRole, Request};
use std::sync::Arc;
use std::time::Instant;

/// Agent execution response
#[derive(Debug, Clone)]
pub struct AgentResponse {
    /// Final response text
    pub text: String,

    /// Number of tool calls made
    pub tool_calls: usize,

    /// Total latency (E2E)
    pub latency_ms: u64,

    /// Tool execution results
    pub tool_results: Vec<ToolResult>,
}

/// Agent executor configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Enable result caching
    pub enable_cache: bool,

    /// Cache capacity (number of entries)
    pub cache_capacity: usize,

    /// Maximum iterations (prevent infinite loops)
    pub max_iterations: usize,

    /// Enable parallel tool execution
    pub parallel_execution: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_capacity: 256,
            max_iterations: 10,
            parallel_execution: true,
        }
    }
}

/// High-performance agent executor
pub struct AgentExecutor {
    /// Tool registry (O(1) lookup)
    registry: Arc<ToolRegistry>,

    /// LLM provider
    provider: Arc<dyn LLMProvider>,

    /// Result cache (optional)
    cache: Option<Arc<ResultCache>>,

    /// Configuration
    config: AgentConfig,
}

impl AgentExecutor {
    /// Create a new executor
    pub fn new(registry: ToolRegistry, provider: Arc<dyn LLMProvider>) -> Self {
        Self::with_config(registry, provider, AgentConfig::default())
    }

    /// Create executor with custom config
    pub fn with_config(
        registry: ToolRegistry,
        provider: Arc<dyn LLMProvider>,
        config: AgentConfig,
    ) -> Self {
        let cache = if config.enable_cache {
            Some(Arc::new(ResultCache::new(config.cache_capacity)))
        } else {
            None
        };

        Self {
            registry: Arc::new(registry),
            provider,
            cache,
            config,
        }
    }

    /// Execute one agent turn (main entry point)
    pub async fn execute(&self, user_message: &str) -> Result<AgentResponse> {
        let start = Instant::now();
        let mut iteration = 0;
        let mut all_tool_results = Vec::new();

        // Build initial prompt with tools
        let mut conversation = vec![
            Message {
                role: MessageRole::System,
                content: MessageContent::Text(self.registry.tools_prompt().to_string()),
                name: None,
                metadata: None,
            },
            Message {
                role: MessageRole::User,
                content: MessageContent::Text(user_message.to_string()),
                name: None,
                metadata: None,
            },
        ];

        // Agent loop (max iterations to prevent infinite loops)
        loop {
            iteration += 1;

            if iteration > self.config.max_iterations {
                return Ok(AgentResponse {
                    text: "Max iterations reached".to_string(),
                    tool_calls: all_tool_results.len(),
                    latency_ms: start.elapsed().as_millis() as u64,
                    tool_results: all_tool_results,
                });
            }

            // Send request to LLM
            let request = Request::new(self.provider.name(), "llamacppturbo")
                .with_system(self.registry.tools_prompt());

            let request = conversation
                .iter()
                .fold(request, |req, msg| req.add_message(msg.clone()));

            let response = self
                .provider
                .send_request(request)
                .await
                .map_err(|e| AgentError::Core(e))?;

            let response_text = response.text().map_err(|e| AgentError::Core(e))?;

            // Parse tool calls
            let mut parser = ToolCallParser::new();
            let tool_calls = parser.parse(&response_text);

            if tool_calls.is_empty() {
                // No tools called - done!
                return Ok(AgentResponse {
                    text: response_text,
                    tool_calls: all_tool_results.len(),
                    latency_ms: start.elapsed().as_millis() as u64,
                    tool_results: all_tool_results,
                });
            }

            // Execute tools
            let tool_results = self.execute_tools(&tool_calls).await?;
            all_tool_results.extend(tool_results.clone());

            // Add assistant response to conversation
            conversation.push(Message {
                role: MessageRole::Assistant,
                content: MessageContent::Text(response_text),
                name: None,
                metadata: None,
            });

            // Add tool results to conversation
            let results_text = tool_results
                .iter()
                .map(|r| format!("Tool result: {}", r.output))
                .collect::<Vec<_>>()
                .join("\n");

            conversation.push(Message {
                role: MessageRole::User,
                content: MessageContent::Text(results_text),
                name: None,
                metadata: None,
            });
        }
    }

    /// Execute multiple tool calls (sequential or parallel based on config)
    async fn execute_tools(&self, calls: &[ToolCall]) -> Result<Vec<ToolResult>> {
        if self.config.parallel_execution {
            self.execute_tools_parallel(calls).await
        } else {
            self.execute_tools_sequential(calls).await
        }
    }

    /// Execute tools sequentially
    async fn execute_tools_sequential(&self, calls: &[ToolCall]) -> Result<Vec<ToolResult>> {
        let mut results = Vec::new();

        for call in calls {
            let result = self.execute_single_tool(call).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute tools in parallel (when safe)
    async fn execute_tools_parallel(&self, calls: &[ToolCall]) -> Result<Vec<ToolResult>> {
        let mut results = Vec::new();

        // Simple sequential for now (tokio::spawn in future)
        for call in calls {
            let result = self.execute_single_tool(call).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute a single tool call
    async fn execute_single_tool(&self, call: &ToolCall) -> Result<ToolResult> {
        let start = Instant::now();

        // Check cache first
        if let Some(cache) = &self.cache {
            let cache_key = format!(
                "{}:{}",
                call.name,
                serde_json::to_string(&call.params.raw).unwrap_or_default()
            );

            if let Some(cached) = cache.get(&cache_key) {
                tracing::debug!("Cache hit for tool: {}", call.name);
                return Ok(cached);
            }
        }

        // Get tool from registry
        let tool = self
            .registry
            .get(&call.name)
            .ok_or_else(|| AgentError::ToolNotFound(call.name.clone()))?;

        // Execute tool
        let mut result = tool.execute(call.params.clone()).await?;

        // Update latency
        result.latency_ms = start.elapsed().as_millis() as u64;

        // Cache result
        if let Some(cache) = &self.cache {
            let cache_key = format!(
                "{}:{}",
                call.name,
                serde_json::to_string(&call.params.raw).unwrap_or_default()
            );
            cache.put(cache_key, result.clone());
        }

        Ok(result)
    }
}
