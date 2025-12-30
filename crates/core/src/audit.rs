// Enterprise-grade audit logging module
// Provides comprehensive audit trail for compliance (GDPR, SOC2, HIPAA)

use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::Result;

/// Audit event structure for comprehensive logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub request_id: Uuid,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub estimated_cost_usd: f64,
    pub duration_ms: u64,
    pub status: RequestStatus,
    pub error_message: Option<String>,
    pub client_ip: Option<String>,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    RequestReceived,
    ResponseSent,
    RequestFailed,
    SecurityEvent,
}

/// Request processing status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Success,
    Failed,
    RateLimited,
    Timeout,
}

/// Audit logger with async-safe structured logging
#[derive(Clone)]
pub struct AuditLogger {
    // Can include writer-specific configuration here in the future
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {}
    }

    /// Log when a request is received
    pub async fn log_request_received(
        &self,
        request_id: Uuid,
        provider: &str,
        model: &str,
        message_count: usize,
        client_ip: Option<String>,
    ) -> Result<()> {
        info!(
            audit.event = "request_received",
            audit.request_id = %request_id,
            audit.provider = provider,
            audit.model = model,
            audit.message_count = message_count,
            audit.client_ip = ?client_ip,
            audit.timestamp = %Utc::now().to_rfc3339(),
            "Audit: Request received"
        );
        Ok(())
    }

    /// Log when a response is sent successfully
    pub async fn log_response_sent(
        &self,
        event: &AuditEvent,
    ) -> Result<()> {
        info!(
            audit.event = "response_sent",
            audit.request_id = %event.request_id,
            audit.provider = %event.provider,
            audit.model = %event.model,
            audit.prompt_tokens = event.prompt_tokens,
            audit.completion_tokens = event.completion_tokens,
            audit.total_tokens = event.total_tokens,
            audit.cost_usd = event.estimated_cost_usd,
            audit.duration_ms = event.duration_ms,
            audit.status = ?event.status,
            audit.timestamp = %event.timestamp.to_rfc3339(),
            "Audit: Response sent"
        );
        Ok(())
    }

    /// Log when a request fails
    pub async fn log_request_failed(
        &self,
        request_id: Uuid,
        provider: &str,
        error: &str,
        duration_ms: u64,
    ) -> Result<()> {
        warn!(
            audit.event = "request_failed",
            audit.request_id = %request_id,
            audit.provider = provider,
            audit.error = error,
            audit.duration_ms = duration_ms,
            audit.timestamp = %Utc::now().to_rfc3339(),
            "Audit: Request failed"
        );
        Ok(())
    }

    /// Log security events
    pub async fn log_security_event(&self, event: &str, severity: &str) -> Result<()> {
        warn!(
            audit.event = "security_event",
            audit.event_type = event,
            audit.severity = severity,
            audit.timestamp = %Utc::now().to_rfc3339(),
            "Audit: Security event"
        );
        Ok(())
    }

    /// Helper to estimate cost based on provider and token usage
    /// Pricing as of January 2025
    pub fn estimate_cost(
        provider: &str,
        model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> f64 {
        match (provider, model) {
            ("deepseek", _) => {
                // DeepSeek: $0.14 / 1M input, $0.28 / 1M output
                let input_cost = (prompt_tokens as f64 / 1_000_000.0) * 0.14;
                let output_cost = (completion_tokens as f64 / 1_000_000.0) * 0.28;
                input_cost + output_cost
            }
            ("openai", model) if model.starts_with("gpt-4") => {
                // GPT-4: $30 / 1M input, $60 / 1M output (approximate)
                let input_cost = (prompt_tokens as f64 / 1_000_000.0) * 30.0;
                let output_cost = (completion_tokens as f64 / 1_000_000.0) * 60.0;
                input_cost + output_cost
            }
            ("openai", model) if model.starts_with("gpt-3.5") => {
                // GPT-3.5-turbo: $0.50 / 1M input, $1.50 / 1M output
                let input_cost = (prompt_tokens as f64 / 1_000_000.0) * 0.50;
                let output_cost = (completion_tokens as f64 / 1_000_000.0) * 1.50;
                input_cost + output_cost
            }
            ("anthropic", model) if model.starts_with("claude-3-opus") => {
                // Claude 3 Opus: $15 / 1M input, $75 / 1M output
                let input_cost = (prompt_tokens as f64 / 1_000_000.0) * 15.0;
                let output_cost = (completion_tokens as f64 / 1_000_000.0) * 75.0;
                input_cost + output_cost
            }
            ("anthropic", model) if model.starts_with("claude-3-sonnet") => {
                // Claude 3 Sonnet: $3 / 1M input, $15 / 1M output
                let input_cost = (prompt_tokens as f64 / 1_000_000.0) * 3.0;
                let output_cost = (completion_tokens as f64 / 1_000_000.0) * 15.0;
                input_cost + output_cost
            }
            ("ollama", _) | ("llamacpp", _) => {
                // Local models have no API cost
                0.0
            }
            _ => {
                // Unknown provider/model
                0.0
            }
        }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_cost_deepseek() {
        let cost = AuditLogger::estimate_cost("deepseek", "deepseek-chat", 1000, 2000);
        // (1000/1M * 0.14) + (2000/1M * 0.28) = 0.00014 + 0.00056 = 0.0007
        assert!((cost - 0.0007).abs() < 0.00001);
    }

    #[test]
    fn test_estimate_cost_openai_gpt4() {
        let cost = AuditLogger::estimate_cost("openai", "gpt-4", 1000, 2000);
        // (1000/1M * 30) + (2000/1M * 60) = 0.03 + 0.12 = 0.15
        assert!((cost - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_estimate_cost_ollama() {
        let cost = AuditLogger::estimate_cost("ollama", "llama3", 1000, 2000);
        assert_eq!(cost, 0.0);
    }
}
