#!/bin/bash
# dllm 更新腳本

set -e

VERSION=${1:-latest}
BACKUP_DIR="/opt/dllm/backups/$(date +%Y%m%d_%H%M%S)"

echo "========================================"
echo "dllm 更新程式"
echo "目標版本: $VERSION"
echo "========================================"

# 備份
echo "[1/3] 備份當前版本..."
mkdir -p "$BACKUP_DIR"
cp /usr/local/bin/dllm "$BACKUP_DIR/" 2>/dev/null || true
cp -r /opt/dllm/config "$BACKUP_DIR/" 2>/dev/null || true

# 停止服務
echo "[2/3] 停止服務..."
systemctl stop dllm-core dllm-rag dllm-agent || true

# 更新二進位檔案
echo "[3/3] 更新二進位檔案..."
cp ./dllm /usr/local/bin/
chmod +x /usr/local/bin/dllm

# 重新啟動
echo "重新啟動服務..."
systemctl start dllm-core
sleep 5
systemctl start dllm-rag dllm-agent

echo ""
echo "========================================"
echo "✅ 更新完成！"
echo "========================================"
echo ""
echo "若發生問題，可執行 rollback："
echo "  sudo $BACKUP_DIR/rollback.sh"
echo ""
