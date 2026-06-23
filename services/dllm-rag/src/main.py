"""dllm-rag: RAG Pipeline Service

負責文件處理、Embedding、向量檢索。
提供 REST API 供 dllm-core 調用。
"""

from fastapi import FastAPI, UploadFile, File, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field
from typing import List, Optional, Dict, Any
import os
import uuid
from datetime import datetime

app = FastAPI(
    title="dllm-rag",
    description="RAG Pipeline Service for dllm",
    version="0.1.0",
)


# ==================== 模型定義 ====================

class KnowledgeBaseCreate(BaseModel):
    name: str = Field(..., description="知識庫名稱")
    description: Optional[str] = Field(None, description="知識庫描述")
    embedding_model: str = Field("BAAI/bge-m3", description="嵌入模型")
    chunk_strategy: str = Field("semantic", description="分塊策略")
    metadata: Optional[Dict[str, Any]] = Field(None, description="附加元數據")


class KnowledgeBaseResponse(BaseModel):
    id: str
    name: str
    status: str
    document_count: int
    created_at: str


class RagQueryRequest(BaseModel):
    knowledge_base_ids: List[str] = Field(..., description="知識庫 ID 列表")
    query: str = Field(..., description="查詢文字")
    top_k: int = Field(5, ge=1, le=50, description="返回結果數量")
    rerank: bool = Field(True, description="是否重排序")
    hybrid_search: bool = Field(True, description="是否混合檢索")
    stream: bool = Field(False, description="是否串流")


class RagSource(BaseModel):
    document_id: str
    filename: str
    page: Optional[int] = None
    chunk_text: str
    score: float


class RagQueryResponse(BaseModel):
    answer: str
    sources: List[RagSource]
    usage: Dict[str, int]


class DocumentUploadResponse(BaseModel):
    id: str
    filename: str
    status: str
    chunks_expected: int
    uploaded_at: str


# ==================== 全局狀態 ====================

knowledge_bases: Dict[str, KnowledgeBaseResponse] = {}
documents: Dict[str, Dict[str, Any]] = {}


# ==================== API 路由 ====================

@app.get("/health")
async def health():
    """健康檢查"""
    return {"status": "healthy", "service": "dllm-rag"}


@app.post("/v1/rag/knowledge-bases", response_model=KnowledgeBaseResponse)
async def create_knowledge_base(request: KnowledgeBaseCreate):
    """建立知識庫"""
    kb_id = f"kb-{uuid.uuid4().hex[:12]}"
    
    kb = KnowledgeBaseResponse(
        id=kb_id,
        name=request.name,
        status="ready",
        document_count=0,
        created_at=datetime.utcnow().isoformat(),
    )
    
    knowledge_bases[kb_id] = kb
    
    # TODO: 在 Qdrant 建立 collection
    
    return kb


@app.get("/v1/rag/knowledge-bases")
async def list_knowledge_bases():
    """列出所有知識庫"""
    return {
        "object": "list",
        "data": list(knowledge_bases.values()),
    }


@app.get("/v1/rag/knowledge-bases/{kb_id}")
async def get_knowledge_base(kb_id: str):
    """取得知識庫資訊"""
    if kb_id not in knowledge_bases:
        raise HTTPException(status_code=404, detail="知識庫不存在")
    return knowledge_bases[kb_id]


@app.post("/v1/rag/knowledge-bases/{kb_id}/documents", response_model=DocumentUploadResponse)
async def upload_document(kb_id: str, file: UploadFile = File(...)):
    """上傳文件到知識庫"""
    if kb_id not in knowledge_bases:
        raise HTTPException(status_code=404, detail="知識庫不存在")
    
    doc_id = f"doc-{uuid.uuid4().hex[:12]}"
    
    # TODO: 儲存文件並異步處理
    # 1. 儲存原始文件
    # 2. 解析文本
    # 3. 分塊
    # 4. Embedding
    # 5. 存入 Qdrant
    
    doc = DocumentUploadResponse(
        id=doc_id,
        filename=file.filename or "unknown",
        status="processing",
        chunks_expected=50,
        uploaded_at=datetime.utcnow().isoformat(),
    )
    
    documents[doc_id] = doc.model_dump()
    
    # 更新知識庫文件數
    kb = knowledge_bases[kb_id]
    kb.document_count += 1
    
    return doc


@app.post("/v1/rag/query", response_model=RagQueryResponse)
async def query_rag(request: RagQueryRequest):
    """RAG 查詢"""
    # TODO: 實現檢索邏輯
    # 1. Query embedding
    # 2. 向量檢索
    # 3. 混合檢索（BM25）
    # 4. 重排序
    # 5. 組裝上下文
    # 6. 調用 LLM 生成答案
    
    return RagQueryResponse(
        answer="（RAG 查詢尚未實現）",
        sources=[],
        usage={"retrieval_tokens": 0, "generation_tokens": 0, "total_tokens": 0},
    )


@app.delete("/v1/rag/knowledge-bases/{kb_id}")
async def delete_knowledge_base(kb_id: str):
    """刪除知識庫"""
    if kb_id not in knowledge_bases:
        raise HTTPException(status_code=404, detail="知識庫不存在")
    
    # TODO: 刪除 Qdrant collection
    
    del knowledge_bases[kb_id]
    return {"success": True, "message": f"知識庫 {kb_id} 已刪除"}


@app.get("/v1/rag/documents/{doc_id}")
async def get_document(doc_id: str):
    """取得文件資訊"""
    if doc_id not in documents:
        raise HTTPException(status_code=404, detail="文件不存在")
    return documents[doc_id]


@app.delete("/v1/rag/documents/{doc_id}")
async def delete_document(doc_id: str):
    """刪除文件"""
    if doc_id not in documents:
        raise HTTPException(status_code=404, detail="文件不存在")
    
    # TODO: 從 Qdrant 刪除對應向量
    
    del documents[doc_id]
    return {"success": True, "message": f"文件 {doc_id} 已刪除"}


# ==================== 啟動 ====================

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
