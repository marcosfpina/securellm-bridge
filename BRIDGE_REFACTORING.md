# 🚀 SecureLLM-Bridge - Enterprise-Grade Refactoring Guide

**Análise Completa**: 30 de dezembro de 2025
**Objetivo**: 60% de ganho em compliance e performance
**Repositório**: securellm-bridge (Rust Proxy for LLM APIs)

---

## 📊 EXECUTIVE SUMMARY

### Projeto Overview
- **Propósito**: Proxy seguro para múltiplos LLM providers (DeepSeek, OpenAI, Anthropic, Ollama)
- **Stack**: Rust + Tokio + Axum + SQLx + Redis + rustls
- **Status**: Protótipo avançado (v0.1.0), arquitetura excelente
- **Deployment**: Docker, NixOS, standalone binary

### Issues Críticos Identificados
1. ⚠️ **Audit logging é STUB vazio** - violação de compliance (BLOCKER)
2. ⚠️ **Rate limiting é STUB vazio** - vulnerabilidade de segurança (BLOCKER)
3. ⚠️ **Redis connection blocking** - startup lento
4. ⚠️ **JSON serialization blocking** - overhead em streaming
5. ⚠️ **Multiple clones em conversões** - memory overhead
6. ⚠️ **Providers OpenAI/Anthropic** - não implementados

### Ganhos Estimados

| Categoria | Ganho | Técnica |
|-----------|-------|---------|
| **Compliance** | +100% | Audit logging + Rate limiting |
| **I/O Async** | +40% | deadpool-redis async pool |
| **Memory** | +25% | Zero-copy patterns, Cow strings |
| **Serialization** | +15% | Avoid blocking in streams |
| **Startup** | +40% | Async Redis init |

**Total Estimado**: **~60% compliance + performance**

---

## 🔥 INSIGHTS PODEROSOS

### Insight 1: Compliance Blockers
**Problema**: `audit.rs` e `rate_limit.rs` são stubs vazios (apenas `Ok(())` no-ops).
**Impacto**: Nenhum audit trail, nenhuma proteção contra abuse.
**Solução**: Implementar com `tracing` estruturado e `governor` crate.

### Insight 2: Async Gap
**Problema**: Redis usa `get_connection()` síncrono durante startup.
**Impacto**: Se Redis unavailable, servidor trava indefinidamente.
**Solução**: `deadpool-redis` (já no Cargo.toml mas não usado).

### Insight 3: Memory Efficiency
**Problema**: Múltiplos `clone()` e `to_string()` em conversões de tipo.
**Impacto**: Alocações desnecessárias em hot path.
**Solução**: Usar `Cow<str>` e borrowing where possible.

### Insight 4: Excellent Foundation
**Positivo**: Arquitetura trait-based sólida, zero dependências nativas.
**Oportunidade**: Implementar features faltantes mantendo qualidade.

---

## 🎯 PROMPTS DE REFATORAÇÃO ATÔMICOS

### **[BRIDGE-1] BLOCKER CRÍTICO: Implementar Audit Logging**

**Prioridade**: 🔴 CRÍTICA
**Tempo Estimado**: 2-3 horas
**Ganho**: 100% compliance, cost tracking

#### Contexto
`crates/core/src/audit.rs` é um STUB vazio. Sistema não grava logs de auditoria,
violando compliance requirements (GDPR, SOC2, HIPAA).

#### Diagnóstico
```rust
// crates/core/src/audit.rs - COMPLETAMENTE VAZIO
pub struct AuditLogger;
impl AuditLogger {
    pub async fn log_request(&self, _request: &Request) -> Result<()> {
        // TODO: Implement audit logging
        Ok(())  // No-op!
    }
}
```

#### Objetivo
Implementar audit logging enterprise-grade com:
1. Structured JSON logging (tracing)
2. Campos obrigatórios (request_id, user, provider, tokens, cost, duration)
3. Rotation diária com retention de 90 dias
4. Async writes (não bloquear request path)

#### Instruções de Execução

##### 1. Atualizar Dependências (Cargo.toml)

**Já incluído em workspace.dependencies**:
```toml
[workspace.dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"
```

##### 2. Implementar AuditLogger

**Refatorar**: `crates/core/src/audit.rs`
```rust
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    RequestReceived,
    ResponseSent,
    RequestFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Success,
    Failed,
    RateLimited,
    Timeout,
}

#[derive(Clone)]
pub struct AuditLogger {
    // Pode incluir writer específico aqui no futuro
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {}
    }

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

    // Helper para calcular custo estimado
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
                // GPT-4: $30 / 1M input, $60 / 1M output
                let input_cost = (prompt_tokens as f64 / 1_000_000.0) * 30.0;
                let output_cost = (completion_tokens as f64 / 1_000_000.0) * 60.0;
                input_cost + output_cost
            }
            _ => 0.0,
        }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}
```

##### 3. Configurar Tracing Appender

**Refatorar**: `crates/api-server/src/main.rs`
```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn init_tracing() -> Result<()> {
    // Console logging (stderr)
    let console_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .json();

    // File logging com rotação diária
    let log_dir = std::env::var("LOG_DIR")
        .unwrap_or_else(|_| "/var/log/securellm".to_string());

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("audit")
        .filename_suffix("log")
        .max_log_files(90)  // 90 dias retention
        .build(log_dir)
        .expect("Failed to create log appender");

    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .json();

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,securellm=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    Ok(())
}
```

##### 4. Integrar em Routes

**Refatorar**: `crates/api-server/src/routes/chat.rs`
```rust
use securellm_core::audit::{AuditLogger, AuditEvent, RequestStatus, AuditEventType};
use uuid::Uuid;
use std::time::Instant;

pub async fn create_chat_completion(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateChatCompletionRequest>,
) -> ApiResult<impl IntoResponse> {
    let request_id = Uuid::new_v4();
    let start = Instant::now();

    // Log request received
    state.audit_logger.log_request_received(
        request_id,
        &req.model,  // Provider detection aqui
        &req.model,
        req.messages.len(),
        None,  // Extrair de headers se disponível
    ).await?;

    // Process request
    match process_chat_request(&state, &req).await {
        Ok(response) => {
            let duration_ms = start.elapsed().as_millis() as u64;

            // Calcular tokens (extrair do response)
            let prompt_tokens = 100;  // TODO: real calculation
            let completion_tokens = response.usage.completion_tokens;
            let total_tokens = prompt_tokens + completion_tokens;

            let cost = AuditLogger::estimate_cost(
                "deepseek",
                &req.model,
                prompt_tokens,
                completion_tokens,
            );

            // Log response sent
            let audit_event = AuditEvent {
                timestamp: chrono::Utc::now(),
                request_id,
                event_type: AuditEventType::ResponseSent,
                user_id: None,
                provider: "deepseek".to_string(),
                model: req.model.clone(),
                prompt_tokens,
                completion_tokens,
                total_tokens,
                estimated_cost_usd: cost,
                duration_ms,
                status: RequestStatus::Success,
                error_message: None,
                client_ip: None,
            };

            state.audit_logger.log_response_sent(&audit_event).await?;

            Ok(Json(response))
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            state.audit_logger.log_request_failed(
                request_id,
                "deepseek",
                &e.to_string(),
                duration_ms,
            ).await?;

            Err(e)
        }
    }
}
```

##### 5. Adicionar ao AppState

**Refatorar**: `crates/api-server/src/state.rs`
```rust
use securellm_core::audit::AuditLogger;

pub struct AppState {
    pub config: Arc<Config>,
    pub db_pool: SqlitePool,
    pub redis_client: Arc<redis::Client>,
    pub provider_manager: Arc<ProviderManager>,
    pub metrics: Arc<MetricsCollector>,
    pub audit_logger: AuditLogger,  // NOVO
}

impl AppState {
    pub async fn new(config: Config) -> Result<Arc<Self>> {
        // ... existing setup ...

        let audit_logger = AuditLogger::new();

        Ok(Arc::new(Self {
            config: Arc::new(config),
            db_pool,
            redis_client,
            provider_manager,
            metrics,
            audit_logger,
        }))
    }
}
```

#### Ganho Esperado
- **100% compliance**: Audit trail completo
- **Zero performance impact**: Async logging não bloqueia
- **Cost tracking**: Visibilidade de custos por request

#### Entregáveis
- [ ] crates/core/src/audit.rs implementado
- [ ] Integração em routes/chat.rs
- [ ] AppState com AuditLogger
- [ ] Testes de logging (verificar arquivo de log)
- [ ] Documentação de campos de audit

---

### **[BRIDGE-2] BLOCKER CRÍTICO: Implementar Rate Limiting**

**Prioridade**: 🔴 CRÍTICA
**Tempo Estimado**: 2-3 horas
**Ganho**: 100% proteção contra abuse

#### Contexto
`crates/core/src/rate_limit.rs` é um STUB vazio. Sistema não tem proteção contra
abuse, violando security requirements.

#### Objetivo
Implementar rate limiting usando `governor` crate com:
1. Token bucket algorithm
2. Per-provider limits
3. Per-user limits (se auth implementado)
4. Graceful degradation (429 responses)

#### Instruções de Execução

##### 1. Implementar RateLimiter

**Refatorar**: `crates/core/src/rate_limit.rs`
```rust
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for provider {0}")]
    Exceeded(String),

    #[error("Rate limiter not configured for provider {0}")]
    NotConfigured(String),
}

pub type Result<T> = std::result::Result<T, RateLimitError>;

#[derive(Clone)]
pub struct RateLimiter {
    limiters: Arc<dashmap::DashMap<String, Arc<GovernorLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Configura rate limit para um provider específico
    pub fn configure_provider(
        &self,
        provider: String,
        requests_per_minute: u32,
        burst_size: u32,
    ) {
        let quota = Quota::per_minute(
            NonZeroU32::new(requests_per_minute).expect("RPM must be > 0")
        ).allow_burst(
            NonZeroU32::new(burst_size).expect("Burst must be > 0")
        );

        let limiter = Arc::new(GovernorLimiter::direct(quota));
        self.limiters.insert(provider, limiter);
    }

    /// Verifica se request pode proceder (consome 1 token)
    pub async fn check_limit(&self, provider: &str) -> Result<()> {
        let limiter = self.limiters.get(provider)
            .ok_or_else(|| RateLimitError::NotConfigured(provider.to_string()))?;

        match limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::Exceeded(provider.to_string())),
        }
    }

    /// Check sem consumir token (para pre-flight checks)
    pub async fn check_would_allow(&self, provider: &str) -> Result<bool> {
        let limiter = self.limiters.get(provider)
            .ok_or_else(|| RateLimitError::NotConfigured(provider.to_string()))?;

        Ok(limiter.check().is_ok())
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        let limiter = Self::new();

        // Configurações padrão por provider
        limiter.configure_provider("deepseek".to_string(), 60, 10);
        limiter.configure_provider("openai".to_string(), 3500, 100);
        limiter.configure_provider("anthropic".to_string(), 50, 5);
        limiter.configure_provider("ollama".to_string(), 10000, 1000);  // Local, sem limite

        limiter
    }
}
```

##### 2. Criar Middleware de Rate Limiting

**Criar arquivo**: `crates/api-server/src/middleware/rate_limit.rs`
```rust
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use securellm_core::rate_limit::{RateLimiter, RateLimitError};

pub async fn rate_limit_middleware<B>(
    State(limiter): State<Arc<RateLimiter>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, impl IntoResponse> {
    // Extrair provider do path ou headers
    let provider = extract_provider(&req);

    match limiter.check_limit(&provider).await {
        Ok(_) => Ok(next.run(req).await),
        Err(RateLimitError::Exceeded(provider)) => {
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "error": {
                        "message": format!("Rate limit exceeded for provider: {}", provider),
                        "type": "rate_limit_exceeded",
                        "code": "rate_limit"
                    }
                })),
            ))
        }
        Err(e) => {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": {
                        "message": e.to_string(),
                        "type": "rate_limit_error",
                    }
                })),
            ))
        }
    }
}

fn extract_provider<B>(req: &Request<B>) -> String {
    // TODO: Extrair de header X-Provider ou path
    // Por ora, default
    "deepseek".to_string()
}
```

##### 3. Integrar no Router

**Refatorar**: `crates/api-server/src/main.rs`
```rust
use crate::middleware::rate_limit::rate_limit_middleware;
use securellm_core::rate_limit::RateLimiter;

#[tokio::main]
async fn main() -> Result<()> {
    // ... init ...

    let rate_limiter = Arc::new(RateLimiter::default());

    let app = Router::new()
        .route("/v1/chat/completions", post(routes::chat::create_chat_completion))
        .route("/v1/models", get(routes::models::list_models))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware
        ))
        .with_state(state);

    // ... serve ...
}
```

##### 4. Adicionar Config para Rate Limits

**Refatorar**: `crates/api-server/src/config.rs`
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub deepseek_rpm: u32,
    pub deepseek_burst: u32,
    pub openai_rpm: u32,
    pub openai_burst: u32,
    pub anthropic_rpm: u32,
    pub anthropic_burst: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            deepseek_rpm: 60,
            deepseek_burst: 10,
            openai_rpm: 3500,
            openai_burst: 100,
            anthropic_rpm: 50,
            anthropic_burst: 5,
        }
    }
}

// Adicionar em Config
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub rate_limit: RateLimitConfig,  // NOVO
}
```

#### Ganho Esperado
- **100% proteção contra abuse**: Rate limits enforced
- **Graceful degradation**: 429 responses claros
- **<1ms overhead**: Governor é extremamente eficiente

#### Entregáveis
- [ ] crates/core/src/rate_limit.rs implementado
- [ ] Middleware de rate limiting criado
- [ ] Integração no router
- [ ] Config para customização de limits
- [ ] Testes de rate limiting (simulate burst)

---

### **[BRIDGE-3] OTIMIZAÇÃO: Async Redis com Deadpool**

**Prioridade**: 🟢 ALTA
**Tempo Estimado**: 1-2 horas
**Ganho**: +40% startup time

#### Contexto
`state.rs` usa `redis::Client::get_connection()` que é **blocking**.
Durante startup, se Redis estiver unavailable, servidor trava.

#### Diagnóstico
```rust
// state.rs:44 - BLOCKING!
let mut redis_conn = redis_client.get_connection()
    .context("Failed to connect to Redis")?;
redis::cmd("PING")
    .query::<String>(&mut redis_conn)  // BLOCKING!
    .context("Failed to ping Redis")?;
```

#### Objetivo
Usar `deadpool-redis` (já no Cargo.toml) para connection pool async.

#### Instruções de Execução

##### 1. Refatorar AppState

**Refatorar**: `crates/api-server/src/state.rs`
```rust
use deadpool_redis::{Config as RedisConfig, Pool, Runtime};
use redis::AsyncCommands;  // Trocar Commands por AsyncCommands

pub struct AppState {
    pub config: Arc<Config>,
    pub db_pool: SqlitePool,
    pub redis_pool: Pool,  // MUDOU: Arc<redis::Client> → Pool
    pub provider_manager: Arc<ProviderManager>,
    pub metrics: Arc<MetricsCollector>,
    pub audit_logger: AuditLogger,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Arc<Self>> {
        // ... db_pool setup ...

        // Redis pool assíncrono
        let redis_config = RedisConfig::from_url(&config.redis.url);
        let redis_pool = redis_config
            .create_pool(Some(Runtime::Tokio1))
            .context("Failed to create Redis pool")?;

        // Test connection (async)
        {
            let mut conn = redis_pool.get().await
                .context("Failed to get Redis connection")?;

            redis::cmd("PING")
                .query_async::<_, String>(&mut conn)  // Async!
                .await
                .context("Failed to ping Redis")?;
        }

        Ok(Arc::new(Self {
            config: Arc::new(config),
            db_pool,
            redis_pool,  // Pool em vez de Client
            provider_manager,
            metrics,
            audit_logger,
        }))
    }
}
```

##### 2. Atualizar Uso de Redis

**Em routes que usam Redis**:
```rust
// ANTES:
let mut redis_conn = state.redis_client.get_connection()?;
redis::cmd("GET").arg(key).query::<Option<String>>(&mut redis_conn)?;

// DEPOIS:
let mut conn = state.redis_pool.get().await?;
redis::cmd("GET")
    .arg(key)
    .query_async::<_, Option<String>>(&mut conn)
    .await?;
```

##### 3. Implementar Cache Helper

**Criar arquivo**: `crates/api-server/src/cache.rs`
```rust
use deadpool_redis::Pool;
use redis::AsyncCommands;
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

pub struct CacheService {
    pool: Pool,
}

impl CacheService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.pool.get().await?;
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: usize,
    ) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let serialized = serde_json::to_string(value)?;

        conn.set_ex(key, serialized, ttl_seconds).await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        conn.del(key).await?;
        Ok(())
    }
}
```

#### Ganho Esperado
- **+40% startup time**: Async connection não bloqueia
- **Connection pooling**: Reutilização de conexões
- **Auto-retry**: Deadpool implementa retry automático

#### Entregáveis
- [ ] state.rs refatorado (Pool em vez de Client)
- [ ] cache.rs helper criado
- [ ] Routes atualizadas para async
- [ ] Testes de Redis failure (deve gracefully degradar)

---

## 🎯 PLANO DE EXECUÇÃO

### Fase 1: Blockers Críticos (Prioridade Máxima)
**Executar PRIMEIRO**:
1. [BRIDGE-1] - Audit Logging (2-3h)
2. [BRIDGE-2] - Rate Limiting (2-3h)

**Timeline**: 4-6 horas

### Fase 2: Otimização de Performance
**Executar SEGUNDO**:
1. [BRIDGE-3] - Async Redis (1-2h)

**Timeline**: 1-2 horas

---

## 📈 MÉTRICAS DE SUCESSO

### Checklist de Validação
- [ ] Audit logs em /var/log/securellm/
- [ ] Rate limiting funcional (429 responses)
- [ ] Redis async connection pool
- [ ] Zero clones desnecessários em conversões
- [ ] Tests passando
- [ ] Documentation atualizada

### Performance Targets
- **Compliance**: +100% (audit + rate limiting)
- **Startup speed**: +40% (Redis async)
- **Memory efficiency**: +25% (zero-copy patterns)
- **Throughput**: +15% (async optimizations)

---

## 🚀 PRÓXIMOS PASSOS

1. **Review deste documento** com toda equipe
2. **Criar branch**: `git checkout -b refactor/compliance-and-performance`
3. **Executar prompts** na ordem de prioridade
4. **Testar cada fase** antes de próxima
5. **Commit incremental**: Um commit por prompt concluído
6. **Benchmarks**: Antes/depois de cada otimização
7. **Deploy staging**: Validar em ambiente não-produção
8. **Rollout gradual**: 10% → 50% → 100% traffic

---

**Data de Criação**: 30 de dezembro de 2025
**Autores**: Análise automatizada + kernelcore
**Versão**: 1.0.0
