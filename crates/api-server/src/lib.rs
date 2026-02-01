use anyhow::{Context, Result};
use axum::{
    http::{header, HeaderValue, Method},
    middleware as axum_middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod config;
pub mod dashboard;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod services;
pub mod state;

use config::Config;
use state::AppState;

pub async fn start_server() -> Result<()> {
    // Initialize tracing
    init_tracing()?;

    info!("🚀 SecureLLM API Server starting...");

    // Load configuration
    let config = Config::load()?;
    info!("✅ Configuration loaded");

    // Initialize application state
    let state = AppState::new(config.clone()).await?;
    info!("✅ Application state initialized");

    // Build router with all routes and middleware
    let app = build_router(state.clone());

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("🌐 Server listening on {}", addr);

    // Create server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;

    info!("✅ Server ready to accept connections");

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    info!("👋 Server shutdown complete");

    Ok(())
}

/// Initialize tracing with file appender and daily rotation
fn init_tracing() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,securellm_api_server=debug"));

    // Console logging (stderr) - structured JSON
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .json();

    // File logging with daily rotation
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| {
        // Default to /tmp/securellm for development, /var/log/securellm for production
        if cfg!(debug_assertions) {
            "/tmp/securellm".to_string()
        } else {
            "/var/log/securellm".to_string()
        }
    });

    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create log directory: {}", log_dir))?;

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("audit")
        .filename_suffix("log")
        .max_log_files(90) // 90 days retention
        .build(&log_dir)
        .context("Failed to create log appender")?;

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false) // No ANSI colors in file logs
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    info!(
        "Audit logging initialized: {}/audit-YYYY-MM-DD.log",
        log_dir
    );

    Ok(())
}

/// Build the application router with all routes and middleware
fn build_router(state: Arc<AppState>) -> Router {
    // API v1 routes - protected by rate limiting
    let api_v1_routes = Router::new()
        .route("/models", get(routes::models::list_models))
        .route("/chat/completions", post(routes::chat::chat_completions))
        .route("/completions", post(routes::completions::text_completions))
        .layer(axum_middleware::from_fn_with_state(
            state.rate_limiter.clone(),
            middleware::rate_limit::rate_limit_middleware,
        ));

    // Management routes - no rate limiting for health checks
    let management_routes = Router::new()
        .route("/health", get(routes::health::health_check))
        .route("/ready", get(routes::health::readiness_check))
        .route("/metrics", get(routes::metrics::metrics))
        .route("/models/sync", post(routes::models::sync_models));

    // Build main router
    Router::new()
        .nest("/v1", api_v1_routes)
        .nest("/api", management_routes)
        .route("/", get(root_handler))
        .layer(
            ServiceBuilder::new()
                // Tracing layer for request logging
                .layer(TraceLayer::new_for_http())
                // Compression layer
                .layer(CompressionLayer::new())
                // Timeout layer (30 seconds)
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                // CORS layer
                .layer(
                    CorsLayer::new()
                        .allow_origin("*".parse::<HeaderValue>().unwrap())
                        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                        .allow_headers([
                            header::CONTENT_TYPE,
                            header::AUTHORIZATION,
                            header::ACCEPT,
                        ])
                        .max_age(Duration::from_secs(3600)),
                ),
        )
        .with_state(state)
}

async fn root_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "SecureLLM API Server",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running",
        "documentation": "/docs",
    }))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Received Ctrl+C signal");
        },
        _ = terminate => {
            warn!("Received SIGTERM signal");
        },
    }

    info!("Starting graceful shutdown...");
}
