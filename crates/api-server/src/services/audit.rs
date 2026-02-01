use async_trait::async_trait;
use securellm_core::audit::{AuditEvent, AuditSink};
use securellm_core::Result;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

pub struct SqliteAuditSink {
    pool: SqlitePool,
}

impl SqliteAuditSink {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditSink for SqliteAuditSink {
    async fn persist(&self, event: &AuditEvent) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let request_id = event.request_id.to_string();
        let event_type = format!("{:?}", event.event_type);
        let status = format!("{:?}", event.status);

        let metadata = serde_json::json!({});

        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, request_id, timestamp, event_type, provider, model,
                prompt_tokens, completion_tokens, total_tokens,
                estimated_cost_usd, duration_ms, status, error_message,
                client_ip, metadata
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(request_id)
        .bind(event.timestamp)
        .bind(event_type)
        .bind(event.provider.clone())
        .bind(event.model.clone())
        .bind(event.prompt_tokens as i64)
        .bind(event.completion_tokens as i64)
        .bind(event.total_tokens as i64)
        .bind(event.estimated_cost_usd)
        .bind(event.duration_ms as i64)
        .bind(status)
        .bind(event.error_message.clone())
        .bind(event.client_ip.clone())
        .bind(metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| securellm_core::Error::Database(e.to_string()))?;

        Ok(())
    }
}
