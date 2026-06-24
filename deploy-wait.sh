#!/bin/bash
# dllm 郵件自動部署腳本
# 監控模型下載，完成後自動啟動 vLLM

MODEL_DIR="/home/daniel/.dllm/models"
MAIN_MODEL="Qwen3-Coder-30B-A3B-Instruct"
VL_MODEL="Qwen2.5-VL-7B-Instruct"

echo "=== dllm 部署監控 $(date) ==="
echo "等待主力模型 $MAIN_MODEL 下載完成..."

# 等待 model.safetensors.index.json 出現
while [ ! -f "$MODEL_DIR/$MAIN_MODEL/model.safetensors.index.json" ]; do
  size=$(du -sh "$MODEL_DIR/$MAIN_MODEL/" 2>/dev/null | cut -f1)
  echo "$(date +%H:%M) 30B: $size"
  sleep 30
done

echo "✅ 30B 模型就緒！"
echo "開始啟動 vLLM..."

# 停止舊服務
pkill -f "vllm.entrypoints" 2>/dev/null
sleep 3

# 啟動新 vLLM
PATH="$HOME/.local/bin:$PATH" CPATH="/tmp/vllm-env/include" \
nohup /tmp/vllm-env/bin/python3 -m vllm.entrypoints.openai.api_server \
  --model "$MODEL_DIR/$MAIN_MODEL" \
  --port 18001 \
  --gpu-memory-utilization 0.80 \
  --max-model-len 8192 \
  --enforce-eager > /tmp/vllm_30b.log 2>&1 &

echo "vLLM PID: $!"
echo "等待服務就緒..."
sleep 60

# 測試
curl -s http://127.0.0.1:18001/v1/models 2>/dev/null && echo "✅ 服務就緒" || echo "❌ 服務啟動失敗"

# 執行快速測試
/tmp/vllm-env/bin/python3 -c "
import urllib.request, json, time
t0=time.time()
req=urllib.request.Request('http://localhost:18001/v1/chat/completions',
  data=json.dumps({'model':'$MODEL_DIR/$MAIN_MODEL','messages':[{'role':'user','content':'你好！'}],'max_tokens':50}).encode(),
  headers={'Content-Type':'application/json'})
resp=json.loads(urllib.request.urlopen(req, timeout=120).read())
print(f\"✅ 第一段對話耗時: {time.time()-t0:.2f}s\")
print(f\"回應: {resp['choices'][0]['message']['content'][:60]}\")
" 2>&1

echo "=== 部署完成 ==="
