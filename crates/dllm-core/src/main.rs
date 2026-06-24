use clap::{Parser, Subcommand};
use tracing::{info, warn};

mod api;
mod api_keys;
mod commands;
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
    /// 查看服務狀態
    Status,
    /// 查看最近日誌
    Log {
        /// 行數（預設 50）
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },
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
    /// 從 HuggingFace 下載模型
    Pull {
        /// HuggingFace repo ID（如 Qwen/Qwen2.5-0.5B-Instruct）
        model: String,
        /// 輸出目錄（預設 ~/.dllm/models/{model_name}）
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 列出已下載模型
    List,
    /// 刪除已下載模型
    Rm {
        /// 模型名稱
        model: String,
    },
    /// 管理 API Key
    Key {
        #[command(subcommand)]
        action: KeyAction,
    },
    /// 在終端機直接與模型對話（類似 ollama run）
    Run {
        /// 模型名稱（目錄名或 repo_id）
        model: String,
        /// 可選的單次提示詞（若有則不進入互動模式）
        prompt: Option<String>,
    },
}

#[derive(clap::Subcommand)]
enum KeyAction {
    /// 建立新的 API Key
    Create {
        /// Key 名稱/標籤
        label: String,
    },
    /// 撤銷 API Key
    Revoke {
        /// Key 的 hash 值
        hash: String,
    },
    /// 列出所有 API Key
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            port,
            model_dir: _,
            config,
            memory_guard: _,
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
            let mut cmd = std::process::Command::new("pkill");
            cmd.arg("-f").arg("dllm serve");
            if cfg!(target_os = "macos") {
                cmd.arg("-x");
            }
            match cmd.status() {
                Ok(_) => println!("✅ 已停止 dllm 服務"),
                Err(_) => println!("⚠️  停止指令執行失敗（可能服務未啟動）"),
            }
        }
        Commands::Status => {
            commands::check_status().await.unwrap_or_else(|e| {
                eprintln!("錯誤: {}", e);
            });
        }
        Commands::Log { lines } => {
            commands::show_log(lines).unwrap_or_else(|e| {
                eprintln!("錯誤: {}", e);
            });
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
        }
        Commands::Pull { model, output } => {
            commands::pull_model(&model, output).unwrap_or_else(|e| {
                eprintln!("錯誤: {}", e);
                std::process::exit(1);
            });
        }
        Commands::List => {
            commands::list_models();
        }
        Commands::Rm { model } => {
            commands::remove_model(&model).unwrap_or_else(|e| {
                eprintln!("錯誤: {}", e);
                std::process::exit(1);
            });
        }
        Commands::Run { model, prompt } => {
            commands::run_model(&model, prompt.as_deref()).await.unwrap_or_else(|e| {
                eprintln!("錯誤: {}", e);
                std::process::exit(1);
            });
        }
        Commands::Key { action } => {
            let store = api_keys::ApiKeyStore::new();
            match action {
                KeyAction::Create { label } => {
                    let key = store.create_key(&label);
                    println!("🔑 已建立 API Key:");
                    println!("   Key: {}", key);
                    println!("   標籤: {}", label);
                    println!("⚠️  請立即儲存此 Key，建立後無法再次檢視。");
                }
                KeyAction::Revoke { hash } => {
                    if store.revoke_key(&hash) {
                        println!("✅ 已撤銷 Key: {}", hash);
                    } else {
                        eprintln!("❌ 找不到指定的 Key");
                    }
                }
                KeyAction::List => {
                    let keys = store.list_keys();
                    if keys.is_empty() {
                        println!("尚未建立任何 API Key");
                    } else {
                        println!("API Key 列表:");
                        for (status, entry) in &keys {
                            println!("  [{status}] {}", entry.label);
                            println!("         Hash: {}", &entry.key_hash[..16]);
                            println!("         建立: {}", &entry.created_at[..19]);
                            println!();
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
