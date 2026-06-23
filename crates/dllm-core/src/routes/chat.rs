//! 聊天完成路由

use axum::{
    extract::Extension,
    response::{sse::Event, Sse},
    Json,
};
use futures::stream::Stream;
use std::sync::Arc;
use std::convert::Infallible;

use dllm_shared::types::ChatRequest;

use crate::engine_pool::EnginePool;

/// 聊天完成
pub async fn chat_completions(
    Extension(pool): Extension<Arc<EnginePool>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<serde_json::Value>, Json<serde_json::Value>> {
    let is_stream = request.stream.unwrap_or(false);
    
    if is_stream {
        // TODO: 實現 SSE 串流
        return Err(Json(serde_json::json!({
            "error": {
                "message": "串流模式尚未實現",
                "type": "not_implemented",
                "code": "stream_not_implemented"
            }
        })));
    }

    // TODO: 取得引擎並執行生成
    let response = serde_json::json!({
        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": request.model,
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "（此為預設回應，實際推理引擎尚未連接）"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        }
    });

    Ok(Json(response))
}
