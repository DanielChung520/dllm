//! 通用類型定義

use serde::{Deserialize, Serialize};

// ==================== Chat API Types ====================

/// 聊天請求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// 聊天訊息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// 聊天回應
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}

/// 聊天選擇
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: usize,
    pub message: ChatMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// 串流聊天區塊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChunkChoice>,
}

/// 串流聊天選擇
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunkChoice {
    pub index: usize,
    pub delta: ChatMessageDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// 訊息增量
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatMessageDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Token 用量
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

// ==================== Tool Types ====================

/// 工具定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

/// 函式定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

/// 工具呼叫選擇
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Auto(String),
    None(String),
    Required(String),
    Named { tool_type: String, function: FunctionChoice },
}

/// 函式選擇
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChoice {
    pub name: String,
}

/// 工具呼叫
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
}

/// 函式呼叫
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

// ==================== Response Format ====================

/// 回應格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,
}

// ==================== Health & System Types ====================

/// 健康狀態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl HealthStatus {
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            message: None,
            details: None,
        }
    }

    pub fn unhealthy(reason: impl Into<String>) -> Self {
        Self {
            status: "unhealthy".to_string(),
            message: Some(reason.into()),
            details: None,
        }
    }

    pub fn degraded(reason: impl Into<String>) -> Self {
        Self {
            status: "degraded".to_string(),
            message: Some(reason.into()),
            details: None,
        }
    }
}

/// API 錯誤回應
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorDetail,
}

/// API 錯誤詳情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dllm: Option<serde_json::Value>,
}

// ==================== RAG Types ====================

/// RAG 查詢請求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQueryRequest {
    pub knowledge_base_ids: Vec<String>,
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default)]
    pub rerank: bool,
    #[serde(default)]
    pub hybrid_search: bool,
    #[serde(default)]
    pub stream: bool,
}

fn default_top_k() -> usize {
    5
}

/// RAG 查詢回應
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQueryResponse {
    pub answer: String,
    pub sources: Vec<RagSource>,
    pub usage: Usage,
}

/// RAG 來源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSource {
    pub document_id: String,
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
    pub chunk_text: String,
    pub score: f32,
}

// ==================== Agent Types ====================

/// Agent 執行請求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,
    #[serde(default)]
    pub stream: bool,
}

fn default_max_iterations() -> usize {
    10
}

/// Agent 執行事件（串流）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    #[serde(rename = "thought")]
    Thought { content: String },
    #[serde(rename = "tool_call")]
    ToolCall { tool: String, input: serde_json::Value },
    #[serde(rename = "tool_result")]
    ToolResult { tool: String, output: serde_json::Value },
    #[serde(rename = "final")]
    Final { content: String },
    #[serde(rename = "error")]
    Error { message: String },
}
