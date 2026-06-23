# dllm 部署指南

## 系統需求

### 硬體需求

| 規格 | 最低需求 | 建議配置 |
|------|---------|---------|
| CPU | ARM64 / x86-64 | ARM64 (GB-10) |
| GPU | NVIDIA CUDA 相容 | NVIDIA GB-10 / RTX 4090 |
| 記憶體 | 32GB | 128GB 統一記憶體 |
| 儲存空間 | 256GB SSD | 1TB NVMe SSD |
| 網路 | 100Mbps | 1Gbps |

### 軟體需求

- OS: Ubuntu 22.04 LTS 或更新版本
- Docker: 24.0+
- Docker Compose: 2.20+
- NVIDIA Driver: 535+
- NVIDIA Container Toolkit

## 快速部署

### 方法 1：Docker Compose（推薦）

```bash
# 1. 克隆專案
git clone https://github.com/dllm-project/dllm.git
cd dllm

# 2. 建立模型目錄
mkdir -p data/models

# 3. 啟動服務
docker-compose up -d

# 4. 檢查狀態
docker-compose ps
curl http://localhost:11400/health
```

### 方法 2：systemd 服務

```bash
# 1. 執行安裝腳本
sudo ./deploy/oem/install.sh

# 2. 啟動服務
sudo systemctl start dllm-core

# 3. 設定開機自啟
sudo systemctl enable dllm-core
```

### 方法 3：OEM 預裝

首次開機時，系統會自動執行 `deploy/oem/first-boot.sh`：

1. 硬體偵測
2. 目錄結構建立
3. 預設配置生成
4. 模型下載（若有網路）
5. 服務啟動

## 配置說明

### 主要配置檔案

`config/settings.toml`:

```toml
[server]
host = "0.0.0.0"
port = 11400

[engine]
model_dirs = ["/opt/dllm/models"]
memory_guard = "balanced"

[rag]
qdrant_url = "http://localhost:6333"
embedding_model = "BAAI/bge-m3"

[cloud]
enabled = false
privacy_mode = true
```

### 環境變數

| 變數 | 說明 | 預設值 |
|------|------|--------|
| `DLLM_CONFIG_PATH` | 配置檔案路徑 | `/opt/dllm/config/settings.toml` |
| `DLLM_MODEL_DIR` | 模型目錄 | `/opt/dllm/models` |
| `RUST_LOG` | 日誌級別 | `info` |
| `DLLM_PORT` | 服務端口 | `11400` |

## 模型管理

### 模型目錄結構

```
/opt/dllm/models/
├── qwen3-30b-a3b-4bit/
│   ├── config.json
│   ├── model.safetensors.index.json
│   └── *.safetensors
├── bge-m3/
│   └── ...
└── ...
```

### 自動發現

服務啟動時會自動掃描 `model_dirs` 中的模型目錄，辨識模型類型並估算記憶體用量。

### 手動載入

```bash
# 透過 API
curl -X POST http://localhost:11400/v1/models/qwen3-30b-a3b-4bit/load

# 透過 CLI
dllm load --model qwen3-30b-a3b-4bit
```

## 監控與維護

### 查看日誌

```bash
# 核心服務
journalctl -u dllm-core -f

# RAG 服務
journalctl -u dllm-rag -f

# Docker 容器
docker-compose logs -f dllm-core
```

### 性能監控

```bash
# Prometheus 指標
curl http://localhost:11400/v1/system/metrics

# 系統狀態
curl http://localhost:11400/v1/system/status
```

### 備份與還原

```bash
# 備份
sudo ./deploy/oem/backup.sh

# 還原
sudo ./deploy/oem/restore.sh /path/to/backup
```

## 故障排除

### 常見問題

#### 1. 模型載入失敗

**症狀**：`insufficient_memory` 錯誤

**解決**：
- 檢查可用記憶體：`free -h`
- 調整 `memory_guard` 為 `aggressive`
- 卸載其他模型

#### 2. vLLM 啟動超時

**症狀**：`engine_start_failed` 錯誤

**解決**：
- 檢查 NVIDIA 驅動：`nvidia-smi`
- 檢查模型檔案完整性
- 增加超時時間

#### 3. RAG 查詢無結果

**症狀**：空回應或無來源

**解決**：
- 檢查文件是否已處理完成
- 檢查 Qdrant 連線：`curl http://localhost:6333`
- 確認 embedding 模型已載入

## 安全建議

1. **防火牆**：僅開放 11400 與 11401 端口
2. **API Key**：生產環境務必啟用 API Key 驗證
3. **TLS**：使用反向代理（nginx/traefik）啟用 HTTPS
4. **更新**：定期執行更新腳本以取得安全修補

## 升級路徑

### 單機 → 叢集

1. 部署 K8s 叢集
2. 使用 Helm chart 部署 dllm
3. 配置共享儲存（NFS/S3）
4. 設定負載均衡

### GB-10 → H100

1. 更換硬體
2. 更新 NVIDIA 驅動
3. 調整 tensor_parallel_size
4. 啟用多節點推理
