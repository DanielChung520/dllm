//! 模型管理路由

use axum::{
    extract::{Extension, Path},
    response::Json,
};
use std::sync::Arc;

use crate::engine_pool::EnginePool;

/// 列出所有模型
pub async fn list_models(Extension(pool): Extension<Arc<EnginePool>>) -> Json<serde_json::Value> {
    let models = pool.list_models();
    
    Json(serde_json::json!({
        "object": "list",
        "data": models
    }))
}

/// 取得單一模型資訊
pub async fn get_model(
    Path(model_id): Path<String>,
    Extension(pool): Extension<Arc<EnginePool>>,
) -> Json<serde_json::Value> {
    match pool.get_model_status(&model_id) {
        Some(status) => Json(serde_json::json!(status)),
        None => Json(serde_json::json!({
            "error": {
                "message": format!("模型 '{}' 不存在", model_id),
                "type": "not_found",
                "code": "model_not_found"
            }
        })),
    }
}

/// 載入模型
pub async fn load_model(
    Path(model_id): Path<String>,
    Extension(pool): Extension<Arc<EnginePool>>,
) -> Json<serde_json::Value> {
    match pool.load_model(model_id.clone()).await {
        Ok(status) => Json(serde_json::json!(status)),
        Err(e) => Json(serde_json::json!({
            "error": {
                "message": e.to_string(),
                "type": "engine_error",
                "code": "load_failed"
            }
        })),
    }
}

/// 卸載模型
pub async fn unload_model(
    Path(model_id): Path<String>,
    Extension(pool): Extension<Arc<EnginePool>>,
) -> Json<serde_json::Value> {
    match pool.unload_model(&model_id).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": format!("模型 '{}' 已卸載", model_id)
        })),
        Err(e) => Json(serde_json::json!({
            "error": {
                "message": e.to_string(),
                "type": "engine_error",
                "code": "unload_failed"
            }
        })),
    }
}

/// 固定模型
pub async fn pin_model(
    Path(model_id): Path<String>,
    Extension(pool): Extension<Arc<EnginePool>>,
) -> Json<serde_json::Value> {
    match pool.pin_model(&model_id).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": format!("模型 '{}' 已固定", model_id)
        })),
        Err(e) => Json(serde_json::json!({
            "error": {
                "message": e.to_string(),
                "type": "engine_error",
                "code": "pin_failed"
            }
        })),
    }
}

/// 解除固定
pub async fn unpin_model(
    Path(model_id): Path<String>,
    Extension(pool): Extension<Arc<EnginePool>>,
) -> Json<serde_json::Value> {
    match pool.unpin_model(&model_id).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": format!("模型 '{}' 已解除固定", model_id)
        })),
        Err(e) => Json(serde_json::json!({
            "error": {
                "message": e.to_string(),
                "type": "engine_error",
                "code": "unpin_failed"
            }
        })),
    }
}
