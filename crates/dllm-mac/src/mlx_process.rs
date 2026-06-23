//! MLX Python 進程管理

use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{error, info, warn};

use dllm_shared::engine::ModelConfig;
use dllm_shared::error::EngineError;

/// MLX Python 子進程封裝
pub struct MLXProcess {
    model_path: std::path::PathBuf,
    port: u16,
    child: Child,
}

impl MLXProcess {
    /// 啟動 MLX 服務
    pub async fn start(
        model_path: &std::path::Path,
        port: u16,
        config: ModelConfig,
    ) -> Result<Self, EngineError> {
        let actual_port = if port == 0 {
            pick_free_port().await?
        } else {
            port
        };

        let mut cmd = Command::new("python");
        cmd.arg("-m")
            .arg("mlx_lm.server")
            .arg("--model")
            .arg(model_path)
            .arg("--port")
            .arg(actual_port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // 添加上下文長度
        if let Some(ctx_len) = config.context_length {
            cmd.arg("--max-tokens").arg(ctx_len.to_string());
        }

        info!("啟動 MLX: {:?}", cmd);

        let child = cmd.spawn().map_err(|e| EngineError::EngineStartFailed {
            reason: format!("無法啟動 MLX: {}", e),
        })?;

        // 等待服務就緒
        let base_url = format!("http://localhost:{}", actual_port);
        wait_for_ready(&base_url).await?;

        info!("MLX 就緒: {}", base_url);

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

    /// 檢查進程是否存活
    pub async fn is_alive(&self) -> bool {
        // 嘗試取得進程 ID，若失敗則進程已終止
        self.child.id().is_some()
    }

    /// 停止 MLX 服務
    pub async fn stop(&mut self) -> Result<(), EngineError> {
        info!("停止 MLX 進程 (PID: {:?})", self.child.id());

        if let Err(e) = self.child.kill().await {
            warn!("停止 MLX 進程失敗: {}", e);
        }

        match tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.child.wait(),
        ).await
        {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => {
                error!("等待 MLX 進程結束失敗: {}", e);
                Err(EngineError::Internal {
                    reason: format!("停止 MLX 失敗: {}", e),
                })
            }
            Err(_) => Err(EngineError::RequestTimeout),
        }
    }
}

async fn pick_free_port() -> Result<u16, EngineError> {
    for _ in 0..100 {
        let port = rand::random::<u16>() % 30000 + 20000;
        if tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await.is_ok() {
            return Ok(port);
        }
    }
    Err(EngineError::Internal {
        reason: "無法找到可用端口".to_string(),
    })
}

async fn wait_for_ready(base_url: &str) -> Result<(), EngineError> {
    let client = reqwest::Client::new();
    let health_url = format!("{}/health", base_url);

    for _ in 0..60 {
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
        reason: "MLX 啟動超時（60秒）".to_string(),
    })
}
