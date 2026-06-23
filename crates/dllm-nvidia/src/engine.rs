//! VLLM 推理引擎實現

use async_trait::async_trait;
use futures::stream::BoxStream;
use tokio::sync::Mutex;
use tracing::{info, warn};

use dllm_shared::{
    engine::{EngineConfig, InferenceEngine},
    error::EngineError,
    memory::MemorySnapshot,
    model::ModelInfo,
    types::{ChatChunk, ChatRequest, ChatResponse, HealthStatus},
};

use crate::vllm_client::VLLMClient;
use crate::vllm_process::VLLMProcess;

/// vLLM 子進程推理引擎
pub struct VLLMProcessEngine {
    engine_id: String,
    model_info: ModelInfo,
    process: Mutex<VLLMProcess>,
    client: VLLMClient,
}

impl VLLMProcessEngine {
    pub async fn new(
        engine_id: String,
        model_info: ModelInfo,
        process: VLLMProcess,
    ) -> Result<Self, EngineError> {
        let base_url = process.base_url();
        let client = VLLMClient::new(base_url);
        
        Ok(Self {
            engine_id,
            model_info,
            process: Mutex::new(process),
            client,
        })
    }
}

#[async_trait]
impl InferenceEngine for VLLMProcessEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }

    async fn generate(&self, request: ChatRequest) -> Result<ChatResponse, EngineError> {
        self.client.chat_completion(request).await
    }

    async fn stream_generate(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatChunk, EngineError>>, EngineError> {
        self.client.chat_completion_stream(request).await
    }

    async fn health(&self) -> HealthStatus {
        match self.client.health().await {
            Ok(true) => HealthStatus::healthy(),
            Ok(false) => HealthStatus::degraded("vLLM 未就緒"),
            Err(e) => HealthStatus::unhealthy(format!("健康檢查失敗: {}", e)),
        }
    }

    async fn memory_usage(&self) -> MemorySnapshot {
        MemorySnapshot::default()
    }

    async fn unload(&self) -> Result<(), EngineError> {
        info!("卸載 vLLM 引擎: {}", self.engine_id);
        let mut process = self.process.lock().await;
        process.stop().await?;
        Ok(())
    }
}
