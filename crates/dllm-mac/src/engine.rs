//! MLX 推理引擎實現

use async_trait::async_trait;
use futures::stream::BoxStream;
use tracing::{info, warn};

use dllm_shared::{
    engine::{EngineConfig, InferenceEngine},
    error::EngineError,
    memory::MemorySnapshot,
    model::ModelInfo,
    types::{ChatChunk, ChatRequest, ChatResponse, HealthStatus},
};

use crate::mlx_process::MLXProcess;

/// MLX Python 子進程推理引擎
pub struct MLXProcessEngine {
    engine_id: String,
    model_info: ModelInfo,
    process: MLXProcess,
}

impl MLXProcessEngine {
    pub async fn new(
        engine_id: String,
        model_info: ModelInfo,
        process: MLXProcess,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            engine_id,
            model_info,
            process,
        })
    }
}

#[async_trait]
impl InferenceEngine for MLXProcessEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }

    async fn generate(&self, request: ChatRequest) -> Result<ChatResponse, EngineError> {
        // TODO: 實現 MLX 生成
        Ok(ChatResponse {
            id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: request.model,
            choices: vec![dllm_shared::types::ChatChoice {
                index: 0,
                message: dllm_shared::types::ChatMessage {
                    role: "assistant".to_string(),
                    content: Some("（MLX 生成尚未實現）".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: dllm_shared::types::Usage::default(),
        })
    }

    async fn stream_generate(
        &self,
        _request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatChunk, EngineError>>, EngineError> {
        // TODO: 實現 MLX 串流生成
        let stream = futures::stream::empty();
        Ok(Box::pin(stream))
    }

    async fn health(&self) -> HealthStatus {
        match self.process.is_alive().await {
            true => HealthStatus::healthy(),
            false => HealthStatus::unhealthy("MLX 進程已終止"),
        }
    }

    async fn memory_usage(&self) -> MemorySnapshot {
        // TODO: 透過 macOS API 查詢統一記憶體使用
        MemorySnapshot::default()
    }

    async fn unload(&self) -> Result<(), EngineError> {
        info!("卸載 MLX 引擎: {}", self.engine_id);
        // TODO: 實現 MLX 進程停止
        Ok(())
    }
}
