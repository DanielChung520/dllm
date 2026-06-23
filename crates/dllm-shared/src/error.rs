//! 錯誤處理

use thiserror::Error;

/// 引擎層級錯誤
#[derive(Error, Debug, Clone)]
pub enum EngineError {
    #[error("模型未找到: {model_id}")]
    ModelNotFound { model_id: String },

    #[error("模型已載入: {model_id}")]
    ModelAlreadyLoaded { model_id: String },

    #[error("記憶體不足: 需要 {required_mb}MB, 可用 {available_mb}MB")]
    InsufficientMemory { required_mb: usize, available_mb: usize },

    #[error("引擎啟動失敗: {reason}")]
    EngineStartFailed { reason: String },

    #[error("引擎通訊失敗: {reason}")]
    CommunicationFailed { reason: String },

    #[error("請求超時")]
    RequestTimeout,

    #[error("生成被取消")]
    GenerationCancelled,

    #[error("無效請求: {reason}")]
    InvalidRequest { reason: String },

    #[error("內部錯誤: {reason}")]
    Internal { reason: String },
}

/// API 層級錯誤
#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("未授權")]
    Unauthorized,

    #[error("禁止訪問")]
    Forbidden,

    #[error("資源未找到: {resource}")]
    NotFound { resource: String },

    #[error("請求過頻")]
    RateLimited,

    #[error("服務過載")]
    ServiceUnavailable,

    #[error("引擎錯誤: {0}")]
    Engine(#[from] EngineError),

    #[error("序列化錯誤: {0}")]
    Serialization(String),

    #[error("內部伺服器錯誤")]
    Internal,
}

impl ApiError {
    pub fn status_code(&self) -> u16 {
        match self {
            ApiError::Unauthorized => 401,
            ApiError::Forbidden => 403,
            ApiError::NotFound { .. } => 404,
            ApiError::RateLimited => 429,
            ApiError::ServiceUnavailable => 503,
            ApiError::Engine(e) => match e {
                EngineError::ModelNotFound { .. } => 404,
                EngineError::ModelAlreadyLoaded { .. } => 409,
                EngineError::InsufficientMemory { .. } => 503,
                EngineError::RequestTimeout => 504,
                EngineError::InvalidRequest { .. } => 400,
                _ => 500,
            },
            ApiError::Serialization(_) => 400,
            ApiError::Internal => 500,
        }
    }

    pub fn error_code(&self) -> String {
        match self {
            ApiError::Unauthorized => "unauthorized".to_string(),
            ApiError::Forbidden => "forbidden".to_string(),
            ApiError::NotFound { .. } => "not_found".to_string(),
            ApiError::RateLimited => "rate_limit_exceeded".to_string(),
            ApiError::ServiceUnavailable => "service_unavailable".to_string(),
            ApiError::Engine(e) => match e {
                EngineError::ModelNotFound { .. } => "model_not_found".to_string(),
                EngineError::ModelAlreadyLoaded { .. } => "model_already_loaded".to_string(),
                EngineError::InsufficientMemory { .. } => "insufficient_memory".to_string(),
                EngineError::EngineStartFailed { .. } => "engine_start_failed".to_string(),
                EngineError::RequestTimeout => "request_timeout".to_string(),
                EngineError::InvalidRequest { .. } => "invalid_request".to_string(),
                _ => "internal_error".to_string(),
            },
            ApiError::Serialization(_) => "serialization_error".to_string(),
            ApiError::Internal => "internal_error".to_string(),
        }
    }
}
