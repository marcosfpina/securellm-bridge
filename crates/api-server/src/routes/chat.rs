use axum::{
    extract::State,
    response::{
        IntoResponse,
        sse::{Event, KeepAlive},
        Sse,
    },
    Json,
};
use futures::{stream::{self, Stream}, StreamExt};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc, time::Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;
use securellm_core::audit::{AuditLogger, AuditEvent, RequestStatus, AuditEventType};

use crate::{error::ApiResult, state::AppState};

/// OpenAI-compatible chat completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub n: Option<u32>,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub presence_penalty: Option<f32>,
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    #[serde(default)]
    pub logit_bias: Option<serde_json::Value>,
    #[serde(default)]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// OpenAI-compatible chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenAI-compatible streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChunkChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunkChoice {
    pub index: u32,
    pub delta: ChatDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// POST /v1/chat/completions - Chat completion endpoint
///
/// Supports both streaming and non-streaming responses.
/// Routes requests to appropriate providers based on model prefix.
pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatCompletionRequest>,
) -> ApiResult<axum::response::Response> {
    let request_id = Uuid::new_v4();
    let start = Instant::now();

    info!(
        "Chat completion request: request_id={}, model={}, messages={}, stream={:?}",
        request_id,
        req.model,
        req.messages.len(),
        req.stream
    );

    // Parse provider and model from model string (format: "provider/model")
    let (provider_name, model_name) = parse_model_identifier(&req.model)?;

    debug!("Routing to provider: {}, model: {}", provider_name, model_name);

    // Log request received
    if let Err(e) = state.audit_logger.log_request_received(
        request_id,
        &provider_name,
        &model_name,
        req.messages.len(),
        None,  // TODO: Extract client IP from headers
    ).await {
        warn!("Failed to log audit event: {}", e);
    }

    // Check if streaming is requested
    if req.stream.unwrap_or(false) {
        // Return SSE stream
        let stream = create_completion_stream(state, req, provider_name, model_name, request_id).await?;
        Ok(Sse::new(stream)
            .keep_alive(KeepAlive::default())
            .into_response())
    } else {
        // Return complete response
        match create_completion(state.clone(), req.clone(), provider_name.clone(), model_name.clone()).await {
            Ok(response) => {
                let duration_ms = start.elapsed().as_millis() as u64;

                // Calculate cost
                let prompt_tokens = response.usage.prompt_tokens;
                let completion_tokens = response.usage.completion_tokens;
                let cost = AuditLogger::estimate_cost(
                    &provider_name,
                    &model_name,
                    prompt_tokens,
                    completion_tokens,
                );

                // Log response sent
                let audit_event = AuditEvent {
                    timestamp: chrono::Utc::now(),
                    request_id,
                    event_type: AuditEventType::ResponseSent,
                    user_id: req.user.clone(),
                    provider: provider_name.clone(),
                    model: model_name.clone(),
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: response.usage.total_tokens,
                    estimated_cost_usd: cost,
                    duration_ms,
                    status: RequestStatus::Success,
                    error_message: None,
                    client_ip: None,
                };

                if let Err(e) = state.audit_logger.log_response_sent(&audit_event).await {
                    warn!("Failed to log audit event: {}", e);
                }

                Ok(Json(response).into_response())
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;

                // Log request failed
                if let Err(log_err) = state.audit_logger.log_request_failed(
                    request_id,
                    &provider_name,
                    &e.to_string(),
                    duration_ms,
                ).await {
                    warn!("Failed to log audit event: {}", log_err);
                }

                Err(e)
            }
        }
    }
}

/// Parse model identifier into provider and model name
fn parse_model_identifier(model: &str) -> ApiResult<(String, String)> {
    if let Some((provider, model_name)) = model.split_once('/') {
        Ok((provider.to_string(), model_name.to_string()))
    } else {
        // If no slash, assume it's a model name and try to find the provider
        // Default to deepseek for now
        Ok(("deepseek".to_string(), model.to_string()))
    }
}

/// Create non-streaming completion
async fn create_completion(
    state: Arc<AppState>,
    req: ChatCompletionRequest,
    provider_name: String,
    model_name: String,
) -> ApiResult<ChatCompletionResponse> {
    // TODO: Implement actual provider call
    // For now, return a mock response
    
    warn!("Using mock response - provider implementation pending");

    let completion_id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
    let created = chrono::Utc::now().timestamp();

    Ok(ChatCompletionResponse {
        id: completion_id,
        object: "chat.completion".to_string(),
        created,
        model: req.model.clone(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: "This is a mock response. Provider implementation is pending.".to_string(),
                name: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        },
    })
}

/// Create streaming completion
async fn create_completion_stream(
    state: Arc<AppState>,
    req: ChatCompletionRequest,
    provider_name: String,
    model_name: String,
    request_id: Uuid,
) -> ApiResult<impl Stream<Item = Result<Event, Infallible>>> {
    // TODO: Implement actual provider streaming
    // For now, return a mock stream
    
    warn!("Using mock stream - provider implementation pending");

    let completion_id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
    let created = chrono::Utc::now().timestamp();
    let model = req.model.clone();

    let chunks = vec![
        ChatCompletionChunk {
            id: completion_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: model.clone(),
            choices: vec![ChatChunkChoice {
                index: 0,
                delta: ChatDelta {
                    role: Some("assistant".to_string()),
                    content: None,
                },
                finish_reason: None,
            }],
        },
        ChatCompletionChunk {
            id: completion_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: model.clone(),
            choices: vec![ChatChunkChoice {
                index: 0,
                delta: ChatDelta {
                    role: None,
                    content: Some("This ".to_string()),
                },
                finish_reason: None,
            }],
        },
        ChatCompletionChunk {
            id: completion_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: model.clone(),
            choices: vec![ChatChunkChoice {
                index: 0,
                delta: ChatDelta {
                    role: None,
                    content: Some("is a mock stream.".to_string()),
                },
                finish_reason: None,
            }],
        },
        ChatCompletionChunk {
            id: completion_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: model.clone(),
            choices: vec![ChatChunkChoice {
                index: 0,
                delta: ChatDelta {
                    role: None,
                    content: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        },
    ];

    let stream = stream::iter(chunks).map(|chunk| {
        let data = serde_json::to_string(&chunk).unwrap();
        Ok(Event::default().data(data))
    });

    Ok(stream)
}