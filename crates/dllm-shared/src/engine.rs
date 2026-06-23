//! 推理引擎 trait 與相關類型

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::EngineError;
use crate::memory::MemorySnapshot;
use crate::model::ModelInfo;
use crate::types::{ChatChunk, ChatRequest, ChatResponse, HealthStatus};

/// 推理引擎抽象介面
/// 所有平台後端（vLLM / MLX / Atlas）皆需實現此 trait
#[async_trait]
pub trait InferenceEngine: Send + Sync {
    /// 引擎唯一識別碼
    fn engine_id(&self) -> &str;

    /// 模型資訊
    fn model_info(&self) -> &ModelInfo;

    /// 同步生成（非串流）
    async fn generate(&self, request: ChatRequest) -> Result<ChatResponse, EngineError>;

    /// 串流生成
    async fn stream_generate(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatChunk, EngineError>>, EngineError>;

    /// 健康檢查
    async fn health(&self) -> HealthStatus;

    /// 記憶體用量統計
    async fn memory_usage(&self) -> MemorySnapshot;

    /// 卸載模型（釋放資源）
    async fn unload(&self) -> Result<(), EngineError>;
}

/// 嵌入引擎抽象介面
#[async_trait]
pub trait EmbeddingEngine: Send + Sync {
    /// 引擎唯一識別碼
    fn engine_id(&self) -> &str;

    /// 嵌入文本
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, EngineError>;

    /// 取得嵌入維度
    fn embedding_dim(&self) -> usize;
}

/// 引擎工廠：根據平台與模型類型建立對應引擎
#[async_trait]
pub trait EngineFactory: Send + Sync {
    /// 是否支援此模型
    fn supports(&self, model_path: &Path, config: &ModelConfig) -> bool;

    /// 建立引擎實例
    async fn create(
        &self,
        model_id: String,
        model_path: PathBuf,
        config: EngineConfig,
    ) -> Result<Box<dyn InferenceEngine>, EngineError>;

    /// 預估記憶體用量（MB）
    fn estimate_memory(&self, model_path: &Path, config: &ModelConfig) -> usize;
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_type: String,
    pub model_id: String,
    pub context_length: Option<usize>,
    pub quantization: Option<String>,
    pub tensor_parallel_size: Option<usize>,
    pub gpu_memory_utilization: Option<f64>,
    pub extra_args: HashMap<String, serde_json::Value>,
}

/// 引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub model_config: ModelConfig,
    pub port: Option<u16>,
    pub timeout_seconds: u64,
    pub max_concurrent_requests: usize,
    pub health_check_interval_seconds: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            model_config: ModelConfig {
                model_type: "llm".to_string(),
                model_id: "unknown".to_string(),
                context_length: None,
                quantization: None,
                tensor_parallel_size: None,
                gpu_memory_utilization: None,
                extra_args: HashMap::new(),
            },
            port: None,
            timeout_seconds: 300,
            max_concurrent_requests: 16,
            health_check_interval_seconds: 30,
        }
    }
}

/// 平台類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    MacAppleSilicon,
    NvidiaLinux,
    NvidiaWindows,
    CpuOnly,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::MacAppleSilicon => write!(f, "mac-apple-silicon"),
            Platform::NvidiaLinux => write!(f, "nvidia-linux"),
            Platform::NvidiaWindows => write!(f, "nvidia-windows"),
            Platform::CpuOnly => write!(f, "cpu-only"),
        }
    }
}

/// 硬體 SKU — 決定預設配置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareSku {
    /// Mac Mini M4 Pro 64GB — 標準方案（2-4 用戶）
    MacMini64,
    /// DGX Spark / ASUS / 銘凡 128GB — 升級方案（4-8 用戶）
    DGXSpark128,
    /// 企業級 H100/H800
    EnterpriseH100,
    /// 未知硬體
    Unknown,
}

impl HardwareSku {
    pub fn label(&self) -> &'static str {
        match self {
            HardwareSku::MacMini64 => "Mac Mini M4 Pro 64GB",
            HardwareSku::DGXSpark128 => "DGX Spark / GB-10 128GB",
            HardwareSku::EnterpriseH100 => "H100 / H800 企業級",
            HardwareSku::Unknown => "未知硬體",
        }
    }

    /// 建議的最大並發請求數
    pub fn max_concurrent_requests(&self) -> usize {
        match self {
            HardwareSku::MacMini64 => 4,
            HardwareSku::DGXSpark128 => 8,
            HardwareSku::EnterpriseH100 => 32,
            HardwareSku::Unknown => 2,
        }
    }

    /// 建議的 GPU 記憶體使用率
    pub fn gpu_memory_utilization(&self) -> f64 {
        match self {
            HardwareSku::MacMini64 => 0.65,
            HardwareSku::DGXSpark128 => 0.80,
            HardwareSku::EnterpriseH100 => 0.90,
            HardwareSku::Unknown => 0.50,
        }
    }
}

/// 偵測當前平台（執行時期，非編譯期）
pub fn detect_platform() -> Platform {
    #[cfg(target_os = "macos")]
    {
        if std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Apple"))
            .unwrap_or(false)
        {
            return Platform::MacAppleSilicon;
        }
    }
    
    if std::process::Command::new("nvidia-smi")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Platform::NvidiaLinux;
    }
    
    Platform::CpuOnly
}

/// 偵測硬體 SKU — 結合平台 + 記憶體大小
pub fn detect_hardware_sku() -> HardwareSku {
    let platform = detect_platform();
    
    match platform {
        Platform::MacAppleSilicon => {
            let mem_gb = get_total_memory_gb();
            if mem_gb >= 128 {
                HardwareSku::DGXSpark128
            } else {
                HardwareSku::MacMini64
            }
        }
        Platform::NvidiaLinux | Platform::NvidiaWindows => {
            let mem_gb = get_total_memory_gb();
            if mem_gb >= 160 {
                HardwareSku::EnterpriseH100
            } else if mem_gb >= 96 {
                HardwareSku::DGXSpark128
            } else {
                HardwareSku::MacMini64
            }
        }
        Platform::CpuOnly => HardwareSku::Unknown,
    }
}

/// 取得總記憶體（GB）— 跨平台實現
fn get_total_memory_gb() -> usize {
    #[cfg(target_os = "linux")]
    {
        let content = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<usize>() {
                        return kb / 1024 / 1024;
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
        {
            let bytes = String::from_utf8_lossy(&output.stdout).trim().parse::<u64>().unwrap_or(0);
            return (bytes / 1024 / 1024 / 1024) as usize;
        }
    }
    
    64 // 預設 64GB
}
