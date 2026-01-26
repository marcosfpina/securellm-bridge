use anyhow::Result;
use axum::{
    http::{HeaderValue, Method},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use securellm_api_server::dashboard::{self, AppState};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Starting SecureLLM Dashboard API Server");

    // Initialize application state
    let state = AppState::new().await?;

    // Configure CORS for development
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3001".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .nest("/api", dashboard::router())
        .with_state(state)
        .layer(cors);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    info!("📡 Listening on http://{}", addr);
    info!("📊 Dashboard available at http://localhost:3001");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
