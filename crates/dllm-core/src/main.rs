use clap::{Parser, Subcommand};
use tracing::{info, warn};

mod api;
mod config;
mod engine_pool;
mod error;
mod memory;
mod middleware;
mod model_discovery;
mod routes;

use crate::config::AppConfig;

#[derive(Parser)]
#[command(name = "dllm")]
#[command(about = "dllm — 跨平台統一 LLM 執行環境")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 啟動 API 伺服器
    Serve {
        /// 監聽端口
        #[arg(short, long, default_value = "11400")]
        port: u16,
        /// 模型目錄
        #[arg(short, long, default_value = "~/.dllm/models")]
        model_dir: String,
        /// 配置文件路徑
        #[arg(short, long)]
        config: Option<String>,
        /// 記憶體守衛模式
        #[arg(long, default_value = "balanced")]
        memory_guard: String,
        /// 日誌級別
        #[arg(long, default_value = "info")]
        log_level: String,
    },
    /// 停止伺服器
    Stop,
    /// 查看狀態
    Status,
    /// 列出可用模型
    Models,
    /// 載入指定模型
    Load {
        #[arg(short, long)]
        model: String,
    },
    /// 卸載指定模型
    Unload {
        #[arg(short, long)]
        model: String,
    },
    /// 診斷系統
    Diagnose,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            port,
            model_dir,
            config,
            memory_guard,
            log_level,
        } => {
            // 初始化日誌
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&log_level)),
                )
                .init();

            info!("dllm v{} 啟動中", env!("CARGO_PKG_VERSION"));
            info!("平台: {}", dllm_shared::detect_platform());
            info!("端口: {}", port);
            info!("模型目錄: {}", model_dir);
            info!("記憶體守衛: {}", memory_guard);

            let config = if let Some(config_path) = config {
                AppConfig::from_file(&config_path)?
            } else {
                AppConfig::default()
            };

            let app = api::create_app(config).await?;
            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
            
            info!("🚀 API 伺服器已就緒: http://0.0.0.0:{}", port);
            info!("   健康檢查: http://0.0.0.0:{}/health", port);
            info!("   API 文件: http://0.0.0.0:{}/docs", port);

            axum::serve(listener, app).await?;
        }
        Commands::Stop => {
            println!("停止伺服器...");
            // TODO: 實現優雅停止
        }
        Commands::Status => {
            println!("檢查伺服器狀態...");
            // TODO: 調用 /health
        }
        Commands::Models => {
            println!("列出可用模型...");
            // TODO: 調用 /v1/models
        }
        Commands::Load { model } => {
            println!("載入模型: {}", model);
            // TODO: 調用載入 API
        }
        Commands::Unload { model } => {
            println!("卸載模型: {}", model);
            // TODO: 調用卸載 API
        }
        Commands::Diagnose => {
            println!("系統診斷...");
            // TODO: 執行診斷
        }
    }

    Ok(())
}
