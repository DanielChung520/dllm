//! 應用程式配置

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use dllm_shared::memory::MemoryGuardMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub engine: EnginePoolConfig,
    pub rag: RagConfig,
    pub agent: AgentConfig,
    pub cloud: CloudConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            engine: EnginePoolConfig::default(),
            rag: RagConfig::default(),
            agent: AgentConfig::default(),
            cloud: CloudConfig::default(),
            auth: AuthConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub request_timeout_seconds: u64,
    pub max_request_size_mb: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 11400,
            workers: 4,
            request_timeout_seconds: 300,
            max_request_size_mb: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnginePoolConfig {
    pub model_dirs: Vec<PathBuf>,
    pub pinned_models: Vec<String>,
    pub default_model: Option<String>,
    pub memory_guard: MemoryGuardMode,
    pub ttl_seconds: Option<u64>,
    pub max_concurrent_requests: usize,
    pub preload_on_startup: bool,
}

impl Default for EnginePoolConfig {
    fn default() -> Self {
        Self {
            model_dirs: vec![PathBuf::from("~/.dllm/models")],
            pinned_models: vec![],
            default_model: None,
            memory_guard: MemoryGuardMode::Balanced,
            ttl_seconds: Some(3600),
            max_concurrent_requests: 8,
            preload_on_startup: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    pub qdrant_url: String,
    pub embedding_model: String,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub top_k: usize,
    pub rerank: bool,
    pub hybrid_search: bool,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            qdrant_url: "http://localhost:6333".to_string(),
            embedding_model: "BAAI/bge-m3".to_string(),
            chunk_size: 512,
            chunk_overlap: 128,
            top_k: 5,
            rerank: true,
            hybrid_search: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_iterations: usize,
    pub timeout_seconds: u64,
    pub enabled_tools: Vec<String>,
    pub mcp_servers: Vec<McpServerConfig>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            timeout_seconds: 300,
            enabled_tools: vec![
                "query_database".to_string(),
                "read_file".to_string(),
                "send_email".to_string(),
            ],
            mcp_servers: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub enabled: bool,
    pub providers: Vec<CloudProviderConfig>,
    pub fallback_rules: Vec<String>,
    pub budget_limit_usd: Option<f64>,
    pub privacy_mode: bool,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            providers: vec![],
            fallback_rules: vec![],
            budget_limit_usd: None,
            privacy_mode: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub api_key_required: bool,
    pub admin_api_key: Option<String>,
    pub rate_limit_requests_per_minute: usize,
    pub jwt_secret: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_key_required: true,
            admin_api_key: None,
            rate_limit_requests_per_minute: 60,
            jwt_secret: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: Option<PathBuf>,
    pub max_file_size_mb: usize,
    pub max_files: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            file: None,
            max_file_size_mb: 100,
            max_files: 10,
        }
    }
}
