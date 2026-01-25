use crate::{ProviderError, Result};
use async_trait::async_trait;
use securellm_core::{
    Choice, ContentPart, Error, HealthStatus, LLMProvider, Message, MessageContent, 
    MessageRole, ModelInfo, ModelPricing, ProviderCapabilities, ProviderHealth, Request, 
    Response, FinishReason, TokenUsage, ResponseMetadata,
};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

const DEFAULT_ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Google Gemini provider configuration
#[derive(Debug, Clone)]
pub struct GeminiConfig {
    pub api_key: SecretString,
    pub endpoint: String,
    pub timeout: Duration,
}

impl GeminiConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::new(api_key.into().into()),
            endpoint: DEFAULT_ENDPOINT.to_string(),
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

pub struct GeminiProvider {
    config: GeminiConfig,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(config: GeminiConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| ProviderError::Http(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self { config, client })
    }

    fn convert_role(role: MessageRole) -> String {
        match role {
            MessageRole::User => "user".to_string(),
            MessageRole::Assistant => "model".to_string(),
            MessageRole::System => "user".to_string(), 
            MessageRole::Function => "function".to_string(),
        }
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn version(&self) -> &str {
        "v1beta"
    }

    fn validate_config(&self) -> securellm_core::Result<()> {
        if self.config.api_key.expose_secret().is_empty() {
            return Err(Error::Config("Gemini API key is empty".to_string()));
        }
        Ok(())
    }

    async fn send_request(&self, request: Request) -> securellm_core::Result<Response> {
        let start = Instant::now();
        
        // 1. Convert Messages
        let mut contents = Vec::new();
        let mut system_instruction = None;

        for msg in &request.messages {
            match msg.role {
                MessageRole::System => {
                    if let MessageContent::Text(text) = &msg.content {
                        system_instruction = Some(GeminiContent {
                            role: None,
                            parts: vec![GeminiPart { text: Some(text.clone()) }],
                        });
                    }
                }
                _ => {
                    let text = match &msg.content {
                        MessageContent::Text(t) => t.clone(),
                        MessageContent::Parts(parts) => {
                            parts.iter().map(|p| match p {
                                ContentPart::Text { text } => text.as_str(),
                                _ => "", // Vision explicitly disabled
                            }).collect::<Vec<_>>().join(" ")
                        }
                    };
                    
                    contents.push(GeminiContent {
                        role: Some(Self::convert_role(msg.role)),
                        parts: vec![GeminiPart { text: Some(text) }],
                    });
                }
            }
        }

        // 2. Configure Safety Settings (OFF)
        // Explicitly disabling all safety filters to prevent false positives
        let safety_settings = vec![
            GeminiSafetySetting { category: "HARM_CATEGORY_HARASSMENT".to_string(), threshold: "BLOCK_NONE".to_string() },
            GeminiSafetySetting { category: "HARM_CATEGORY_HATE_SPEECH".to_string(), threshold: "BLOCK_NONE".to_string() },
            GeminiSafetySetting { category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(), threshold: "BLOCK_NONE".to_string() },
            GeminiSafetySetting { category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(), threshold: "BLOCK_NONE".to_string() },
        ];

        // 3. Build Request Body
        let gemini_req = GeminiRequest {
            contents,
            system_instruction,
            generation_config: Some(GeminiGenerationConfig {
                temperature: request.parameters.temperature,
                top_p: request.parameters.top_p,
                max_output_tokens: request.parameters.max_tokens,
                stop_sequences: request.parameters.stop.clone(),
            }),
            safety_settings: Some(safety_settings),
        };

        // 4. Send HTTP Request
        let url = format!("{}/models/{}:generateContent", self.config.endpoint, request.model);

        let response = self.client.post(&url)
            .header("x-goog-api-key", self.config.api_key.expose_secret())
            .json(&gemini_req)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Gemini request failed: {}", e)))?;

        let status = response.status();
        
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(Error::Provider {
                provider: "gemini".to_string(),
                message: format!("API Error {}: {}", status, err_text),
            });
        }

        let gemini_resp: GeminiResponse = response.json().await
            .map_err(|e| Error::Serialization(format!("Failed to parse Gemini response: {}", e)))?;

        // 5. Convert Response
        let processing_time = start.elapsed();
        
        if gemini_resp.candidates.is_empty() {
             // Even with BLOCK_NONE, Google might block severe policy violations
             return Err(Error::Provider {
                provider: "gemini".to_string(),
                message: "No candidates returned (External policy block?)".to_string(),
            });
        }

        let first_candidate = &gemini_resp.candidates[0];
        let content_text = first_candidate.content.parts.first()
            .and_then(|p| p.text.clone())
            .unwrap_or_default();

        let usage = TokenUsage {
            prompt_tokens: gemini_resp.usage_metadata.as_ref().map(|u| u.prompt_token_count).unwrap_or(0),
            completion_tokens: gemini_resp.usage_metadata.as_ref().map(|u| u.candidates_token_count).unwrap_or(0),
            total_tokens: gemini_resp.usage_metadata.as_ref().map(|u| u.total_token_count).unwrap_or(0),
            estimated_cost: None,
        };

        Ok(Response {
            request_id: request.id,
            id: format!("gemini-{}", uuid::Uuid::new_v4()),
            provider: "gemini".to_string(),
            model: request.model,
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: MessageRole::Assistant,
                    content: MessageContent::Text(content_text),
                    name: None,
                    metadata: None,
                },
                finish_reason: match first_candidate.finish_reason.as_deref() {
                    Some("STOP") => FinishReason::Stop,
                    Some("MAX_TOKENS") => FinishReason::Length,
                    Some("SAFETY") => FinishReason::ContentFilter,
                    _ => FinishReason::Unknown,
                },
                logprobs: None,
            }],
            usage,
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
        let url = format!("{}/models/gemini-2.0-flash?key={}", self.config.endpoint, self.config.api_key.expose_secret());
        let start = Instant::now();
        let response = self.client.get(&url).send().await;

        Ok(ProviderHealth {
            status: if response.is_ok() { HealthStatus::Healthy } else { HealthStatus::Unhealthy },
            latency_ms: Some(start.elapsed().as_millis() as u64),
            message: None,
            timestamp: chrono::Utc::now(),
        })
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: false,
            function_calling: true,
            vision: false, // EXPLICITLY DISABLED per requirement
            embeddings: true,
            max_tokens: Some(8192),
            max_context_window: Some(2000000), // 2M
            supports_system_prompts: true,
        }
    }

    async fn list_models(&self) -> securellm_core::Result<Vec<ModelInfo>> {
        // Updated for 2026 hypothetical model lineup
        Ok(vec![
            ModelInfo {
                id: "gemini-2.0-pro".to_string(),
                name: "Gemini 2.0 Pro".to_string(),
                description: Some("Flagship 2026 reasoning model with massive context".to_string()),
                context_window: Some(4000000), // 4M Context
                max_output_tokens: Some(16384),
                capabilities: vec!["chat".to_string(), "reasoning".to_string()],
                pricing: Some(ModelPricing {
                    input_cost_per_1k: 0.0025, // Price drop assumption
                    output_cost_per_1k: 0.0075,
                    currency: "USD".to_string(),
                }),
            },
            ModelInfo {
                id: "gemini-2.0-flash".to_string(),
                name: "Gemini 2.0 Flash".to_string(),
                description: Some("Ultra-low latency production model".to_string()),
                context_window: Some(2000000),
                max_output_tokens: Some(8192),
                capabilities: vec!["chat".to_string()],
                pricing: Some(ModelPricing {
                    input_cost_per_1k: 0.00020,
                    output_cost_per_1k: 0.00060,
                    currency: "USD".to_string(),
                }),
            },
        ])
    }
}

// --- Gemini API Structs ---

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "systemInstruction")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "generationConfig")]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "safetySettings")]
    safety_settings: Option<Vec<GeminiSafetySetting>>,
}

#[derive(Debug, Serialize)]
struct GeminiSafetySetting {
    category: String,
    threshold: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsage>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiUsage {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    total_token_count: u32,
}