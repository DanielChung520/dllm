//! vLLM 引擎工廠

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tracing::info;

use dllm_shared::{
    engine::{EngineConfig, EngineFactory, InferenceEngine, ModelConfig},
    error::EngineError,
    model::ModelInfo,
};

use crate::engine::VLLMProcessEngine;
use crate::vllm_process::VLLMProcess;

/// vLLM 引擎工廠
pub struct VLLMEngineFactory;

impl VLLMEngineFactory {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EngineFactory for VLLMEngineFactory {
    fn supports(&self, _model_path: &Path, config: &ModelConfig) -> bool {
        // vLLM 支援 HuggingFace 格式與 GGUF
        config.model_type == "llm" || config.model_type == "vlm"
    }

    async fn create(
        &self,
        model_id: String,
        model_path: PathBuf,
        config: EngineConfig,
    ) -> Result<Box<dyn InferenceEngine>, EngineError> {
        info!("建立 vLLM 引擎: {}", model_id);

        let port = config.port.unwrap_or(0); // 0 表示自動分配
        let model_info = ModelInfo::new(&model_id, dllm_shared::model::ModelType::Llm);

        let process = VLLMProcess::start(
            &model_path,
            port,
            config.model_config.clone(),
        ).await?;

        let engine = VLLMProcessEngine::new(
            model_id.clone(),
            model_info,
            process,
        ).await?;

        Ok(Box::new(engine))
    }

    fn estimate_memory(&self, model_path: &Path, _config: &ModelConfig) -> usize {
        // 從 safetensors 計算或 config 估算
        // TODO: 實現更精確的估算
        let index_path = model_path.join("model.safetensors.index.json");
        if index_path.exists() {
            // 讀取 index 計算總大小
            if let Ok(content) = std::fs::read_to_string(&index_path) {
                if let Ok(index) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(weight_map) = index.get("weight_map").and_then(|v| v.as_object()) {
                        // 估計參數數量
                        let param_count = weight_map.len();
                        // 假設平均每個張量 ~10MB（粗略估算）
                        return param_count * 10;
                    }
                }
            }
        }

        // 退回到目錄大小
        if let Ok(metadata) = std::fs::metadata(model_path) {
            if metadata.is_dir() {
                let total_size: u64 = walkdir::WalkDir::new(model_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.metadata().ok())
                    .filter(|m| m.is_file())
                    .map(|m| m.len())
                    .sum();
                return (total_size / 1024 / 1024) as usize;
            }
        }

        8192 // 預設 8GB
    }
}
