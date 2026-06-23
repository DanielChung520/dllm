//! 聊天完成路由

use axum::{
    extract::Extension,
    response::Json,
};
use std::sync::Arc;

use dllm_shared::types::ChatRequest;

use crate::engine_pool::EnginePool;

/// 聊天完成
pub async fn chat_completions(
    Extension(pool): Extension<Arc<EnginePool>>,
    Json(request): Json<ChatRequest>,
) -> Json<serde_json::Value> {
    let is_stream = request.stream.unwrap_or(false);
    
    if is_stream {
        return Json(serde_json::json!({
            "error": {
                "message": "串流模式尚未實現",
                "type": "not_implemented",
                "code": "stream_not_implemented"
            }
        }));
    }

    // 開發模式：直接代理到本地 vLLM（當 EnginePool 尚未就緒時）
    if let Ok(vllm_url) = std::env::var("VLLM_DIRECT_URL") {
        return proxy_to_vllm(&vllm_url, request).await;
    }

    match pool.get_engine(&request.model).await {
        Ok(engine) => {
            match engine.generate(request).await {
                Ok(response) => Json(serde_json::to_value(&response).unwrap_or_default()),
                Err(e) => Json(serde_json::json!({
                    "error": {
                        "message": e.to_string(),
                        "type": "engine_error",
                        "code": "generation_failed"
                    }
                })),
            }
        }
        Err(e) => Json(serde_json::json!({
            "error": {
                "message": e.to_string(),
                "type": "engine_error",
                "code": "engine_not_found"
            }
        })),
    }
}

/// 直接代理請求到本地 vLLM（開發模式）
async fn proxy_to_vllm(vllm_url: &str, request: ChatRequest) -> Json<serde_json::Value> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/chat/completions", vllm_url);
    
    match client.post(&url).json(&request).send().await {
        Ok(response) => {
            match response.json::<serde_json::Value>().await {
                Ok(value) => Json(value),
                Err(e) => Json(serde_json::json!({
                    "error": {"message": format!("vLLM 回應解析失敗: {}", e), "type": "proxy_error", "code": "parse_failed"}
                })),
            }
        }
        Err(e) => Json(serde_json::json!({
            "error": {"message": format!("vLLM 連線失敗: {}", e), "type": "proxy_error", "code": "connection_failed"}
        })),
    }
}
