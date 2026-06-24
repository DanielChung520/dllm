//! 記憶體管理實現

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use dashmap::DashMap;
use tracing::info;

use dllm_shared::memory::{MemoryEnforcer, MemoryGuardMode, MemorySnapshot, SystemMemoryInfo};

/// 預設記憶體強制執行器
pub struct DefaultMemoryEnforcer {
    guard_mode: MemoryGuardMode,
    engine_usage: DashMap<String, usize>,
    total_memory_mb: AtomicUsize,
    used_memory_mb: AtomicUsize,
}

impl DefaultMemoryEnforcer {
    pub fn new(guard_mode: MemoryGuardMode) -> Self {
        let sys_info = Self::get_system_memory();
        
        Self {
            guard_mode,
            engine_usage: DashMap::new(),
            total_memory_mb: AtomicUsize::new(sys_info.total_mb),
            used_memory_mb: AtomicUsize::new(sys_info.used_mb),
        }
    }

    fn get_system_memory() -> SystemMemoryInfo {
        // TODO: 實現跨平台記憶體查詢
        // Linux: /proc/meminfo
        // macOS: sysctl vm_statistics
        // Windows: GlobalMemoryStatusEx
        SystemMemoryInfo {
            total_mb: 128 * 1024, // 預設 128GB
            used_mb: 32 * 1024,   // 預設 32GB 已用
            free_mb: 96 * 1024,
            available_mb: 96 * 1024,
            buffers_mb: 0,
            cached_mb: 0,
        }
    }

    fn update_system_memory(&self) {
        let sys_info = Self::get_system_memory();
        self.total_memory_mb.store(sys_info.total_mb, Ordering::Relaxed);
        self.used_memory_mb.store(sys_info.used_mb, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl MemoryEnforcer for DefaultMemoryEnforcer {
    async fn start_monitoring(&self) {
        // EnginePool 已啟動監控，此處為空實現
        info!("記憶體監控已由 EnginePool 管理");
    }

    async fn stop_monitoring(&self) {
        // TODO: 實現停止監控
    }

    fn can_load(&self, required_mb: usize) -> bool {
        let snapshot = self.snapshot();
        snapshot.has_enough(required_mb, self.guard_mode)
    }

    fn suggest_eviction(&self, required_mb: usize) -> Vec<String> {
        let snapshot = self.snapshot();
        let available = snapshot.available_mb;
        
        if available >= required_mb {
            return vec![];
        }

        let need_to_free = required_mb.saturating_sub(available);
        let mut freed = 0;
        let mut to_evict = vec![];

        // 按使用量從大到小排序建議卸載
        let mut engines: Vec<(String, usize)> = self.engine_usage
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        
        engines.sort_by(|a, b| b.1.cmp(&a.1));

        for (engine_id, usage_mb) in engines {
            if freed >= need_to_free {
                break;
            }
            to_evict.push(engine_id);
            freed += usage_mb;
        }

        to_evict
    }

    fn snapshot(&self) -> MemorySnapshot {
        let total = self.total_memory_mb.load(Ordering::Relaxed);
        let used = self.used_memory_mb.load(Ordering::Relaxed);
        let engine_total = self.engine_usage.iter().map(|e| *e.value()).sum::<usize>();
        
        let mut engine_usage_mb = HashMap::new();
        for entry in self.engine_usage.iter() {
            engine_usage_mb.insert(entry.key().clone(), *entry.value());
        }

        MemorySnapshot {
            total_mb: total,
            used_mb: used + engine_total,
            available_mb: total.saturating_sub(used + engine_total),
            engine_usage_mb,
        }
    }

    fn register_engine(&self, engine_id: String, usage_mb: usize) {
        self.engine_usage.insert(engine_id, usage_mb);
    }

    fn update_engine(&self, engine_id: &str, usage_mb: usize) {
        self.engine_usage.insert(engine_id.to_string(), usage_mb);
    }

    fn unregister_engine(&self, engine_id: &str) {
        self.engine_usage.remove(engine_id);
    }
}
