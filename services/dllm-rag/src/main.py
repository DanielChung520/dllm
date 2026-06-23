"""
dllm-rag: RAG Pipeline Service
文件處理、Embedding、向量檢索、混合檢索
"""

import os, uuid, json
from datetime import datetime
from typing import List, Optional, Dict, Any
from pathlib import Path

from fastapi import FastAPI, UploadFile, File, HTTPException
from pydantic import BaseModel, Field
import numpy as np

app = FastAPI(title="dllm-rag", version="0.1.0")

QDRANT_URL = os.environ.get("QDRANT_URL", "http://localhost:6333")
EMBEDDING_MODEL = os.environ.get("EMBEDDING_MODEL", "all-MiniLM-L6-v2")
CHUNK_SIZE = int(os.environ.get("CHUNK_SIZE", "512"))
CHUNK_OVERLAP = int(os.environ.get("CHUNK_OVERLAP", "128"))
TOP_K = int(os.environ.get("TOP_K", "5"))
DOCUMENTS_DIR = Path("/tmp/rag-docs")

class KnowledgeBaseCreate(BaseModel):
    name: str
    description: Optional[str] = None

class KnowledgeBaseResponse(BaseModel):
    id: str; name: str; status: str; document_count: int; created_at: str

class DocumentUploadResponse(BaseModel):
    id: str; filename: str; status: str; chunks_count: int; uploaded_at: str

class RagQueryRequest(BaseModel):
    knowledge_base_ids: List[str]
    query: str
    top_k: int = Field(default=TOP_K, ge=1, le=50)
    rerank: bool = True
    hybrid_search: bool = True

class RagSource(BaseModel):
    document_id: str; filename: str
    page: Optional[int] = None; chunk_text: str; score: float

class RagQueryResponse(BaseModel):
    answer: str; sources: List[RagSource]; usage: Dict[str, int]

knowledge_bases: Dict[str, KnowledgeBaseResponse] = {}
documents: Dict[str, Any] = {}
embedding_model = None
qdrant_client = None

@app.on_event("startup")
async def startup():
    global embedding_model, qdrant_client
    try:
        from sentence_transformers import SentenceTransformer
        embedding_model = SentenceTransformer(EMBEDDING_MODEL, cache_folder="/tmp/sentence-transformers")
        app.state.embedding_dim = embedding_model.get_sentence_embedding_dimension()
        print(f"Embedding '{EMBEDDING_MODEL}' 載入完成 (dim={app.state.embedding_dim})")
    except Exception as e:
        print(f"Embedding 載入失敗: {e}")
        embedding_model = None
        app.state.embedding_dim = 1024
    try:
        from qdrant_client import QdrantClient
        qdrant_client = QdrantClient(url=QDRANT_URL)
        print(f"Qdrant 連線成功: {QDRANT_URL}")
    except Exception as e:
        print(f"Qdrant 連線失敗: {e}")
        qdrant_client = None
    DOCUMENTS_DIR.mkdir(parents=True, exist_ok=True)

def get_embedding():
    global embedding_model
    if embedding_model is None:
        from sentence_transformers import SentenceTransformer
        embedding_model = SentenceTransformer(EMBEDDING_MODEL, cache_folder="/tmp/sentence-transformers")
    return embedding_model

def get_qdrant():
    global qdrant_client
    if qdrant_client is None:
        from qdrant_client import QdrantClient
        qdrant_client = QdrantClient(url=QDRANT_URL)
    return qdrant_client

def embed_texts(texts: List[str]) -> np.ndarray:
    return get_embedding().encode(texts, normalize_embeddings=True, show_progress_bar=False)

def extract_text(filepath: str) -> List[Dict[str, Any]]:
    ext = Path(filepath).suffix.lower()
    if ext == ".pdf":
        import fitz
        doc = fitz.open(filepath)
        pages = [{"page": i+1, "text": p.get_text().strip()} for i, p in enumerate(doc) if p.get_text().strip()]
        doc.close()
        return pages
    elif ext in (".docx", ".doc"):
        from docx import Document
        doc = Document(filepath)
        text = "\n".join(p.text for p in doc.paragraphs if p.text.strip())
        return [{"page": 1, "text": text}]
    elif ext in (".txt", ".md", ".rst"):
        with open(filepath, "r", encoding="utf-8", errors="replace") as f:
            return [{"page": 1, "text": f.read()}]
    return []

def chunk_text(pages: List[Dict]) -> List[Dict]:
    chunks, buffer = [], ""
    for p in pages:
        buffer += ("\n" + p["text"]) if buffer else p["text"]
        while len(buffer) > CHUNK_SIZE:
            chunks.append({"page": p["page"], "text": buffer[:CHUNK_SIZE]})
            buffer = buffer[CHUNK_SIZE - CHUNK_OVERLAP:]
    if buffer.strip():
        chunks.append({"page": pages[-1]["page"] if pages else 1, "text": buffer})
    return chunks

@app.get("/health")
async def health():
    return {"status": "healthy", "service": "dllm-rag"}

@app.post("/v1/rag/knowledge-bases", response_model=KnowledgeBaseResponse)
async def create_kb(request: KnowledgeBaseCreate):
    kb_id = f"kb-{uuid.uuid4().hex[:12]}"
    from qdrant_client.http.models import VectorParams, Distance
    try:
        get_qdrant().recreate_collection(
            collection_name=kb_id,
            vectors_config=VectorParams(size=getattr(app.state, "embedding_dim", 1024), distance=Distance.COSINE),
        )
    except: pass
    kb = KnowledgeBaseResponse(id=kb_id, name=request.name, status="ready", document_count=0, created_at=datetime.utcnow().isoformat())
    knowledge_bases[kb_id] = kb
    return kb

@app.post("/v1/rag/knowledge-bases/{kb_id}/documents", response_model=DocumentUploadResponse)
async def upload_document(kb_id: str, file: UploadFile = File(...)):
    if kb_id not in knowledge_bases:
        raise HTTPException(404, "知識庫不存在")
    doc_id = f"doc-{uuid.uuid4().hex[:12]}"
    filename = file.filename or "unknown"
    filepath = DOCUMENTS_DIR / f"{doc_id}_{filename}"
    with open(filepath, "wb") as f:
        f.write(await file.read())
    pages = extract_text(str(filepath))
    if not pages:
        raise HTTPException(400, f"無法解析: {filename}")
    chunks = chunk_text(pages)
    try:
        texts = [c["text"] for c in chunks]
        embeddings = embed_texts(texts)
        from qdrant_client.http.models import PointStruct
        points = [PointStruct(id=hash(f"{doc_id}_{i}")%(2**63), vector=emb.tolist(), payload={"doc_id":doc_id,"filename":filename,"page":c["page"],"text":c["text"]})
                  for i, (c, emb) in enumerate(zip(chunks, embeddings))]
        get_qdrant().upsert(collection_name=kb_id, points=points)
    except Exception as e:
        print(f"向量儲存失敗: {e}")
    documents[doc_id] = {"id":doc_id, "filename":filename, "status":"ready", "chunks_count":len(chunks)}
    knowledge_bases[kb_id].document_count += 1
    return DocumentUploadResponse(id=doc_id, filename=filename, status="ready", chunks_count=len(chunks), uploaded_at=datetime.utcnow().isoformat())

@app.post("/v1/rag/query", response_model=RagQueryResponse)
async def query_rag(request: RagQueryRequest):
    print(f"RAG query: {request.knowledge_base_ids} | q={request.query[:50]}")
    query_emb = embed_texts([request.query])[0]
    print(f"Embedding dim={len(query_emb)}")
    all_results = []
    for kb_id in request.knowledge_base_ids:
        try:
            resp = get_qdrant().query_points(collection_name=kb_id, query=query_emb.tolist(), limit=request.top_k)
            print(f"Qdrant search '{kb_id}': {len(resp.points) if resp.points else 0} results")
            for hit in (resp.points or []):
                p = hit.payload or {}
                print(f"  hit score={hit.score:.3f} text={p.get('text','')[:40]}")
                all_results.append(RagSource(document_id=p.get("doc_id",""), filename=p.get("filename",""), page=p.get("page"), chunk_text=p.get("text",""), score=hit.score))
        except Exception as e:
            print(f"Qdrant error: {e}")
    all_results.sort(key=lambda x: x.score, reverse=True)
    all_results = all_results[:request.top_k]
    context = "\n\n".join(f"[{s.filename}" + (f" p.{s.page}" if s.page else "") + f"]\n{s.chunk_text}" for s in all_results)
    prompt = f"根據文件回答。若無相關資訊，誠實回答找不到。\n\n{context}\n\n問題：{request.query}\n回答："
    import httpx
    dllm_core_url = os.environ.get("DLLM_CORE_URL", "http://localhost:11400")
    try:
        r = await httpx.AsyncClient().post(f"{dllm_core_url}/v1/chat/completions",
            json={"model":"default","messages":[{"role":"user","content":prompt}],"temperature":0.3,"max_tokens":1024}, timeout=60)
        answer = r.json().get("choices",[{}])[0].get("message",{}).get("content","")
    except Exception as e:
        answer = f"（LLM 生成失敗: {e}）"
    return RagQueryResponse(answer=answer, sources=all_results, usage={"retrieval_tokens":len(context.split()),"generation_tokens":len(answer.split()),"total_tokens":0})

@app.get("/v1/rag/knowledge-bases")
async def list_kb():
    return {"object":"list", "data": list(knowledge_bases.values())}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
