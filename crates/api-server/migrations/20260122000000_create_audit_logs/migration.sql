-- Add migration script here
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY NOT NULL, -- UUID
    request_id TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    event_type TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    prompt_tokens INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    estimated_cost_usd REAL NOT NULL,
    duration_ms INTEGER NOT NULL,
    status TEXT NOT NULL,
    error_message TEXT,
    client_ip TEXT,
    metadata JSON -- Flexible field for extra data
);

CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_audit_request_id ON audit_logs(request_id);
CREATE INDEX idx_audit_provider ON audit_logs(provider);
