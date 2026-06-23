"""dllm-agent: Agent Core Service

負責工具調用、MCP 整合、ReAct Agent 執行。
提供 REST API 供 dllm-core 調用。
"""

from fastapi import FastAPI, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, Field
from typing import List, Optional, Dict, Any, AsyncGenerator
import os
import json
import uuid
from datetime import datetime

app = FastAPI(
    title="dllm-agent",
    description="Agent Core Service for dllm",
    version="0.1.0",
)


# ==================== 模型定義 ====================

class ChatMessage(BaseModel):
    role: str
    content: Optional[str] = None
    tool_calls: Optional[List[Dict[str, Any]]] = None
    tool_call_id: Optional[str] = None
    name: Optional[str] = None


class AgentRunRequest(BaseModel):
    agent_id: Optional[str] = Field(None, description="Agent ID")
    messages: List[ChatMessage] = Field(..., description="對話歷史")
    tools: Optional[List[str]] = Field(None, description="可用工具列表")
    max_iterations: int = Field(10, ge=1, le=50, description="最大迭代次數")
    stream: bool = Field(False, description="是否串流")


class ToolDefinition(BaseModel):
    name: str
    description: str
    parameters: Dict[str, Any]


class ToolCall(BaseModel):
    tool: str
    input: Dict[str, Any]


class ToolResult(BaseModel):
    tool: str
    output: Any


class AgentEvent(BaseModel):
    type: str
    content: Optional[str] = None
    tool: Optional[str] = None
    input: Optional[Dict[str, Any]] = None
    output: Optional[Any] = None
    message: Optional[str] = None


# ==================== 工具註冊表 ====================

class ToolRegistry:
    """工具註冊表"""
    
    def __init__(self):
        self.tools: Dict[str, ToolDefinition] = {}
        self.handlers: Dict[str, callable] = {}
    
    def register(self, tool: ToolDefinition, handler: callable):
        """註冊工具"""
        self.tools[tool.name] = tool
        self.handlers[tool.name] = handler
    
    def get(self, name: str) -> Optional[ToolDefinition]:
        """取得工具定義"""
        return self.tools.get(name)
    
    def list(self) -> List[ToolDefinition]:
        """列出所有工具"""
        return list(self.tools.values())
    
    async def execute(self, name: str, input_data: Dict[str, Any]) -> Any:
        """執行工具"""
        if name not in self.handlers:
            raise ValueError(f"工具 {name} 未註冊")
        return await self.handlers[name](input_data)


# 全局工具註冊表
registry = ToolRegistry()


# ==================== 內建工具 ====================

async def query_database_handler(input_data: Dict[str, Any]) -> Any:
    """查詢資料庫"""
    # TODO: 實現資料庫查詢
    sql = input_data.get("sql", "")
    return {
        "sql": sql,
        "rows": [],
        "summary": "（資料庫查詢尚未實現）",
    }


async def read_file_handler(input_data: Dict[str, Any]) -> Any:
    """讀取文件"""
    # TODO: 實現文件讀取
    path = input_data.get("path", "")
    return {
        "path": path,
        "content": "（文件讀取尚未實現）",
    }


async def send_email_handler(input_data: Dict[str, Any]) -> Any:
    """發送郵件"""
    # TODO: 實現郵件發送
    return {
        "to": input_data.get("to", ""),
        "subject": input_data.get("subject", ""),
        "status": "sent",
        "message": "（郵件發送尚未實現）",
    }


# 註冊內建工具
registry.register(
    ToolDefinition(
        name="query_database",
        description="查詢企業資料庫",
        parameters={
            "type": "object",
            "properties": {
                "sql": {"type": "string", "description": "SQL 查詢語句"},
            },
            "required": ["sql"],
        },
    ),
    query_database_handler,
)

registry.register(
    ToolDefinition(
        name="read_file",
        description="讀取本地文件",
        parameters={
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "文件路徑"},
            },
            "required": ["path"],
        },
    ),
    read_file_handler,
)

registry.register(
    ToolDefinition(
        name="send_email",
        description="發送電子郵件",
        parameters={
            "type": "object",
            "properties": {
                "to": {"type": "string"},
                "subject": {"type": "string"},
                "body": {"type": "string"},
            },
            "required": ["to", "subject", "body"],
        },
    ),
    send_email_handler,
)


# ==================== ReAct Agent ====================

class ReActAgent:
    """ReAct Agent 實現"""
    
    def __init__(self, registry: ToolRegistry, max_iterations: int = 10):
        self.registry = registry
        self.max_iterations = max_iterations
    
    async def run(
        self,
        messages: List[ChatMessage],
        tools: Optional[List[str]] = None,
    ) -> AsyncGenerator[AgentEvent, None]:
        """執行 Agent"""
        
        # 初始思考
        yield AgentEvent(
            type="thought",
            content="分析用戶需求並規劃執行步驟...",
        )
        
        for iteration in range(self.max_iterations):
            # TODO: 實現真正的 ReAct loop
            # 1. Thought: LLM 推理
            # 2. Action: 選擇工具
            # 3. Observation: 執行工具並觀察結果
            
            yield AgentEvent(
                type="thought",
                content=f"迭代 {iteration + 1}/{self.max_iterations}",
            )
            
            # 模擬工具調用
            if iteration == 0:
                yield AgentEvent(
                    type="tool_call",
                    tool="query_database",
                    input={"sql": "SELECT * FROM sales LIMIT 10"},
                )
                
                yield AgentEvent(
                    type="tool_result",
                    tool="query_database",
                    output={"rows": 10, "summary": "總銷售額 NT$ 500,000"},
                )
            
            # 模擬完成
            if iteration >= 1:
                break
        
        yield AgentEvent(
            type="final",
            content="（Agent 執行尚未完整實現）",
        )


# ==================== API 路由 ====================

@app.get("/health")
async def health():
    """健康檢查"""
    return {"status": "healthy", "service": "dllm-agent"}


@app.post("/v1/agent/run")
async def run_agent(request: AgentRunRequest):
    """執行 Agent"""
    if request.stream:
        async def event_generator():
            agent = ReActAgent(registry, request.max_iterations)
            async for event in agent.run(request.messages, request.tools):
                yield f"data: {json.dumps(event.model_dump(), ensure_ascii=False)}\n\n"
            yield "data: [DONE]\n\n"
        
        return StreamingResponse(
            event_generator(),
            media_type="text/event-stream",
        )
    
    # 非串流模式
    agent = ReActAgent(registry, request.max_iterations)
    events = []
    async for event in agent.run(request.messages, request.tools):
        events.append(event)
    
    return {
        "result": events[-1].content if events else "",
        "steps": [e.model_dump() for e in events[:-1]],
    }


@app.get("/v1/agent/tools")
async def list_tools():
    """列出可用工具"""
    return {
        "tools": [tool.model_dump() for tool in registry.list()],
        "mcp_servers": [],  # TODO: 實現 MCP 整合
    }


@app.post("/v1/agent/tools/{tool_name}")
async def execute_tool(tool_name: str, input_data: Dict[str, Any]):
    """直接執行工具"""
    try:
        result = await registry.execute(tool_name, input_data)
        return {"success": True, "result": result}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


# ==================== MCP 整合（TODO）====================

@app.post("/v1/agent/mcp/connect")
async def connect_mcp_server(config: Dict[str, Any]):
    """連接 MCP 伺服器"""
    # TODO: 實現 MCP client
    return {"success": True, "message": "MCP 連接尚未實現"}


@app.get("/v1/agent/mcp/servers")
async def list_mcp_servers():
    """列出已連接的 MCP 伺服器"""
    return {"servers": []}


# ==================== 啟動 ====================

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
