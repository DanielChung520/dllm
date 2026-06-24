# dllm 架構規格文件

> **版本**：v0.1.0-alpha
> **定位**：跨平台統一 LLM 執行環構

---

## 一、架構目標

1. **平台抽象**：上層邏輯與底層硬體解耦，同一套 Rust 控制層運行於 Mac / GB-10 / H100
2. **單一二進位**：`dllm-core` 編譯為單一可執行檔，部署極簡
3. **插件化推理引擎**：透過 trait 抽象，未來可無縫替換 vLLM / MLX / Atlas
4. **零停機擴展**：從單機邊緣設備到 K8s 叢集，API 與管理介面不變
5. **資料不離境**：本地推理為預設，雲端連接需明確授權與規則

## 一之一、硬體限制與實際配置（以 DGX Spark 128GB 為基準）

> ⚠️ **重要**：本專案以 **128GB DGX Spark（GB-10）為首要目標硬體**，所有預設配置必須在此硬體上可運行。

### 128GB 記憶體分配實際計算

```
總計: 128GB 統一記憶體
├── 系統保留 (Linux, Docker): ~12GB
├── Rust 控制層: ~0.5GB
├── 系統服務 (PostgreSQL): ~1GB
├── 結構化資料庫 (PostgreSQL): ~1GB
├── 快取 (Redis): ~0.5GB
├── Tokenizer / 日誌服務: ~1-2GB
└── 可用於 LLM 推理: ~100-105GB
```

### 兩種硬體規格（軟體完全相同）

| 規格 | Mac Mini M4 Pro | DGX Spark (GB-10) |
|------|----------------|-------------------|
| **記憶體** | 64GB 統一記憶體 | 128GB 統一記憶體 |
| **引擎** | MLX (Metal) | vLLM (CUDA) |
| **部署方式** | 原生 CLI（無 Docker） | Docker Compose |
| **並發用戶** | **2-4 人** | **4-8 人** |
| **價格帶** | $2,500-3,000 | $4,000-5,000 |

**4 個模型共 ~38GB，兩台都能跑：**

| 模型 | 用途 | 記憶體 |
|------|------|--------|
| Qwen3-Coder-30B-A3B | 程式開發、問答（主力） | ~26GB |
| Qwen2.5-VL-8B | 圖片辨識 | ~9GB |
| BGE-M3 / Embedding | 向量嵌入 | ~2GB |
| Qwen3.5-0.8B | 備用降載 | ~1GB |

### 記憶體分配對比

**64GB Mac Mini（2-4 用戶）：**
```
├── macOS + 服務: ~16GB
├── 4 個模型:     ~38GB
└── 緩衝:         ~10GB ⚠️
```
→ `memory_guard = safe`、`max_concurrent_requests = 4`、KV 上限 ~6K

**128GB DGX Spark（4-8 用戶）：**
```
├── Linux + 服務: ~12GB
├── 4 個模型:     ~38GB
└── 緩衝:         ~78GB ✅
```
→ `memory_guard = balanced`、`max_concurrent_requests = 8`、KV 上限 ~8K

> 核心差異：128GB 的緩衝空間可容納更大的 KV Cache，因此能同時服務更多用戶。
> 模型與功能完全相同，客戶可從 64GB 無痛升級到 128GB。

---

## 二、系統架構圖

### 2.1 總體架構

```
┌─────────────────────────────────────────────────────────────────────┐
│                          外部客戶端                                  │
│  OpenAI SDK │ Claude Code │ Cursor │ 企業內部系統 │ dllm-admin     │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ HTTP / WebSocket
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        dllm-core（Rust）                             │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  API Layer（Axum）                                           │    │
│  │  ├── OpenAI-compatible 路由（/v1/chat/completions）           │    │
│  │  ├── Anthropic-compatible 路由（/v1/messages）                │    │
│  │  ├── Tokenize 路由（/v1/tokenize）                            │    │
│  │  ├── Admin 路由（/v1/admin/*）                                │    │
│  │  ├── 管理路由（/admin/*、/v1/models、/health）                │    │
│  │  └── WebSocket（串流、監控）                                   │    │
│  ├─────────────────────────────────────────────────────────────┤    │
│  │  Business Logic Layer                                        │    │
│  │  ├── Request Router（請求分發到對應 Engine）                 │    │
│  │  ├── Engine Pool（多模型 LRU + TTL + Pin）                   │    │
│  │  ├── Memory Enforcer（記憶體壓力監控與自動卸載）              │    │
│  │  ├── Model Discovery（啟動時掃描、動態註冊）                  │    │
│  │  ├── Cloud Connector（混合雲路由與計費）                      │    │
│  │  └── Auth & Rate Limit（API Key、請求限流）                   │    │
│  ├─────────────────────────────────────────────────────────────┤    │
│  │  Engine Interface Layer（Trait 抽象）                         │    │
│  │  ├── InferenceEngine trait（generate / stream_generate）      │    │
│  │  ├── EmbeddingEngine trait（embed）                           │    │
│  │  └── HealthCheck trait（health / metrics）                    │    │
│  └─────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                        Platform Adapters                             │
│  ┌─────────────────────┐              ┌─────────────────────┐       │
│  │   dllm-nvidia       │              │     dllm-mac        │       │
│  │   （條件編譯）       │              │   （條件編譯）       │       │
│  │                     │              │                     │       │
│  │  VLLMProcessEngine  │              │   MLXProcessEngine  │       │
│  │  ├── 子進程管理      │              │   ├── MLX Python    │       │
│  │  ├── gRPC/HTTP 溝通  │              │   │   subprocess    │       │
│  │  ├── VRAM 監控       │              │   ├── Metal 記憶體  │       │
│  │  └── CUDA 健康檢查   │              │   └── 統一記憶體    │       │
│  └─────────────────────┘              └─────────────────────┘       │
└─────────────────────────────────────────────────────────────────────┘
                           │ 本地 IPC（gRPC / Unix Socket / HTTP）
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Service Layer（Docker / 子進程）                 │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐   │
│  │    dllm-nvidia   │  │   dllm-mac       │  │（平台適配層）   │   │
│  │  （Python/Rust）  │  │  （Python/Rust）  │  │    （Rust）       │   │
│  │                  │  │                  │  │                  │   │
│  │  文件解析         │  │  工具註冊         │  │  雲端 LLM 連接   │   │
│  │  Embedding       │  │  MCP client      │  │  請求轉換        │   │
│  │  向量檢索         │  │  ReAct Loop      │  │  計費追蹤        │   │
│  │  混合檢索         │  │  工作流引擎       │  │  Fallback 路由   │   │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Data Layer                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │   PostgreSQL │  │   Local FS   │              │
│  │  向量資料庫   │  │  + pgvector  │  │  模型/文件   │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Engine Pool 詳細設計

```
┌─────────────────────────────────────────────────────────────┐
│                    Engine Pool（單例）                        │
├─────────────────────────────────────────────────────────────┤
│  Config                                                     │
│  ├── model_dirs: Vec<PathBuf>      # 掃描路徑               │
│  ├── pinned_models: Vec<String>    # 固定不卸載             │
│  ├── default_model: Option<String> # 預設模型               │
│  ├── memory_guard: MemoryGuardMode # safe/balanced/aggressive│
│  └── ttl_seconds: Option<u64>      # 閒置卸載時間            │
├─────────────────────────────────────────────────────────────┤
│  State                                                      │
│  ├── engines: HashMap<String, Box<dyn InferenceEngine>>     │
│  ├── lru_list: LinkedList<String>  # 最近使用順序            │
│  ├── pinned: HashSet<String>       # 被固定的模型            │
│  └── memory_usage: MemorySnapshot  # 當前記憶體使用          │
├─────────────────────────────────────────────────────────────┤
│  Operations                                                 │
│  ├── discover_models()             # 掃描目錄               │
│  ├── load_model(id)                # 載入模型               │
│  ├── unload_model(id)              # 卸載模型               │
│  ├── evict_if_needed()             # LRU 卸載               │
│  ├── pin_model(id)                 # 固定模型               │
│  ├── unpin_model(id)               # 解除固定               │
│  ├── get_engine(id) -> Option      # 取得引擎               │
│  └── health_check_all()            # 健康檢查               │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 請求生命週期

```
Client Request
    │
    ▼
┌─────────────────┐
│  API Router     │ ──▶ 認證（API Key）
│  (Axum)         │ ──▶ 限流（Rate Limit）
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Request Parser  │ ──▶ 解析 model 欄位
│                 │ ──▶ 解析 model 欄位，決定目標引擎
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Engine Pool    │ ──▶ 模型已載入？→ 直接路由
│                 │ ──▶ 模型未載入？→ load_model() + LRU eviction
│                 │ ──▶ 記憶體不足？→ 返回 503
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Engine Pool    │ ──▶ 模型已載入？→ 直接路由
│                 │ ──▶ 模型未載入？→ load_model() + LRU eviction
│                 │ ──▶ 記憶體不足？→ 返回 503 + retry-after
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ InferenceEngine │ ──▶ 非同步串流生成（tokio::sync::mpsc）
│  (trait)        │ ──▶ 支援 cancel（Drop token）
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Response Stream │ ──▶ Server-Sent Events (SSE)
│ Formatter       │ ──▶ OpenAI-compatible chunk 格式
└─────────────────┘
```

---

## 三、核心 Trait 定義

### 3.1 InferenceEngine

```rust
/// 推理引擎抽象介面
/// 所有平台後端（vLLM / MLX / Atlas）皆需實現
#[async_trait]
pub trait InferenceEngine: Send + Sync {
    /// 引擎唯一識別碼
    fn engine_id(&self) -> &str;

    /// 模型資訊
    fn model_info(&self) -> &ModelInfo;

    /// 同步生成（非串流）
    async fn generate(&self, request: ChatRequest) -> Result<ChatResponse, EngineError>;

    /// 串流生成
    async fn stream_generate(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatChunk, EngineError>>, EngineError>;

    /// 健康檢查
    async fn health(&self) -> HealthStatus;

    /// 記憶體用量統計
    async fn memory_usage(&self) -> MemorySnapshot;

    /// 卸載模型（釋放資源）
    async fn unload(&self) -> Result<(), EngineError>;
}

/// 模型基本資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String, // "model"
    pub created: i64,
    pub owned_by: String,
    pub model_type: ModelType, // LLM / VLM / Embedding / Reranker
    pub max_context_length: usize,
    pub quantization: Option<String>, // "int4", "int8", "fp16"
    pub estimated_memory_mb: usize,
    pub capabilities: Vec<String>, // "chat", "vision", "tools", "json_mode"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    Llm,
    Vlm,
    Embedding,
    Reranker,
    AudioStt,
    AudioTts,
}
```

### 3.2 EngineFactory

```rust
/// 引擎工廠：根據平台與模型類型建立對應引擎
pub trait EngineFactory: Send + Sync {
    /// 是否支援此模型
    fn supports(&self, model_path: &Path, config: &ModelConfig) -> bool;

    /// 建立引擎實例
    async fn create(
        &self,
        model_id: String,
        model_path: PathBuf,
        config: EngineConfig,
    ) -> Result<Box<dyn InferenceEngine>, EngineError>;

    /// 預估記憶體用量（MB）
    fn estimate_memory(&self, model_path: &Path, config: &ModelConfig) -> usize;
}

/// 平台自動偵測
pub fn detect_platform() -> Platform {
    #[cfg(target_os = "macos")]
    {
        if std::process::Command::new("system_profiler")
            .args(&["SPHardwareDataType"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Apple M"))
            .unwrap_or(false)
        {
            return Platform::MacAppleSilicon;
        }
    }
    
    #[cfg(all(target_os = "linux", feature = "nvidia"))]
    {
        if nvml::Nvml::init().is_ok() {
            return Platform::NvidiaLinux;
        }
    }
    
    Platform::CpuOnly
}
```

### 3.3 Memory Management

```rust
/// 記憶體守衛模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryGuardMode {
    /// 保守模式：保留較多系統記憶體
    Safe,
    /// 平衡模式：預設
    Balanced,
    /// 積極模式：允許 AI 使用更多記憶體
    Aggressive,
    /// 自訂上限
    Custom { max_gb: f64 },
}

/// 記憶體快照
#[derive(Debug, Clone, Copy, Default)]
pub struct MemorySnapshot {
    pub total_mb: usize,
    pub used_mb: usize,
    pub available_mb: usize,
    pub engine_usage_mb: HashMap<String, usize>,
}

/// 記憶體強制執行器
#[async_trait]
pub trait MemoryEnforcer: Send + Sync {
    /// 啟動背景監控
    async fn start_monitoring(&self);
    
    /// 檢查是否有足夠記憶體載入新模型
    fn can_load(&self, required_mb: usize) -> bool;
    
    /// 建議應卸載的模型（按 LRU）
    fn suggest_eviction(&self, required_mb: usize) -> Vec<String>;
    
    /// 取得當前記憶體狀態
    fn snapshot(&self) -> MemorySnapshot;
}
```

---

## 四、跨平台適配策略

### 4.1 條件編譯配置

```toml
# Cargo.toml（dllm-core）
[features]
default = ["nvidia"]

# 平台互斥
nvidia = ["dllm-nvidia"]
mac = ["dllm-mac"]

# 功能選項
# 核心功能（無需 feature flag）
cloud = ["dllm-connector"]
admin = []

[dependencies]
dllm-shared = { path = "../dllm-shared" }
dllm-nvidia = { path = "../dllm-nvidia", optional = true }
dllm-mac = { path = "../dllm-mac", optional = true }
# 僅有 dllm-shared（核心類型）、dllm-nvidia（NVIDIA）、dllm-mac（Mac）
```

```rust
// 平台引擎初始化
cfg_if::cfg_if! {
    if #[cfg(feature = "nvidia")] {
        pub use dllm_nvidia::VLLMProcessEngine as DefaultEngine;
        pub use dllm_nvidia::NvidiaMemoryEnforcer as DefaultEnforcer;
    } else if #[cfg(feature = "mac")] {
        pub use dllm_mac::MLXProcessEngine as DefaultEngine;
        pub use dllm_mac::MacMemoryEnforcer as DefaultEnforcer;
    } else {
        pub use dllm_shared::MockEngine as DefaultEngine;
        pub use dllm_shared::MockEnforcer as DefaultEnforcer;
    }
}
```

### 4.2 NVIDIA 後端（dllm-nvidia）

```
dllm-nvidia/
├── Cargo.toml
└── src/
    ├── lib.rs              # 公開介面
    ├── vllm_engine.rs      # VLLMProcessEngine 實現
    ├── vllm_process.rs     # vLLM 子進程管理（啟動/停止/重啟）
    ├── vllm_client.rs      # vLLM HTTP/gRPC 客戶端
    ├── memory/
    │   ├── mod.rs          # NVIDIA 記憶體管理模組
    │   ├── nvml_monitor.rs # NVML VRAM 監控
    │   └── cuda_guard.rs   # CUDA 記憶體保護
    └── health/
        ├── mod.rs
        └── gpu_checker.rs  # GPU 溫度/利用率/驅動檢查
```

**VLLMProcessEngine 設計**：
- 每個模型一個 vLLM subprocess（透過 `tokio::process::Command`）
- 子進程參數：`--model`、`--port`（動態分配）、`--gpu-memory-utilization`、`--max-model-len`
- 通訊：HTTP REST API（vLLM 原生 OpenAI-compatible）
- 健康檢查：定時 `GET /health`，失敗則重啟
- 卸載：`SIGTERM` → 等待 graceful shutdown → `SIGKILL`

### 4.3 Mac 後端（dllm-mac）

```
dllm-mac/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── mlx_engine.rs       # MLXProcessEngine 實現
    ├── mlx_process.rs      # Python mlx-lm subprocess 管理
    ├── memory/
    │   ├── mod.rs
    │   └── metal_monitor.rs # Metal 記憶體監控
    └── health/
        └── mod.rs
```

**MLXProcessEngine 設計**：
- 呼叫 Python mlx-lm server subprocess
- 或使用 PyO3 直接嵌入 Python runtime（效能更好，但編譯複雜）
- 監控透過 macOS `vm_statistics64` 與 `task_info`

---

---

## 五、Token 計算

### 5.1 Token 計數

請求前 token 計數，避免超出模型上限：

```rust
/// Token 計數器
pub trait TokenCounter: Send + Sync {
    /// 計算 prompt 的 token 數量
    fn count_tokens(&self, model: &str, messages: &[ChatMessage]) -> Result<usize>;
    
    /// 計算文字的 token 數量
    fn count_text(&self, text: &str) -> usize;
    
    /// 估算生長的 token 數量
    fn estimate_completion_tokens(&self, model: &str, prompt_tokens: usize) -> usize;
}
```

### 5.2 用量統計

```
API 請求
    │
    ▼
┌─────────────────┐
│  Token Counter  │ ──▶ 請求前計算 prompt tokens
│                 │ ──▶ 檢查是否超出 max_tokens
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Engine         │ ──▶ 推理完成後回傳用量
│  (vLLM)         │ ──▶ usage.prompt_tokens
│                 │ ──▶ usage.completion_tokens
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Audit Log      │ ──▶ 記錄每筆請求用量
│                 │ ──▶ 可匯出統計報表
└─────────────────┘
```

---

## 六、雲端連接器設計

### 7.1 混合雲路由邏輯

```rust
/// 路由決策引擎
pub struct CloudRouter {
    /// 本地可用模型
    local_models: HashSet<String>,
    /// 雲端供應商配置
    providers: Vec<CloudProvider>,
    /// 路由規則
    rules: Vec<RoutingRule>,
    /// 用戶預算上限
    budget_limit: Option<f64>,
    /// 已用額度
    budget_used: Arc<AtomicF64>,
}

impl CloudRouter {
    /// 決定請求路由
    pub async fn route(&self, request: &ChatRequest) -> RouteDecision {
        // 1. 檢查隱私規則
        if self.is_sensitive(&request.messages) {
            return RouteDecision::LocalOnly;
        }
        
        // 2. 檢查預算
        if self.budget_exceeded() {
            return RouteDecision::LocalOnly;
        }
        
        // 3. 檢查本地模型能力
        if self.local_capable(request) {
            return RouteDecision::Local;
        }
        
        // 4. 評估請求複雜度
        let complexity = self.assess_complexity(request);
        match complexity {
            Complexity::Simple => RouteDecision::Local,
            Complexity::Moderate if self.local_available() => RouteDecision::Local,
            _ => RouteDecision::Cloud(self.select_provider(request)),
        }
    }
}
```

### 7.2 支援的雲端供應商

| 供應商 | API 格式 | 特色 |
|--------|---------|------|
| **OpenAI** | 原生 | GPT-4o、o1、DALL-E |
| **Anthropic** | Messages API | Claude 3.5、長上下文 |
| **Google** | Gemini API | Gemini Pro、多模態 |
| **通義千問** | OpenAI-compatible | 中文最佳 |
| **DeepSeek** | OpenAI-compatible | 高性價比 |
| **Azure OpenAI** | OpenAI-compatible | 企業合規 |

---

## 八、部署架構

### 8.1 開發環境（docker-compose）

```yaml
# docker-compose.yml
version: "3.8"

services:
  dllm-core:
    build:
      context: .
      dockerfile: deploy/docker/Dockerfile.core
    ports:
      - "11400:11400"
    volumes:
      - ./data/models:/models
      - ./data/config:/config
    environment:
      - DLLM_CONFIG_PATH=/config/settings.toml
      - DLLM_MODEL_DIR=/models
      - RUST_LOG=info
    depends_on:
      - vllm
    networks:
      - dllm-net

  vllm:
    image: vllm/vllm-openai:latest
    runtime: nvidia
    environment:
      - CUDA_VISIBLE_DEVICES=0
      - GPU_MEMORY_UTILIZATION=0.80
      - MAX_MODEL_LEN=32768
    volumes:
      - ./data/models:/models
    networks:
      - dllm-net

networks:
  dllm-net:
    driver: bridge
```

### 8.2 生產環境（OEM 預裝）

```bash
# OEM 首次開機腳本（deploy/oem/first-boot.sh）
#!/bin/bash
set -e

echo "=== dllm AI Box 首次設定 ==="

# 1. 硬體檢測
./scripts/detect-hardware.sh

# 2. 下載預設模型（若網路可用）
./scripts/download-default-models.sh

# 3. 啟動服務
systemctl enable dllm-core
systemctl enable dllm-rag
systemctl enable dllm-agent
systemctl enable qdrant
systemctl start dllm-core

# 4. 等待服務就緒
sleep 10

# 5. 顯示狀態
curl -s http://localhost:11400/health | jq .

echo "=== 設定完成 ==="
echo "管理後台: http://$(hostname -I | awk '{print $1}'):11401"
echo "API 端點: http://$(hostname -I | awk '{print $1}'):11400/v1"
```

---

## 九、監控與可觀測性

### 9.1 Metrics（Prometheus 格式）

```
# /metrics
# 請求指標
dllm_requests_total{model="qwen3-30b",status="success"} 1024
dllm_request_duration_seconds_bucket{model="qwen3-30b",le="1.0"} 950
dllm_tokens_generated_total{model="qwen3-30b"} 1048576

# 引擎指標
dllm_engine_loaded{model="qwen3-30b"} 1
dllm_engine_memory_mb{model="qwen3-30b"} 45056
dllm_engine_lru_position{model="qwen3-30b"} 1

# 系統指標
dllm_system_memory_total_mb 131072
dllm_system_memory_available_mb 65536
dllm_gpu_memory_total_mb 81920
dllm_gpu_memory_used_mb 45056

# RAG 指標
dllm_rag_documents_total{kb="company-docs"} 1500
dllm_rag_queries_total{kb="company-docs"} 512
dllm_rag_latency_seconds_bucket{le="0.5"} 480
```

### 9.2 日誌結構（tracing）

```json
{
  "timestamp": "2026-06-23T10:30:00Z",
  "level": "INFO",
  "target": "dllm_core::engine_pool",
  "span": {
    "request_id": "req-abc123",
    "model": "qwen3-30b-a3b-4bit",
    "user": "api-key-xxx"
  },
  "fields": {
    "message": "模型載入完成",
    "load_time_ms": 12000,
    "memory_mb": 45056
  }
}
```

---

## 十、安全設計

### 10.1 認證與授權

| 層級 | 機制 | 說明 |
|------|------|------|
| API Key | Bearer Token | 每用戶獨立 API Key，可撤銷 |
| Admin | Session + CSRF | 管理後台獨立認證 |
| 模型權限 | RBAC | 不同角色可用不同模型 |
| 審計日誌 | 不可變儲存 | 所有請求記錄，合規需求 |

### 10.2 資料保護

- **靜態加密**：模型權重、用戶文件、向量資料庫皆加密儲存
- **傳輸加密**：TLS 1.3（生產環境）
- **記憶體保護**：模型卸載後立即釋放記憶體（mlock / madvise）
- **隱私規則**：用戶可設定「不上雲」標籤，強制本地處理

---

## 十一、商業模式驅動的架構設計

> 本專案採用「軟體租用 + 硬體借用」模式（NTD 10,000/月）。以下架構設計直接對應此商業模式。

### 11.1 License 驗證系統

```
客戶設備
    │
    ├── dllm-core 啟動時
    │       │
    │       ├── 讀取本地 License 檔案（/etc/dllm/license.key）
    │       ├── 檢查過期時間（離線驗證，RSA 簽章）
    │       ├── 選擇性線上驗證（非必備，支援離線運行）
    │       │
    │       ├── ✅ 有效 → 正常啟動
    │       └── ❌ 過期 → 進入降級模式（僅 API 查詢，推理引擎不載入）
    │
    └── 每月續約時：
            └── 你上傳新的 License 簽章檔到客戶設備
                    ├── 透過你的 Admin Portal（客戶可自行貼上）
                    └── 或透過遠端管理 agent 自動更新
```

**離線優先設計**：客戶設備可能無網際網路連線，License 驗證必須支援完全離線。

### 11.2 硬體自動偵測

```rust
/// 自動識別設備類型，無需手動配置
pub fn detect_hardware() -> HardwareSku {
    match (platform::current(), total_memory_gb()) {
        // Mac Mini M4 Pro 64GB → 標準方案
        (Platform::MacAppleSilicon, 64..=128) => HardwareSku::MacMini64,
        // DGX Spark / ASUS / 銘凡 128GB → 升級方案
        (Platform::NvidiaLinux, 128..=192) => HardwareSku::DGXSpark128,
        // 未知硬體 → 降級模式
        _ => HardwareSku::Unknown,
    }
}

/// 串流自動切換配置
pub fn apply_hardware_profile(sku: HardwareSku) {
    match sku {
        HardwareSku::MacMini64 => {
            memory_guard = MemoryGuardMode::Safe;
            max_concurrent_requests = 4;
            // MLX 引擎
        }
        HardwareSku::DGXSpark128 => {
            memory_guard = MemoryGuardMode::Balanced;
            max_concurrent_requests = 8;
            // vLLM 引擎
        }
        HardwareSku::Unknown => {
            panic!("不支援的硬體平台");
        }
    }
}
```

### 11.3 OTA 自動更新

```
你的發布伺服器
    │
    ├── 推送更新通知（dllm-core 定時輪詢）
    │       │
    │       ├── 下載更新套件（差分更新，減少頻寬）
    │       ├── 驗證簽章（防止篡改）
    │       ├── 啟動備份容器
    │       ├── 切流（藍綠部署）
    │       └── 失敗自動回退
    │
    └── 你可以在 Admin Portal 查看：
            ├── 當前版本
            ├── 最後更新時間
            └── 回退歷史
```

### 11.4 硬體回收與重新部署流程

```
客戶退租
    │
    ├── 你收到設備
    │       │
    │       ├── 執行 secure_wipe.sh（清除客戶資料與知識庫）
    │       ├── 重建韌體到出廠狀態
    │       ├── 寫入新 License 金鑰
    │       └── 出貨給下一客戶
    │
    └── 技術上支援：
            ├── 一鍵恢復出廠設定（deploy/oem/factory-reset.sh）
            ├── 客戶資料加密儲存，清除後無法復原
            └── License 與設備綁定，確保前客戶無法繼續使用
```

### 11.5 遠端管理系統（你的後台）

```
你的管理後台（獨立系統）
    ├── 設備管理
    │   ├── 設備清單（硬體型號、版本、記憶體、儲存）
    │   ├── 在線狀態（心跳，30 秒一次）
    │   ├── 模型載入狀態（當前已載入模型、記憶體使用）
    │   └── 客戶資訊（租約期限、聯絡人）
    │
    ├── License 管理
    │   ├── 發放新 License
    │   ├── 延長/取消訂閱
    │   └── 歷史記錄
    │
    ├── 更新管理
    │   ├── 推送更新（個別/批次）
    │   ├── 回退版本
    │   └── 更新歷史
    │
    └── 監控與警報
        ├── 設備離線通知
        ├── 磁碟空間不足通知
        ├── 異常重啟通知
        └── 使用量統計（月活躍用戶、請求數）
```

---

## 十二、未來擴展

### 11.1 純 Rust 引擎替代（長期）

當 Atlas / rvLLM / vllm.rs 等純 Rust 引擎成熟時，可無縫替換 vLLM：

```rust
// 未來：條件編譯切換引擎
#[cfg(feature = "atlas")]
pub use atlas_backend::AtlasEngine as DefaultEngine;

#[cfg(feature = "vllm")]
pub use dllm_nvidia::VLLMProcessEngine as DefaultEngine;
```

### 11.2 企業級 K8s 模式

```
┌─────────────────────────────────────────┐
│           K8s Ingress                    │
│         （dllm-core Gateway）            │
└─────────────────┬───────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌────────┐  ┌────────┐  ┌────────┐
│vLLM Pod│  │vLLM Pod│  │vLLM Pod│  ← HPA 自動擴展
│Model A │  │Model B │  │Model C │
└────────┘  └────────┘  └────────┘
    │             │             │
    └─────────────┴─────────────┘
                  │
          ┌───────┴───────┐
          ▼               ▼
    ┌──────────┐    ┌──────────┐
    │ Qdrant   │    │PostgreSQL│
    │ Cluster  │    │ Cluster  │
    └──────────┘    └──────────┘
```

### 11.3 消費級輕量版

- **硬體**：RTX 5090（24GB）或 MacBook Pro（36GB）
- **模型**：8B-13B 級，INT4 量化
- **功能**：基礎 Chat + 輕量 RAG（< 1000 文件）
- **定價**：$99-199 一次性授權

---

*本文件為 dllm 專案的架構規格，將隨技術演進持續更新。*
