#!/bin/bash
# dllm AI Box 首次開機設定腳本
# 此腳本在設備首次啟動時自動執行

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/var/log/dllm-first-boot.log"

exec > >(tee -a "$LOG_FILE")
exec 2>&1

echo "========================================"
echo "dllm AI Box 首次開機設定"
echo "時間: $(date)"
echo "========================================"

# 1. 硬體檢測
echo "[1/6] 偵測硬體..."
$SCRIPT_DIR/detect-hardware.sh

# 2. 建立目錄結構
echo "[2/6] 建立目錄結構..."
mkdir -p /opt/dllm/{models,config,logs,data}
mkdir -p /opt/dllm/data/{documents,cache,backups}
chown -R dllm:dllm /opt/dllm

# 3. 生成預設配置
echo "[3/6] 生成預設配置..."
if [ ! -f /opt/dllm/config/settings.toml ]; then
    cat > /opt/dllm/config/settings.toml << 'EOF'
[server]
host = "0.0.0.0"
port = 11400
workers = 4

[engine]
model_dirs = ["/opt/dllm/models"]
pinned_models = []
memory_guard = "balanced"
ttl_seconds = 3600
preload_on_startup = false

[rag]
qdrant_url = "http://localhost:6333"
embedding_model = "BAAI/bge-m3"
chunk_size = 512
chunk_overlap = 128

[agent]
max_iterations = 10
timeout_seconds = 300
enabled_tools = ["query_database", "read_file", "send_email"]

[cloud]
enabled = false
privacy_mode = true

[auth]
api_key_required = true
rate_limit_requests_per_minute = 60

[logging]
level = "info"
format = "json"
EOF
fi

# 4. 下載預設模型（若網路可用）
echo "[4/6] 檢查預設模型..."
if ping -c 1 huggingface.co &> /dev/null; then
    echo "網路連線正常，準備下載預設模型..."
    # TODO: 下載預設模型（如 Qwen3-8B）
    echo "預設模型下載功能尚未啟用"
else
    echo "無網路連線，跳過模型下載"
fi

# 5. 啟動服務
echo "[5/6] 啟動系統服務..."
systemctl daemon-reload
systemctl enable dllm-core
systemctl enable dllm-rag
systemctl enable dllm-agent
systemctl enable qdrant

systemctl start qdrant
sleep 5
systemctl start dllm-core
sleep 10
systemctl start dllm-rag
systemctl start dllm-agent

# 6. 驗證狀態
echo "[6/6] 驗證系統狀態..."
sleep 5

if curl -sf http://localhost:11400/health > /dev/null; then
    echo "✅ dllm-core 服務正常"
else
    echo "⚠️ dllm-core 服務未就緒，請檢查日誌"
fi

IP_ADDR=$(hostname -I | awk '{print $1}')

echo ""
echo "========================================"
echo "✅ 設定完成！"
echo "========================================"
echo ""
echo "設備資訊:"
echo "  IP 位址: $IP_ADDR"
echo "  API 端點: http://$IP_ADDR:11400/v1"
echo "  管理後台: http://$IP_ADDR:11401"
echo "  健康檢查: http://$IP_ADDR:11400/health"
echo ""
echo "日誌位置:"
echo "  $LOG_FILE"
echo "  journalctl -u dllm-core -f"
echo ""
echo "========================================"

# 標記首次開機完成
touch /opt/dllm/.first-boot-complete
