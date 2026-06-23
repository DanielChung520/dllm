//! NVIDIA 健康檢查

use dllm_shared::types::HealthStatus;

/// GPU 健康檢查
pub fn check_gpu_health() -> HealthStatus {
    #[cfg(feature = "nvml")]
    {
        use nvml_wrapper::NVML;
        
        match NVML::init() {
            Ok(nvml) => {
                match nvml.device_by_index(0) {
                    Ok(device) => {
                        let mut issues = vec![];
                        
                        // 檢查溫度
                        if let Ok(temp) = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
                            if temp > 85 {
                                issues.push(format!("GPU 溫度過高: {}°C", temp));
                            }
                        }
                        
                        // 檢查利用率
                        if let Ok(util) = device.utilization_rates() {
                            if util.gpu > 95 {
                                issues.push(format!("GPU 利用率過高: {}%", util.gpu));
                            }
                        }
                        
                        if issues.is_empty() {
                            HealthStatus::healthy()
                        } else {
                            HealthStatus::degraded(issues.join(", "))
                        }
                    }
                    Err(e) => HealthStatus::unhealthy(format!("無法取得 GPU 設備: {}", e)),
                }
            }
            Err(e) => HealthStatus::unhealthy(format!("NVML 初始化失敗: {}", e)),
        }
    }
    
    #[cfg(not(feature = "nvml"))]
    {
        HealthStatus::healthy()
    }
}
