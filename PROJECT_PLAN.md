# dllm 專案總體計畫書

> **版本**：v0.1.0-alpha
> **日期**：2026-06-23
> **定位**：中小企業本地 AI Box 統一執行環境
> **願景**：讓每一間中小企業都能擁有一台「插電即用」的 AI 中樞

---

## 一、專案概述

### 1.1 背景與動機

隨著 GB-10（NVIDIA Grace Blackwell）內核設備的興起，以及 Apple Silicon Mac 的普及，邊緣端運行大型語言模型已從實驗走向實用。然而，現有方案存在明顯斷層：

- **Ollama / LM Studio**：面向個人開發者，缺乏企業級管理與 RAG 整合
- **vLLM**：面向雲端資料中心，單一進程單一模型，缺乏邊緣端的多模型動態管理
- **oMLX**：Mac 專屬，NVIDIA 生態無法直接使用
- **Dify / LangChain**：需自備雲端資源，非本地化硬體方案

中小企業需要的是一台**開箱即用、資料不離境、可漸進式上雲**的 AI 設備。

### 1.2 產品定義

**dllm**（Distributed Local LLM Manager）是一套跨平台統一 LLM 執行環境，專為中小企業 AI Box 設計：

- **統一 API**：所有平台皆暴露相同的 OpenAI-compatible API（Port 11400）
- **跨平台**：Mac（MLX/Metal）+ NVIDIA（CUDA/GB-10/RTX）+ 未來消費級
- **多模型動態管理**：Engine Pool + LRU eviction，記憶體不足自動卸載
- **內建 RAG**：本地知識庫處理，文件上傳即問即答
- **資料庫 Agent**：NL2SQL，連接企業現有資料庫
- **工具生態**：MCP 整合，可接第三方工具
- **混合雲路由**：本地優先，雲端為輔，預算可控

### 1.3 目標市場

| 市場區隔 | 場景 | 硬體建議 | 模型上限 |
|---------|------|---------|---------|
| **小型企業**（10-50人）| 內部知識庫問答、文件處理 | GB-10（128GB 統一記憶體） | 1x 70B INT4 + Embedding |
| **中型企業**（50-200人）| 多部門知識庫、資料庫查詢、自動化流程 | 多台 GB-10 或單台 H100 | 1x 122B+ 或 多機 |
| **專業工作室** | 設計、法律、顧問等專業領域 | Mac Studio / GB-10 | Mac: 1x 122B+ |
| **消費級進階用戶** | 個人知識管理、AI 助理 | RTX 5090 / MacBook Pro | 1x 8-13B |

---

## 二、技術架構總覽

### 2.1 分層架構

```
┌─────────────────────────────────────────────────────────────┐
│  用戶界面層（User Interface）                                │
│  ├── Web Admin Dashboard（React + TypeScript）              │
│  ├── Desktop App（Tauri / Electron）                         │
│  └── 第三方整合（OpenAI SDK, Claude Code, Cursor）           │
├─────────────────────────────────────────────────────────────┤
│  控制平面層（Control Plane）— Rust 統一實現                   │
│  ├── dllm-core: API Gateway (Axum), Engine Pool, LRU        │
│  ├── dllm-shared: 共享類型、trait、序列化格式                 │
│  ├── dllm-nvidia: NVIDIA 後端適配（條件編譯）                │
│  └── dllm-mac: Mac MLX 後端適配（條件編譯）                  │
├─────────────────────────────────────────────────────────────┤
│  AI 核心引擎層（AI Engine）                                   │
│  ├── dllm-rag: 文件處理、Embedding、向量檢索                 │
│  ├── dllm-agent: 工具調用、MCP、ReAct Agent                  │
│  └── dllm-connector: 雲端 LLM 連接與路由                    │
├─────────────────────────────────────────────────────────────┤
│  資料與記憶層（Data Layer）                                   │
│  ├── Qdrant（向量資料庫，Rust 原生）                          │
│  ├── PostgreSQL + pgvector（結構化資料）                      │
│  └── 本地檔案系統（模型權重、文件快取）                       │
├─────────────────────────────────────────────────────────────┤
│  推理引擎層（Inference Runtime）                              │
│  ├── Mac: MLX / Metal（透過 dllm-mac 調用）                   │
│  ├── NVIDIA: vLLM / SGLang（透過 dllm-nvidia 管理）           │
│  └── 未來: Atlas / rvLLM（純 Rust 引擎，無縫替換）             │
├─────────────────────────────────────────────────────────────┤
│  硬體抽象層（Hardware Abstraction）                           │
│  ├── GB-10 / DGX Spark（ARM64 + CUDA）                        │
│  ├── x86-64 + RTX（消費級）                                   │
│  ├── Apple Silicon（M1/M2/M3/M4）                             │
│  └── H100/H800（企業級叢集，K8s 模式）                         │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心設計原則

1. **統一介面，異構實現**：API 層 100% 統一，底層按平台適配
2. **單一二進位**：Rust 控制層編譯為單一可執行檔，部署極簡
3. **容器化後端**：vLLM / RAG / 向量資料庫以 Docker 運行，版本可控
4. **插件化引擎**：推理引擎透過 trait 抽象，未來可無縫替換
5. **資料不離境**：本地推理為預設，雲端連接需明確授權

---

## 三、專案結構與倉庫規劃

### 3.1 多倉庫工作區（Multi-Repo Workspace）

```
dllm/
├── crates/                     # Rust 工作區（核心控制層）
│   ├── dllm-core/              # API Gateway + Engine Pool + 模型管理
│   ├── dllm-shared/            # 共享類型、trait、錯誤處理、序列化
│   ├── dllm-nvidia/            # NVIDIA 後端：vLLM 進程管理、CUDA 監控
│   └── dllm-mac/               # Mac 後端：MLX 調用、Metal 記憶體監控
├── services/                   # 服務層（獨立容器或子進程）
│   ├── dllm-rag/               # RAG Pipeline（文件處理、Embedding、檢索）
│   ├── dllm-agent/             # Agent Core（工具調用、MCP、工作流）
│   └── dllm-connector/         # 雲端連接器（OpenAI/Claude/通義路由）
├── admin/                      # 管理後台
│   └── dllm-admin/             # Web UI（React + TypeScript + Vite）
├── deploy/                     # 部署與維運
│   ├── docker/                 # Docker Compose、Dockerfile
│   ├── systemd/                # systemd service 檔案
│   └── oem/                    # OEM 預裝腳本、首次開機設定
├── docs/                       # 文件
│   ├── api/                    # API 規格
│   ├── architecture/           # 架構設計文件
│   └── deployment/             # 部署指南
├── Makefile                    # 統一構建入口
├── Cargo.toml                  # Rust workspace 定義
├── docker-compose.yml          # 開發環境快速啟動
└── PROJECT_PLAN.md             # 本文件
```

### 3.2 倉庫職責

| 倉庫 | 語言 | 職責 | 部署方式 |
|------|------|------|---------|
| `dllm-core` | Rust | HTTP API、請求路由、Engine Pool、LRU、記憶體監控、模型發現 | 單一二進位 |
| `dllm-shared` | Rust | 類型定義、trait、序列化、錯誤處理、配置解析 | 函式庫 |
| `dllm-nvidia` | Rust | vLLM 子進程生命周期管理、CUDA VRAM 監控、GPU 健康檢查 | 條件編譯 |
| `dllm-mac` | Rust | MLX 引擎調用、Metal 記憶體監控、Mac 平台適配 | 條件編譯 |
| `dllm-rag` | Python/Rust | 文件解析、OCR、Embedding、向量索引、混合檢索 | Docker 容器 |
| `dllm-agent` | Python/Rust | 工具註冊、MCP client、ReAct loop、工作流引擎 | Docker 容器 |
| `dllm-connector` | Rust | 雲端 LLM 連接池、請求轉換、計費追蹤 | 內嵌於 core |
| `dllm-admin` | TS/React | 模型管理、知識庫管理、監控面板、系統設定 | 靜態網站 |

---

## 四、開發時程與里程碑

### Phase 0：基礎建設（第 1-2 週）

**目標**：專案結構就緒，開發環境可運行

- [x] 多倉庫工作區初始化
- [ ] Rust workspace 配置（Cargo.toml、條件編譯）
- [ ] 共享類型與 trait 設計（dllm-shared）
- [ ] CI/CD 基礎（GitHub Actions：build、test、lint）
- [ ] 開發環境 Docker Compose（core + vLLM + Qdrant）
- [ ] 程式碼規範（rustfmt、clippy、pre-commit hooks）

**交付物**：
- `cargo build --workspace` 成功
- `docker-compose up` 啟動核心服務
- 單元測試框架就緒

### Phase 1：核心引擎 MVP + License（第 3-6 週）

**目標**：單一端口 11400 可回應 OpenAI API，支援硬體偵測與 License 驗證

- [ ] dllm-core：Axum HTTP server、OpenAI-compatible路由
- [ ] dllm-core：Engine Pool（多模型載入/卸載/固定）
- [ ] dllm-core：硬體自動偵測（64GB Mac vs 128GB GB-10）
- [ ] dllm-core：License 驗證系統（RSA 離線簽章）
- [ ] dllm-nvidia：vLLM 子進程管理（啟動/停止/健康檢查）
- [ ] dllm-mac：MLX 引擎適配（條件編譯）
- [ ] dllm-shared：模型發現（掃描目錄、辨識類型）
- [ ] dllm-shared：配置系統 + 硬體 profile 自動套用

**交付物**：
- 硬體插電即識別，自動套用對應配置
- License 過期自動降級
- `curl http://localhost:11400/v1/chat/completions` 成功對話
- Mac 與 NVIDIA 兩平台編譯通過

### Phase 2：RAG + 零接觸部署（第 7-10 週）

**目標**：文件上傳即可問答，設備開箱即用

- [ ] dllm-rag：文件解析（PDF、Word、Excel、Markdown）
- [ ] dllm-rag：Embedding 模型整合（BGE-M3）
- [ ] dllm-rag：向量索引（Qdrant 整合）
- [ ] dllm-rag：混合檢索（向量 + BM25 + 重排序）
- [ ] dllm-core：RAG API 擴展（`/v1/rag/upload`、`/v1/rag/query`）
- [ ] OEM：首次開機精靈（硬體檢測、模型下載、License 啟動）
- [ ] OEM：零接觸部署腳本
- [ ] dllm-admin MVP：設備狀態、模型管理

**交付物**：
- 開機 → 連網 → 自動下載模型 → 可問答（全程無命令列）
- 上傳 PDF 後可問答，附來源出處
- 支援中英文混合文件

### Phase 3：Agent + 遠端管理（第 11-14 週）

**目標**：客戶能查資料庫、發郵件；你能遠端管理所有設備

- [ ] dllm-agent：工具註冊與發現系統
- [ ] dllm-agent：MCP client + 內建工具（資料庫查詢、郵件）
- [ ] dllm-agent：ReAct Agent loop
- [ ] dllm-connector：雲端 LLM 路由（OpenAI / Claude / 通義）
- [ ] dllm-connector：計費追蹤與預算上限
- [ ] **遠端管理後台**：設備清單、心跳狀態、License 管理
- [ ] **遠端管理後台**：使用量統計（月活躍用戶、請求數）
- [ ] **OTA 自動更新**：差分更新、簽章驗證、藍綠部署

**交付物**：
- Agent 可查詢客戶資料庫並回答
- 你的後台可查看 100 台設備的在線狀態
- 批次推送更新，失敗自動回退

### Phase 4：硬體回收 + Admin UX（第 15-18 週）

**目標**：完整產品閉環，支援硬體回收與重新部署

- [ ] 一鍵恢復出廠設定（secure_wipe.sh）
- [ ] License 與設備綁定，換設備需重新啟用
- [ ] dllm-admin：知識庫管理、監控面板、用戶管理
- [ ] dllm-admin：多語言支援（繁中、簡中、英文、日文）
- [ ] 客戶資料加密備份（可選雲端）
- [ ] 效能優化：請求批次合併、KV Cache 共享
- [ ] 安全強化：API Key 管理、請求限流

**交付物**：
- 退租設備 30 分鐘內可重新部署給下一客戶
- Web Admin 可完成 90% 操作
- 首次開機到可用 < 30 分鐘

### Phase 5：企業升級路徑（第 19-24 週）

**目標**：當客戶成長到 50+ 人時，可無痛遷移到更大硬體

- [ ] 消費級適配：RTX 5090 輕量版、MacBook 版
- [ ] 高可用模式：多節點負載均衡
- [ ] SSO / LDAP 整合
- [ ] 審計日誌（誰問了什麼、用了什麼模型）
- [ ] 叢集模式：K8s operator

**交付物**：
- 客戶從 64GB Mac Mini 遷移到 H100 叢集，API 與設定不變
- 消費級版可 NT$ 1,000-2,000/月（個人方案）

---

## 五、技術選型詳細說明

### 5.1 控制平面（Rust）

| 元件 | 選型 | 理由 |
|------|------|------|
| HTTP Framework | Axum | 生態成熟、與 Tower 整合、效能優異 |
| 序列化 | serde + serde_json | 標準選擇 |
| 配置 | figment / config-rs | 支援多層級覆蓋（預設 < 檔案 < 環境變數） |
| 日誌 | tracing + tracing-subscriber | 結構化日誌、OpenTelemetry 相容 |
| 非同步 | tokio | Rust async 標準 |
| CLI | clap | 強大的命令列解析 |
| 資料庫連接 | sqlx | 編譯時檢查 SQL、async |
| gRPC | tonic | 若需與 vLLM gRPC 溝通 |
| 監控 | prometheus-client | 內建 metrics endpoint |

### 5.2 NVIDIA 後端

| 元件 | 選型 | 理由 |
|------|------|------|
| 推理引擎 | vLLM | 模型覆蓋率最高、PagedAttention、OpenAI API |
| 溝通方式 | HTTP / gRPC | vLLM 原生支援 |
| 進程管理 | tokio::process | Rust async 子進程管理 |
| VRAM 監控 | nvml-wrapper | NVIDIA Management Library Rust binding |
| 容器 | Docker + NVIDIA Container Toolkit | 隔離、版本可控 |

### 5.3 Mac 後端

| 元件 | 選型 | 理由 |
|------|------|------|
| 推理引擎 | mlx-lm / mlx-vlm | Apple 官方、統一記憶體最佳化 |
| 溝通方式 | 直接 FFI / Python subprocess | mlx-lm 為 Python 庫 |
| 記憶體監控 | system-monitoring | macOS 系統 API |

### 5.4 RAG 管道

| 元件 | 選型 | 理由 |
|------|------|------|
| 文件解析 | unstructured / marker | 支援多格式、版面分析 |
| OCR | surya / easyocr | 開源、多語言 |
| Embedding | sentence-transformers / BGE | 中文效果最佳 |
| 向量資料庫 | Qdrant | Rust 原生、輕量、無 JVM |
| 重排序 | BGE-Reranker | 提升檢索準確率 |

### 5.5 Agent 與工具

| 元件 | 選型 | 理由 |
|------|------|------|
| MCP client | 官方 SDK (Python) | MCP 為標準協議 |
| 工具調用 | 自研 + LangChain 參考 | 輕量、可控 |
| 資料庫連接 | SQLAlchemy + asyncpg | Python async DB |
| 郵件 | lettre (Rust) / yagmail (Python) | 依場景選擇 |

### 5.6 管理後台

| 元件 | 選型 | 理由 |
|------|------|------|
| 框架 | React 19 + TypeScript | 生態最大 |
| 構建 | Vite | 快速、現代 |
| UI 元件 | shadcn/ui + Tailwind CSS | 美觀、可客製 |
| 狀態管理 | Zustand | 輕量、TypeScript 友好 |
| 圖表 | Recharts / Tremor | 監控面板 |
| 桌面封裝 | Tauri (Rust) | 輕量、安全、與 Rust 後端天然整合 |

---

## 六、風險評估與緩解

| 風險 | 可能性 | 影響 | 緩解措施 |
|------|--------|------|---------|
| **硬體物流成本**（寄送/回收） | 中 | 毛利侵蝕 | 硬體折舊 3 年攤提；回收設備重新部署；快遞費談企業合約 |
| **客戶欠費 / 不歸還設備** | 中 | 硬體損失 | 簽訂設備借用合約；軟體 License 綁定訂閱狀態，過期自動停用；可選遠端鎖定 |
| **License 被破解** | 低 | 收入損失 | 核心控制層用 Rust 編譯，增加逆向難度；定期更換驗證邏輯 |
| **支援成本過高** | 中 | 毛利侵蝕 | 零接觸部署降低支援量；遠端診斷減少到場服務；建立知識庫自助排解 |
| **vLLM subprocess 啟動過慢** | 中 | 首次請求延遲高 | 預載 pinned models；非固定模型提示用戶「首次載入中」 |
| **GPU VRAM 無法動態共享** | 高 | 多模型切換受限 | 積極 eviction + 記憶體估算；產品定位明確為「邊緣設備」 |
| **硬體供應鏈波動** | 中 | 無法出貨 | 支援多硬體平台（Mac Mini / DGX Spark / ASUS / 銘凡），不依賴單一供應商 |
| **客戶資料遺失** | 中 | 法律責任 | 內建定期本地備份；可選雲端備份（加密）；服務條款明確責任歸屬 |
| **競品（Google/Apple）降價** | 低 | 價格壓力 | 差異化在「在地化支援 + RAG + Agent」而非單純硬體；客戶轉換成本高 |

---

## 七、商業模式與產品策略

### 7.1 核心商業模式：軟體租用 + 硬體借用

**一句話**：月租 NTD 10,000，免費借用 Mac Mini 64GB 一台，硬體升級另付差價。

### 7.2 定價方案

| 方案 | 月費 | 硬體 | 並發用戶 | 對標競品 |
|------|------|------|---------|---------|
| **標準方案** | **NTD 10,000** (~$310 USD) | Mac Mini M4 Pro 64GB（免費借用） | 2-4 人 | Google 同級方案 $300-500/月（不附硬體） |
| **升級方案 A** | NTD 10,000 + 硬體差價 | DGX Spark 128GB | 4-8 人 | — |
| **升級方案 B** | NTD 10,000 + 硬體差價 | ASUS GB-10 / 銘凡 GB-10 | 4-8 人 | — |

> **競爭優勢**：Google 同級方案月費 $300-500 USD 且不附硬體。你的方案 NTD 10,000（~$310 USD）**包含一台 Mac Mini**，對客戶來說等於硬體免費。

### 7.3 對產品設計的影響

這個商業模式決定了以下設計優先權：

| 影響面向 | 設計要求 | 優先級 |
|---------|---------|--------|
| **零接觸部署** | 客戶收到設備，插電即用，無需命令列 | 🔴 最高 |
| **遠端管理** | 可遠端監控設備健康狀態、用量統計 | 🔴 高 |
| **License 驗證** | 軟體綁定訂閱狀態，過期自動降級或鎖定 | 🔴 高 |
| **自動更新** | OTA 推送更新，客戶無需手動操作 | 🟡 中 |
| **資料備份** | 客戶知識庫可遠端備份（可選） | 🟡 中 |
| **使用分析** | 匿名用量統計，輔助產品決策 | 🟢 低 |
| **硬體偵測** | 自動識別 Mac Mini / DGX Spark / 其他 GB-10 設備 | 🔴 最高 |

### 7.4 版本策略（簡化為單一產品線）

> 因為是租用模式，不需要多版本。所有客戶都用同一套軟體，差異只在硬體。

| 硬體 | 月費 | 硬體成本（你的） | 硬體折舊（3 年攤提） | 你的毛利（約） |
|------|------|----------------|-------------------|--------------|
| Mac Mini M4 Pro 64GB | NTD 10,000 | ~$1,500 | ~$42/月 | **~$268/月** |
| DGX Spark 128GB | NTD 10,000 + $1,500-$2,000 差價 | ~$4,000 | ~$111/月 | **~$199/月 + 硬體回本** |

> 實務上：Mac Mini 客戶 4 個月回本，之後每個月毛利 ~$268。DGX Spark 客戶收到硬體差價時已打平硬體成本。

### 7.5 升級流程

```
客戶月租 NTD 10,000（Mac Mini）
        │
        ├── 業務成長，需要更多並發
        │       │
        │       ▼
        ├── 你寄出一台 DGX Spark
        │       │
        │       ├── 客戶插電
        │       ├── dllm 自動識別硬體
        │       ├── 載入相同設定檔
        │       └── 無痛升級，API 端口不變
        │
        └── 你收回 Mac Mini（重新部署給下個客戶）
```

### 7.6 對標分析：你的護城河

| 面向 | Google 同級方案 | 你的方案 | 你的優勢 |
|------|---------------|---------|---------|
| **月費** | $300-500 USD | ~$310 USD (NTD 10,000) | 價格相當 |
| **硬體** | 客戶自備 | Mac Mini 免費借用 | **硬體免費** |
| **本地推理** | ✅ | ✅ | 持平 |
| **OpenAI API** | ✅ | ✅ | 持平 |
| **中文支援** | 普通 | 最佳化（Qwen/BGE 原生中文） | **更強** |
| **RAG 知識庫** | 無或需另購 | 內建 | **有優勢** |
| **資料庫 Agent** | 無 | 規劃中 | **未來優勢** |
| **混合雲** | 僅雲端 | 本地+雲端可混合 | **更靈活** |
| **升級路徑** | 需更換方案 | 換硬體，軟體不變 | **無痛升級** |
| **技術支援** | 原廠（英文） | 你（繁體中文） | **在地化支援** |

---

## 八、成功指標（KPI）

### 8.1 技術指標

| 指標 | 目標 | 驗證方式 |
|------|------|---------|
| API 延遲（TTFT） | < 3s（30B 模型） | 壓力測試 |
| 並發用戶數 | > 4（64GB）/ > 8（128GB） | 負載測試 |
| 記憶體保護命中率 | > 99%（無 OOM） | 長期監控 |
| RAG 準確率 | > 80%（Top-3 召回） | 評測集 |
| 零接觸部署成功率 | > 95%（開機到可用） | OEM 出貨測試 |
| 自動更新失敗復原率 | > 99%（自動回退） | 整合測試 |

### 8.2 商業指標

| 指標 | 目標 | 時間 |
|------|------|------|
| 客戶續約率（年約） | > 85% | 上線 6 個月 |
| 零接觸開機到可用 | < 30 分鐘（含模型下載） | Phase 4 |
| 客戶技術支援工單 | < 5 張/月/100 設備 | 上線 3 個月 |
| 硬體回收良率 | > 95%（可重新部署） | 上線 6 個月 |
| 硬體回本週期（Mac Mini）| < 4 個月 | 財務模型 |

---

## 九、附錄

### 9.1 命名規範

- 倉庫名：`dllm-{module}`
- Crate 名：`dllm_{module}`（Rust 使用底線）
- Docker image：`dllm/{module}:{version}`
- 分支：`feature/{module}-{description}`、`fix/{issue-id}`
- 標籤：`v{major}.{minor}.{patch}-{prerelease}`

### 9.2 參考資料

- [oMLX](https://github.com/jundot/omlx) — Mac 本地 LLM 伺服器參考
- [vLLM](https://github.com/vllm-project/vllm) — NVIDIA 推理引擎
- [Atlas](https://github.com/Avarok-Cybersecurity/atlas) — DGX Spark 純 Rust 引擎
- [Qdrant](https://github.com/qdrant/qdrant) — 向量資料庫
- [MCP](https://modelcontextprotocol.io/) — 模型上下文協議

---

*本文件為動態文件，將隨專案進展持續更新。*
