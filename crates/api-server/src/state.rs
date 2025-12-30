use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::sync::Arc;
use tokio::sync::RwLock;
use deadpool_redis::{Config as RedisConfig, Pool as RedisPool, Runtime};

use crate::config::Config;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db_pool: SqlitePool,
    pub redis_pool: RedisPool,  // Changed from Arc<redis::Client> to RedisPool
    pub provider_manager: Arc<ProviderManager>,
    pub metrics: Arc<MetricsCollector>,
    pub audit_logger: securellm_core::audit::AuditLogger,
    pub rate_limiter: Arc<securellm_core::rate_limit::RateLimiter>,
}

impl AppState {
    /// Create new application state
    pub async fn new(config: Config) -> Result<Arc<Self>> {
        // Initialize database pool
        let db_pool = SqlitePoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.database.connect_timeout_secs))
            .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout_secs))
            .connect(&config.database.url)
            .await
            .context("Failed to connect to database")?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .context("Failed to run database migrations")?;

        // Initialize Redis connection pool (async)
        let redis_config = RedisConfig::from_url(&config.redis.url);
        let redis_pool = redis_config
            .create_pool(Some(Runtime::Tokio1))
            .context("Failed to create Redis pool")?;

        // Test Redis connection (async)
        {
            let mut conn = redis_pool.get().await
                .context("Failed to get Redis connection from pool")?;

            redis::cmd("PING")
                .query_async::<String>(&mut *conn)
                .await
                .context("Failed to ping Redis")?;

            tracing::info!("✅ Redis connection pool initialized and verified");
        }

        // Initialize provider manager
        let provider_manager = Arc::new(ProviderManager::new(config.clone()).await?);

        // Initialize metrics collector
        let metrics = Arc::new(MetricsCollector::new());

        // Initialize audit logger
        let audit_logger = securellm_core::audit::AuditLogger::new();

        // Initialize rate limiter with default configurations
        let rate_limiter = Arc::new(securellm_core::rate_limit::RateLimiter::default());

        Ok(Arc::new(Self {
            config: Arc::new(config),
            db_pool,
            redis_pool,  // Changed from redis_client
            provider_manager,
            metrics,
            audit_logger,
            rate_limiter,
        }))
    }
}

/// Provider manager handles all LLM providers
pub struct ProviderManager {
    providers: RwLock<Vec<Arc<dyn Provider>>>,
    circuit_breakers: RwLock<std::collections::HashMap<String, CircuitBreaker>>,
}

impl ProviderManager {
    pub async fn new(config: Config) -> Result<Self> {
        let providers: Vec<Arc<dyn Provider>> = Vec::new();

        // Initialize providers based on configuration
        if let Some(deepseek_config) = &config.providers.deepseek {
            if deepseek_config.enabled {
                // TODO: Initialize DeepSeek provider
            }
        }

        if let Some(openai_config) = &config.providers.openai {
            if openai_config.enabled {
                // TODO: Initialize OpenAI provider
            }
        }

        if let Some(anthropic_config) = &config.providers.anthropic {
            if anthropic_config.enabled {
                // TODO: Initialize Anthropic provider
            }
        }

        if let Some(groq_config) = &config.providers.groq {
            if groq_config.enabled {
                // TODO: Initialize Groq provider
            }
        }

        if let Some(cohere_config) = &config.providers.cohere {
            if cohere_config.enabled {
                // TODO: Initialize Cohere provider
            }
        }

        if let Some(llamacpp_config) = &config.providers.llamacpp {
            if llamacpp_config.enabled {
                // TODO: Initialize LlamaCpp provider
            }
        }

        Ok(Self {
            providers: RwLock::new(providers),
            circuit_breakers: RwLock::new(std::collections::HashMap::new()),
        })
    }

    pub async fn list_providers(&self) -> Vec<String> {
        let providers = self.providers.read().await;
        providers.iter().map(|p| p.name().to_string()).collect()
    }

    pub async fn get_provider(&self, name: &str) -> Option<Arc<dyn Provider>> {
        let providers = self.providers.read().await;
        providers.iter().find(|p| p.name() == name).cloned()
    }
}

/// Circuit breaker for provider fault tolerance
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: Option<std::time::Instant>,
    pub config: crate::config::CircuitBreakerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(config: crate::config::CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            config,
        }
    }

    pub fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            _ => {}
        }
    }

    pub fn record_failure(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count += 1;
                self.last_failure_time = Some(std::time::Instant::now());
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
                self.failure_count = 1;
                self.success_count = 0;
                self.last_failure_time = Some(std::time::Instant::now());
            }
            _ => {}
        }
    }

    pub fn can_attempt(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    let timeout = std::time::Duration::from_secs(self.config.timeout_secs);
                    if last_failure.elapsed() >= timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
}

/// Metrics collector for Prometheus
pub struct MetricsCollector {
    // TODO: Add Prometheus metrics
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {}
    }
}

/// Provider trait - will be implemented by each LLM provider
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
}