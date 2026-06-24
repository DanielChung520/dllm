//! 模型管理指令（pull / list / rm）

use std::path::PathBuf;
use std::process::Command;

const DEFAULT_MODEL_DIR: &str = ".dllm/models";

fn models_dir() -> PathBuf {
    let home = std::env::var("DLLM_MODEL_DIR")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/{}", home, DEFAULT_MODEL_DIR)
        });
    PathBuf::from(home)
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

    // 顯示模型資訊
    if let Some(config) = read_config_json(&dest) {
        let model_type = config.get("model_type").and_then(|v| v.as_str()).unwrap_or("unknown");
        let hidden_size = config.get("hidden_size").and_then(|v| v.as_u64()).unwrap_or(0);
        let num_layers = config.get("num_hidden_layers").and_then(|v| v.as_u64()).unwrap_or(0);
        let vocab_size = config.get("vocab_size").and_then(|v| v.as_u64()).unwrap_or(0);
        let ctx_len = config.get("max_position_embeddings")
            .or_else(|| config.get("max_seq_len"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let size = format_size(dir_size(&dest));
        println!();
        println!("📋 模型資訊:");
        println!("   類型: {}", model_type);
        println!("   大小: {}", size);
        if ctx_len > 0 { println!("   上下文: {} tokens", ctx_len); }
        if hidden_size > 0 { println!("   維度: {}", hidden_size); }
        if num_layers > 0 { println!("   層數: {}", num_layers); }
        if vocab_size > 0 { println!("   詞彙量: {}", vocab_size); }
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
