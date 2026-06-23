//! API 文件路由

use axum::response::Html;

pub async fn handler() -> Html<String> {
    Html(String::from(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>dllm API 文件</title>
    <meta charset="utf-8">
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; max-width: 900px; margin: 0 auto; padding: 40px; }
        h1 { color: #333; }
        h2 { color: #666; margin-top: 40px; }
        code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }
        pre { background: #f4f4f4; padding: 16px; border-radius: 8px; overflow-x: auto; }
        .endpoint { background: #e8f4f8; padding: 16px; border-radius: 8px; margin: 16px 0; }
        .method { font-weight: bold; color: #0066cc; }
    </style>
</head>
<body>
    <h1>dllm API 文件</h1>
    <p>版本: v0.1.0-alpha | Base URL: <code>http://localhost:11400/v1</code></p>
    
    <h2>OpenAI-compatible API</h2>
    <div class="endpoint">
        <span class="method">POST</span> /v1/chat/completions
    </div>
    <div class="endpoint">
        <span class="method">GET</span> /v1/models
    </div>
    <div class="endpoint">
        <span class="method">POST</span> /v1/embeddings
    </div>
    
    <h2>RAG API</h2>
    <div class="endpoint">
        <span class="method">POST</span> /v1/rag/knowledge-bases
    </div>
    <div class="endpoint">
        <span class="method">POST</span> /v1/rag/query
    </div>
    
    <h2>Agent API</h2>
    <div class="endpoint">
        <span class="method">POST</span> /v1/agent/run
    </div>
    <div class="endpoint">
        <span class="method">GET</span> /v1/agent/tools
    </div>
    
    <h2>管理 API</h2>
    <div class="endpoint">
        <span class="method">GET</span> /v1/system/status
    </div>
    <div class="endpoint">
        <span class="method">GET</span> /v1/system/metrics
    </div>
    
    <p>詳細規格請參閱 <a href="https://github.com/dllm-project/dllm/blob/main/API_SPEC.md">API_SPEC.md</a></p>
</body>
</html>
        "#
    ))
}
