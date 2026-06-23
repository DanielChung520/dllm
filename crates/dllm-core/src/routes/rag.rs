//! RAG 路由

use axum::{
    extract::{Extension, Path},
    response::Json,
};
use std::sync::Arc;

use crate::engine_pool::EnginePool;

/// 建立知識庫
pub async fn create_kb(
    Extension(pool): Extension<Arc<EnginePool>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    // TODO: 實現知識庫建立
    Json(serde_json::json!({
        "id": format!("kb-{}", uuid::Uuid::new_v4()),
        "status": "created",
        "name": body.get("name").and_then(|v| v.as_str()).unwrap_or("未命名")
    }))
}

/// 上傳文件
pub async fn upload_document(
    Path(kb_id): Path<String>,
    Extension(pool): Extension<Arc<EnginePool>>,
) -> Json<serde_json::Value> {
    // TODO: 實現文件上傳與處理
    Json(serde_json::json!({
        "id": format!("doc-{}", uuid::Uuid::new_v4()),
        "kb_id": kb_id,
        "status": "processing"
    }))
}

/// RAG 查詢
pub async fn query(
    Extension(pool): Extension<Arc<EnginePool>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    // TODO: 實現 RAG 查詢
    Json(serde_json::json!({
        "answer": "（RAG 查詢尚未實現）",
        "sources": [],
        "usage": {
            "retrieval_tokens": 0,
            "generation_tokens": 0,
            "total_tokens": 0
        }
    }))
}
