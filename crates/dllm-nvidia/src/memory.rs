//! NVIDIA 記憶體監控

use tracing::warn;

use dllm_shared::memory::SystemMemoryInfo;

/// 透過 NVML 查詢 GPU 記憶體
pub fn get_gpu_memory() -> Option<SystemMemoryInfo> {
    // 先嘗試用 nvidia-smi 查詢（更可靠）
    if let Some(info) = get_gpu_memory_smi() {
        return Some(info);
    }
    
    // 退回到 NVML（若有安裝）
    get_gpu_memory_nvml().ok().flatten()
}

fn get_gpu_memory_smi() -> Option<SystemMemoryInfo> {
    let output = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=memory.total,memory.used,memory.free", "--format=csv,noheader"])
        .output().ok()?;
    
    let stdout = String::from_utf8(output.stdout).ok()?;
    let parts: Vec<&str> = stdout.trim().split(", ").collect();
    if parts.len() < 3 { return None; }
    
    let parse_mb = |s: &str| {
        s.trim()
            .trim_end_matches(" MiB")
            .parse::<usize>()
            .ok()
    };
    
    let total_mb = parse_mb(parts[0])?;
    let used_mb = parse_mb(parts[1])?;
    let free_mb = parse_mb(parts[2])?;
    
    Some(SystemMemoryInfo {
        total_mb,
        used_mb,
        free_mb,
        available_mb: free_mb,
        buffers_mb: 0,
        cached_mb: 0,
    })
}

fn get_gpu_memory_nvml() -> Result<Option<SystemMemoryInfo>, String> {
    // 嘗試動態載入 NVML
    #[cfg(feature = "nvml")]
    {
        match nvml_wrapper::Nvml::init() {
            Ok(nvml) => {
                let device = nvml.device_by_index(0).map_err(|e| format!("NVML device: {}", e))?;
                let memory = device.memory_info().map_err(|e| format!("NVML memory: {}", e))?;
                let total_mb = (memory.total / 1024 / 1024) as usize;
                let used_mb = (memory.used / 1024 / 1024) as usize;
                let free_mb = (memory.free / 1024 / 1024) as usize;
                Ok(Some(SystemMemoryInfo { total_mb, used_mb, free_mb, available_mb: free_mb, buffers_mb: 0, cached_mb: 0 }))
            }
            Err(e) => {
                warn!("NVML 初始化失敗: {:?}", e);
                Ok(None)
            }
        }
    }
    
    #[cfg(not(feature = "nvml"))]
    {
        let _ = warn;
        Ok(None)
    }
}
