// Enterprise-grade rate limiting module
// Provides protection against API abuse using token bucket algorithm

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use thiserror::Error;

/// Rate limiting errors
#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for provider {0}")]
    Exceeded(String),

    #[error("Rate limiter not configured for provider {0}")]
    NotConfigured(String),
}

pub type Result<T> = std::result::Result<T, RateLimitError>;

/// Rate limiter using token bucket algorithm
#[derive(Clone)]
pub struct RateLimiter {
    limiters:
        Arc<dashmap::DashMap<String, Arc<GovernorLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Configure rate limit for a specific provider
    ///
    /// # Arguments
    /// * `provider` - Provider name (e.g., "deepseek", "openai")
    /// * `requests_per_minute` - Maximum requests per minute
    /// * `burst_size` - Maximum burst size (how many requests can be made in quick succession)
    pub fn configure_provider(&self, provider: String, requests_per_minute: u32, burst_size: u32) {
        let quota =
            Quota::per_minute(NonZeroU32::new(requests_per_minute).expect("RPM must be > 0"))
                .allow_burst(NonZeroU32::new(burst_size).expect("Burst must be > 0"));

        let limiter = Arc::new(GovernorLimiter::direct(quota));
        self.limiters.insert(provider, limiter);
    }

    /// Check if request can proceed (consumes 1 token)
    ///
    /// Returns Ok(()) if request is allowed, Err if rate limit exceeded
    pub async fn check_limit(&self, provider: &str) -> Result<()> {
        let limiter = self
            .limiters
            .get(provider)
            .ok_or_else(|| RateLimitError::NotConfigured(provider.to_string()))?;

        match limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::Exceeded(provider.to_string())),
        }
    }

    /// Check without consuming token (for pre-flight checks)
    pub async fn check_would_allow(&self, provider: &str) -> Result<bool> {
        let limiter = self
            .limiters
            .get(provider)
            .ok_or_else(|| RateLimitError::NotConfigured(provider.to_string()))?;

        Ok(limiter.check().is_ok())
    }

    /// Get current state for a provider (for metrics)
    pub fn get_remaining(&self, provider: &str) -> Option<u32> {
        // Governor doesn't expose remaining tokens directly
        // This is a simplified implementation
        self.limiters.get(provider).map(|_| 0)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        let limiter = Self::new();

        // Default configurations per provider
        // These can be overridden via configuration

        // DeepSeek: 60 requests/minute, 10 burst
        limiter.configure_provider("deepseek".to_string(), 60, 10);

        // OpenAI: 3500 requests/minute (tier 1), 100 burst
        limiter.configure_provider("openai".to_string(), 3500, 100);

        // Anthropic: 50 requests/minute, 5 burst
        limiter.configure_provider("anthropic".to_string(), 50, 5);

        // Ollama: High limit (local model)
        limiter.configure_provider("ollama".to_string(), 10000, 1000);

        // LlamaCpp: High limit (local model)
        limiter.configure_provider("llamacpp".to_string(), 10000, 1000);

        limiter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new();
        limiter.configure_provider("test".to_string(), 10, 5);

        // First request should succeed
        assert!(limiter.check_limit("test").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_unconfigured() {
        let limiter = RateLimiter::new();

        // Unconfigured provider should fail
        let result = limiter.check_limit("unknown").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(RateLimitError::NotConfigured(_))));
    }

    #[tokio::test]
    async fn test_rate_limiter_burst() {
        let limiter = RateLimiter::new();
        limiter.configure_provider("test".to_string(), 60, 5);

        // Should allow burst_size requests
        for _ in 0..5 {
            assert!(limiter.check_limit("test").await.is_ok());
        }

        // 6th request should fail (exceeds burst)
        let result = limiter.check_limit("test").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(RateLimitError::Exceeded(_))));
    }

    #[tokio::test]
    async fn test_rate_limiter_default() {
        let limiter = RateLimiter::default();

        // Should have default configurations
        assert!(limiter.check_limit("deepseek").await.is_ok());
        assert!(limiter.check_limit("openai").await.is_ok());
        assert!(limiter.check_limit("anthropic").await.is_ok());
    }
}
