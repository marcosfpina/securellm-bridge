// Enterprise-grade audit logging module
// Provides comprehensive audit trail for compliance (GDPR, SOC2, HIPAA)

use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Pluggable sink for audit logs (SQLite, GCP, Kafka, etc)
#[async_trait]
pub trait AuditSink: Send + Sync {
    async fn persist(&self, event: &AuditEvent) -> Result<()>;
}

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

/// Audit logger with async-safe structured logging and pluggable persistence
#[derive(Clone)]
pub struct AuditLogger {
    sink: Option<Arc<dyn AuditSink>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self { sink: None }
    }

    /// Create with a persistence sink
    pub fn with_sink(sink: Arc<dyn AuditSink>) -> Self {
        Self { sink: Some(sink) }
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

        if let Some(sink) = &self.sink {
            if let Err(e) = sink.persist(event).await {
                warn!("Failed to persist audit event (Response): {}", e);
            }
        }
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

        // Also persist failure events
        if let Some(sink) = &self.sink {
            let event = AuditEvent {
                timestamp: Utc::now(),
                request_id,
                event_type: AuditEventType::RequestFailed,
                user_id: None,
                provider: provider.to_string(),
                model: "unknown".to_string(),
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
                estimated_cost_usd: 0.0,
                duration_ms,
                status: RequestStatus::Failed,
                error_message: Some(error.to_string()),
                client_ip: None,
            };
            if let Err(e) = sink.persist(&event).await {
                warn!("Failed to persist audit event (Failure): {}", e);
            }
        }
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

    // Cost estimation helper
    pub fn estimate_cost(
        provider: &str,
        model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> f64 {
       // ... logic handled by PricingRegistry now, but kept for legacy/test ...
       0.0 
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}