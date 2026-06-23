//! 嵌入路由

use axum::{
    extract::Extension,
    response::Json,
};
use std::sync::Arc;

use crate::engine_pool::EnginePool;

/// 建立嵌入
pub async fn create_embeddings(
    Extension(pool): Extension<Arc<EnginePool>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    // TODO: 實現嵌入生成
    Json(serde_json::json!({
        "object": "list",
        "data": [],
        "model": body.get("model").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "usage": {
            "prompt_tokens": 0,
            "total_tokens": 0
        }
    }))
}
