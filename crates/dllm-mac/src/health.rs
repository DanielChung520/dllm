//! Mac 健康檢查

use dllm_shared::types::HealthStatus;

/// Mac 系統健康檢查
pub fn check_mac_health() -> HealthStatus {
    // 檢查是否為 Apple Silicon
    match std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
    {
        Ok(output) => {
            let brand = String::from_utf8_lossy(&output.stdout);
            if brand.contains("Apple") {
                HealthStatus::healthy()
            } else {
                HealthStatus::degraded("非 Apple Silicon 晶片，MLX 效能可能受限")
            }
        }
        Err(e) => HealthStatus::unhealthy(format!("無法檢查 CPU: {}", e)),
    }
}
