//! Agent 路由

use axum::{
    extract::Extension,
    response::Json,
};
use std::sync::Arc;

use crate::engine_pool::EnginePool;

/// 執行 Agent
pub async fn run_agent(
    Extension(pool): Extension<Arc<EnginePool>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    // TODO: 實現 Agent 執行
    Json(serde_json::json!({
        "result": "（Agent 執行尚未實現）",
        "steps": []
    }))
}

/// 列出可用工具
pub async fn list_tools(
    Extension(pool): Extension<Arc<EnginePool>>,
) -> Json<serde_json::Value> {
    // TODO: 實現工具列表
    Json(serde_json::json!({
        "tools": [
            {
                "name": "query_database",
                "description": "查詢企業資料庫"
            },
            {
                "name": "read_file",
                "description": "讀取本地文件"
            },
            {
                "name": "send_email",
                "description": "發送電子郵件"
            }
        ],
        "mcp_servers": []
    }))
}
