//! vLLM HTTP 客戶端

use futures::stream::BoxStream;
use reqwest::Client;
use tracing::warn;

use dllm_shared::{
    error::EngineError,
    types::{ChatChunk, ChatRequest, ChatResponse},
};

/// vLLM API 客戶端
pub struct VLLMClient {
    base_url: String,
    client: Client,
}

impl VLLMClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    /// 聊天完成
    pub async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, EngineError> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| EngineError::CommunicationFailed {
                reason: format!("HTTP 請求失敗: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EngineError::CommunicationFailed {
                reason: format!("vLLM 返回錯誤 {}: {}", status, body),
            });
        }

        response
            .json::<ChatResponse>()
            .await
            .map_err(|e| EngineError::CommunicationFailed {
                reason: format!("解析回應失敗: {}", e),
            })
    }

    /// 串流聊天完成
    pub async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatChunk, EngineError>>, EngineError> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        
        let mut request = request;
        request.stream = Some(true);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| EngineError::CommunicationFailed {
                reason: format!("HTTP 請求失敗: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EngineError::CommunicationFailed {
                reason: format!("vLLM 返回錯誤 {}: {}", status, body),
            });
        }

        // TODO: 實現 SSE 串流解析
        let stream = futures::stream::empty();
        Ok(Box::pin(stream))
    }

    /// 健康檢查
    pub async fn health(&self) -> Result<bool, EngineError> {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!("vLLM 健康檢查失敗: {}", e);
                Ok(false)
            }
        }
    }
}
