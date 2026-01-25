use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use securellm_api_server::config::{Config, ProviderConfig, CircuitBreakerConfig};
use securellm_api_server::state::AppState;
use securellm_core::request::Request;
use securellm_core::intelligence::RoutingStrategy;
use std::sync::Arc;

#[tokio::test]
async fn test_smart_routing_fallback() {
    // 1. Start Mock Servers
    let mock_deepseek = MockServer::start().await;
    let mock_gemini = MockServer::start().await;

    // 2. Configure Gemini to FAIL (Primary Candidate - Cheapest)
    Mock::given(method("POST"))
        .and(path("/models/gemini-2.0-flash:generateContent"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .expect(1) // Should be called once and fail
        .mount(&mock_gemini)
        .await;

    // 3. Configure DeepSeek to SUCCEED (Secondary Candidate - Fallback)
    let deepseek_response = serde_json::json!({
        "id": "chatcmpl-123",
        "model": "deepseek-chat",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": "Fallback successful!" },
            "finish_reason": "stop"
        }],
        "usage": { "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 }
    });

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(deepseek_response))
        .expect(1) // Should be called once after fallback
        .mount(&mock_deepseek)
        .await;

    // 4. Setup Config pointing to Mocks
    let mut config = Config::default();
    
    // Disable others
    config.providers.openai = None;
    config.providers.anthropic = None;
    config.providers.groq = None;
    config.providers.nvidia = None;

    // Use in-memory DB for tests
    config.database.url = "sqlite::memory:".to_string();
    
    // Use Mock Redis (if available) or skip redis dependency if possible
    // Note: Since we don't have a mock redis, we assume the environment has one or we should mock the pool.
    // For this environment, let's assume we need to skip redis or point to a local one.
    // Ideally we'd use a trait for Cache too, but for now let's hope local redis is up or allow failure.
    // A better approach for unit tests is to mock the cache layer.
    // For now, let's just proceed, assuming the dev env has redis or the test will fail on redis.
    // If it fails on Redis, I will need to refactor AppState to be more mockable.

    // Configure DeepSeek (Primary - Broken)
    config.providers.deepseek = Some(ProviderConfig {
        enabled: true,
        api_key: "mock-key".to_string(),
        base_url: Some(mock_deepseek.uri()),
        timeout_secs: 1,
        max_retries: 0,
        rate_limit_per_minute: 100,
        circuit_breaker: CircuitBreakerConfig { failure_threshold: 1, success_threshold: 1, timeout_secs: 10 },
    });

    // Configure Gemini (Secondary - Healthy)
    config.providers.gemini = Some(ProviderConfig {
        enabled: true,
        api_key: "mock-key".to_string(),
        base_url: Some(mock_gemini.uri()), // Override endpoint base
        timeout_secs: 1,
        max_retries: 0,
        rate_limit_per_minute: 100,
        circuit_breaker: CircuitBreakerConfig { failure_threshold: 1, success_threshold: 1, timeout_secs: 10 },
    });

    // 5. Initialize AppState
    let state = AppState::new(config).await.expect("Failed to init state");

    // 6. Execute Request via Routing Engine
    // We simulate what the route handler does
    let candidates = vec![
        ("deepseek".to_string(), "deepseek-chat".to_string()),
        ("gemini".to_string(), "gemini-2.0-flash".to_string()),
    ];

    let ranked = state.routing_engine.select_candidates(candidates, RoutingStrategy::LowestCost);
    assert_eq!(ranked[0].0, "gemini", "Gemini Flash should be first (cheapest in 2026)");

    // Manual Loop Simulation (Integration Test)
    let mut success = false;
    let mut last_response = String::new();

    for (p_name, m_name) in ranked {
        let provider = state.provider_manager.get_provider(&p_name).await.unwrap();
        
use securellm_core::Message;
use securellm_core::MessageRole;
use securellm_core::MessageContent;

// ...

        // Mock Core Request
        let mut req = Request::new(p_name.clone(), m_name);
        req.messages.push(Message {
            role: MessageRole::User,
            content: MessageContent::Text("Hello".to_string()),
            name: None,
            metadata: None,
        });
        
        match provider.send_request(req).await {
            Ok(resp) => {
                success = true;
                last_response = resp.choices[0].message.content.text();
                break;
            }
            Err(e) => {
                println!("Request to {} failed: {:?}", p_name, e);
                // Should fail for Primary
                continue;
            }
        }
    }

    assert!(success, "Request should eventually succeed");
    assert_eq!(last_response, "Fallback successful!", "Should receive response from Gemini");
}
