use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use tracing::{info, warn};

use crate::api_keys::ApiKeyStore;

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    let store = ApiKeyStore::new();
    let has_keys = store.list_keys().len() > 0;

    if has_keys {
        let auth_header = request.headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok());

        match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let key = header.trim_start_matches("Bearer ");
                if store.validate_key(key).is_none() {
                    warn!("無效的 API Key");
                    return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                        "error": {"message": "無效的 API Key", "type": "auth_error", "code": "unauthorized"}
                    }))).into_response();
                }
            }
            _ => {
                return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                    "error": {"message": "請提供 Authorization: Bearer <key>", "type": "auth_error", "code": "missing_auth"}
                }))).into_response();
            }
        }
    }

    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    let status = response.status().as_u16();
    info!("{} {} {}", method, path, status);
    response
}
