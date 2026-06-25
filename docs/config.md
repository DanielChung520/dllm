# 配置說明

## 設定檔位置

dllm 設定檔位於 `~/.dllm/config.json`，首次執行 `dllm config set` 時自動建立。

## 完整設定項

| 名稱 | 型態 | 預設值 | 說明 |
|------|------|--------|------|
| `port` | number | 11400 | 服務器監聽端口 |
| `default_model` | string | null | `dllm run` 的預設模型名稱 |
| `backend` | string | null | GPU 後端：null=自動, nvidia, amd, intel |
| `log_dir` | string | ~/.dllm/logs | 日誌輸出目錄 |
| `model_dir` | string | ~/.dllm/models | 模型儲存目錄 |
| `vllm_url` | string | http://127.0.0.1:18001 | vLLM 後端 URL |
| `memory_guard` | string | balanced | 記憶體保護模式 |
| `api_key` | string | null | API 認證密鑰 |

## 環境變數

環境變數優先於設定檔：

| 變數 | 對應配置 | 說明 |
|------|---------|------|
| `DLLM_MODEL_DIR` | model_dir | 模型目錄路徑 |
| `DLLM_CONFIG_PATH` | — | 設定檔路徑 |
| `DLLM_LICENSE_PATH` | — | License 檔案路徑 |
| `DLLM_PYTHON` | — | Python 執行檔路徑 |
| `DLLM_RUN_API` | — | `dllm run` 的 API 端點 |
| `DLLM_API_URL` | — | API 服務端點（status 用） |
| `VLLM_DIRECT_URL` | vllm_url | vLLM 後端 URL |
| `HF_TOKEN` | — | HuggingFace 存取權杖 |
| `RUST_LOG` | — | 日誌級別（debug/info/warn/error） |

優先順序：**CLI 參數 > 環境變數 > 設定檔 > 預設值**

## 範例設定檔

```json
{
  "port": 11400,
  "default_model": "Qwen3-Coder-30B-A3B-Instruct",
  "log_dir": "~/.dllm/logs",
  "model_dir": "~/.dllm/models",
  "vllm_url": "http://127.0.0.1:18001",
  "memory_guard": "balanced",
  "api_key": null,
  "backend": "auto"
}
```
