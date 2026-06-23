//! 記憶體管理類型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 記憶體守衛模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MemoryGuardMode {
    /// 保守模式：保留較多系統記憶體
    Safe,
    /// 平衡模式：預設
    Balanced,
    /// 積極模式：允許 AI 使用更多記憶體
    Aggressive,
    /// 自訂上限
    Custom { max_gb: f64 },
}

impl Default for MemoryGuardMode {
    fn default() -> Self {
        MemoryGuardMode::Balanced
    }
}

impl MemoryGuardMode {
    /// 取得記憶體使用上限（佔總記憶體比例）
    pub fn usage_ratio(&self) -> f64 {
        match self {
            MemoryGuardMode::Safe => 0.6,
            MemoryGuardMode::Balanced => 0.75,
            MemoryGuardMode::Aggressive => 0.9,
            MemoryGuardMode::Custom { max_gb } => {
                // 自訂模式需要知道總記憶體，此處返回 1.0 由外部計算
                let _ = max_gb;
                1.0
            }
        }
    }
}

/// 記憶體快照
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub total_mb: usize,
    pub used_mb: usize,
    pub available_mb: usize,
    pub engine_usage_mb: HashMap<String, usize>,
}

impl MemorySnapshot {
    pub fn new(total_mb: usize, used_mb: usize) -> Self {
        Self {
            total_mb,
            used_mb,
            available_mb: total_mb.saturating_sub(used_mb),
            engine_usage_mb: HashMap::new(),
        }
    }

    /// 取得引擎總使用量
    pub fn engine_total_mb(&self) -> usize {
        self.engine_usage_mb.values().sum()
    }

    /// 計算載入新模型後的預估可用量
    pub fn projected_available(&self, required_mb: usize) -> usize {
        self.available_mb.saturating_sub(required_mb)
    }

    /// 檢查是否有足夠記憶體
    pub fn has_enough(&self, required_mb: usize, guard_mode: MemoryGuardMode) -> bool {
        let max_allowed = (self.total_mb as f64 * guard_mode.usage_ratio()) as usize;
        let current_used = self.used_mb + self.engine_total_mb();
        current_used + required_mb <= max_allowed
    }
}

/// 記憶體強制執行器 trait
#[async_trait::async_trait]
pub trait MemoryEnforcer: Send + Sync {
    /// 啟動背景監控
    async fn start_monitoring(&self);
    
    /// 停止背景監控
    async fn stop_monitoring(&self);
    
    /// 檢查是否有足夠記憶體載入新模型
    fn can_load(&self, required_mb: usize) -> bool;
    
    /// 建議應卸載的模型（按 LRU）
    fn suggest_eviction(&self, required_mb: usize) -> Vec<String>;
    
    /// 取得當前記憶體狀態
    fn snapshot(&self) -> MemorySnapshot;
    
    /// 註冊引擎記憶體用量
    fn register_engine(&self, engine_id: String, usage_mb: usize);
    
    /// 更新引擎記憶體用量
    fn update_engine(&self, engine_id: &str, usage_mb: usize);
    
    /// 移除引擎記憶體用量
    fn unregister_engine(&self, engine_id: &str);
}

/// 系統記憶體資訊
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SystemMemoryInfo {
    pub total_mb: usize,
    pub used_mb: usize,
    pub free_mb: usize,
    pub available_mb: usize,
    pub buffers_mb: usize,
    pub cached_mb: usize,
}
