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

| 市場區隔 | 場景 | 硬體建議 |
|---------|------|---------|
| **小型企業**（10-50人）| 內部知識庫問答、文件處理 | GB-10（128GB 統一記憶體） |
| **中型企業**（50-200人）| 多部門知識庫、資料庫查詢、自動化流程 | 多台 GB-10 或單台 H100 |
| **專業工作室** | 設計、法律、顧問等專業領域 | Mac Studio / GB-10 |
| **消費級進階用戶** | 個人知識管理、AI 助理 | RTX 5090 / MacBook Pro |

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

### Phase 1：核心引擎 MVP（第 3-6 週）

**目標**：單一端口 11400 可回應 OpenAI API，支援多模型切換

- [ ] dllm-core：Axum HTTP server、OpenAI-compatible路由
- [ ] dllm-core：Engine Pool（多模型載入/卸載/固定）
- [ ] dllm-core：LRU eviction 策略
- [ ] dllm-nvidia：vLLM 子進程管理（啟動/停止/健康檢查）
- [ ] dllm-nvidia：CUDA VRAM 監控與記憶體保護
- [ ] dllm-mac：MLX 引擎適配（條件編譯）
- [ ] dllm-shared：模型發現（掃描目錄、辨識類型）
- [ ] dllm-shared：配置系統（TOML/JSON、環境變數覆蓋）

**交付物**：
- `curl http://localhost:11400/v1/models` 列出可用模型
- `curl http://localhost:11400/v1/chat/completions` 成功對話
- 記憶體不足時自動卸載 LRU 模型
- Mac 與 NVIDIA 兩平台編譯通過

### Phase 2：RAG 與知識庫（第 7-10 週）

**目標**：文件上傳即可問答，本地知識庫可用

- [ ] dllm-rag：文件解析（PDF、Word、Excel、Markdown）
- [ ] dllm-rag：OCR 支援（掃描件、圖片中的文字）
- [ ] dllm-rag：文本分塊與清洗
- [ ] dllm-rag：Embedding 模型整合（BGE-M3、 multilingual）
- [ ] dllm-rag：向量索引（Qdrant 整合）
- [ ] dllm-rag：混合檢索（向量 + BM25 + 重排序）
- [ ] dllm-core：RAG API 擴展（`/v1/rag/upload`、`/v1/rag/query`）
- [ ] dllm-admin：知識庫管理介面

**交付物**：
- 上傳 PDF 後可問答
- 支援中英文混合文件
- 檢索結果附帶來源出處

### Phase 3：Agent 與工具（第 11-14 週）

**目標**：不只是聊天機器人，而是企業自動化中樞

- [ ] dllm-agent：工具註冊與發現系統
- [ ] dllm-agent：MCP client（連接外部 MCP servers）
- [ ] dllm-agent：ReAct Agent loop（推理-行動-觀察）
- [ ] dllm-agent：內建工具（資料庫查詢、郵件、檔案操作、網頁爬蟲）
- [ ] dllm-connector：雲端 LLM 路由（OpenAI / Claude / 通義 / Gemini）
- [ ] dllm-connector：請求複雜度評估（本地 vs 雲端）
- [ ] dllm-connector：計費追蹤與預算上限
- [ ] dllm-core：Agent API（`/v1/agent/run`、`/v1/agent/tools`）

**交付物**：
- Agent 可查詢本地資料庫並回答
- Agent 可發送郵件摘要
- 雲端 fallback 自動觸發
- 用戶可設定「財務資料不上雲」規則

### Phase 4：管理後台與產品化（第 15-18 週）

**目標**：非技術用戶也能使用，OEM 可預裝

- [ ] dllm-admin：模型管理（下載、啟動、設定）
- [ ] dllm-admin：知識庫管理（上傳、分類、權限）
- [ ] dllm-admin：監控面板（GPU/記憶體、請求量、延遲）
- [ ] dllm-admin：系統設定（雲端連接、預算、隱私規則）
- [ ] dllm-admin：用戶管理（多帳號、權限控制）
- [ ] OEM：首次開機精靈（硬體檢測、模型下載、網路設定）
- [ ] OEM：遠端監控 agent（健康狀態回報）
- [ ] OEM：自動更新機制（OTA、差分更新、rollback）

**交付物**：
- Web Admin 可完成 90% 日常操作
- 首次開機 15 分鐘內可用
- 支援多語言（繁中、簡中、英文、日文）

### Phase 5：性能優化與企業級（第 19-24 週）

**目標**：支援 50+ 並發用戶，可擴展到 H100 叢集

- [ ] 性能優化：請求批次合併、KV Cache 共享
- [ ] 企業功能：SSO / LDAP 整合
- [ ] 企業功能：審計日誌（誰問了什麼、用了什麼模型）
- [ ] 企業功能：模型權限（不同部門可用不同模型）
- [ ] 叢集模式：K8s operator、多節點模型分片
- [ ] 消費級適配：RTX 5090 輕量版、量化優化
- [ ] 安全強化：API Key 管理、請求限流、WAF

**交付物**：
- 單機 50 並發 < 2s TTFT
- K8s Helm chart
- SOC 2 合規基礎

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
| vLLM subprocess 啟動過慢 | 中 | 首次請求延遲高 | 預載 pinned models；非固定模型提示用戶「首次載入中」 |
| GPU VRAM 無法動態共享 | 高 | 多模型切換受限 | 積極 eviction + 記憶體估算；產品定位明確為「邊緣設備」 |
| Rust 開發速度較 Python 慢 | 中 | 時程延誤 | 服務層（RAG/Agent）先用 Python，控制層用 Rust；逐步遷移 |
| 模型授權風險 | 中 | 法律問題 | 不提供模型下載，只提供管理框架；用戶自行下載開源模型 |
| 競品（NVIDIA NIM）免費化 | 低 | 價格壓力 | 差異化在「本地化 RAG + Agent + 管理」而非單純推理 |
| 硬體供應鏈波動 | 中 | 無法出貨 | 支援多硬體平台（GB-10、RTX、Mac），不依賴單一供應商 |

---

## 七、商業模式建議

### 7.1 收入來源

| 項目 | 模式 | 定價參考 |
|------|------|---------|
| **硬體銷售** | 一次買斷 + 服務費 | GB-10 AI Box: $5,000-8,000 |
| **軟體授權** | 按設備 / 按年 | 基礎版免費，企業版 $500/年/設備 |
| **雲端點數** | 預付 + 用量計費 | 雲端 fallback 代充值，抽成 10-15% |
| **專業服務** | 客製化 RAG / Agent | 按專案計費 |
| **OEM 授權** | 販售給硬體廠商預裝 | 每設備 $50-100 授權費 |

### 7.2 版本分級

| 版本 | 功能 | 目標客戶 |
|------|------|---------|
| **Community** | 單機、2 模型、基礎 RAG、社群支援 | 個人、開發者 |
| **Pro** | 多機、無限模型、進階 RAG、Agent、Email 支援 | 小型企業 |
| **Enterprise** | K8s、SSO、審計、SLA、專屬客服 | 中型企業 |

---

## 八、成功指標（KPI）

### 8.1 技術指標

| 指標 | 目標 | 驗證方式 |
|------|------|---------|
| 單模型載入時間 | < 30s（70B INT4） | 自動測試 |
| API 延遲（TTFT） | < 2s（本地模型） | 壓力測試 |
| 並發用戶數 | > 20（GB-10 單機） | 負載測試 |
| 記憶體保護命中率 | > 95%（無 OOM） | 長期監控 |
| RAG 準確率 | > 80%（Top-3 召回） | 評測集 |

### 8.2 產品指標

| 指標 | 目標 | 時間 |
|------|------|------|
| 首次開機到可用 | < 15 分鐘 | Phase 4 |
| 用戶無命令列操作比例 | > 90% | Phase 4 |
| 客戶續約率 | > 80% | 上線 6 個月 |
| NPS（淨推薦值） | > 50 | 上線 3 個月 |

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
