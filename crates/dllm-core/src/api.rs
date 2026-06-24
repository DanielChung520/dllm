//! API 應用程式建構

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::AppConfig;
use crate::engine_pool::EnginePool;
use crate::middleware::auth::auth_middleware;
use crate::routes;

pub async fn create_app(config: AppConfig) -> anyhow::Result<Router> {
    // 初始化 Engine Pool
    let engine_pool = Arc::new(EnginePool::new(config.engine.clone()).await?);

    // 啟動背景任務（熱載入 + 備援監控）
    engine_pool.clone().start_background_tasks().await;

    // 建立路由
    let app = Router::new()
        // 健康檢查
        .route("/health", get(routes::health::handler))
        // OpenAI-compatible API
        .route("/v1/models", get(routes::models::list_models))
        .route("/v1/models/:model_id", get(routes::models::get_model))
        .route("/v1/models/:model_id/load", post(routes::models::load_model))
        .route("/v1/models/:model_id/unload", post(routes::models::unload_model))
        .route("/v1/models/:model_id/pin", post(routes::models::pin_model))
        .route("/v1/models/:model_id/unpin", post(routes::models::unpin_model))
        .route("/v1/chat/completions", post(routes::chat::chat_completions))
        .route("/v1/embeddings", post(routes::embeddings::create_embeddings))
        // 管理 API
        .route("/v1/system/status", get(routes::admin::system_status))
        .route("/v1/system/config", get(routes::admin::get_config).put(routes::admin::update_config))
        // API 文件
        .route("/docs", get(routes::docs::handler))
        // 全域狀態
        .layer(axum::extract::Extension(engine_pool))
        // 中間件
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(auth_middleware));

    Ok(app)
}
