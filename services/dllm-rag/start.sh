#!/bin/bash
export DOCUMENTS_DIR=/tmp/rag-docs
export SENTENCE_TRANSFORMERS_HOME=/tmp/sentence-transformers
export HF_HOME=/tmp/huggingface
export DLLM_CORE_URL=http://localhost:11400
mkdir -p /tmp/rag-docs
cd /home/daniel/github/dllm/services/dllm-rag
exec /tmp/vllm-env/bin/python3 -m uvicorn src.main:app --host 0.0.0.0 --port 11402
