//! NVIDIA 健康檢查

use dllm_shared::types::HealthStatus;

/// GPU 健康檢查
pub fn check_gpu_health() -> HealthStatus {
    // 檢查 nvidia-smi 是否可用
    match std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,temperature.gpu", "--format=csv,noheader"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let info = String::from_utf8_lossy(&output.stdout);
            if info.contains("ERROR") {
                HealthStatus::degraded("GPU 驅動異常")
            } else {
                HealthStatus::healthy()
            }
        }
        Ok(_) => HealthStatus::unhealthy("nvidia-smi 回傳錯誤"),
        Err(e) => HealthStatus::unhealthy(format!("無法存取 GPU: {}", e)),
    }
}
