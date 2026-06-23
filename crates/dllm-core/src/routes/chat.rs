use axum::{
    extract::Extension,
    response::{
        sse::{Event, Sse},
        IntoResponse, Json, Response,
    },
};
use futures::StreamExt;
use std::sync::Arc;

use dllm_shared::types::ChatRequest;

use crate::engine_pool::EnginePool;

pub async fn chat_completions(
    Extension(pool): Extension<Arc<EnginePool>>,
    Json(request): Json<ChatRequest>,
) -> Response {
    let is_stream = request.stream.unwrap_or(false);

    if let Ok(vllm_url) = std::env::var("VLLM_DIRECT_URL") {
        return proxy_to_vllm(&vllm_url, request, is_stream).await;
    }

    match pool.get_engine(&request.model).await {
        Ok(engine) => {
            if is_stream {
                match engine.stream_generate(request).await {
                    Ok(stream) => {
                        let mapped = stream.filter_map(|chunk| async move {
                            match chunk {
                                Ok(c) => {
                                    let json = serde_json::to_string(&c).ok()?;
                                    Some(Ok::<_, std::convert::Infallible>(
                                        Event::default().data(json)
                                    ))
                                }
                                Err(_) => None,
                            }
                        });
                        return Sse::new(mapped).into_response();
                    }
                    Err(e) => {
                        return Json(serde_json::json!({
                            "error": {"message": e.to_string(), "type": "engine_error", "code": "stream_failed"}
                        })).into_response();
                    }
                }
            }
            match engine.generate(request).await {
                Ok(response) => {
                    return Json(serde_json::to_value(&response).unwrap_or_default()).into_response();
                }
                Err(e) => {
                    return Json(serde_json::json!({
                        "error": {"message": e.to_string(), "type": "engine_error", "code": "generation_failed"}
                    })).into_response();
                }
            }
        }
        Err(e) => {
            Json(serde_json::json!({
                "error": {"message": e.to_string(), "type": "engine_error", "code": "engine_not_found"}
            })).into_response()
        }
    }
}

async fn proxy_to_vllm(
    vllm_url: &str,
    request: ChatRequest,
    is_stream: bool,
) -> Response {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/chat/completions", vllm_url);

    if is_stream {
        let mut req = request;
        req.stream = Some(true);
        match client.post(&url).json(&req).send().await {
            Ok(resp) if resp.status().is_success() => {
                let stream = resp.bytes_stream().filter_map(|chunk| async move {
                    let bytes = chunk.ok()?;
                    let text = String::from_utf8_lossy(&bytes);
                    // 上游 vLLM 回傳 data: {...}\n\n，移除 data: prefix 後交由 Axum SSE 重新包裝
                    let json = text.lines().find_map(|line| {
                        let trimmed = line.trim();
                        if trimmed.starts_with("data: ") {
                            Some(trimmed.trim_start_matches("data: "))
                        } else {
                            None
                        }
                    })?;
                    if json == "[DONE]" {
                        return None;
                    }
                    Some(Ok::<_, std::convert::Infallible>(
                        Event::default().data(json.to_string())
                    ))
                });
                return Sse::new(stream).into_response();
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Json(serde_json::json!({
                    "error": {"message": format!("vLLM 錯誤 {}: {}", status, body), "type": "proxy_error", "code": "vllm_error"}
                })).into_response();
            }
            Err(e) => {
                return Json(serde_json::json!({
                    "error": {"message": format!("vLLM 連線失敗: {}", e), "type": "proxy_error", "code": "connection_failed"}
                })).into_response();
            }
        }
    }

    match client.post(&url).json(&request).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => Json(json).into_response(),
            Err(e) => Json(serde_json::json!({
                "error": {"message": format!("vLLM 解析失敗: {}", e), "type": "proxy_error", "code": "parse_failed"}
            })).into_response(),
        },
        Err(e) => Json(serde_json::json!({
            "error": {"message": format!("vLLM 連線失敗: {}", e), "type": "proxy_error", "code": "connection_failed"}
        })).into_response(),
    }
}
