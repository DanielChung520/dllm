//! Mac 記憶體監控

use tracing::warn;

use dllm_shared::memory::SystemMemoryInfo;

/// 透過 macOS API 查詢統一記憶體
pub fn get_unified_memory() -> Option<SystemMemoryInfo> {
    // macOS: sysctl hw.memsize
    match std::process::Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
    {
        Ok(output) => {
            let total_bytes = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<u64>()
                .unwrap_or(0);
            
            let total_mb = (total_bytes / 1024 / 1024) as usize;
            
            // 取得已用記憶體（vm_statistics）
            let used_mb = get_used_memory().unwrap_or(total_mb / 4);
            let available_mb = total_mb.saturating_sub(used_mb);
            
            Some(SystemMemoryInfo {
                total_mb,
                used_mb,
                free_mb: available_mb,
                available_mb,
                buffers_mb: 0,
                cached_mb: 0,
            })
        }
        Err(e) => {
            warn!("無法取得 Mac 記憶體資訊: {}", e);
            None
        }
    }
}

fn get_used_memory() -> Option<usize> {
    // vm_statistics64
    // TODO: 實現更精確的已用記憶體查詢
    None
}
