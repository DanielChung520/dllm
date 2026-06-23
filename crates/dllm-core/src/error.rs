//! 錯誤處理

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

use dllm_shared::error::ApiError;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = Json(serde_json::json!({
            "error": {
                "message": self.to_string(),
                "type": self.error_code(),
                "code": self.error_code()
            }
        }));
        
        (status, body).into_response()
    }
}
