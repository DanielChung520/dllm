# API 使用指南

## 基礎資訊

- **Base URL**: `http://localhost:11400/v1`
- **認證**: `Authorization: Bearer {api_key}`
- **內容類型**: `application/json`

## 快速開始

### 1. 健康檢查

```bash
curl http://localhost:11400/health
```

### 2. 列出模型

```bash
curl http://localhost:11400/v1/models \
  -H "Authorization: Bearer your-api-key"
```

### 3. 聊天完成

```bash
curl -X POST http://localhost:11400/v1/chat/completions \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-30b-a3b-4bit",
    "messages": [
      {"role": "user", "content": "你好"}
    ]
  }'
```

### 4. 建立知識庫

```bash
curl -X POST http://localhost:11400/v1/rag/knowledge-bases \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "產品手冊",
    "description": "公司產品說明文件"
  }'
```

### 5. 上傳文件

```bash
curl -X POST http://localhost:11400/v1/rag/knowledge-bases/{kb_id}/documents \
  -H "Authorization: Bearer your-api-key" \
  -F "file=@product-manual.pdf"
```

### 6. RAG 查詢

```bash
curl -X POST http://localhost:11400/v1/rag/query \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "knowledge_base_ids": ["{kb_id}"],
    "query": "產品規格是什麼？"
  }'
```

### 7. 執行 Agent

```bash
curl -X POST http://localhost:11400/v1/agent/run \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "查詢上個月銷售額"}
    ]
  }'
```

## SDK 範例

### Python

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:11400/v1",
    api_key="your-api-key"
)

response = client.chat.completions.create(
    model="qwen3-30b-a3b-4bit",
    messages=[
        {"role": "user", "content": "你好"}
    ]
)

print(response.choices[0].message.content)
```

### JavaScript/TypeScript

```typescript
import OpenAI from 'openai';

const client = new OpenAI({
  baseURL: 'http://localhost:11400/v1',
  apiKey: 'your-api-key',
});

const response = await client.chat.completions.create({
  model: 'qwen3-30b-a3b-4bit',
  messages: [
    { role: 'user', content: '你好' }
  ],
});

console.log(response.choices[0].message.content);
```

## 進階功能

### 串流回應

```bash
curl -X POST http://localhost:11400/v1/chat/completions \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-30b-a3b-4bit",
    "messages": [{"role": "user", "content": "你好"}],
    "stream": true
  }'
```

### 工具調用

```bash
curl -X POST http://localhost:11400/v1/chat/completions \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-30b-a3b-4bit",
    "messages": [{"role": "user", "content": "查詢資料庫"}],
    "tools": [
      {
        "type": "function",
        "function": {
          "name": "query_database",
          "description": "查詢資料庫",
          "parameters": {
            "type": "object",
            "properties": {
              "sql": {"type": "string"}
            }
          }
        }
      }
    ]
  }'
```

### 混合雲路由

當本地模型無法處理請求時，自動路由到雲端：

```bash
curl -X POST http://localhost:11400/v1/chat/completions \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "複雜推理問題"}]
  }'
```

> 注意：雲端路由需在配置中啟用並設定供應商。
