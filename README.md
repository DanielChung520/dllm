# dllm

> NTD 10,000/月，免費借用 Mac Mini 64GB——中小企業的本地 AI Box

## 專案概述

**dllm**（Distributed Local LLM Manager）是一套專為中小企業設計的本地化 LLM 執行環境。軟體租用（NTD 10,000/月）包含一台預裝好的設備，客戶插電即可使用本地 LLM，保持與 OpenAI 完全相容的 API 格式。

## 核心特性

- **統一 API**：所有平台皆暴露相同的 OpenAI-compatible API（Port 11400）
- **跨平台**：Mac（MLX/Metal）+ NVIDIA（CUDA/GB-10/RTX/H100）
- **模型管理**：`dllm pull` / `dllm list` / `dllm rm`，像 Ollama 一樣管理模型
- **多模型載入策略**：常駐（pinned）、熱載入（hot）、冷載入（cold）、備援（standby）
- **硬體自動感知**：Mac Mini 64GB 自動保守配置，DGX Spark 128GB 自動最佳化
- **安全審計**：API Key 管理 + 完整請求日誌

## 硬體選擇指南

> 同一套軟體，兩種硬體選擇。差異只在並發用戶數。

| 規格 | Mac Mini M4 Pro | DGX Spark (GB-10) |
|------|----------------|-------------------|
| **記憶體** | 64GB | 128GB |
| **引擎** | MLX (Metal) | vLLM (CUDA) |
| **並發用戶** | **2-4 人** | **4-8 人** |
| **部署方式** | 原生 CLI | Docker |
| **價格帶** | $2,500-3,000 | $4,000-5,000 |

**2 個主力模型共 ~27GB，兩台都能跑：**

| 模型 | 用途 | 記憶體 |
|------|------|--------|
| Qwen3-Coder-30B-A3B | 主力模型（程式開發、企業問答） | ~18GB |
| Qwen2.5-VL-8B | 多模態備用模型 | ~5GB |

- **64GB Mac Mini**：適合 2-3 人小型團隊
- **128GB DGX Spark**：適合 4-8 人中型團隊

詳見 [硬體規格指南](docs/deployment/GB10_128GB.md)

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

## 授權

Apache-2.0

## 貢獻

歡迎提交 Issue 與 PR！請參閱 [CONTRIBUTING.md](CONTRIBUTING.md)。
