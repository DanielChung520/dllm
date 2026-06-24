# dllm

> **硬體無關的 LLM 執行層** — 同一套 OpenAI API，管你是 Mac、GB-10 還是 H100。

## 緣起

大語言模型正在改變企業運作方式，但現狀是破碎的：

- **Mac 用戶**用 oMLX（MLX/Metal），體驗好但只能跑在 Apple 生態
- **NVIDIA 用戶**用 vLLM 或 Ollama，效能高但缺乏統一管理
- **Ollama 的自訂 API**跟業界標準 OpenAI API 格格不入
- 客戶被迫選擇硬體平台，跨平台遷移等於重寫整合程式碼

GB-10（Grace Blackwell）核心的出現改變了遊戲規則——ASUS、Dell、HP、銘凡等多家 OEM 都在推出這個規格的設備，128GB 統一記憶體、100W 功耗、可放辦公室，對中小企業來說是合理的本地 AI 起點。

**dllm 的目標是：不管底層是什麼硬體，開發者永遠面對同一套 OpenAI API。**

```
你的應用程式（OpenAI SDK）
         │
         ▼
  base_url = "http://你的設備:11400/v1"
         │
    ┌────┴────┐
    ▼          ▼
  Mac Mini   GB-10 設備
  (oMLX)     (dllm + vLLM)
  2-4 用戶   4-8 用戶
```

## 核心特性

- **統一 OpenAI API**：所有平台都是 Port 11400，`/v1/chat/completions`—客戶端只改 `base_url`
- **跨平台**：Mac（MLX）+ NVIDIA（GB-10 / RTX / H100）+ 更多 ARM64 Linux 設備
- **模型管理**：`dllm pull` / `dllm list` / `dllm rm` — 跟 Ollama 一樣直覺
- **多模型載入策略**：常駐 (pinned)、熱載入 (hot)、冷載入 (cold)、備援 (standby)
- **硬體自動感知**：64GB Mac Mini 自動保守配置，128GB DGX Spark 自動最佳化
- **安全審計**：API Key 管理、請求日誌、License 驗證

## 硬體生態系

GB-10（Grace Blackwell）核心已成多家 OEM 的標準規格：

| 廠商 | 設備 | 狀態 |
|------|------|------|
| **NVIDIA** | DGX Spark | ✅ 已出貨 |
| **ASUS** | 未命名 GB-10 設備 | 🔜 |
| **Dell** | GB-10 工作站 | 🔜 |
| **HP** | GB-10 工作站 | 🔜 |
| **銘凡** | GB-10 迷你 PC | 🔜 |
| **Apple** | Mac Mini M4 Pro（64GB） | ✅ 可搭配 oMLX |

所有 GB-10 設備執行相同的 ARM64 Linux binary，不需重新編譯。

**2 個主力模型共 ~27GB：**

| 模型 | 用途 | 記憶體 |
|------|------|--------|
| Qwen3-Coder-30B-A3B | 主力模型（程式開發、企業問答） | ~18GB |
| Qwen2.5-VL-8B | 多模態備用模型 | ~5GB |

## 與 oMLX 的關係

dllm 與 oMLX 是同一個產品理念在不同硬體上的實現：

```
               統一 OpenAI API（Port 11400）
                        │
          ┌─────────────┴─────────────┐
          ▼                           ▼
    oMLX（MLX 引擎）             dllm（vLLM 引擎）
    Apple Silicon Mac            NVIDIA GB-10 / RTX / H100
    2-4 用戶                     4-8 用戶
    原生 CLI                     原生 CLI
```

- Mac 用戶 → oMLX（MLX 原生，Apple 生態最佳化）
- NVIDIA 用戶 → dllm（vLLM 原生，CUDA 生態最佳化）
- **客戶端程式碼完全一致**，只差 `base_url` 指向不同設備

詳見 [docs/OMX_VS_DLLM.md](docs/OMX_VS_DLLM.md)

## 專案結構

```
dllm/
├── crates/              # Rust 核心
│   ├── dllm-shared/     # 共享類型與 trait
│   ├── dllm-core/       # API + Engine Pool + CLI
│   ├── dllm-nvidia/     # NVIDIA 後端（vLLM）
│   └── dllm-mac/        # Mac 後端（MLX）
├── deploy/              # 部署腳本
│   ├── docker/
│   ├── systemd/
│   └── oem/
└── docs/                # 文件
```

## 快速開始

### 開發環境

```bash
# 1. 克隆專案
git clone https://github.com/dllm-project/dllm.git
cd dllm

# 2. 啟動開發環境（Docker Compose）
docker-compose up -d

# 3. 檢查服務狀態
curl http://localhost:11400/health

# 4. 測試 API
curl http://localhost:11400/v1/models
```

### 本地編譯（Rust）

```bash
# 編譯整個工作區
cargo build --workspace

# 執行核心服務
cargo run -p dllm-core -- serve --port 11400

# 執行測試
cargo test --workspace
```

## 文件

- [專案計畫書](PROJECT_PLAN.md) — 開發時程與里程碑
- [架構規格](ARCHITECTURE.md) — 系統設計與技術選型
- [API 規格](API_SPEC.md) — REST API 與 WebSocket 規格
- [部署指南](docs/deployment/README.md) — 生產環境部署

## 貢獻

dllm 還很年輕，任何形式的貢獻都歡迎：

| 貢獻方式 | 說明 |
|---------|------|
| **使用回報** | 你在 GB-10 設備上遇到什麼問題？開個 Issue |
| **程式碼** | Rust、vLLM 整合、模型管理策略 |
| **模型格式支援** | HuggingFace、GGUF、MLX 格式的模型管理 |
| **文件** | 繁體中文、英文、部署指南 |
| **測試** | 在不同 GB-10 設備（ASUS / Dell / 銘凡）上驗證 |

請參閱 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 授權

Apache-2.0

## 緣起

這個專案最初是為了解決一個實際問題：**我的 Mac 用 oMLX，我的 DGX Spark 用 Ollama，兩套 API 格式不一樣，每次遷移都要改程式碼，很煩。**

於是把 Mac 交給 oMLX（它做得很好），在 Spark 上自己寫一套相容 OpenAI API 的執行環境。結果發現 GB-10 這顆核心比我想像的更有潛力——ASUS、Dell、HP、銘凡都在出這個規格。於是決定把它做好、開源，讓所有 GB-10 設備都有一個統一的管理層。

期望有一天，不管是開發者還是企業用戶，插上任何一台設備，`:11400/v1` 永遠可以用。
