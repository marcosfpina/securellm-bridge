// Rate limiting middleware for API routes

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use securellm_core::rate_limit::{RateLimitError, RateLimiter};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, warn};

/// Rate limiting middleware
///
/// Checks rate limits before processing requests.
/// Returns 429 Too Many Requests if rate limit is exceeded.
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Extract provider from path or use default
    let provider = extract_provider(&req);

    debug!("Checking rate limit for provider: {}", provider);

    match limiter.check_limit(&provider).await {
        Ok(_) => {
            debug!("Rate limit check passed for provider: {}", provider);
            Ok(next.run(req).await)
        }
        Err(RateLimitError::Exceeded(provider)) => {
            warn!("Rate limit exceeded for provider: {}", provider);
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "error": {
                        "message": format!("Rate limit exceeded for provider: {}", provider),
                        "type": "rate_limit_exceeded",
                        "code": "rate_limit",
                        "param": provider,
                    }
                })),
            ))
        }
        Err(e) => {
            warn!("Rate limit error: {}", e);
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

/// Extract provider from request
///
/// Tries to extract provider from:
/// 1. Request header X-Provider
/// 2. Path parameter
/// 3. Defaults to "deepseek"
fn extract_provider(req: &Request<Body>) -> String {
    // Try to extract from header first
    if let Some(provider_header) = req.headers().get("X-Provider") {
        if let Ok(provider) = provider_header.to_str() {
            return provider.to_string();
        }
    }

    // Try to extract from path
    // Example: /v1/chat/completions with model "deepseek/deepseek-chat"
    // This is handled in the route handler, so we default here

    // Default to deepseek
    "deepseek".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;

    #[test]
    fn test_extract_provider_from_header() {
        let mut req = Request::builder()
            .uri("/v1/chat/completions")
            .header("X-Provider", "openai")
            .body(Body::empty())
            .unwrap();

        let provider = extract_provider(&req);
        assert_eq!(provider, "openai");
    }

    #[test]
    fn test_extract_provider_default() {
        let req = Request::builder()
            .uri("/v1/chat/completions")
            .body(Body::empty())
            .unwrap();

        let provider = extract_provider(&req);
        assert_eq!(provider, "deepseek");
    }
}
