//! RAG 路由 — 代理到 dllm-rag 服務

use axum::{
    extract::{Path, Query, Request},
    response::{IntoResponse, Json, Response},
    body::Body,
};
use std::collections::HashMap;

const RAG_SERVICE_URL: &str = "http://localhost:11402";

/// 建立知識庫（代理到 RAG 服務）
pub async fn create_kb(Json(body): Json<serde_json::Value>) -> Response {
    proxy_post("/v1/rag/knowledge-bases", body).await
}

/// 上傳文件
pub async fn upload_document(
    Path(kb_id): Path<String>,
    body: String,
) -> Response {
    let url = format!("{}/v1/rag/knowledge-bases/{}/documents", RAG_SERVICE_URL, kb_id);
    proxy_request("POST", &url, Some(body)).await
}

/// RAG 查詢
pub async fn query(Json(body): Json<serde_json::Value>) -> Response {
    proxy_post("/v1/rag/query", body).await
}

/// 列出知識庫
pub async fn list_kb() -> Response {
    proxy_get("/v1/rag/knowledge-bases").await
}

async fn proxy_post(path: &str, body: serde_json::Value) -> Response {
    let url = format!("{}{}", RAG_SERVICE_URL, path);
    proxy_request("POST", &url, Some(serde_json::to_string(&body).unwrap_or_default())).await
}

async fn proxy_get(path: &str) -> Response {
    let url = format!("{}{}", RAG_SERVICE_URL, path);
    proxy_request("GET", &url, None).await
}

async fn proxy_request(method: &str, url: &str, body: Option<String>) -> Response {
    let client = reqwest::Client::new();
    let req = if method == "POST" {
        let mut r = client.post(url);
        if let Some(b) = body {
            r = r.header("Content-Type", "application/json").body(b);
        }
        r
    } else {
        client.get(url)
    };

    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            let mut response = Json(serde_json::from_str::<serde_json::Value>(&body).unwrap_or_default()).into_response();
            *response.status_mut() = status;
            response
        }
        Err(e) => Json(serde_json::json!({
            "error": {"message": format!("RAG 服務連線失敗: {}", e), "type": "proxy_error", "code": "rag_unreachable"}
        })).into_response(),
    }
}
