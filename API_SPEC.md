# dllm API 規格文件

> **版本**：v0.1.0-alpha
> **Base URL**：`http://localhost:11400/v1`
> **認證**：`Authorization: Bearer {api_key}`

---

## 一、OpenAI-compatible API

### 1.1 Chat Completions

```http
POST /v1/chat/completions
Content-Type: application/json
Authorization: Bearer {api_key}
```

**Request Body**：

```json
{
  "model": "qwen3-30b-a3b-4bit",
  "messages": [
    {"role": "system", "content": "你是一個專業的企業助理"},
    {"role": "user", "content": "幫我查一下上個月的銷售報表"}
  ],
  "temperature": 0.7,
  "max_tokens": 2048,
  "stream": false,
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "query_database",
        "description": "查詢企業資料庫",
        "parameters": {
          "type": "object",
          "properties": {
            "sql": {"type": "string"}
          }
        }
      }
    }
  ],
  "tool_choice": "auto"
}
```

**Response（非串流）**：

```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1719234567,
  "model": "qwen3-30b-a3b-4bit",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "根據資料庫查詢結果，上個月銷售總額為 NT$ 1,234,567..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 45,
    "completion_tokens": 128,
    "total_tokens": 173
  }
}
```

**Response（串流，SSE）**：

```
data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","created":1719234567,"model":"qwen3-30b-a3b-4bit","choices":[{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]}

data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","created":1719234567,"model":"qwen3-30b-a3b-4bit","choices":[{"index":0,"delta":{"content":"根據"},"finish_reason":null}]}

...（持續輸出）...

data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","created":1719234567,"model":"qwen3-30b-a3b-4bit","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

### 1.2 Models

```http
GET /v1/models
Authorization: Bearer {api_key}
```

**Response**：

```json
{
  "object": "list",
  "data": [
    {
      "id": "qwen3-30b-a3b-4bit",
      "object": "model",
      "created": 1719234567,
      "owned_by": "dllm-local",
      "dllm": {
        "status": "loaded",
        "memory_mb": 45056,
        "quantization": "int4",
        "capabilities": ["chat", "tools", "json_mode"],
        "pinned": true
      }
    },
    {
      "id": "bge-m3",
      "object": "model",
      "created": 1719234567,
      "owned_by": "dllm-local",
      "dllm": {
        "status": "available",
        "model_type": "embedding",
        "capabilities": ["embeddings"]
      }
    }
  ]
}
```

### 1.3 Embeddings

```http
POST /v1/embeddings
Content-Type: application/json
Authorization: Bearer {api_key}
```

**Request Body**：

```json
{
  "model": "bge-m3",
  "input": ["這是一段測試文本", "This is a test sentence"],
  "encoding_format": "float"
}
```

**Response**：

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "index": 0,
      "embedding": [0.0123, -0.0456, ...]
    },
    {
      "object": "embedding",
      "index": 1,
      "embedding": [0.0789, -0.0234, ...]
    }
  ],
  "model": "bge-m3",
  "usage": {
    "prompt_tokens": 20,
    "total_tokens": 20
  }
}
```

---

## 二、Admin API（管理用）

### 2.1 系統狀態

```http
GET /v1/system/status
Authorization: Bearer {admin_api_key}
```

**Response**：

```json
{
  "version": "0.1.0",
  "platform": "nvidia-linux",
  "gpu": {
    "name": "NVIDIA GB10",
    "driver_version": "550.90",
    "cuda_version": "12.4",
    "memory_total_mb": 81920,
    "memory_used_mb": 45056,
    "temperature_c": 65,
    "utilization_percent": 45
  },
  "memory": {
    "total_mb": 131072,
    "used_mb": 98304,
    "available_mb": 32768
  },
  "models": {
    "loaded": 2,
    "available": 10,
    "pinned": 1
  },
  "uptime_seconds": 86400
}
```

### 2.2 模型管理

```http
# 手動載入模型
POST /v1/models/{model_id}/load
Authorization: Bearer {admin_api_key}

# 手動卸載模型
POST /v1/models/{model_id}/unload
Authorization: Bearer {admin_api_key}

# 固定模型（不被 LRU 卸載）
POST /v1/models/{model_id}/pin
Authorization: Bearer {admin_api_key}

# 解除固定
POST /v1/models/{model_id}/unpin
Authorization: Bearer {admin_api_key}
```

### 2.3 設定管理

```http
GET /v1/system/config
Authorization: Bearer {admin_api_key}

PUT /v1/system/config
Content-Type: application/json
Authorization: Bearer {admin_api_key}

{
  "memory_guard": "balanced",
  "default_model": "qwen3-30b-a3b-4bit",
  "cloud_fallback": true,
  "cloud_budget_usd": 100.0
}
```

---

## 三、錯誤處理

### 3.1 錯誤格式

```json
{
  "error": {
    "message": "模型記憶體不足，無法載入",
    "type": "insufficient_memory",
    "param": "model_id",
    "code": "memory_limit_exceeded",
    "dllm": {
      "required_mb": 65536,
      "available_mb": 24576,
      "suggested_action": "請卸載其他模型或升級硬體"
    }
  }
}
```

### 3.2 錯誤碼對照

| HTTP Status | Code | 說明 |
|-------------|------|------|
| 400 | `invalid_request` | 請求格式錯誤 |
| 401 | `unauthorized` | API Key 無效 |
| 403 | `forbidden` | 權限不足 |
| 404 | `model_not_found` | 模型不存在 |
| 409 | `model_already_loaded` | 模型已載入 |
| 429 | `rate_limit_exceeded` | 請求過頻 |
| 503 | `engine_overloaded` | 引擎過載 |
| 503 | `insufficient_memory` | 記憶體不足 |
| 504 | `cloud_timeout` | 雲端連接超時 |



*本文件為 dllm API 規格，將隨功能迭代持續更新。*
