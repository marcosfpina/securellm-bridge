//! High-performance Tool Registry with O(1) lookup
//!
//! Uses hashbrown::HashMap for better performance than std::HashMap

use crate::{DynTool, Tool};
use hashbrown::HashMap;
use std::sync::Arc;

/// Tool registry with O(1) lookup
pub struct ToolRegistry {
    /// Tools indexed by name (O(1) lookup)
    tools: HashMap<String, DynTool>,

    /// Pre-computed system prompt for LLM
    tools_prompt: String,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            tools_prompt: String::new(),
        }
    }

    /// Create registry with default tools
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register default tools
        // TODO: Add default tools when implemented
        // registry.register(Box::new(CodeExecutor::new()));
        // registry.register(Box::new(FileOps::new()));

        registry.build_prompt();
        registry
    }

    /// Register a tool (O(1) insertion)
    pub fn register<T: Tool + 'static>(&mut self, tool: T) -> &mut Self {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
        self.build_prompt();
        self
    }

    /// Get a tool by name (O(1) lookup)
    pub fn get(&self, name: &str) -> Option<&DynTool> {
        self.tools.get(name)
    }

    /// Check if tool exists (O(1))
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Get pre-computed tools prompt for LLM
    ///
    /// This prompt is built once and cached for efficiency.
    /// Format: XML-style tool definitions
    pub fn tools_prompt(&self) -> &str {
        &self.tools_prompt
    }

    /// Build the system prompt from registered tools
    ///
    /// Called automatically after registration.
    /// Format optimized for XML parsing.
    fn build_prompt(&mut self) {
        if self.tools.is_empty() {
            self.tools_prompt = String::new();
            return;
        }

        let mut prompt = String::from("Available tools:\n\n");

        for tool in self.tools.values() {
            prompt.push_str(&format!(
                "<tool name=\"{}\">\n  <description>{}</description>\n  <parameters>{}</parameters>\n</tool>\n\n",
                tool.name(),
                tool.description(),
                tool.parameters_schema()
            ));
        }

        prompt.push_str("\nTo use a tool, respond with:\n");
        prompt.push_str("<tool_call name=\"tool_name\">\n  {\"param\": \"value\"}\n</tool_call>");

        self.tools_prompt = prompt;
    }

    /// Get all tool names (sorted)
    pub fn tool_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.tools.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ToolParams, ToolResult};
    use async_trait::async_trait;
    use serde_json::json;

    struct DummyTool;

    #[async_trait]
    impl Tool for DummyTool {
        fn name(&self) -> &str {
            "dummy"
        }
        fn description(&self) -> &str {
            "A dummy tool"
        }
        fn parameters_schema(&self) -> serde_json::Value {
            json!({"type": "object"})
        }
        async fn execute(&self, _params: ToolParams) -> crate::Result<ToolResult> {
            Ok(ToolResult::success("ok"))
        }
    }

    #[test]
    fn test_registry_operations() {
        let mut registry = ToolRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        registry.register(DummyTool);
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert!(registry.contains("dummy"));

        let tool = registry.get("dummy");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "dummy");
    }
}
