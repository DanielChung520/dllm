# CLI 指令參考

## 服務管理

### `dllm serve`

啟動 API 伺服器。

```bash
dllm serve                          # 預設 Port 11400
dllm serve --port 11400             # 指定端口
dllm serve --model-dir ~/models     # 指定模型目錄
dllm serve --log-level debug        # 除錯模式
```

### `dllm stop`

停止執行中的 dllm 服務。

```bash
dllm stop
```

### `dllm status`

查看服務狀態、GPU 資訊、系統資源。

```bash
dllm status
```

輸出範例：
```
📊 dllm 服務狀態
{"status":"healthy","timestamp":"...","version":"0.1.0-alpha"}
vLLM 後端: 運行中 (1 個模型載入)
```

### `dllm log`

查看最近日誌。

```bash
dllm log             # 預設 50 行
dllm log --lines 100 # 顯示 100 行
```

## 模型管理

### `dllm pull <repo_id>`

從 HuggingFace 下載模型。

```bash
dllm pull Qwen/Qwen3-Coder-30B-A3B-Instruct
dllm pull BAAI/bge-m3
dllm pull Qwen/Qwen2.5-VL-7B-Instruct
```

下載完成後自動顯示模型資訊（類型、大小、上下文長度）。

### `dllm list`

列出已下載的模型。

```bash
dllm list
```

輸出範例：
```
已下載的模型 (目錄: ~/.dllm/models):

  Qwen3-Coder-30B-A3B-Instruct  57.0 GB  8192 ctx
  Qwen2.5-VL-7B-Instruct        15.0 GB  32768 ctx
  Qwen3-0.6B                     1.5 GB  40960 ctx
```

### `dllm rm <model>`

刪除已下載的模型。

```bash
dllm rm Qwen3-0.6B
```

## 對話

### `dllm run <model> [prompt]`

在終端機直接與模型對話（類似 `ollama run`）。

```bash
# 互動模式
dllm run Qwen3-Coder-30B-A3B-Instruct

# 單次問答
dllm run Qwen3-Coder-30B-A3B-Instruct "你好，請自我介紹"

# 使用預設模型（需 config set default_model）
dllm run
```

互動模式下支援的指令：
- `/bye` — 結束對話
- `/clear` — 清除對話歷史
- `/help` — 顯示說明

## 配置管理

### `dllm config show`

檢視目前配置。

```bash
dllm config show
```

### `dllm config set <key> <value>`

設定配置值。

```bash
dllm config set port 11400
dllm config set default_model Qwen3-Coder-30B-A3B-Instruct
dllm config set backend auto
dllm config set log_dir ~/.dllm/logs
dllm config set vllm_url http://127.0.0.1:18001
dllm config set api_key sk-xxx
```

可用配置項：

| 名稱 | 預設值 | 說明 |
|------|--------|------|
| `port` | 11400 | 服務器端口 |
| `default_model` | — | `dllm run` 不指定模型時使用 |
| `backend` | auto | GPU 後端：auto / nvidia / amd / intel |
| `log_dir` | ~/.dllm/logs | 日誌目錄 |
| `model_dir` | ~/.dllm/models | 模型儲存目錄 |
| `vllm_url` | http://127.0.0.1:18001 | vLLM 後端 URL |
| `memory_guard` | balanced | 記憶體守衛：safe / balanced / aggressive |
| `api_key` | — | API 密鑰 |

## API Key 管理

### `dllm key create <label>`

建立新的 API Key。

```bash
dllm key create "測試金鑰"
# 輸出: 🔑 Key: dllm-xxxx-xxxx-xxxx
# ⚠️  請立即儲存此 Key，建立後無法再次檢視。
```

### `dllm key revoke <hash>`

撤銷 API Key。

```bash
dllm key revoke 82e841c4176576d4
```

### `dllm key list`

列出所有 API Key。

```bash
dllm key list
```

## 其他

### `dllm load <model>`

手動載入模型到記憶體。

```bash
dllm load Qwen3-Coder-30B-A3B-Instruct
```

### `dllm unload <model>`

手動卸載模型。

```bash
dllm unload Qwen3-0.6B
```

### `dllm models`

列出可用模型（API 端點）。

### `dllm diagnose`

系統診斷。

### `dllm help`

顯示所有指令。
