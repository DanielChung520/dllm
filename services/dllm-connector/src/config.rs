use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub dllm_core_url: String,
    pub redis_url: Option<String>,
    pub providers: Vec<CloudProviderConfig>,
}

impl ConnectorConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            dllm_core_url: std::env::var("DLLM_CORE_URL")
                .unwrap_or_else(|_| "http://localhost:11400".to_string()),
            redis_url: std::env::var("REDIS_URL").ok(),
            providers: vec![],
        })
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
