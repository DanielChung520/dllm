#!/bin/bash
# dllm 安裝腳本

set -e

VERSION="0.1.0-alpha"
INSTALL_DIR="/opt/dllm"
USER="dllm"

echo "========================================"
echo "dllm AI Box 安裝程式"
echo "版本: $VERSION"
echo "========================================"

# 檢查 root 權限
if [ "$EUID" -ne 0 ]; then
    echo "請使用 sudo 執行此腳本"
    exit 1
fi

# 建立用戶
echo "[1/5] 建立系統用戶..."
if ! id "$USER" &>/dev/null; then
    useradd -r -s /bin/false -d "$INSTALL_DIR" "$USER"
fi

# 安裝依賴
echo "[2/5] 安裝系統依賴..."
apt-get update
apt-get install -y \
    curl \
    jq \
    docker.io \
    docker-compose \
    nginx \
    systemd \
    python3 \
    python3-pip \
    python3-venv

# 啟動 Docker
systemctl enable docker
systemctl start docker

# 建立目錄
echo "[3/5] 建立目錄結構..."
mkdir -p "$INSTALL_DIR"/{models,config,logs,data}
mkdir -p "$INSTALL_DIR"/data/{documents,cache,backups}
chown -R "$USER:$USER" "$INSTALL_DIR"

# 安裝二進位檔案
echo "[4/5] 安裝二進位檔案..."
cp ./dllm /usr/local/bin/
chmod +x /usr/local/bin/dllm

# 安裝 systemd 服務
echo "[5/5] 安裝系統服務..."
cp ./deploy/systemd/*.service /etc/systemd/system/
systemctl daemon-reload

echo ""
echo "========================================"
echo "✅ 安裝完成！"
echo "========================================"
echo ""
echo "請執行以下命令啟動服務："
echo "  sudo systemctl start dllm-core"
echo ""
echo "查看狀態："
echo "  sudo systemctl status dllm-core"
echo ""
echo "查看日誌："
echo "  sudo journalctl -u dllm-core -f"
echo ""
echo "========================================"
