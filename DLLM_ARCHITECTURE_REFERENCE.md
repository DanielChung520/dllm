# dllm 服務架構參考文件

> 用途：作為在 DGX Spark（NVIDIA Grace Blackwell）上建立類似服務的架構藍圖對照

---

## 一、專案定位

**dllm** 是一個 **OpenAI-compatible LLM 推理伺服器**，專為 **Apple Silicon Mac** 最佳化。

- 專案：https://github.com/jundot/dllm
- 底層 Framework：**MLX**（Apple 自家的 ML framework，使用 Metal GPU）
- 對比 vLLM：使用 **CUDA + PyTorch**，目標是 NVIDIA GPU
- Port：`11400`（預設）
- 授權：Apache 2.0

---

## 二、系統架構總覽

```
┌─────────────────────────────────────────────────┐
│                 CLI (dllm)                       │
│    serve / start / stop / launch / diagnose      │
├─────────────────────────────────────────────────┤
│              FastAPI + Uvicorn                    │
│         OpenAI / Anthropic / 管理後台             │
├──────────────────────┬──────────────────────────┤
│    Engine Pool       │    Scheduler             │
│  (LRU multi-model)   │  (Continuous Batching)   │
├──────────────────────┴──────────────────────────┤
│              Model Discovery                      │
│         (掃描目錄、辨識類型、估計記憶體)          │
├──────────────────────┬──────────────────────────┤
│     Cache Layer      │  Memory Management       │
│  Paged / Prefix/ SSD │  Memory Guard / Enforcer │
├──────────────────────┴──────────────────────────┤
│              Engine Layer                         │
│  Batched / VLM / Embedding / Reranker / Audio    │
├─────────────────────────────────────────────────┤
│              MLX Runtime                          │
│     mlx-lm / mlx-vlm / mlx-embeddings / mlx-audio│
└─────────────────────────────────────────────────┘
```

---

## 三、核心子系統（逐層說明）

### 3.1 底層推理引擎

| 組件 | 檔案 | 說明 |
|------|------|------|
| `BaseEngine` | `engine/base.py` | 抽象基底類別，定義 `generate()`、`stream_generate()` 介面 |
| `BatchedEngine` | `engine/batched.py` | 主力 LLM Engine，包裝 `AsyncEngineCore` 提供 continuous batching |
| `VLMBatchedEngine` | `engine/vlm.py` | Vision-Language 多模態引擎（支援圖片） |
| `DFlashEngine` | `engine/dflash.py` | Apple G17 NAX DFlash 加速模型 |
| `EmbeddingEngine` | `engine/embedding.py` | 向量嵌入引擎（mlx-embeddings） |
| `RerankerEngine` | `engine/reranker.py` | 文件重排序引擎 |
| `STTEngine` | `engine/stt.py` | 語音轉文字 |
| `TTSEngine` | `engine/tts.py` | 文字轉語音 |
| `STSEngine` | `engine/sts.py` | 語音轉語音 |

### 3.2 Engine Pool（多模型管理）

**檔案**：`engine_pool.py`（1717 行）

這是 **dllm 最關鍵的設計**——支援同時載入多個模型並做 LRU 管理：

| 功能 | 說明 |
|------|------|
| **多模型載入** | 同時載入多個 model 目錄下的模型，透過 engine pool 共用記憶體 |
| **LRU Eviction** | 當記憶體不足時，自動卸載最近最少使用的模型 |
| **Model Pinning** | 支援固定（pin）模型，永遠不被 evict |
| **Pre-load Check** | 載入前檢查記憶體是否足夠，避免 OOM crash |
| **記憶體估算** | 從 `.safetensors` 自動估算模型記憶體佔用 |
| **類型自動偵測** | 自動判斷是 LLM / VLM / Embedding / Reranker / Audio |

### 3.3 Scheduler（排程器）

**檔案**：`scheduler.py`（10114 行——dllm 最大單一檔案）

基於 **mlx-lm 的 BatchGenerator** 實現 continuous batching：

| 概念 | 對應 vLLM 概念 | 說明 |
|------|----------------|------|
| Waiting Queue | waiting queue | 等待排隊的請求 |
| Running Set | running set | 正在執行的請求 |
| BatchGenerator | Tensor Parallel Engine | 批次執行的核心 |
| Max Concurrent Requests | max_num_seqs | 最大同時處理請求數 |
| Paged KV Cache | PagedAttention | 分頁式 KV Cache |
| Prefix Cache | Automatic Prefix Caching | 前綴共享 |
| Prefill Memory Guard | — | 預填充階段記憶體保護 |

### 3.4 Cache 系統

**檔案**：`cache/` 目錄（16 個檔案）

```
cache/
├── paged_cache.py        # 分頁式 KV Cache（主體）
├── paged_ssd_cache.py    # SSD 備援層
├── hybrid_cache.py       # 熱 + 冷快取混合
├── prefix_cache.py       # 前綴共享快取
├── vision_feature_cache.py # VLM 特徵快取
├── boundary_snapshot_store.py # 邊界快照
├── observability.py      # 快取命中率觀測
└── recovery.py           # 異常復原
```

三層架構：
```
      Hot Cache (in-memory, ~8GB)
              ↓
  Paged Cache (in-memory KV blocks)
              ↓
  Paged SSD Cache (SSD-backed, ~100GB)
```

### 3.5 記憶體管理

| 組件 | 檔案 | 說明 |
|------|------|------|
| `Memory Monitor` | `memory_monitor.py` | MLX Metal API 記憶體用量追蹤 |
| `ProcessMemoryEnforcer` | `process_memory_enforcer.py` | 動態記憶體壓力管理後臺執行緒 |
| `Memory Guard` |（設定層） | 三層保護：safe / balanced / aggressive |
| `Prefill Memory Guard` | `prefill_progress.py` | 預填充階段防止記憶體爆量 |

Memory Guard 運作邏輯：
```
safe:       保留較多系統記憶體（給其他 app 使用）
balanced:   中間值（預設）
aggressive: 允許 dllm 使用更多記憶體
custom:     自訂上限 (--memory-guard-gb N)
```

### 3.6 API 層

**檔案**：`api/` 目錄（21 個檔案）

| API | 路由 | 說明 |
|-----|------|------|
| OpenAI Chat Completions | `POST /v1/chat/completions` | 主要聊天 API |
| OpenAI Completions | `POST /v1/completions` | 純文字補全（legacy） |
| OpenAI Embeddings | `POST /v1/embeddings` | 向量嵌入 |
| Anthropic Messages | `POST /v1/messages` | Anthropic 相容 API |
| OpenAI Responses | `POST /v1/responses` | Codex 相容 API |
| Model List | `GET /v1/models` | 列出可用模型 |
| Model Status | `GET /v1/models/status` | 模型詳細狀態 |
| Rerank | `POST /v1/rerank` | 文檔重排序 |
| Health | `GET /health` | 健康檢查 |
| MCP Tools | `GET /v1/mcp/tools` | MCP 工具查詢 |
| Admin | `/admin/` | Web 管理後台 |

### 3.7 Model Discovery（模型發現）

**檔案**：`model_discovery.py`（1257 行）

啟動時掃描 model 目錄，自動：
1. 讀取每個子目錄的 `config.json`
2. 根據 `model_type` 自動分類（llm / vlm / embedding / reranker / audio_stt 等）
3. 估算記憶體用量（從 `.safetensors` index 計算）
4. 辨識支援的架構（Qwen / Llama / Mistral / Gemma 等）

支援的 VLM 架構（29+ 種）：
```
qwen2_vl, qwen2_5_vl, qwen3_vl, qwen3_vl_moe, gemma3, gemma4, 
llava, llava_next, idefics3, internvl_chat, phi3_v, pixtral, 
mulmo, mistral3, deepseekocr, florence2, ...
```

---

## 四、部署模式

### macOS App Bundle 模式
```
dllm.app/                   # SwiftUI 原生 macOS app
├── Contents/
│   ├── MacOS/dllm          # Swift 原生中介層
│   └── Resources/
│       └── bin/dllm        # Python server（venvstacks 封裝）
```

### CLI 啟動
```bash
dllm serve --model-dir ~/.dllm/models/ --port 11400
```

### launchd 背景服務
```bash
dllm.sh start     # launchctl load
dllm.sh stop      # launchctl bootout
dllm.sh status    # curl http://127.0.0.1:11400/v1/models
```

### 目錄結構
```
~/.dllm/
├── models/              # 模型目錄（每個子目錄一個模型）
│   ├── Qwen3-Coder-30B-A3B-Instruct-4bit/
│   │   ├── config.json
│   │   └── *.safetensors
│   └── bge-m3-mlx-fp16/
├── settings.json        # 伺服器設定
├── model_settings.json  # 模型個別設定
├── global_settings.json # 全域設定
├── logs/
└── cache/
    ├── hot/             # 熱快取（in-memory swap）
    └── paged/           # SSD 分頁快取
```

---

## 五、啟動流程（初始化順序）

```
dllm serve
  │
  ├── 1. 設定初始化
  │     ├── 讀取 CLI args
  │     ├── 讀取 settings.json / global_settings.json
  │     └── 寫回 CLI override
  │
  ├── 2. Socket Binding（先綁定 port，失敗早退）
  │
  ├── 3. 建立 Engine Pool
  │     ├── 掃描 model_dirs（model_discovery）
  │     ├── 載入 model_settings.json
  │     ├── 標記 pinned models
  │     └── 設定 default model
  │
  ├── 4. FastAPI lifespan（啟動時被呼叫）
  │     ├── Preload pinned models
  │     ├── 啟動 ProcessMemoryEnforcer（背景執行緒）
  │     ├── 設定 TTL check loop
  │     └── 初始化 MCP config
  │
  ├── 5. uvicorn.run()
  │
  └── 6. API ready（/v1/chat/completions 等）
```

---

## 六、vLLM 功能對照表（DGX Spark 建置參考）

| dllm 功能 | vLLM 對應 | 備註 |
|-----------|-----------|------|
| OpenAI-compatible API | ✅ 原生支援 | 兩者皆跟隨 OpenAI API 規範 |
| Continuous Batching | ✅ PagedAttention | vLLM 的核心功能 |
| 多模型服務 | ⚠️ LoRA / 實驗性 | vLLM 單一 process 單一 model，需多 process 或多 container |
| LRU Eviction | ❌ 無 | vLLM 不支援動態卸載模型 |
| Model Pinning | ❌ 不適用 | — |
| Paged KV Cache | ✅ PagedAttention | 兩者都有 |
| Prefix Caching | ✅ Automatic Prefix Caching | vLLM 也有 |
| SSD-backed Cache | ❌ 無 | — |
| 記憶體動態管理 | ❌ 無 | vLLM 預先分配 GPU memory |
| Vision-Language | ✅ 支援 | vLLM 支援多模態 |
| Embeddings | ✅ 支援 | vLLM 也有 embedding endpoint |
| Anthropic API 相容 | ❌ 無 | vLLM 僅支援 OpenAI API |
| 管理後台 | ❌ 無 | vLLM 無 web admin |
| 模型自動發現掃描 | ❌ 無 | vLLM 需手動指定 model name |
| MCP 整合 | ❌ 無 | — |
| Multimodal（工具呼叫） | ✅ 支援 | |
| Structured Output | ✅ 支援 | vLLM 使用 xgrammar / outlines |
| 模型下載工具 | ✅ huggingface-cli | |
| 單機輕量部署 | ✅ | vLLM 同樣可單機部署 |

---

## 七、DGX Spark 上的建議對應方案

### 建議方案：**vLLM + 若干補充**

| 層級 | dllm 做法 | DGX Spark 建議 |
|------|-----------|----------------|
| **底層框架** | MLX (Metal GPU) | CUDA (NVIDIA GPU) |
| **推理引擎** | mlx-lm BatchGenerator | vLLM (連續批次 + PagedAttention) |
| **多模型管理** | EnginePool + LRU eviction | 多個 vLLM 實例，或管理腳本動態起停 |
| **伺服器** | FastAPI + uvicorn | vLLM 內建 FastAPI server 🎯 |
| **KV Cache** | Paged SSD + Hot cache | vLLM PagedAttention + GPU memory |
| **前端 UI** | SwiftUI macOS App | Open WebUI（開源方案） |
| **管理** | Web Admin Dashboard | 自建或使用 Open WebUI |
| **背景服務** | launchd | systemd / Docker compose |
| **模型下載** | HuggingFace Hub | `huggingface-cli` |
| **整合工具** | MCP / Claude Code launch | 直接使用 OpenAI SDK 連接 vLLM |

### 快速起步指令（DGX Spark）

```bash
# 安裝 vLLM（pip）
pip install vllm

# 啟動服務（單一模型）
vllm serve Qwen/Qwen3-Coder-30B-A3B-Instruct \
  --port 11400 \
  --max-model-len 65536 \
  --gpu-memory-utilization 0.90

# 測試 API
curl http://localhost:11400/v1/models
```

### 如需多模型服務

```yaml
# docker-compose.yml（每個模型一個 container）
services:
  llm-main:
    image: vllm/vllm-openai:latest
    command: --model Qwen/Qwen3-Coder-30B-A3B-Instruct --port 8000
    ports:
      - "11401:8000"
    volumes:
      - ~/.cache/huggingface:/root/.cache/huggingface
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1

  llm-embedding:
    image: vllm/vllm-openai:latest
    command: --model BAAI/bge-m3 --task embedding --port 8000
    ports:
      - "11402:8000"
    # ...
```

---

## 八、關鍵差異總結

| 項目 | dllm（Mac） | vLLM（DGX Spark/GPU） |
|------|-------------|----------------------|
| **加速硬體** | Apple Silicon GPU (Metal) | NVIDIA GPU (CUDA) |
| **記憶體模型** | 統一記憶體（CPU+GPU 共享） | 分離式 GPU VRAM |
| **多模型動態管理** | ✅ 內建 LRU Pool | ❌ 需多實例或多容器 |
| **模型格式** | MLX 格式（GGUF 需轉換） | HuggingFace 格式（原生支援） |
| **性能特色** | 統一記憶體可載入超大模型 | 較高 raw throughput |
| **部署複雜度** | 低（macOS app） | 中（Python + NVIDIA driver） |
| **生態成熟度** | 較小（MLX 社群） | 極大（CUDA 生態） |

---

*文件建立日期：2026-06-23*
*參考專案：dllm (https://github.com/jundot/dllm)*
