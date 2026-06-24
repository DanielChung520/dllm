# dllm: 一個為 DGX Spark 打造的 OpenAI-compatible LLM 執行環境

> 像 oMLX 之於 Mac，dllm 之於 NVIDIA GB-10。

## 前言

如果你有一台 DGX Spark（或任何 GB-10 核心設備），你可能已經遇到這個問題：

- **vLLM** 很強，但沒有模型管理（沒有 `ollama pull` 這種東西）
- **Ollama** 有模型管理，但底層是 llama.cpp，沒有發揮 GB-10 的 CUDA 效能
- **oMLX** 很棒，但只支援 Mac / Apple Silicon
- 你的 Mac 用 oMLX（OpenAI API），Spark 用 Ollama（自訂格式），**兩套 API 不一致**

dllm 就是為了解決這個問題而寫的。

## dllm 是什麼？

dllm（Distributed Local LLM Manager）是一個輕量的 LLM 執行環境，基於 **vLLM** 但加上完整的模型管理層。你可以把它想成「NVIDIA 版的 oMLX」。

```
dllm = vLLM（高效推理） + Ollama 風格的模型管理 + 多模型載入策略
```

### 核心功能

- **OpenAI-compatible API**（Port 11400）— `/v1/chat/completions`、`/v1/models`
- **`dllm pull` / `dllm list` / `dllm rm`** — 跟 `ollama pull` 一樣直覺
- **多模型同時載入** — 常駐 (pinned)、熱載入 (hot)、備援 (standby) 策略
- **硬體自動感知** — 64GB 設備自動保守配置，128GB 自動最佳化
- **API Key 管理 + 審計日誌** — 企業級安全
- **SSE Streaming** — 標準 OpenAI SSE 格式
- **全部用 Rust 寫成** — 單一二進位檔 3.1MB，無 Python runtime 負擔

## 效能實測（DGX Spark / Qwen2.5-0.5B）

| 指標 | 結果 |
|------|------|
| Token 生成速度 | 151 TPS |
| 平均延遲 | 0.29 秒 |
| P95 延遲 | 0.38 秒 |
| 連續 20 請求成功率 | 100% |

## 為什麼不用 Ollama？

Ollama 在個人開發場景很好用，但在 GB-10 上有幾個問題：

| 面向 | Ollama | dllm |
|------|--------|------|
| 底層引擎 | llama.cpp（CPU/GPU 混合） | vLLM（CUDA 原生） |
| API 格式 | 自訂格式 | OpenAI 標準 |
| 多模型管理 | 一次一個模型 | 常駐 / 熱載入 / 備援 |
| 硬體感知 | 無 | 64GB / 128GB 自動配置 |
| 企業功能 | 無 | API Key / 審計日誌 |
| Binary 大小 | ~30MB（Go） | ~3.1MB（Rust） |

## 快速開始

```bash
# 安裝
git clone https://github.com/DanielChung520/dllm.git
cd dllm
cargo build --release
cp target/release/dllm ~/.local/bin/

# 下載模型
export HF_TOKEN="your_token"
dllm pull Qwen/Qwen3-Coder-30B-A3B-Instruct

# 啟動服務
dllm serve --port 11400

# 對話（另一終端機）
curl http://localhost:11400/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"模型名稱","messages":[{"role":"user","content":"你好"}]}'

# 或直接用 dllm run
dllm run 模型名稱
```

## 與 oMLX 的關係

Mac 用戶的最佳選擇是 oMLX（MLX 原生），NVIDIA 用戶的最佳選擇是 dllm（vLLM 原生）。兩者都提供相同的 OpenAI API（Port 11400）：

```
你的應用程式（OpenAI SDK）
  → base_url = "http://你的機器:11400/v1"
  → 不管是 Mac / DGX Spark / ASUS GB-10
  → API 格式完全一致，客戶端不用改
```

## GB-10 生態系

GB-10（Grace Blackwell）核心已成為多家 OEM 的標準規格：

| 廠商 | 設備 |
|------|------|
| NVIDIA | DGX Spark |
| ASUS | GB-10 設備（開發中） |
| Dell | GB-10 工作站（開發中） |
| HP | GB-10 工作站（開發中） |
| 銘凡 | GB-10 迷你 PC（開發中） |

所有 GB-10 設備執行相同的 ARM64 Linux binary，不需重新編譯。

## 技術架構

```
Client → Port 11400
         │
         ├── dllm-core（Rust, 3.1MB）
         │     ├── OpenAI API 路由
         │     ├── Engine Pool（多模型 LRU）
         │     ├── API Key 驗證
         │     └── 審計日誌
         │
         └── vLLM（推理引擎）
               ├── PagedAttention
               ├── Continuous Batching
               └── CUDA / NVIDIA GB-10
```

## 下一步

- [GitHub 專案](https://github.com/DanielChung520/dllm)
- [專案計畫書](https://github.com/DanielChung520/dllm/blob/main/PROJECT_PLAN.md)
- Benchmark 數據與測試腳本在 `benchmark.py`

## 貢獻

dllm 還很年輕，任何形式的貢獻都歡迎——尤其是不同 GB-10 設備的測試回報。有興趣請開 GitHub Issue 或 PR。


