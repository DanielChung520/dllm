# API 參考

## 基本資訊

- **Base URL**: `http://localhost:11400/v1`
- **Format**: OpenAI-compatible JSON
- **Streaming**: Server-Sent Events (SSE)

## 端點

### 健康檢查

```http
GET /health
```

回應：
```json
{"status":"healthy","version":"0.1.0-alpha","timestamp":"..."}
```

### 列出模型

```http
GET /v1/models
```

回應：
```json
{
  "object": "list",
  "data": [
    {"id": "模型名稱", "object": "model", ...}
  ]
}
```

### 聊天完成

```http
POST /v1/chat/completions
Content-Type: application/json
```

請求：
```json
{
  "model": "模型名稱",
  "messages": [
    {"role": "system", "content": "設定"},
    {"role": "user", "content": "你好"}
  ],
  "temperature": 0.7,
  "max_tokens": 2048,
  "stream": false
}
```

非串流回應：
```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "模型名稱",
  "choices": [{
    "index": 0,
    "message": {"role": "assistant", "content": "你好！"},
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 5,
    "total_tokens": 15
  }
}
```

串流回應（`stream: true`）：
```
data: {"id":"xxx","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"你"},"finish_reason":null}]}

data: {"id":"xxx","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"好"},"finish_reason":null}]}

data: [DONE]
```

### 嵌入

```http
POST /v1/embeddings
Content-Type: application/json
```

```json
{
  "model": "模型名稱",
  "input": "要嵌入的文字"
}
```

### 模型管理

```http
POST /v1/models/{model_id}/load     # 載入模型
POST /v1/models/{model_id}/unload   # 卸載模型
POST /v1/models/{model_id}/pin      # 固定模型
POST /v1/models/{model_id}/unpin    # 解除固定
```

### Token 計算

```http
POST /v1/tokenize
Content-Type: application/json
```

```json
{
  "model": "模型名稱",
  "text": "要計算的文字"
}
```

### 系統狀態

```http
GET /v1/system/status
```

回應包含 GPU、記憶體、模型載入狀況。

## SDK 範例

### Python

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:11400/v1",
    api_key="not-needed"
)

response = client.chat.completions.create(
    model="模型名稱",
    messages=[{"role": "user", "content": "你好"}]
)
print(response.choices[0].message.content)
```

### JavaScript

```javascript
import OpenAI from 'openai';

const client = new OpenAI({
  baseURL: 'http://localhost:11400/v1',
  apiKey: 'not-needed',
});

const response = await client.chat.completions.create({
  model: '模型名稱',
  messages: [{ role: 'user', content: '你好' }],
});
```

### curl

```bash
curl http://localhost:11400/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "模型名稱",
    "messages": [{"role": "user", "content": "你好"}]
  }'
```

## 與 OpenAI API 的差異

| 項目 | OpenAI | dllm |
|------|--------|------|
| Base URL | `https://api.openai.com/v1` | `http://localhost:11400/v1` |
| 認證 | API Key 必填 | 可選（無 key 時跳過驗證） |
| 模型 | `gpt-4o` 等 | 本地模型路徑或名稱 |
| 串流 | SSE | SSE |
