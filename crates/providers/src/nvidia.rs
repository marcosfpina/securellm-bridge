use crate::{ProviderError, Result};
use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use securellm_core::{
    Choice, Error, FinishReason, HealthStatus, LLMProvider, Message, MessageContent, MessageRole,
    ModelInfo, ModelPricing, ProviderCapabilities, ProviderHealth, Request, Response,
    ResponseMetadata, TokenUsage,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

const DEFAULT_ENDPOINT: &str = "https://integrate.api.nvidia.com/v1";

pub struct NvidiaConfig {
    pub api_key: SecretString,
    pub endpoint: String,
}

impl NvidiaConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::new(api_key.into().into()),
            endpoint: DEFAULT_ENDPOINT.to_string(),
        }
    }
}

pub struct NvidiaProvider {
    config: NvidiaConfig,
    client: reqwest::Client,
}

impl NvidiaProvider {
    pub fn new(config: NvidiaConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| ProviderError::Http(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }
}

#[async_trait]
impl LLMProvider for NvidiaProvider {
    fn name(&self) -> &str {
        "nvidia"
    }
    fn version(&self) -> &str {
        "v1"
    }

    fn validate_config(&self) -> securellm_core::Result<()> {
        if self.config.api_key.expose_secret().is_empty() {
            return Err(Error::Config("NVIDIA API key is empty".to_string()));
        }
        Ok(())
    }

    async fn send_request(&self, request: Request) -> securellm_core::Result<Response> {
        let start = Instant::now();
        let url = format!("{}/chat/completions", self.config.endpoint);

        let response = self
            .client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.expose_secret()),
            )
            .json(&serde_json::json!({
                "model": request.model,
                "messages": request.messages,
                "temperature": request.parameters.temperature,
                "max_tokens": request.parameters.max_tokens,
            }))
            .send()
            .await
            .map_err(|e| Error::Network(format!("NVIDIA request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            return Err(Error::Provider {
                provider: "nvidia".to_string(),
                message: format!("API Error {}", status),
            });
        }

        let oai_resp: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::Serialization(format!("Failed to parse NVIDIA response: {}", e)))?;

        let processing_time = start.elapsed();

        Ok(Response {
            request_id: request.id,
            id: oai_resp["id"].as_str().unwrap_or_default().to_string(),
            provider: "nvidia".to_string(),
            model: request.model,
            choices: vec![],
            usage: TokenUsage {
                prompt_tokens: oai_resp["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: oai_resp["usage"]["completion_tokens"].as_u64().unwrap_or(0)
                    as u32,
                total_tokens: oai_resp["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
                estimated_cost: None,
            },
            metadata: ResponseMetadata {
                created_at: chrono::Utc::now(),
                processing_time_ms: processing_time.as_millis() as u64,
                cached: false,
                rate_limit_info: None,
                extra: std::collections::HashMap::new(),
            },
        })
    }

    async fn health_check(&self) -> securellm_core::Result<ProviderHealth> {
        Ok(ProviderHealth {
            status: HealthStatus::Healthy,
            latency_ms: None,
            message: None,
            timestamp: chrono::Utc::now(),
        })
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            function_calling: true,
            vision: true,
            embeddings: true,
            max_tokens: Some(4096),
            max_context_window: Some(128000),
            supports_system_prompts: true,
        }
    }

    async fn list_models(&self) -> securellm_core::Result<Vec<ModelInfo>> {
        Ok(vec![ModelInfo {
            id: "nvidia/llama-3.1-405b-instruct".to_string(),
            name: "Llama 3.1 405B".to_string(),
            description: Some("NVIDIA NIM optimized Llama 3.1 405B".to_string()),
            context_window: Some(128000),
            max_output_tokens: Some(4096),
            capabilities: vec!["chat".to_string()],
            pricing: Some(ModelPricing {
                input_cost_per_1k: 0.001,
                output_cost_per_1k: 0.003,
                currency: "USD".to_string(),
            }),
        }])
    }
}
