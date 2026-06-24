//! 模型管理指令（pull / list / rm）

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

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
    let models_dir = models_dir();
    let model_path = models_dir.join(model);
    
    if !model_path.exists() || !model_path.join("config.json").exists() {
        return Err(format!("模型 '{}' 不存在。請先用 `dllm pull {}` 下載。", model, model));
    }

    // 從 index 或 config 取得模型資訊
    let index = read_index();
    let repo_id = index.get(model)
        .and_then(|m| m.get("repo_id"))
        .and_then(|v| v.as_str())
        .unwrap_or(model);

    println!("🚀 正在載入模型: {} ({})", model, repo_id);
    println!("   輸入 /bye 或 Ctrl+C 結束對話");
    println!("   輸入 /clear 清除對話歷史");
    println!("   輸入 /help 查看更多指令");
    println!();

    // 先測試 API 是否可用（連接到本機 dllm-core 或 vLLM）
    let api_base = if let Ok(url) = std::env::var("DLLM_RUN_API") {
        url
    } else {
        "http://localhost:11400/v1".to_string()
    };

    let client = reqwest::Client::new();

    // 測試連線
    match client.get(&format!("{}/models", api_base)).send().await {
        Ok(_) => {}
        Err(_) => {
            println!("⚠️  無法連接到 API 伺服器 ({})", api_base);
            println!("   請先啟動 `dllm serve` 或設定 DLLM_RUN_API");
            return Err("連線失敗".to_string());
        }
    }
    
    // 取得 API 用的 model id
    let model_api_id = model_path.to_string_lossy();

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
