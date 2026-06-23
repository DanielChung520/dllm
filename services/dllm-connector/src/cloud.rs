use axum::{extract::Json, response::Json as AxumJson};
use serde_json::json;

/// 雲端聊天完成
pub async fn chat_completions(
    AxumJson(body): AxumJson<serde_json::Value>,
) -> AxumJson<serde_json::Value> {
    // TODO: 實現雲端路由邏輯
    AxumJson(json!({
        "id": "chatcmpl-cloud",
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": body.get("model").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "（雲端連接尚未實現）"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        }
    }))
}

/// 列出雲端供應商
pub async fn list_providers() -> AxumJson<serde_json::Value> {
    AxumJson(json!({
        "providers": [
            {
                "name": "openai",
                "type": "openai",
                "enabled": false,
                "priority": 1
            },
            {
                "name": "anthropic",
                "type": "anthropic",
                "enabled": false,
                "priority": 2
            }
        ]
    }))
}
