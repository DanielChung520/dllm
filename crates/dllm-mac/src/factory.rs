//! MLX 引擎工廠

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tracing::info;

use dllm_shared::{
    engine::{EngineConfig, EngineFactory, InferenceEngine, ModelConfig},
    error::EngineError,
    model::ModelInfo,
};

use crate::engine::MLXProcessEngine;
use crate::mlx_process::MLXProcess;

/// MLX 引擎工廠
pub struct MLXEngineFactory;

impl MLXEngineFactory {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EngineFactory for MLXEngineFactory {
    fn supports(&self, _model_path: &Path, config: &ModelConfig) -> bool {
        // MLX 支援 MLX 格式的模型
        config.model_type == "llm" || config.model_type == "vlm"
    }

    async fn create(
        &self,
        model_id: String,
        model_path: PathBuf,
        config: EngineConfig,
    ) -> Result<Box<dyn InferenceEngine>, EngineError> {
        info!("建立 MLX 引擎: {}", model_id);

        let port = config.port.unwrap_or(0);
        let model_info = ModelInfo::new(&model_id, dllm_shared::model::ModelType::Llm);

        let process = MLXProcess::start(
            &model_path,
            port,
            config.model_config.clone(),
        ).await?;

        let engine = MLXProcessEngine::new(
            model_id.clone(),
            model_info,
            process,
        ).await?;

        Ok(Box::new(engine))
    }

    fn estimate_memory(&self, model_path: &Path, _config: &ModelConfig) -> usize {
        // 從 MLX 格式估算
        // MLX 使用統一記憶體，估算方式與 NVIDIA 不同
        
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

        4096 // 預設 4GB
    }
}
