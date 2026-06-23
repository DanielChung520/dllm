# dllm

> 跨平台統一 LLM 執行環境 —— 中小企業 AI Box 的核心引擎

## 專案概述

**dllm**（Distributed Local LLM Manager）是一套專為中小企業設計的本地化 AI 執行環境。它讓企業能在自有設備上運行大型語言模型，處理本地知識庫與資料庫，同時保持與 OpenAI 完全相容的 API 格式。

## 核心特性

- **統一 API**：所有平台皆暴露相同的 OpenAI-compatible API（Port 11400）
- **跨平台**：Mac（MLX/Metal）+ NVIDIA（CUDA/GB-10/RTX/H100）
- **多模型動態管理**：Engine Pool + LRU eviction，記憶體不足自動卸載
- **內建 RAG**：本地知識庫處理，文件上傳即問即答
- **資料庫 Agent**：NL2SQL，連接企業現有資料庫
- **工具生態**：MCP 整合，可接第三方工具
- **混合雲路由**：本地優先，雲端為輔，預算可控

## ⚠️ 硬體需求（DGX Spark 128GB）

> 本專案以 **128GB DGX Spark** 為首要目標硬體。以下為實際部署測試配置。

**實際載入 4 個模型（共 ~38GB），128GB 非常充裕：**

| 模型 | 用途 | 記憶體 |
|------|------|--------|
| **Qwen3-Coder-30B-A3B** | 程式開發、企業問答（主力） | ~26GB |
| **Qwen2.5-VL-8B** | 圖片辨識、多模態 | ~9GB |
| **BGE-M3** | RAG 嵌入檢索 | ~2GB |
| **Qwen3.5-0.8B** | 備用降載 | ~1GB |
| **總計** | | **~38GB** |

剩餘 ~60GB 提供系統、服務、KV Cache 擴展與緩衝，空間非常充裕。

| 硬體 | 記憶體 | 可載入模型 |
|------|--------|-----------|
| **DGX Spark** | 128GB | 4 個（30B Coder + 8B VLM + Embedding + 0.8B） |
| Mac Studio | 192GB | 更多更大 |
| RTX 5090 | 32GB | 1-2 個小模型 |
| H100 | 80GB+ | 彈性擴展 |

詳見 [GB-10 128GB 配置指南](docs/deployment/GB10_128GB.md)

## 專案結構

```
dllm/
├── crates/              # Rust 核心控制層
│   ├── dllm-shared/     # 共享類型與 trait
│   ├── dllm-core/       # API Gateway + Engine Pool
│   ├── dllm-nvidia/     # NVIDIA 後端適配
│   └── dllm-mac/        # Mac MLX 後端適配
├── services/            # 服務層（Docker）
│   ├── dllm-rag/        # RAG Pipeline
│   ├── dllm-agent/      # Agent Core
│   └── dllm-connector/  # 雲端連接器
├── admin/               # 管理後台
│   └── dllm-admin/      # Web UI
├── deploy/              # 部署腳本
│   ├── docker/
│   ├── systemd/
│   └── oem/
└── docs/                # 文件
    ├── api/
    ├── architecture/
    └── deployment/
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
