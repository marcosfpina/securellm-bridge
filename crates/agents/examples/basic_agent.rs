//! Example: Basic agent execution with echo tool

use securellm_agents::{
    AgentExecutor, ToolRegistry,
    tools::EchoTool,
};
use securellm_providers::llamacpp::LlamaCppProvider;
use securellm_core::LLMProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("🚀 SecureLLM Agent Demo - Low Latency Edition\n");
    
    // 1. Create tool registry with echo tool
    let mut registry = ToolRegistry::new();
    registry.register(EchoTool::new());
    
    println!("✅ Registered {} tool(s)", registry.len());
    println!("📋 Tools: {:?}\n", registry.tool_names());
    
    // 2. Create LlamaCpp provider (port 8081)
    let provider = LlamaCppProvider::new(8081, "llamacppturbo")?;
    println!("✅ Connected to LlamaCpp (localhost:8081)\n");
    
    // 3. Create agent executor
    let agent = AgentExecutor::new(registry, Arc::new(provider));
    println!("✅ Agent executor initialized\n");
    
    // 4. Execute agent with tool call
    println!("📤 Sending: Please use the echo tool to say 'Hello from agent!'\n");
    
    let response = agent.execute(
        "Please use the echo tool to say 'Hello from agent!'"
    ).await?;
    
    // 5. Display results
    println!("📥 Response:\n{}\n", response.text);
    println!("⚡ Performance:");
    println!("   - Tool calls: {}", response.tool_calls);
    println!("   - Latency: {}ms", response.latency_ms);
    
    if !response.tool_results.is_empty() {
        println!("\n🔧 Tool Results:");
        for (i, result) in response.tool_results.iter().enumerate() {
            println!("   {}. Success: {} | Output: {} | Latency: {}ms",
                i + 1,
                result.success,
                result.output.lines().next().unwrap_or(""),
                result.latency_ms
            );
        }
    }
    
    Ok(())
}
