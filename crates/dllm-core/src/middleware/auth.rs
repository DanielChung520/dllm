//! 認證中間件

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// 認證中間件
pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // TODO: 實現 API Key 驗證
    // 暫時允許所有請求
    Ok(next.run(request).await)
}
