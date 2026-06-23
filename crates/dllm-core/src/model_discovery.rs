//! 模型發現

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use parking_lot::RwLock;
use tracing::{info, warn};

use dllm_shared::model::{ModelDiscoveryResult, ModelType};

pub struct ModelDiscovery {
    model_dirs: Vec<PathBuf>,
    /// 掃描結果快取（model_id → ModelDiscoveryResult）
    cache: RwLock<HashMap<String, ModelDiscoveryResult>>,
}

impl ModelDiscovery {
    pub fn new(model_dirs: &[PathBuf]) -> Self {
        Self {
            model_dirs: model_dirs.to_vec(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// 依 model_id 查找對應的模型路徑
    pub fn find_model_path(&self, model_id: &str) -> Option<PathBuf> {
        self.cache.read().get(model_id).map(|r| r.model_path.clone())
    }

    /// 掃描所有模型目錄
    pub async fn scan(&self) -> anyhow::Result<Vec<ModelDiscoveryResult>> {
        let mut results = vec![];

        for dir in &self.model_dirs {
            let expanded = Self::expand_tilde(dir);
            if !expanded.exists() {
                warn!("模型目錄不存在: {:?}", expanded);
                continue;
            }

            info!("掃描模型目錄: {:?}", expanded);
            let mut read_dir = tokio::fs::read_dir(&expanded).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let path = entry.path();
                if path.is_dir() && path.join("config.json").exists() {
                    if let Ok(Some(result)) = self.scan_model_dir(&path).await {
                        results.push(result);
                    }
                }
            }
        }

        // 更新快取
        {
            let mut cache = self.cache.write();
            cache.clear();
            for r in &results {
                cache.insert(r.model_id.clone(), r.clone());
            }
        }

        Ok(results)
    }

    /// 掃描單一模型目錄
    async fn scan_model_dir(&self, dir: &Path) -> anyhow::Result<Option<ModelDiscoveryResult>> {
        let config_path = dir.join("config.json");
        
        if !config_path.exists() {
            return Ok(None);
        }

        let config_content = tokio::fs::read_to_string(&config_path).await?;
        let config: serde_json::Value = serde_json::from_str(&config_content)?;

        let model_type = Self::detect_model_type(&config);
        let model_id = Self::extract_model_id(dir, &config);
        let estimated_memory = Self::estimate_memory(dir, &config);

        Ok(Some(ModelDiscoveryResult {
            model_id,
            model_path: dir.to_path_buf(),
            model_type,
            config,
            estimated_memory_mb: estimated_memory,
        }))
    }

    /// 偵測模型類型
    fn detect_model_type(config: &serde_json::Value) -> ModelType {
        let model_type = config
            .get("model_type")
            .and_then(|v| v.as_str())
            .unwrap_or("llm");

        match model_type {
            "llama" | "mistral" | "qwen2" | "gemma" | "phi" | "gpt2" => ModelType::Llm,
            "qwen2_vl" | "llava" | "gemma3" => ModelType::Vlm,
            "bert" | "bge" | "e5" | "gte" => ModelType::Embedding,
            _ => ModelType::Llm,
        }
    }

    /// 提取模型 ID
    fn extract_model_id(dir: &Path, config: &serde_json::Value) -> String {
        // 優先使用目錄名稱
        dir.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                config
                    .get("_name_or_path")
                    .or_else(|| config.get("model_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string()
            })
    }

    /// 估算模型記憶體用量
    fn estimate_memory(dir: &Path, config: &serde_json::Value) -> usize {
        // 優先從 safetensors index 計算
        let index_path = dir.join("model.safetensors.index.json");
        if index_path.exists() {
            // TODO: 讀取 index 計算總大小
        }

        // 退回到 config 中的參數估算
        let hidden_size = config
            .get("hidden_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(4096);
        
        let num_layers = config
            .get("num_hidden_layers")
            .and_then(|v| v.as_u64())
            .unwrap_or(32);
        
        let vocab_size = config
            .get("vocab_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(32000);

        // 粗略估算: ~2 bytes/param * params + vocab
        let params = hidden_size * hidden_size * num_layers * 4; // 簡化估算
        let vocab_params = vocab_size * hidden_size;
        let total_params = params + vocab_params;
        
        // INT4 量化約 0.5 bytes/param
        let estimated_mb = (total_params as f64 * 0.5 / 1024.0 / 1024.0) as usize;
        
        estimated_mb.max(1024) // 最小 1GB
    }

    /// 展開 ~ 為家目錄
    fn expand_tilde(path: &Path) -> PathBuf {
        let path_str = path.to_string_lossy();
        if path_str.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(&path_str[2..]);
            }
        }
        path.to_path_buf()
    }
}
