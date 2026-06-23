//! 模型相關類型

use serde::{Deserialize, Serialize};

/// 模型類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// 大型語言模型
    Llm,
    /// 視覺語言模型
    Vlm,
    /// 嵌入模型
    Embedding,
    /// 重排序模型
    Reranker,
    /// 語音轉文字
    AudioStt,
    /// 文字轉語音
    AudioTts,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Llm => write!(f, "llm"),
            ModelType::Vlm => write!(f, "vlm"),
            ModelType::Embedding => write!(f, "embedding"),
            ModelType::Reranker => write!(f, "reranker"),
            ModelType::AudioStt => write!(f, "audio_stt"),
            ModelType::AudioTts => write!(f, "audio_tts"),
        }
    }
}

/// 模型基本資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
    pub model_type: ModelType,
    pub max_context_length: usize,
    pub quantization: Option<String>,
    pub estimated_memory_mb: usize,
    pub capabilities: Vec<String>,
}

impl ModelInfo {
    pub fn new(id: impl Into<String>, model_type: ModelType) -> Self {
        let id = id.into();
        Self {
            object: "model".to_string(),
            created: chrono::Utc::now().timestamp(),
            owned_by: "dllm-local".to_string(),
            max_context_length: 4096,
            quantization: None,
            estimated_memory_mb: 0,
            capabilities: vec!["chat".to_string()],
            id,
            model_type,
        }
    }

    pub fn with_quantization(mut self, q: impl Into<String>) -> Self {
        self.quantization = Some(q.into());
        self
    }

    pub fn with_memory(mut self, mb: usize) -> Self {
        self.estimated_memory_mb = mb;
        self
    }

    pub fn with_capabilities(mut self, caps: Vec<String>) -> Self {
        self.capabilities = caps;
        self
    }
}

/// 模型發現結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDiscoveryResult {
    pub model_id: String,
    pub model_path: std::path::PathBuf,
    pub model_type: ModelType,
    pub config: serde_json::Value,
    pub estimated_memory_mb: usize,
}

/// 模型載入狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelLoadStatus {
    Available,
    Loading,
    Loaded,
    Unloading,
    Error,
}

/// 模型狀態（含執行時資訊）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub info: ModelInfo,
    pub status: ModelLoadStatus,
    pub load_time_ms: Option<u64>,
    pub memory_mb: Option<usize>,
    pub pinned: bool,
    pub lru_position: Option<usize>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub request_count: u64,
}
