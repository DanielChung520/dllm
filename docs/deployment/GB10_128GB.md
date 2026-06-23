# dllm 硬體規格指南：64GB vs 128GB

> 本產品提供兩種硬體選擇，軟體完全相同，僅並發用戶數不同。

---

## 一、兩款規格總覽

| 項目 | Mac Mini M4 Pro | DGX Spark (GB-10) |
|------|----------------|-------------------|
| **記憶體** | 64GB 統一記憶體 | 128GB 統一記憶體 |
| **GPU** | Apple M4 Pro (Metal) | NVIDIA GB-10 (CUDA) |
| **CPU** | ARM64 | ARM64 (Grace) |
| **作業系統** | macOS | Linux (Ubuntu) |
| **推理引擎** | MLX (dllm-mac) | vLLM (dllm-nvidia) |
| **管理方式** | CLI + Web Admin | Docker + CLI + Web Admin |
| **目標客戶** | 小型團隊 / 個人工作室 | 中型團隊 / 企業 |
| **並發用戶** | **2-4 人** | **4-8 人** |
| **建議定價** | 中階 | 高階 |

---

## 二、記憶體分配對比

### 兩者完全相同的部分（都是 4 個模型，共 ~38GB）

| 模型 | 用途 | 量化 | 權重 | KV Cache | 總計 |
|------|------|------|------|----------|------|
| Qwen3-Coder-30B-A3B | 程式開發、企業問答（主力） | INT4 | ~18GB | ~8GB | **~26GB** |
| Qwen2.5-VL-8B | 圖片辨識、多模態 | INT4 | ~5GB | ~4GB | **~9GB** |
| BGE-M3 / Qwen3-Embedding | 文本嵌入、RAG 檢索 | FP16 | ~2GB | — | **~2GB** |
| Qwen3.5-0.8B | 簡單對話、備用降載 | INT4 | ~0.5GB | ~0.5GB | **~1GB** |
| **模型總計** | | | **~25.5GB** | **~12.5GB** | **~38GB** |

### Mac Mini M4 Pro（64GB）

```
64GB 總計
├── macOS + 系統服務:       ~16GB（macOS 比 Linux 重）
├── Docker/服務容器:         ~4GB
├── RAG + Agent 服務:        ~4GB
├── 4 個模型:                ~38GB（同上）
└── 閒置緩衝:                 ~2GB  ⚠️ 緊繃
```

**結論：剛好能跑，但幾乎沒有多餘空間。**

- 無法同時處理大量並發（KV Cache 會隨用戶數線性成長）
- 每個用戶的 KV Cache 上限約 **4-6K tokens**
- 建議 `max_concurrent_requests = 4`
- `memory_guard` 設為 `safe`
- 不建議開啟 `preload_on_startup`，讓備用模型按需載入

### DGX Spark（128GB）

```
128GB 總計
├── Linux + 系統服務:        ~12GB（Linux 更輕量）
├── Docker/服務容器:         ~4GB
├── RAG + Agent 服務:        ~4GB
├── 4 個模型:                ~38GB（同上）
└── 閒置緩衝:                ~70GB  ✅ 非常充裕
```

**結論：非常舒服，有大量空間處理並發。**

- KV Cache 可支援 **8K+ tokens × 8 並發**
- 建議 `max_concurrent_requests = 8`
- `memory_guard` 設為 `balanced`
- 可開啟 `preload_on_startup = true`，所有模型開機載入

---

## 三、並發用戶差距的根源

兩台設備跑相同的 4 個模型，使用者感受到的延遲差異來自於 **KV Cache 的爭搶**：

```
記憶體中:
├── 模型權重（固定，無法壓縮）: ~25.5GB
├── KV Cache（隨用戶數成長）:  隨並發線性成長
│   ├── 1 用戶:   ~3GB
│   ├── 4 用戶:   ~12GB
│   ├── 8 用戶:   ~24GB
│   └── 16 用戶:  ~48GB ❌ 兩台都不夠
└── 系統保留:     ~20-30GB
```

| 記憶體 | 可容納 KV Cache | 可支援並發（32K ctx）|
|--------|----------------|-------------------|
| **64GB** | ~8-10GB | **2-4 用戶** |
| **128GB** | ~60-70GB | **4-8 用戶** |

> 關鍵差異：128GB 的剩餘空間可以容納更大的 KV Cache，因此能同時服務更多用戶而不觸發記憶體壓力。

---

## 四、配置檔對照

### Mac Mini M4 Pro（64GB）

```toml
[server]
host = "0.0.0.0"
port = 11400

[engine]
pinned_models = ["qwen3-coder-30b-a3b-4bit", "qwen2.5-vl-8b", "bge-m3"]
default_model = "qwen3-coder-30b-a3b-4bit"
memory_guard = "safe"          # 保留較多記憶體
max-concurrent-requests = 4    # 低並發
preload_on_startup = false     # 備用模型按需載入

[rag]
chunk_size = 512
top_k = 3                      # 檢索數降低，節省記憶體

[agent]
max_iterations = 5             # 降低迭代次數
```

```bash
# macOS 啟動指令
dllm serve --port 11400 --model-dir ~/.dllm/models
# 無需 Docker，MLX 原生運行
```

### DGX Spark（128GB）

```toml
[server]
host = "0.0.0.0"
port = 11400

[engine]
pinned_models = ["qwen3-coder-30b-a3b-4bit", "qwen2.5-vl-8b", "bge-m3"]
default_model = "qwen3-coder-30b-a3b-4bit"
memory_guard = "balanced"
max_concurrent_requests = 8
preload_on_startup = true

[rag]
chunk_size = 1024
top_k = 5
rerank = true                  # 可開啟重排序

[agent]
max_iterations = 10
```

```bash
# DGX Spark 啟動指令（Docker）
docker-compose up -d
```

---

## 五、產品定位總結

```
                ┌──────────────────────┐
                │    dllm 統一軟體       │
                │  Port 11400           │
                │  OpenAI API            │
                │  4 個模型             │
                └──────┬───────────────┘
                       │
          ┌────────────┴────────────┐
          ▼                         ▼
┌──────────────────┐   ┌──────────────────────┐
│  Mac Mini M4 Pro │   │   DGX Spark (GB-10)  │
│  64GB            │   │   128GB               │
│  2-4 用戶        │   │   4-8 用戶            │
│  MLX 引擎        │   │   vLLM 引擎           │
│  CLI 管理        │   │   Docker 管理          │
│  💰 $2,500-3,000│   │   💰 $4,000-5,000     │
└──────────────────┘   └──────────────────────┘
```

**客戶選擇標準：**
- 僅 2-3 人使用 → 64GB Mac Mini 已足夠
- 4-8 人團隊 → 升級到 128GB DGX Spark
- 軟體完全一致，遷移只需換硬體

## Docker Compose 記憶體限制

已為各服務設定 `mem_limit`：

| 服務 | 限制 | 說明 |
|------|------|------|
| vLLM | 96GB | 保留 32GB 給系統與其他服務 |
| dllm-rag | 8GB | Embedding 模型 + 文件處理 |
| dllm-agent | 4GB | Python runtime |
| Qdrant | 8GB | 向量資料庫 |
| PostgreSQL | 2GB | 結構化資料 |
| Redis | 1GB | 快取 |

## 關鍵設定值

### vLLM 啟動參數

```bash
# 128GB 專用
--gpu-memory-utilization 0.70    # 而非 0.85
--max-model-len 32768            # 而非 65536
--max-num-seqs 8                 # 而非 16
--enable-chunked-prefill         # 降低記憶體峰值
```

### 模型下載建議

優先下載 **30B-70B 級別**的模型：

```bash
# 推薦（記憶體友好）
Qwen/Qwen3-30B-A3B-Instruct-INT4      # ~18GB
mlx-community/Qwen3-8B-Instruct-4bit   # ~5GB（備用）
BAAI/bge-m3-mlx-fp16                   # ~2GB（Embedding）

# 可嘗試（需關閉其他服務）
mlx-community/Qwen3-70B-Instruct-4bit  # ~40GB

# 不建議（128GB 跑不動）
mlx-community/Qwen3.5-122B-4bit        # ~65GB，只剩 35GB 給系統
```

## 實際測試數據參考

根據 oMLX 在 M3 Ultra 512GB 上的 benchmark，推估 GB10 128GB 表現：

| 模型 | Prompt TPS | Token TPS | 記憶體峰值 |
|------|-----------|-----------|-----------|
| Qwen3-30B-A3B-4bit | ~400-500 | ~30-40 | ~28GB |
| Qwen3-70B-4bit | ~200-300 | ~15-20 | ~55GB |
| Qwen3.5-122B-4bit | ~100-150 | ~8-12 | ~85GB |

> 註：GB10 的 CUDA 核心數少於 M3 Ultra，實際數字可能更低。

## 升級路徑

當客戶需要更大模型或多模型並行時：

| 硬體 | 記憶體 | 可同時載入 |
|------|--------|-----------|
| GB10 | 128GB | 1x 70B 或 2x 30B |
| H100 (單卡) | 80GB VRAM | 1x 70B（需 CPU offloading） |
| H100 (雙卡) | 160GB VRAM | 1x 122B |
| Mac Studio (M2 Ultra) | 192GB | 1x 122B + 1x 30B |

## 總結

**128GB DGX Spark 能跑什麼？**

✅ **能跑**：
- 1 個 30-70B 主力模型（INT4）
- 1 個 Embedding 模型（BGE-M3）
- 完整的 RAG Pipeline
- 基礎 Agent 工具
- 同時服務 4-8 個並發用戶

❌ **不能跑**：
- 122B+ 大模型 + 其他服務同時運行
- 多個 70B 模型並行
- 超長上下文（65536）高並發
- 無限制的 Engine Pool LRU（需手動管理）

**建議產品定位**：「1 個主力模型 + 知識庫」的輕量企業 AI Box，而非「多模型任切換」的通用平台。
