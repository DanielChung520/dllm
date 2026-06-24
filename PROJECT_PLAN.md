# dllm 專案總體計畫書

> **版本**：v0.1.0-alpha
> **日期**：2026-06-23
> **定位**：中小企業本地 AI Box 統一執行環境
> **願景**：讓每一間中小企業都能擁有一台「插電即用」的 AI 中樞

---

## 一、專案概述

### 1.1 背景與動機

隨著 GB-10（NVIDIA Grace Blackwell）內核設備的興起，以及 Apple Silicon Mac 的普及，邊緣端運行大型語言模型已從實驗走向實用。然而，現有方案存在明顯斷層：

- **Ollama / LM Studio**：面向個人開發者，缺乏多模型管理、無企業級功能
- **vLLM**：單一進程單一模型，無模型下載管理、無多模型動態管理
- **oMLX**：Mac 專屬，NVIDIA 生態無法直接使用

中小企業需要的是一台**插電即用、可集中管理 LLM 推理**的 AI 設備。

### 1.2 產品定義

**dllm** 是一套跨平台 LLM 執行環境，專為中小企業 AI Box 設計：

- **統一 API**：所有平台皆暴露相同的 OpenAI-compatible API（Port 11400）
- **模型管理**：`dllm pull` / `dllm list` / `dllm rm`，像 Ollama 一樣直覺
- **多模型載入策略**：常駐（pinned）、熱載入（hot）、冷載入（cold）、備援（standby）
- **跨平台**：Mac（MLX）+ NVIDIA（CUDA/GB-10/RTX/H100）
- **硬體自動感知**：64GB Mac Mini 自動保守配置，128GB DGX Spark 自動最佳化
- **安全審計**：API Key 管理 + 請求日誌
- **License 驗證**：離線 RSA 簽章，支援月租商業模式

### 1.3 目標市場

| 市場區隔 | 場景 | 硬體建議 | 模型上限 |
|---------|------|---------|---------|
| **小型企業**（10-50人）| 私有 LLM 推理、企業內部使用 | GB-10（128GB 統一記憶體） | 1x 30B + 1x 8B |
| **中型企業**（50-200人）| 多團隊共享推理服務 | 多台 GB-10 或單台 H100 | 多機負載均衡 |
| **專業工作室** | 設計、法律、顧問等專業領域 | Mac Studio / GB-10 | Mac: 1x 122B+ |
| **消費級進階用戶** | 個人知識管理、AI 助理 | RTX 5090 / MacBook Pro | 1x 8-13B |

---

## 二、技術架構總覽

### 2.1 分層架構

```
┌─────────────────────────────────────────────────────────────┐
│  用戶界面層（User Interface）                                │
│  └── OpenAI SDK / curl（客戶端任何 OpenAI-compatible 工具）  │
├─────────────────────────────────────────────────────────────┤
│  控制平面層（Control Plane）— Rust 統一實現                   │
│  ├── dllm-core: API Gateway (Axum), Engine Pool, LRU        │
│  ├── dllm-shared: 共享類型、trait、序列化格式                 │
│  ├── dllm-nvidia: NVIDIA 後端適配（條件編譯）                │
│  └── dllm-mac: Mac MLX 後端適配（條件編譯）                  │
├─────────────────────────────────────────────────────────────┤
│  模型管理層（Model Management）                               │
│  ├── dllm pull/list/rm: HuggingFace 模型下載與管理           │
│  ├── Engine Profile: 常駐/熱載入/冷載入/備援策略             │
│  └── Tokenize: 請求前 token 計數 + 用量統計                  │
├─────────────────────────────────────────────────────────────┤
│  資料與記憶層（Data Layer）                                   │
│  └── 本地檔案系統（模型權重、設定檔）                        │
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
3. **容器化後端**：vLLM 以 Docker 運行，版本可控
4. **插件化引擎**：推理引擎透過 trait 抽象，未來可無縫替換
5. **資料不離境**：本地推理為預設，雲端連接需明確授權

---

## 三、專案結構與倉庫規劃

### 3.1 多倉庫工作區（Multi-Repo Workspace）

```
dllm/
├── crates/                     # Rust 工作區（核心控制層）
│   ├── dllm-core/              # API + Engine Pool + 模型管理 + CLI
│   ├── dllm-shared/            # 共享類型、trait、錯誤處理、序列化
│   ├── dllm-nvidia/            # NVIDIA 後端：vLLM 進程管理、CUDA 監控
│   └── dllm-mac/               # Mac 後端：MLX 調用、Metal 記憶體監控
├── deploy/                     # 部署與維運
│   ├── docker/                 # Dockerfile
│   ├── systemd/                # systemd service 檔案
│   └── oem/                    # OEM 預裝腳本、首次開機設定
├── docs/                       # 文件
├── Makefile                    # 統一構建入口
├── Cargo.toml                  # Rust workspace 定義
└── PROJECT_PLAN.md             # 本文件
```

### 3.2 倉庫職責

| 倉庫 | 語言 | 職責 | 部署方式 |
|------|------|------|---------|
| `dllm-core` | Rust | HTTP API、Engine Pool、`dllm pull/list/rm`、CLI | 單一二進位 |
| `dllm-shared` | Rust | 類型定義、trait、序列化、錯誤處理、配置、License | 函式庫 |
| `dllm-nvidia` | Rust | vLLM 子進程管理、CUDA VRAM 監控、GPU 健康檢查 | 條件編譯 |
| `dllm-mac` | Rust | MLX 引擎調用、Metal 記憶體監控 | 條件編譯 |

---

## 四、開發時程與里程碑

### Phase 0：基礎建設（第 1-2 週）

**目標**：專案結構就緒，開發環境可運行

- [x] 多倉庫工作區初始化
- [ ] Rust workspace 配置（Cargo.toml、條件編譯）
- [ ] 共享類型與 trait 設計（dllm-shared）
- [ ] CI/CD 基礎（GitHub Actions：build、test、lint）
- [ ] 開發環境 Docker Compose（core + vLLM）
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

### Phase 2：模型下載與管理（第 7-10 週）

**目標**：`dllm pull` / `dllm list` / `dllm rm` 完整 CLI，像 Ollama 一樣管理模型

- [ ] dllm-core：`dllm pull <model>` HuggingFace 模型下載（HF / GGUF）
- [ ] dllm-core：`dllm list` 列出已下載模型（量化、大小、context）
- [ ] dllm-core：`dllm rm <model>` 刪除模型
- [ ] dllm-core：`dllm info <model>` 顯示模型詳情
- [ ] dllm-shared：模型目錄管理（`~/.dllm/models/` 結構）
- [ ] dllm-shared：模型 metadata 快取（config.json 解析 + 記憶體估算）
- [ ] OEM：首次開機精靈（硬體檢測、模型下載、License 啟動）

**交付物**：
- `dllm pull Qwen/Qwen3-Coder-30B-A3B-Instruct` 一鍵下載
- `dllm list` 顯示已下載模型與狀態
- 模型資訊快取，重啟不遺失

### Phase 3：多模型載入策略（第 11-14 週）

**目標**：常駐 / 熱載入 / 冷載入 / 備援 四種模型策略

- [ ] dllm-core：常駐（pinned）— 開機載入，永不被 evict
- [ ] dllm-core：熱載入（hot）— 預載未使用，保持 warm
- [ ] dllm-core：冷載入（cold）— 按需載入，首次較慢
- [ ] dllm-core：備援（standby）— 主力模型忙碌時自動切換至降級模型
- [ ] dllm-core：Engine Pool 策略排程器（依據記憶體壓力自動調整）
- [ ] dllm-core：vLLM subprocess 管理（啟動/停止/健康檢查/重啟）
- [ ] dllm-core：Token 計算（tiktoken 整合，請求前計數 + 用量統計）

**交付物**：
- 設定 `pinned = ["qwen3-coder"]`，開機自動載入，永不卸載
- 設定 `hot = ["qwen2.5-vl"]`，記憶體足夠時預載
- 設定 `cold = ["mixtral"]`，按需載入
- 設定 `standby = ["qwen3.5-0.8b"]`，主模型忙碌時自動降級
- `/v1/tokenize` 端點可計算 prompt token 數

### Phase 4：安全審計 + 企業功能（第 15-18 週）

**目標**：API Key 管理、請求審計、License 綁定

- [ ] dllm-core：API Key 管理（建立 / 撤銷 / 權限）
- [ ] dllm-core：請求審計日誌（誰、何時、哪個模型、token 用量）
- [ ] dllm-core：請求限流（per-key rate limit）
- [ ] dllm-core：License 設備綁定 + 到期自動降級
- [ ] dllm-core：硬體回收腳本（secure_wipe + 出廠重置）
- [ ] dllm-core：簡易 Admin API（`GET /v1/admin/stats` 用量統計）

**交付物**：
- 每個客戶獨立 API Key，可撤銷
- 完整審計日誌，可追蹤每個請求
- License 過期自動停用推理，保留管理 API

### Phase 5：效優化 + 跨平台（第 19-24 週）

**目標**：極致效能、Mac 支援、消費級擴展

- [ ] dllm-nvidia：多實例 vLLM GPU 記憶體共享與隔離優化
- [ ] dllm-mac：MLX 引擎實作（條件編譯，Mac Mini 原生運行）
- [ ] 消費級適配：RTX 5090 輕量版、16-32GB 設備支援
- [ ] SSD KV Cache：評估階段（確認何時需要實作）
- [ ] 高可用模式：多節點負載均衡（企業客戶升級路徑）

**交付物**：
- Mac Mini 原生運行（無 Docker）
- NVIDIA + Mac 雙平台同一 binary 條件編譯
- 消費級設備模型載入策略指南

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

### 5.4 Token 計算

| 元件 | 選型 | 理由 |
|------|------|------|
| tokenizer | tiktoken / tokenizers | OpenAI 相容、多語言 |
| 整合方式 | Rust binding (tiktoken-rs) | 無需 Python runtime |

### 5.5 安全審計

| 元件 | 選型 | 理由 |
|------|------|------|
| API Key 管理 | 自研（sqlite + sha256） | 輕量、無外部依賴 |
| 請求日誌 | tracing + file rotate | 與現有日誌系統整合 |
| 限流 | tower-governor | 整合 Axum 中間件 |

---

## 六、風險評估與緩解

| 風險 | 可能性 | 影響 | 緩解措施 |
|------|--------|------|---------|
| **硬體物流成本**（寄送/回收） | 中 | 毛利侵蝕 | 硬體折舊 3 年攤提；回收設備重新部署；快遞費談企業合約 |
| **客戶欠費 / 不歸還設備** | 中 | 硬體損失 | 簽訂設備借用合約；軟體 License 綁定訂閱狀態，過期自動停用；可選遠端鎖定 |
| **License 被破解** | 低 | 收入損失 | 核心控制層用 Rust 編譯，增加逆向難度；定期更換驗證邏輯 |
| **支援成本過高** | 中 | 毛利侵蝕 | 零接觸部署降低支援量；遠端診斷減少到場服務 |
| **vLLM subprocess 啟動過慢** | 中 | 首次請求延遲高 | 預載 pinned models；非固定模型提示用戶「首次載入中」 |
| **GPU VRAM 無法動態共享** | 高 | 多模型切換受限 | 積極 eviction + 記憶體估算；產品定位明確為「邊緣設備」 |
| **硬體供應鏈波動** | 中 | 無法出貨 | 支援多硬體平台（Mac Mini / DGX Spark / ASUS / 銘凡），不依賴單一供應商 |
| **客戶資料遺失** | 中 | 法律責任 | 服務條款明確責任歸屬；客戶資料自行管理 |
| **競品（Google/Apple）降價** | 低 | 價格壓力 | 差異化在「在地化支援 + 多模型管理 + License 綁定硬體」；客戶轉換成本高 |

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
| **硬體回收** | 退租設備可重置並重新部署 | 🟡 中 |
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
| **多模型管理** | 無（一次一模型） | 常駐/熱/冷/備援 | **核心優勢** |
| **硬體自動感知** | 無 | 64GB→保守 / 128GB→最佳 | **核心優勢** |
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
| 模型載入時間（cold start） | < 15s（30B INT4） | 自動測試 |
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
- [Ollama](https://github.com/ollama/ollama) — 模型下載與管理參考
- [llama-swap](https://github.com/mARTin-B78/dgx-spark_lite-llm_llama-swap_vllm_llama-cpp_ollama) — 動態模型切換參考

---

*本文件為動態文件，將隨專案進展持續更新。*
