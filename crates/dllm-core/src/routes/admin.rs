//! 管理路由

use axum::{
    extract::Extension,
    response::Json,
};
use std::sync::Arc;

use crate::engine_pool::EnginePool;

/// 系統狀態
pub async fn system_status(
    Extension(pool): Extension<Arc<EnginePool>>,
) -> Json<serde_json::Value> {
    let snapshot = pool.memory_snapshot();
    
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "platform": dllm_shared::detect_platform().to_string(),
        "memory": {
            "total_mb": snapshot.total_mb,
            "used_mb": snapshot.used_mb,
            "available_mb": snapshot.available_mb
        },
        "models": {
            "loaded": snapshot.engine_usage_mb.len(),
            "total": pool.list_models().len()
        },
        "uptime_seconds": 0
    }))
}

/// 取得設定
pub async fn get_config() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "memory_guard": "balanced",
        "default_model": null,
        "cloud_fallback": false
    }))
}

/// 更新設定
pub async fn update_config(
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    // TODO: 實現設定更新
    Json(serde_json::json!({
        "success": true,
        "updated": body
    }))
}

/// Prometheus 指標
pub async fn metrics() -> String {
    // TODO: 實現 Prometheus 格式指標
    String::from("# 指標尚未實現")
}

/// 服務管理後台
pub async fn serve_admin() -> &'static str {
    // TODO: 實現靜態檔案服務
    "管理後台尚未建構。請使用 API 端點。"
}
