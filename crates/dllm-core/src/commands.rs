//! 模型管理指令（pull / list / rm）

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const DEFAULT_MODEL_DIR: &str = ".dllm/models";
const INDEX_FILE: &str = ".index.json";

fn models_dir() -> PathBuf {
    std::env::var("DLLM_MODEL_DIR")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/{}", home, DEFAULT_MODEL_DIR)
        })
        .into()
}

fn index_path() -> PathBuf {
    models_dir().join(INDEX_FILE)
}

fn read_index() -> HashMap<String, serde_json::Value> {
    let path = index_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_index(index: &HashMap<String, serde_json::Value>) {
    if let Some(parent) = index_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = serde_json::to_string_pretty(index) {
        let _ = std::fs::write(index_path(), content);
    }
}

fn read_config_json(model_path: &PathBuf) -> Option<serde_json::Value> {
    let config_path = model_path.join("config.json");
    let content = std::fs::read_to_string(config_path).ok()?;
    serde_json::from_str(&content).ok()
}

fn format_size(bytes: u64) -> String {
    if bytes > 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes > 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes > 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

fn dir_size(path: &PathBuf) -> u64 {
    path.read_dir()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.metadata().ok())
                .filter(|m| m.is_file())
                .map(|m| m.len())
                .sum()
        })
        .unwrap_or(0)
}

pub fn pull_model(repo_id: &str, output_path: Option<String>) -> Result<(), String> {
    let dest = if let Some(path) = output_path {
        PathBuf::from(path)
    } else {
        let model_name = repo_id.split('/').last().unwrap_or(repo_id);
        models_dir().join(model_name)
    };

    if dest.exists() {
        let existing_size = format_size(dir_size(&dest));
        // 檢查是否已有 config.json
        if dest.join("config.json").exists() {
            println!("⚠️  模型已存在於 {} ({}), 跳過下載", dest.display(), existing_size);
            return Ok(());
        }
        println!("⚠️  目錄已存在 ({}), 將繼續下載...", existing_size);
    } else {
        std::fs::create_dir_all(&dest).map_err(|e| format!("無法建立目錄: {}", e))?;
    }

    println!("📥 正在下載模型: {}", repo_id);
    println!("   目標: {}", dest.display());

    // 使用 Python hugggingface_hub 下載
    let python_code = format!(
        r#"import sys
try:
    from huggingface_hub import snapshot_download
    path = snapshot_download("{}", local_dir=r"{}", local_dir_use_symlinks=False)
    print("✅ 下載完成:", path)
except Exception as e:
    print("❌ 下載失敗:", str(e), file=sys.stderr)
    sys.exit(1)
"#,
        repo_id,
        dest.display().to_string().replace("\\", "\\\\")
    );

    let python_exe = std::env::var("DLLM_PYTHON").unwrap_or_else(|_| "python3".to_string());

    let status = Command::new(&python_exe)
        .arg("-c")
        .arg(&python_code)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| format!("無法執行 Python: {}", e))?;

    if !status.success() {
        return Err("模型下載失敗，請檢查網路連線與模型名稱".to_string());
    }

    // 顯示模型資訊並寫入 index
    if let Some(config) = read_config_json(&dest) {
        let model_type = config.get("model_type").and_then(|v| v.as_str()).unwrap_or("unknown");
        let ctx_len = config.get("max_position_embeddings")
            .or_else(|| config.get("max_seq_len"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let hidden_size = config.get("hidden_size").and_then(|v| v.as_u64()).unwrap_or(0);
        let num_layers = config.get("num_hidden_layers").and_then(|v| v.as_u64()).unwrap_or(0);
        let size = format_size(dir_size(&dest));

        println!();
        println!("📋 模型資訊:");
        println!("   類型: {}", model_type);
        println!("   大小: {}", size);
        if ctx_len > 0 { println!("   上下文: {} tokens", ctx_len); }
        if hidden_size > 0 { println!("   維度: {}", hidden_size); }
        if num_layers > 0 { println!("   層數: {}", num_layers); }

        // 寫入 metadata index
        let model_name = dest.file_name().and_then(|n| n.to_str()).unwrap_or("model");
        let mut index = read_index();
        index.insert(model_name.to_string(), serde_json::json!({
            "repo_id": repo_id,
            "model_type": model_type,
            "context_length": ctx_len,
            "size_bytes": dir_size(&dest),
            "hidden_size": hidden_size,
            "num_layers": num_layers,
        }));
        write_index(&index);
    }

    Ok(())
}

pub fn list_models() {
    let dir = models_dir();
    if !dir.exists() {
        println!("尚未下載任何模型。使用 `dllm pull <repo_id>` 開始下載。");
        return;
    }

    let mut has_models = false;
    let entries = match dir.read_dir() {
        Ok(e) => e,
        Err(_) => {
            println!("無法讀取模型目錄: {}", dir.display());
            return;
        }
    };

    println!("已下載的模型 (目錄: {}):", dir.display());
    println!();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let model_name = entry.file_name().to_string_lossy().to_string();
        let size = format_size(dir_size(&path));

        if let Some(config) = read_config_json(&path) {
            let model_type = config.get("model_type").and_then(|v| v.as_str()).unwrap_or("?");
            let ctx = config.get("max_position_embeddings")
                .or_else(|| config.get("max_seq_len"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let quantization = config.get("quantization_config")
                .and_then(|q| q.get("quantization_method").or_else(|| q.get("quant_method")))
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            println!("  {:<28}  {:>10}  {} ctx  {} quant", model_name, size, ctx, quantization);
            has_models = true;
        } else {
            println!("  {:<28}  {:>10}  (未知格式)", model_name, size);
            has_models = true;
        }
    }

    if !has_models {
        println!("（模型目錄為空）");
    }
}

pub fn remove_model(model: &str) -> Result<(), String> {
    let dir = models_dir().join(model);
    if !dir.exists() {
        return Err(format!("模型 '{}' 不存在於 {}", model, models_dir().display()));
    }

    let size = format_size(dir_size(&dir));
    println!("🗑️  正在刪除模型: {} ({})", model, size);

    std::fs::remove_dir_all(&dir)
        .map_err(|e| format!("刪除失敗: {}", e))?;

    println!("✅ 已刪除: {}", model);
    Ok(())
}

/// 在終端機直接與模型對話（類似 ollama run）
pub async fn run_model(model: &str, prompt: Option<&str>) -> Result<(), String> {
    // 檢查是否為 Mac 後端（由 oMLX 管理模型，不需本地目錄）
    let is_mac = matches!(DllmConfig::effective_backend(), dllm_shared::engine::GpuBackend::AppleSilicon);
    
    if !is_mac {
        let model_path = models_dir().join(model);
        if !model_path.exists() || !model_path.join("config.json").exists() {
            return Err(format!("模型 '{}' 不存在。請先用 `dllm pull {}` 下載。", model, model));
        }
    }

    let model_api_id = if is_mac { model.to_string() } else {
        models_dir().join(model).to_string_lossy().to_string()
    };

    println!("🚀 正在載入模型: {} ({})", model, model_api_id);
    println!("   輸入 /bye 或 Ctrl+C 結束對話");
    println!("   輸入 /clear 清除對話歷史");
    println!("   輸入 /help 查看更多指令");
    println!();

    let api_base = if let Ok(url) = std::env::var("DLLM_RUN_API") {
        url
    } else {
        "http://localhost:11400/v1".to_string()
    };

    let client = reqwest::Client::new();

    match client.get(&format!("{}/models", api_base)).send().await {
        Ok(_) => {}
        Err(_) => {
            println!("⚠️  無法連接到 API 伺服器 ({})", api_base);
            println!("   請先啟動 `dllm serve` 或設定 DLLM_RUN_API");
            return Err("連線失敗".to_string());
        }
    }

    if let Some(single_prompt) = prompt {
        let body = serde_json::json!({
            "model": model_api_id,
            "messages": [{"role": "user", "content": single_prompt}],
            "stream": false,
            "max_tokens": 2048,
        });

        let resp = client.post(&format!("{}/chat/completions", api_base))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("請求失敗: {}", e))?;

        let reply = resp.json::<serde_json::Value>().await
            .map_err(|e| format!("解析回應失敗: {}", e))?;

        let content = reply["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("（無回應）");

        println!("{}", content);
        return Ok(());
    }

    // 互動模式
    let mut history: Vec<serde_json::Value> = vec![
        serde_json::json!({"role": "system", "content": "你是一個有用的 AI 助手。請用繁體中文回答。"})
    ];
    
    let mut input = String::new();

    loop {
        print!("\n>>> ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();

        input.clear();
        if std::io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim().to_string();
        if input.is_empty() { continue; }

        match input.as_str() {
            "/bye" | "/exit" | "/quit" => {
                println!("再見！");
                break;
            }
            "/clear" => {
                history.truncate(1);
                println!("對話歷史已清除");
                continue;
            }
            "/help" => {
                println!("可用指令:");
                println!("  /bye    結束對話");
                println!("  /clear  清除對話歷史");
                println!("  /help   顯示此說明");
                continue;
            }
            _ => {}
        }

        history.push(serde_json::json!({"role": "user", "content": input}));

        let body = serde_json::json!({
            "model": model_api_id,
            "messages": history,
            "stream": false,
            "max_tokens": 2048,
        });

        let resp = match client.post(&format!("{}/chat/completions", api_base))
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("請求失敗: {}", e);
                history.pop();
                continue;
            }
        };

        let reply = resp.json::<serde_json::Value>().await
            .map_err(|e| format!("解析回應失敗: {}", e))?;

        let content = reply["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("（無回應）");

        println!("\n{}", content);

        history.push(serde_json::json!({"role": "assistant", "content": content}));
    }

    Ok(())
}

/// 檢查 dllm 服務狀態
pub async fn check_status() -> Result<(), String> {
    use reqwest::Client;

    let api = std::env::var("DLLM_API_URL").unwrap_or_else(|_| "http://localhost:11400".to_string());
    let client = Client::new();

    // 健康檢查
    match client.get(&format!("{}/health", api)).send().await {
        Ok(resp) => {
            let status: serde_json::Value = resp.json().await.map_err(|e| format!("解析失敗: {}", e))?;
            println!("📊 dllm 服務狀態");
            println!("{}", serde_json::to_string_pretty(&status).unwrap_or_default());
        }
        Err(_) => {
            println!("❌ dllm 服務未運行 (預期端口: {})", if api.contains(":") { &api } else { "11400" });
            println!("   請執行 `dllm serve` 啟動服務");
        }
    }

    // 檢查 vLLM 後端
    let backend = DllmConfig::effective_backend();
    let backend_title = match backend {
        dllm_shared::engine::GpuBackend::AppleSilicon => "oMLX/MLX 後端",
        _ => "vLLM 後端",
    };
    let backend_url = if matches!(backend, dllm_shared::engine::GpuBackend::AppleSilicon) {
        std::env::var("VLLM_DIRECT_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string())
    } else {
        std::env::var("VLLM_DIRECT_URL").unwrap_or_else(|_| "http://127.0.0.1:18001".to_string())
    };
    match client.get(&format!("{}/v1/models", backend_url)).send().await {
        Ok(resp) => {
            if let Ok(models) = resp.json::<serde_json::Value>().await {
                let count = models["data"].as_array().map(|a| a.len()).unwrap_or(0);
                println!("{}: 運行中 ({} 個模型載入)", backend_title, count);
            }
        }
        Err(_) => println!("{}: 未運行", backend_title),
    }

    // 系統資源
    let mem = std::process::Command::new("free")
        .arg("-h").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    for line in mem.lines().take(2) {
        println!("{}", line);
    }

    Ok(())
}

/// 顯示日誌
pub fn show_log(lines: usize) -> Result<(), String> {
    let log_dirs = vec![
        "/var/log/dllm.log",
        "/tmp/dllm.log",
        "/home/daniel/.dllm/logs/server.log",
    ];

    let log_file = log_dirs.iter().find(|p| std::path::Path::new(p).exists());
    
    match log_file {
        Some(path) => {
            let output = std::process::Command::new("tail")
                .arg("-n").arg(lines.to_string())
                .arg(path)
                .output()
                .map_err(|e| format!("讀取日誌失敗: {}", e))?;
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        None => {
            // 退回到 journalctl
            let output = std::process::Command::new("journalctl")
                .args(["-u", "dllm-core", "--no-pager", "-n", &lines.to_string()])
                .output()
                .map_err(|e| format!("讀取日誌失敗: {}", e))?;
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }

    Ok(())
}

// ==================== 配置系統 ====================

const CONFIG_FILE: &str = ".dllm/config.json";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DllmConfig {
    pub port: Option<u16>,
    pub default_model: Option<String>,
    pub log_dir: Option<String>,
    pub model_dir: Option<String>,
    pub vllm_url: Option<String>,
    pub memory_guard: Option<String>,
    pub api_key: Option<String>,
    /// GPU 後端：auto, nvidia, amd, intel
    pub backend: Option<String>,
}

impl DllmConfig {
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(CONFIG_FILE)
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::config_path())
            .ok().and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(p) = path.parent() { std::fs::create_dir_all(p).map_err(|e| format!("{}", e))?; }
        std::fs::write(&path, serde_json::to_string_pretty(self).map_err(|e| format!("{}", e))?)
            .map_err(|e| format!("{}", e))
    }

    /// 取得有效的 GPU 後端（config 指定 or 自動偵測）
    pub fn effective_backend() -> dllm_shared::engine::GpuBackend {
        let cfg = Self::load();
        match cfg.backend.as_deref() {
            Some("nvidia") => dllm_shared::engine::GpuBackend::NvidiaCuda,
            Some("amd") => dllm_shared::engine::GpuBackend::AmdRocm,
            Some("intel") => dllm_shared::engine::GpuBackend::IntelXpu,
            Some("mac") => dllm_shared::engine::GpuBackend::AppleSilicon,
            _ => dllm_shared::engine::detect_gpu_backend(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "port" => self.port = Some(value.parse().map_err(|_| "port 需為整數")?),
            "default_model" => self.default_model = Some(value.to_string()),
            "log_dir" => self.log_dir = Some(value.to_string()),
            "model_dir" => self.model_dir = Some(value.to_string()),
            "vllm_url" => self.vllm_url = Some(value.to_string()),
            "memory_guard" => self.memory_guard = Some(value.to_string()),
            "api_key" => self.api_key = Some(value.to_string()),
            "backend" => {
                match value {
                    "nvidia" | "cuda" => self.backend = Some("nvidia".to_string()),
                    "amd" | "rocm" => self.backend = Some("amd".to_string()),
                    "intel" | "xpu" => self.backend = Some("intel".to_string()),
                    "mac" | "apple" | "mlx" => self.backend = Some("mac".to_string()),
                    "auto" => self.backend = None,
                    _ => return Err(format!("不支援的後端: {}\n可選: auto, nvidia, amd, intel, mac", value)),
                }
            }
            _ => return Err(format!("未知設定: {}\n可用: port, default_model, log_dir, model_dir, vllm_url, memory_guard, api_key, backend", key)),
        }
        self.save()?;
        println!("✅ {} = {}", key, value);
        Ok(())
    }
}

impl Default for DllmConfig {
    fn default() -> Self {
        Self {
            port: Some(11400),
            default_model: None,
            log_dir: Some("~/.dllm/logs".to_string()),
            model_dir: Some("~/.dllm/models".to_string()),
            vllm_url: Some("http://127.0.0.1:18001".to_string()),
            memory_guard: Some("balanced".to_string()),
            api_key: None,
            backend: None,
        }
    }
}

pub fn handle_config(action: super::ConfigAction) -> Result<(), String> {
    let mut cfg = DllmConfig::load();
    match action {
        super::ConfigAction::Show => {
            // 偵測硬體資訊
            let backend = dllm_shared::engine::detect_gpu_backend();
            let hw = dllm_shared::engine::detect_hardware_sku();
            println!("📋 dllm 配置 ({})", DllmConfig::config_path().display());
            println!("{}", serde_json::to_string_pretty(&cfg).unwrap_or_default());
            println!();
            println!("🔍 偵測到的硬體:");
            println!("   平台: {}", std::env::consts::ARCH);
            println!("   GPU:  {} ({})", backend.label(), hw.label());
            println!("   vLLM: {} 套件", backend.pip_package());
            println!();
            println!("設定方式: dllm config set <key> <value>");
            println!("  可用: port, default_model, log_dir, model_dir, vllm_url, memory_guard, api_key, backend");
            println!("  backend 可設: auto, nvidia, amd, intel, mac");
        }
        super::ConfigAction::Set { key, value } => {
            cfg.set(&key, &value)?;
        }
    }
    Ok(())
}
