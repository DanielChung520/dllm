//! vLLM 進程管理

use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{error, info, warn};

use dllm_shared::engine::ModelConfig;
use dllm_shared::error::EngineError;

/// vLLM 子進程封裝
pub struct VLLMProcess {
    model_path: std::path::PathBuf,
    port: u16,
    child: Child,
}

impl VLLMProcess {
    /// 啟動 vLLM 服務
    pub async fn start(
        model_path: &std::path::Path,
        port: u16,
        config: ModelConfig,
    ) -> Result<Self, EngineError> {
        let actual_port = if port == 0 {
            // 自動分配端口
            pick_free_port().await?
        } else {
            port
        };

        let gpu_mem = config
            .gpu_memory_utilization
            .unwrap_or(0.85);
        
        let max_model_len = config
            .context_length
            .unwrap_or(4096);

        let mut cmd = Command::new("python");
        cmd.arg("-m")
            .arg("vllm.entrypoints.openai.api_server")
            .arg("--model")
            .arg(model_path)
            .arg("--port")
            .arg(actual_port.to_string())
            .arg("--gpu-memory-utilization")
            .arg(gpu_mem.to_string())
            .arg("--max-model-len")
            .arg(max_model_len.to_string())
            .arg("--disable-log-requests")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // 添加額外參數
        for (key, value) in &config.extra_args {
            cmd.arg(format!("--{}", key))
                .arg(value.to_string());
        }

        info!("啟動 vLLM: {:?}", cmd);

        let child = cmd.spawn().map_err(|e| EngineError::EngineStartFailed {
            reason: format!("無法啟動 vLLM: {}", e),
        })?;

        // 等待服務就緒
        let base_url = format!("http://localhost:{}", actual_port);
        wait_for_ready(&base_url).await?;

        info!("vLLM 就緒: {}", base_url);

        Ok(Self {
            model_path: model_path.to_path_buf(),
            port: actual_port,
            child,
        })
    }

    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// 停止 vLLM 服務
    pub async fn stop(&mut self) -> Result<(), EngineError> {
        info!("停止 vLLM 進程 (PID: {:?})", self.child.id());

        // 先嘗試 SIGTERM
        if let Err(e) = self.child.kill().await {
            warn!("SIGTERM 失敗: {}", e);
        }

        // 等待進程結束
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.child.wait(),
        ).await
        {
            Ok(Ok(status)) => {
                info!("vLLM 進程已結束: {:?}", status);
                Ok(())
            }
            Ok(Err(e)) => {
                error!("等待 vLLM 進程結束失敗: {}", e);
                Err(EngineError::Internal {
                    reason: format!("停止 vLLM 失敗: {}", e),
                })
            }
            Err(_) => {
                error!("vLLM 進程停止超時");
                Err(EngineError::RequestTimeout)
            }
        }
    }
}

/// 選擇可用端口
async fn pick_free_port() -> Result<u16, EngineError> {
    // 簡易實現：嘗試綁定隨機高端口
    for _ in 0..100 {
        let port = rand::random::<u16>() % 30000 + 20000; // 20000-50000
        if tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await.is_ok() {
            return Ok(port);
        }
    }
    Err(EngineError::Internal {
        reason: "無法找到可用端口".to_string(),
    })
}

/// 等待 vLLM 就緒
async fn wait_for_ready(base_url: &str) -> Result<(), EngineError> {
    let client = reqwest::Client::new();
    let health_url = format!("{}/health", base_url);

    for _attempt in 0..60 {
        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => {
                return Ok(());
            }
            _ => {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }

    Err(EngineError::EngineStartFailed {
        reason: "vLLM 啟動超時（60秒）".to_string(),
    })
}
