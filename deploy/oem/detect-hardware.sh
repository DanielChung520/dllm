#!/bin/bash
# 硬體偵測腳本

echo "=== 硬體偵測報告 ==="
echo "時間: $(date)"
echo ""

# CPU
echo "[CPU]"
if [ -f /proc/cpuinfo ]; then
    grep "model name" /proc/cpuinfo | head -1 | sed 's/model name.*: //'
    grep "cpu cores" /proc/cpuinfo | head -1 | sed 's/cpu cores.*: //'
fi

# 記憶體
echo ""
echo "[記憶體]"
if [ -f /proc/meminfo ]; then
    grep "MemTotal" /proc/meminfo
    grep "MemAvailable" /proc/meminfo
fi

# GPU
echo ""
echo "[GPU]"
if command -v nvidia-smi &> /dev/null; then
    nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader
else
    echo "未偵測到 NVIDIA GPU"
fi

# 儲存空間
echo ""
echo "[儲存空間]"
df -h / | tail -1

# 作業系統
echo ""
echo "[作業系統]"
if [ -f /etc/os-release ]; then
    source /etc/os-release
    echo "名稱: $NAME"
    echo "版本: $VERSION"
fi

echo ""
echo "=== 偵測完成 ==="
