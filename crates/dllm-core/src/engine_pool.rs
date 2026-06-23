//! Engine Pool — 多模型管理核心

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;
use tokio::sync::Semaphore;
use tracing::{info, warn, error};

use dllm_shared::{
    engine::{EngineConfig, EngineFactory, InferenceEngine},
    error::EngineError,
    memory::{MemoryEnforcer, MemoryGuardMode, MemorySnapshot},
    model::{ModelInfo, ModelLoadStatus, ModelStatus},
};

use crate::config::EnginePoolConfig;
use crate::memory::DefaultMemoryEnforcer;
use crate::model_discovery::ModelDiscovery;

/// 引擎池 — 多模型 LRU 管理
pub struct EnginePool {
    config: EnginePoolConfig,
    /// 已載入的引擎
    engines: DashMap<String, Box<dyn InferenceEngine>>,
    /// 模型狀態
    statuses: DashMap<String, ModelStatus>,
    /// LRU 順序（最近使用在後方）
    lru: RwLock<VecDeque<String>>,
    /// 固定模型集合
    pinned: RwLock<HashSet<String>>,
    /// 記憶體強制執行器
    memory_enforcer: Arc<dyn MemoryEnforcer>,
    /// 並發控制
    semaphore: Semaphore,
    /// 模型發現器
    discovery: ModelDiscovery,
    /// 引擎工廠
    factories: Vec<Box<dyn EngineFactory>>,
}

impl EnginePool {
    pub async fn new(config: EnginePoolConfig) -> anyhow::Result<Self> {
        let memory_enforcer = Arc::new(DefaultMemoryEnforcer::new(config.memory_guard));
        let discovery = ModelDiscovery::new(&config.model_dirs);
        
        let pool = Self {
            config: config.clone(),
            engines: DashMap::new(),
            statuses: DashMap::new(),
            lru: RwLock::new(VecDeque::new()),
            pinned: RwLock::new(HashSet::new()),
            memory_enforcer: memory_enforcer.clone(),
            semaphore: Semaphore::new(config.max_concurrent_requests),
            discovery,
            factories: Self::init_factories(),
        };

        // 啟動記憶體監控
        memory_enforcer.start_monitoring().await;

        // 掃描模型
        pool.discover_models().await?;

        // 載入固定模型
        for model_id in &config.pinned_models {
            if let Err(e) = pool.load_model(model_id.clone()).await {
                warn!("無法載入固定模型 {}: {}", model_id, e);
            }
        }

        info!("Engine Pool 初始化完成");
        Ok(pool)
    }

    /// 初始化平台對應的引擎工廠
    fn init_factories() -> Vec<Box<dyn EngineFactory>> {
        let mut factories: Vec<Box<dyn EngineFactory>> = vec![];

        #[cfg(feature = "nvidia")]
        {
            factories.push(Box::new(dllm_nvidia::VLLMEngineFactory::new()));
        }

        #[cfg(feature = "mac")]
        {
            factories.push(Box::new(dllm_mac::MLXEngineFactory::new()));
        }

        factories
    }

    /// 掃描並註冊可用模型
    pub async fn discover_models(&self) -> anyhow::Result<Vec<ModelInfo>> {
        let results = self.discovery.scan().await?;
        let mut models = vec![];

        for result in results {
            let model_info = ModelInfo::new(&result.model_id, result.model_type)
                .with_memory(result.estimated_memory_mb);
            
            let status = ModelStatus {
                info: model_info.clone(),
                status: ModelLoadStatus::Available,
                load_time_ms: None,
                memory_mb: None,
                pinned: false,
                lru_position: None,
                last_used_at: None,
                request_count: 0,
            };

            self.statuses.insert(result.model_id.clone(), status);
            models.push(model_info);
        }

        info!("發現 {} 個可用模型", models.len());
        Ok(models)
    }

    /// 載入模型
    pub async fn load_model(&self, model_id: String) -> Result<ModelStatus, EngineError> {
        // 檢查是否已載入
        if self.engines.contains_key(&model_id) {
            return Err(EngineError::ModelAlreadyLoaded { model_id });
        }

        // 取得模型資訊
        let model_info = self.statuses
            .get(&model_id)
            .map(|s| s.info.clone())
            .ok_or_else(|| EngineError::ModelNotFound { model_id: model_id.clone() })?;

        let required_mb = model_info.estimated_memory_mb;

        // 檢查記憶體
        if !self.memory_enforcer.can_load(required_mb) {
            // 嘗試 LRU 卸載
            self.evict_if_needed(required_mb).await?;
        }

        // 更新狀態
        if let Some(mut status) = self.statuses.get_mut(&model_id) {
            status.status = ModelLoadStatus::Loading;
        }

        // 建立引擎
        let engine = self.create_engine(&model_id, &model_info).await?;
        
        // 註冊引擎
        self.engines.insert(model_id.clone(), engine);
        self.memory_enforcer.register_engine(model_id.clone(), required_mb);
        
        // 更新 LRU
        {
            let mut lru = self.lru.write();
            if !lru.contains(&model_id) {
                lru.push_back(model_id.clone());
            }
        }

        // 更新狀態
        let now = chrono::Utc::now();
        if let Some(mut status) = self.statuses.get_mut(&model_id) {
            status.status = ModelLoadStatus::Loaded;
            status.load_time_ms = Some(0); // TODO: 計算實際載入時間
            status.memory_mb = Some(required_mb);
            status.last_used_at = Some(now);
        }

        info!("模型 {} 載入完成 ({}MB)", model_id, required_mb);
        
        self.statuses
            .get(&model_id)
            .map(|s| s.clone())
            .ok_or_else(|| EngineError::Internal { reason: "狀態更新失敗".to_string() })
    }

    /// 卸載模型
    pub async fn unload_model(&self, model_id: &str) -> Result<(), EngineError> {
        // 檢查是否為固定模型
        {
            let pinned = self.pinned.read();
            if pinned.contains(model_id) {
                return Err(EngineError::InvalidRequest {
                    reason: format!("模型 {} 已被固定，無法卸載", model_id),
                });
            }
        }

        if let Some((_, engine)) = self.engines.remove(model_id) {
            engine.unload().await?;
            self.memory_enforcer.unregister_engine(model_id);
        }

        // 從 LRU 移除
        {
            let mut lru = self.lru.write();
            lru.retain(|id| id != model_id);
        }

        if let Some(mut status) = self.statuses.get_mut(model_id) {
            status.status = ModelLoadStatus::Available;
            status.memory_mb = None;
            status.lru_position = None;
        }

        info!("模型 {} 已卸載", model_id);
        Ok(())
    }

    /// 取得引擎（自動載入）
    pub async fn get_engine(&self, model_id: &str) -> Result<Box<dyn InferenceEngine>, EngineError> {
        // 檢查是否已載入
        if let Some(engine) = self.engines.get(model_id) {
            // 更新 LRU
            {
                let mut lru = self.lru.write();
                lru.retain(|id| id != model_id);
                lru.push_back(model_id.to_string());
            }
            
            // 更新使用統計
            if let Some(mut status) = self.statuses.get_mut(model_id) {
                status.last_used_at = Some(chrono::Utc::now());
                status.request_count += 1;
            }

            // TODO: 這裡需要解決 DashMap 引用生命週期問題
            // 暫時返回複製或 Arc
            return Err(EngineError::Internal { reason: "引擎引用未實現".to_string() });
        }

        // 自動載入
        self.load_model(model_id.to_string()).await?;
        
        // 重新取得
        self.engines
            .get(model_id)
            .map(|e| {
                // TODO: 複製引擎或改為 Arc
                Err::<Box<dyn InferenceEngine>, EngineError>(
                    EngineError::Internal { reason: "引擎複製未實現".to_string() }
                )
            })
            .unwrap_or_else(|| {
                Err(EngineError::ModelNotFound { model_id: model_id.to_string() })
            })
    }

    /// 固定模型
    pub async fn pin_model(&self, model_id: &str) -> Result<(), EngineError> {
        {
            let mut pinned = self.pinned.write();
            pinned.insert(model_id.to_string());
        }
        
        if let Some(mut status) = self.statuses.get_mut(model_id) {
            status.pinned = true;
        }

        info!("模型 {} 已固定", model_id);
        Ok(())
    }

    /// 解除固定
    pub async fn unpin_model(&self, model_id: &str) -> Result<(), EngineError> {
        {
            let mut pinned = self.pinned.write();
            pinned.remove(model_id);
        }
        
        if let Some(mut status) = self.statuses.get_mut(model_id) {
            status.pinned = false;
        }

        info!("模型 {} 已解除固定", model_id);
        Ok(())
    }

    /// LRU 卸載
    async fn evict_if_needed(&self, required_mb: usize) -> Result<(), EngineError> {
        let suggestions = self.memory_enforcer.suggest_eviction(required_mb);
        
        for model_id in suggestions {
            // 檢查是否為固定模型
            {
                let pinned = self.pinned.read();
                if pinned.contains(&model_id) {
                    continue;
                }
            }

            info!("LRU 卸載模型: {}", model_id);
            self.unload_model(&model_id).await?;
        }

        if !self.memory_enforcer.can_load(required_mb) {
            return Err(EngineError::InsufficientMemory {
                required_mb,
                available_mb: self.memory_enforcer.snapshot().available_mb,
            });
        }

        Ok(())
    }

    /// 建立引擎
    async fn create_engine(
        &self,
        model_id: &str,
        model_info: &ModelInfo,
    ) -> Result<Box<dyn InferenceEngine>, EngineError> {
        // 尋找支援的工廠
        for factory in &self.factories {
            // TODO: 實現工廠匹配邏輯
        }

        Err(EngineError::EngineStartFailed {
            reason: format!("無可用工廠載入模型 {}", model_id),
        })
    }

    /// 列出所有模型狀態
    pub fn list_models(&self) -> Vec<ModelStatus> {
        self.statuses
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 取得單一模型狀態
    pub fn get_model_status(&self, model_id: &str) -> Option<ModelStatus> {
        self.statuses.get(model_id).map(|s| s.clone())
    }

    /// 取得記憶體快照
    pub fn memory_snapshot(&self) -> MemorySnapshot {
        self.memory_enforcer.snapshot()
    }
}
