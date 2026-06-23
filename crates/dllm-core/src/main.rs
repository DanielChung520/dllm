use clap::{Parser, Subcommand};
use tracing::{info, warn};

mod api;
mod config;
mod engine_pool;
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
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&log_level)),
                )
                .init();

            let hw_sku = dllm_shared::detect_hardware_sku();
            info!("dllm v{} 啟動中", env!("CARGO_PKG_VERSION"));
            info!("硬體: {} (SKU: {:?})", hw_sku.label(), hw_sku);
            info!("端口: {}", port);

            // License 驗證
            let license_path = std::env::var("DLLM_LICENSE_PATH")
                .unwrap_or_else(|_| "/etc/dllm/license.json".to_string());
            match dllm_shared::license::License::from_file(&license_path) {
                Ok(license) => {
                    let status = license.verify("PUBLIC_KEY_PLACEHOLDER");
                    match &status {
                        dllm_shared::license::LicenseStatus::Valid => {
                            info!("✅ License 有效");
                        }
                        dllm_shared::license::LicenseStatus::ExpiringSoon { days_left } => {
                            info!("⚠️ License 將於 {days_left} 天後到期");
                        }
                        _ => {
                            warn!("❌ License 驗證失敗: {}，進入降級模式", status);
                            info!("降級模式：僅允許 API 查詢，模型推理已停用");
                        }
                    }
                }
                Err(e) => {
                    warn!("❌ License 檔案不存在 ({}): {}，進入降級模式", license_path, e);
                    info!("降級模式：僅允許 API 查詢，模型推理已停用");
                }
            }

            let app_config = if let Some(config_path) = config {
                AppConfig::from_file(&config_path)?
            } else {
                AppConfig::default()
            };

            let app = api::create_app(app_config).await?;
            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
            
            info!("🚀 API 伺服器已就緒: http://0.0.0.0:{}", port);
            info!("   健康檢查: http://0.0.0.0:{}/health", port);

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
