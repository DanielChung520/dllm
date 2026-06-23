use axum::{
    routing::{get, post},
    Router,
};
use tracing::{info, warn};

mod cloud;
mod config;
mod router;

use crate::config::ConnectorConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("dllm-connector 啟動中");

    let config = ConnectorConfig::from_env()?;
    let app = create_app(config).await?;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    info!("🚀 雲端連接器已就緒: http://0.0.0.0:8000");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn create_app(config: ConnectorConfig) -> anyhow::Result<Router> {
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/v1/chat/completions", post(cloud::chat_completions))
        .route("/v1/providers", get(cloud::list_providers));

    Ok(app)
}

async fn health_handler() -> &'static str {
    "healthy"
}
