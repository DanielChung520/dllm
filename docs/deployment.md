# 部署指南

## 系統需求

### 硬體

| 需求 | 最低 | 建議 |
|------|------|------|
| CPU | ARM64 / x86-64 | ARM64（GB-10 / DGX Spark） |
| GPU | NVIDIA CUDA / AMD ROCm / Intel XPU | NVIDIA GB-10 / H100 |
| 記憶體 | 32GB | 64GB+（統一記憶體佳） |
| 儲存 | 256GB SSD | 1TB NVMe |
| 網路 | 100Mbps | 1Gbps |

### 軟體

- Linux（Ubuntu 22.04+ 建議）/ macOS
- Python 3.10+（vLLM 需要）
- Rust 1.80+（編譯需要）
- Docker（可選，容器化部署）

## 安裝

### 從原始碼編譯

```bash
git clone https://github.com/DanielChung520/dllm.git
cd dllm
cargo build --release
cp target/release/dllm ~/.local/bin/
```

編譯後只有一個 3.1MB 的二進位檔，不需要 Python runtime。

### 安裝 vLLM（建議使用虛擬環境）

```bash
python3 -m venv vllm-env
source vllm-env/bin/activate

# NVIDIA GPU
pip install vllm

# AMD GPU
pip install vllm-rocm

# Intel GPU
pip install vllm-intel
```

### 設定 HuggingFace Token（加速下載）

1. 到 https://huggingface.co/settings/tokens 申請 token
2. 設定環境變數：

```bash
export HF_TOKEN="hf_你的token"
```

## 快速部署

### 1. 下載模型

```bash
dllm pull Qwen/Qwen3-Coder-30B-A3B-Instruct
```

### 2. 啟動服務

```bash
# 啟動 vLLM 後端
python3 -m vllm.entrypoints.openai.api_server \
  --model ~/.dllm/models/Qwen3-Coder-30B-A3B-Instruct \
  --port 18001 \
  --gpu-memory-utilization 0.80 \
  --enforce-eager

# 啟動 dllm-core（另一終端機）
dllm serve --port 11400
```

### 3. 測試

```bash
curl http://localhost:11400/health
curl http://localhost:11400/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"模型路徑","messages":[{"role":"user","content":"你好"}]}'

# 或直接對話
dllm run 模型名稱
```

## Docker 部署

```yaml
# docker-compose.yml
services:
  dllm-core:
    build: .
    ports:
      - "11400:11400"
    volumes:
      - ./models:/models
    environment:
      - VLLM_URL=http://vllm:8000

  vllm:
    image: vllm/vllm-openai:latest
    runtime: nvidia
    environment:
      - GPU_MEMORY_UTILIZATION=0.80
    volumes:
      - ./models:/models
```

## 效能測試

```bash
# 完整測試（TPS、延遲、穩定性）
python3 benchmark.py

# 只測算力
python3 benchmark.py --mode tps

# 只測延遲
python3 benchmark.py --mode latency

# 測試指定 API
python3 benchmark.py --api http://localhost:11400/v1
```

## 常見問題

### Q: `dllm run` 無法連線

確認 vLLM 是否有在背景運行。預設 dllm-core 會代理請求到 `http://127.0.0.1:18001`。

### Q: 記憶體不足

調整 `dllm config set memory_guard safe`，或降低 `--gpu-memory-utilization`。

### Q: 下載模型很慢

設定 `export HF_TOKEN="hf_你的token"` 可顯著提升下載速度。
