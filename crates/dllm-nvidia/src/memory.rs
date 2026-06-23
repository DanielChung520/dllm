//! NVIDIA 記憶體監控

use tracing::warn;

use dllm_shared::memory::{MemorySnapshot, SystemMemoryInfo};

/// 透過 NVML 查詢 GPU 記憶體
#[cfg(feature = "nvml")]
pub fn get_gpu_memory() -> Option<SystemMemoryInfo> {
    use nvml_wrapper::NVML;
    
    match NVML::init() {
        Ok(nvml) => {
            match nvml.device_by_index(0) {
                Ok(device) => {
                    match device.memory_info() {
                        Ok(memory) => {
                            let total_mb = (memory.total / 1024 / 1024) as usize;
                            let used_mb = (memory.used / 1024 / 1024) as usize;
                            let free_mb = (memory.free / 1024 / 1024) as usize;
                            
                            Some(SystemMemoryInfo {
                                total_mb,
                                used_mb,
                                free_mb,
                                available_mb: free_mb,
                                buffers_mb: 0,
                                cached_mb: 0,
                            })
                        }
                        Err(e) => {
                            warn!("無法取得 GPU 記憶體資訊: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("無法取得 GPU 設備: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("NVML 初始化失敗: {}", e);
            None
        }
    }
}

#[cfg(not(feature = "nvml"))]
pub fn get_gpu_memory() -> Option<SystemMemoryInfo> {
    None
}
