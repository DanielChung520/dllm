//! 混合雲路由邏輯

use serde::{Deserialize, Serialize};

/// 路由決策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteDecision {
    Local,
    LocalOnly,
    Cloud(String), // 供應商名稱
}

/// 路由引擎
pub struct CloudRouter {
    local_models: Vec<String>,
    budget_limit: Option<f64>,
    budget_used: f64,
}

impl CloudRouter {
    pub fn new() -> Self {
        Self {
            local_models: vec![],
            budget_limit: None,
            budget_used: 0.0,
        }
    }

    pub fn route(&self, _model: &str, _prompt_tokens: usize) -> RouteDecision {
        // TODO: 實現路由邏輯
        RouteDecision::Local
    }
}
