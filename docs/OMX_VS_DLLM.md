# oMLX vs dllm 功能對標與進度評估

> **評估日期**：2026-06-23
> **oMLX 版本**：v0.4.4（17k stars，成熟產品）
> **dllm 版本**：v0.1.0-alpha（專案初始化階段）

---

## 一、功能對標總表

### 1.1 核心推理引擎

| 功能 | oMLX | dllm（設計） | dllm（實現） | 差距評估 |
|------|------|-------------|-------------|---------|
| **LLM 推理** | ✅ mlx-lm BatchGenerator | ✅ vLLM (NVIDIA) / MLX (Mac) | ⚠️ 骨架 | oMLX 已成熟；dllm 需接 vLLM subprocess |
| **VLM 推理** | ✅ mlx-vlm，支援多圖輸入 | ✅ 規劃中（vLLM 原生支援） | ❌ 未開始 | vLLM 原生支援多模態，理論上可對標 |
| **OCR 模型** | ✅ DeepSeek-OCR, DOTS-OCR, GLM-OCR | ❌ 未規劃 | ❌ 未開始 | **差距大**：需評估是否優先支援 |
| **Embedding** | ✅ BERT, BGE-M3, ModernBERT | ✅ BGE-M3（規劃） | ❌ 未開始 | 可對標，需實現服務層 |
| **Reranker** | ✅ ModernBERT, XLM-RoBERTa | ❌ 未規劃 | ❌ 未開始 | **差距大**：RAG 準確率會受影響 |
| **Continuous Batching** | ✅ 原生支援 | ✅ vLLM 原生支援 | ⚠️ 透過 vLLM | 可對標，但 mac 端 mlx-lm 需額外處理 |
| **Tiered KV Cache** | ✅ Hot RAM + Cold SSD（safetensors） | ⚠️ 僅 Hot Cache（vLLM PagedAttention） | ❌ 未開始 | **差距極大**：oMLX 核心特色，dllm 無 SSD cache 設計 |
| **Prefix Sharing** | ✅ Copy-on-Write | ✅ vLLM Automatic Prefix Caching | ⚠️ 透過 vLLM | 可對標 |
| **量化支援** | ✅ MLX 格式（4bit/8bit） | ✅ vLLM 支援 AWQ/GPTQ/FP8/INT4 | ⚠️ 透過 vLLM | NVIDIA 端量化選項更多 |

### 1.2 多模型管理（Engine Pool）

| 功能 | oMLX | dllm（設計） | dllm（實現） | 差距評估 |
|------|------|-------------|-------------|---------|
| **多模型同時服務** | ✅ LLM+VLM+Embedding+Reranker | ✅ 規劃中 | ⚠️ 骨架（EnginePool struct） | 架構設計一致 |
| **LRU Eviction** | ✅ 自動卸載最近最少使用 | ✅ 已設計 | ⚠️ 邏輯已寫，未測試 | 可對標 |
| **Manual Load/Unload** | ✅ Admin 面板手動控制 | ✅ API 已設計 | ⚠️ 路由已寫，未接引擎 | 可對標 |
| **Model Pinning** | ✅ 固定模型不被卸載 | ✅ 已設計 | ⚠️ 邏輯已寫 | 可對標 |
| **Per-Model TTL** | ✅ 閒置超時自動卸載 | ✅ 配置已定義 | ❌ 未實現定時檢查 | 小差距，容易補上 |
| **Process Memory Enforcer** | ✅ 系統級 OOM 防護（預設保留 8GB） | ⚠️ 記憶體監控，無系統級保護 | ⚠️ 基礎監控骨架 | **差距中**：dllm 無主動 kill 機制 |
| **記憶體估算** | ✅ 從 safetensors 自動估算 | ✅ 規劃中（model_discovery） | ⚠️ 粗略估算公式 | 可對標 |
| **模型別名（Alias）** | ✅ 自訂 API 可見名稱 | ❌ 未規劃 | ❌ 未開始 | **差距中**：企業用戶需要 |
| **設定檔（Profiles）** | ✅ 同一模型多組設定 | ❌ 未規劃 | ❌ 未開始 | **差距中**：進階功能 |

### 1.3 API 相容性

| 功能 | oMLX | dllm（設計） | dllm（實現） | 差距評估 |
|------|------|-------------|-------------|---------|
| **OpenAI Chat Completions** | ✅ /v1/chat/completions | ✅ 路由已實現 | ⚠️ 回傳假資料 | **核心差距**：需接引擎 |
| **OpenAI Completions** | ✅ /v1/completions | ✅ 路由已實現 | ⚠️ 未實現 | 小差距 |
| **Anthropic Messages** | ✅ /v1/messages | ✅ 路由已實現 | ⚠️ 未實現 | 小差距 |
| **OpenAI Embeddings** | ✅ /v1/embeddings | ✅ 路由已實現 | ⚠️ 未實現 | 小差距 |
| **OpenAI Models** | ✅ /v1/models | ✅ 路由已實現 | ⚠️ 回傳假資料 | 小差距 |
| **Rerank API** | ✅ /v1/rerank | ❌ 未規劃 | ❌ 未開始 | **差距大**：需補 Reranker 引擎 |
| **Streaming（SSE）** | ✅ 支援 usage stats | ✅ 規劃中 | ❌ 未實現 | **核心差距**：Agent 工具必需 |
| **Tool Calling** | ✅ 12+ 格式（JSON/XML/Gemma/GLM/Mistral 等） | ⚠️ Agent 層規劃 | ❌ 未開始 | **差距大**：oMLX 原生支援多格式 |
| **Structured Output** | ✅ JSON Schema 驗證 | ⚠️ 規劃中 | ❌ 未開始 | **差距中**：企業應用必需 |
| **Vision Input** | ✅ base64/URL/file | ⚠️ 規劃中 | ❌ 未開始 | **差距中**：VLM 必需 |
| **API Key 認證** | ✅ 可選啟用 | ✅ 規劃中 | ⚠️ 中間件骨架 | 小差距 |

### 1.4 管理後台（Admin Dashboard）

| 功能 | oMLX | dllm（設計） | dllm（實現） | 差距評估 |
|------|------|-------------|-------------|---------|
| **Web Admin UI** | ✅ 功能完整（React/Vue） | ✅ React 規劃 | ⚠️ Hello World 頁面 | **差距極大**：幾乎從零開始 |
| **內建 Chat UI** | ✅ /admin/chat | ❌ 未規劃 | ❌ 未開始 | **差距大**：oMLX 特色功能 |
| **Real-time Monitoring** | ✅ GPU/記憶體/請求即時圖表 | ✅ WebSocket 規劃 | ⚠️ 骨架 | **差距大**：需接 Prometheus/Grafana |
| **模型管理面板** | ✅ 載入/卸載/固定/設定 | ✅ API 已設計 | ❌ UI 未開始 | **差距大**：Admin 核心功能 |
| **模型下載器** | ✅ HuggingFace 瀏覽+一鍵下載 | ❌ 未規劃 | ❌ 未開始 | **差距中**：提升易用性 |
| **Per-Model 設定** | ✅ 採樣參數/Template/TTL/Alias | ⚠️ 配置系統支援 | ❌ UI 未開始 | **差距中**：需 Admin UI |
| **Performance Benchmark** | ✅ 一鍵測速 | ❌ 未規劃 | ❌ 未開始 | **差距中**：專業用戶需要 |
| **Integrations** | ✅ Claude Code/Cursor 一鍵配置 | ❌ 未規劃 | ❌ 未開始 | **差距中**：降低上手門檻 |
| **Dark/Light Mode** | ✅ 支援 | ❌ 未規劃 | ❌ 未開始 | 小差距 |
| **多語言支援** | ✅ 8+ 語言（含繁中） | ❌ 未規劃 | ❌ 未開始 | **差距中**：台灣市場必需 |
| **離線運作** | ✅ CDN 資源內嵌 | ❌ 未規劃 | ❌ 未開始 | **差距中**：企業內網環境 |

### 1.5 部署與運維

| 功能 | oMLX | dllm（設計） | dllm（實現） | 差距評估 |
|------|------|-------------|-------------|---------|
| **macOS App Bundle** | ✅ Swift/SwiftUI + Menu Bar | ❌ 未規劃（Headless Server） | ❌ 不適用 | **差異化**：dllm 走伺服器路線 |
| **Homebrew 安裝** | ✅ `brew install omlx` | ❌ 未規劃 | ❌ 未開始 | 小差距（可後補） |
| **Docker 部署** | ❌ 無原生支援 | ✅ Docker Compose 完整規劃 | ✅ 5 個 Dockerfile | **dllm 優勢**：oMLX 無 Docker |
| **systemd 服務** | ✅ `omlx start/stop` | ✅ 3 個 service 檔案 | ✅ 已撰寫 | dllm 更完整 |
| **OEM 預裝** | ❌ 無 | ✅ 首次開機腳本 | ✅ 已撰寫 | **dllm 優勢**：企業級 |
| **自動更新** | ✅ Sparkle（macOS） | ❌ 未規劃 | ❌ 未開始 | **差距中**：長期維運必需 |
| **Auto-restart** | ✅ 崩潰自動重啟 | ⚠️ Docker restart policy | ⚠️ 基本支援 | 可對標 |
| **CLI 工具** | ✅ `omlx serve/start/stop` | ✅ `dllm serve` 骨架 | ⚠️ 基礎 CLI | 小差距 |
| **日誌管理** | ✅ 結構化日誌 + 檔案 | ⚠️ tracing 規劃 | ⚠️ 基礎配置 | 小差距 |

### 1.6 RAG 與知識庫（dllm 獨有領域）

| 功能 | oMLX | dllm（設計） | dllm（實現） | 備註 |
|------|------|-------------|-------------|------|
| **文件解析（PDF/DOCX）** | ❌ 無 | ✅ marker / unstructured | ⚠️ 依賴規劃 | **dllm 優勢**：企業必需 |
| **OCR（掃描件）** | ❌ 無 | ✅ surya / easyocr | ❌ 未開始 | **dllm 優勢** |
| **向量資料庫** | ❌ 無 | ✅ Qdrant | ⚠️ Docker 配置 | **dllm 優勢** |
| **混合檢索** | ❌ 無 | ✅ Dense + BM25 + Rerank | ❌ 未開始 | **dllm 優勢** |
| **知識庫管理** | ❌ 無 | ✅ CRUD API 設計 | ⚠️ 骨架 | **dllm 優勢** |
| **NL2SQL** | ❌ 無 | ✅ Agent 層規劃 | ❌ 未開始 | **dllm 優勢** |

### 1.7 Agent 與工具（dllm 獨有領域）

| 功能 | oMLX | dllm（設計） | dllm（實現） | 備註 |
|------|------|-------------|-------------|------|
| **MCP Integration** | ✅ 支援 | ✅ 規劃中 | ⚠️ 骨架 | 可對標 |
| **Tool Calling（多格式）** | ✅ 12+ 解析器 | ⚠️ 基礎工具註冊 | ❌ 未開始 | **差距大** |
| **ReAct Agent Loop** | ❌ 無 | ✅ 規劃中 | ⚠️ 骨架 | **dllm 優勢** |
| **資料庫 Agent** | ❌ 無 | ✅ NL2SQL 規劃 | ❌ 未開始 | **dllm 優勢** |
| **郵件/檔案工具** | ❌ 無 | ✅ 內建工具規劃 | ⚠️ 骨架 | **dllm 優勢** |
| **Workflow Engine** | ❌ 無 | ⚠️ 長期規劃 | ❌ 未開始 | **dllm 優勢** |

### 1.8 雲端與混合（dllm 獨有領域）

| 功能 | oMLX | dllm（設計） | dllm（實現） | 備註 |
|------|------|-------------|-------------|------|
| **混合雲路由** | ❌ 純本地 | ✅ 本地優先 + 雲端 fallback | ⚠️ 骨架 | **dllm 優勢** |
| **雲端供應商支援** | ❌ 無 | ✅ OpenAI/Claude/Gemini/通義 | ❌ 未開始 | **dllm 優勢** |
| **預算控制** | ❌ 無 | ✅ 計費追蹤 + 上限 | ❌ 未開始 | **dllm 優勢** |
| **隱私規則** | ❌ 無 | ✅ 資料不上雲標籤 | ❌ 未開始 | **dllm 優勢** |

---

## 二、進度評估（量化）

### 2.1 整體完成度

| 模組 | 規劃完成度 | 代碼完成度 | 可運作度 |
|------|-----------|-----------|---------|
| **Rust 控制層（dllm-core）** | 90% | 30% | 10% |
| **NVIDIA 後端（dllm-nvidia）** | 80% | 25% | 5% |
| **Mac 後端（dllm-mac）** | 70% | 20% | 5% |
| **RAG 服務（dllm-rag）** | 75% | 15% | 5% |
| **Agent 服務（dllm-agent）** | 70% | 10% | 5% |
| **雲端連接器（dllm-connector）** | 60% | 10% | 0% |
| **管理後台（dllm-admin）** | 50% | 5% | 0% |
| **部署腳本（Docker/systemd）** | 85% | 70% | 30% |
| **文件** | 90% | 100% | 100% |

### 2.2 與 oMLX 的功能覆蓋率

| 類別 | oMLX 功能數 | dllm 規劃數 | dllm 實現數 | 覆蓋率（規劃/實現） |
|------|------------|------------|------------|------------------|
| 核心推理 | 9 | 7 | 2 | 78% / 22% |
| 多模型管理 | 9 | 7 | 3 | 78% / 33% |
| API 相容性 | 10 | 8 | 1 | 80% / 10% |
| Admin 後台 | 11 | 4 | 0 | 36% / 0% |
| 部署運維 | 8 | 7 | 4 | 88% / 50% |
| **RAG/知識庫** | 0 | 6 | 0 | **N/A（dllm 獨有）** |
| **Agent/工具** | 2 | 6 | 1 | **N/A（dllm 獨有）** |
| **雲端混合** | 0 | 4 | 0 | **N/A（dllm 獨有）** |
| **總計（對標功能）** | **47** | **33** | **10** | **70% / 21%** |

---

## 三、核心差距分析

### 3.1 🔴 關鍵差距（必須補上）

1. **Tiered KV Cache（SSD Cache）**
   - oMLX 的核心競爭力，讓長上下文推理快 5-10 倍
   - dllm 目前僅規劃了 vLLM PagedAttention，無 SSD 層
   - **影響**：對於 Agent 場景（反覆修改上下文），TTFT 會非常差
   - **難度**：高（需改動 vLLM 或自研）

2. **Admin Dashboard 完整性**
   - oMLX 的 Admin 是產品體驗的核心，功能極其完整
   - dllm 目前只有一個 Hello World 頁面
   - **影響**：非技術用戶無法使用，違背「中小企業 AI Box」定位
   - **難度**：中（前端開發量大，但技術門檻不高）

3. **Streaming API 實現**
   - Tool Calling、Agent、Chat 都依賴 SSE Streaming
   - dllm 路由已設計，但完全未實現
   - **影響**：基礎功能無法運作
   - **難度**：低（Axum 原生支援 SSE）

4. **Tool Calling 多格式支援**
   - oMLX 支援 12+ 工具調用格式（JSON/XML/Gemma/GLM 等）
   - dllm 僅規劃了基礎工具註冊
   - **影響**：相容性不如 oMLX，部分模型無法使用工具
   - **難度**：中（需實現多種解析器）

### 3.2 🟡 中等差距（重要但可分期）

5. **OCR 模型支援**
   - oMLX 內建 DeepSeek-OCR 等
   - dllm 未規劃，但 RAG 可部分替代（文件解析時 OCR）
   - **建議**：Phase 2 再考慮

6. **Reranker**
   - oMLX 有專用 Reranker 引擎
   - dllm RAG 規劃了重排序，但無專用引擎
   - **建議**：Phase 2 補上

7. **模型下載器**
   - oMLX Admin 可瀏覽 HuggingFace 並一鍵下載
   - dllm 未規劃
   - **建議**：Phase 3 補上（可用 hf-transfer）

8. **多語言支援**
   - oMLX 支援 8+ 語言含繁中
   - dllm 文件為繁中，但 Admin UI 未規劃 i18n
   - **建議**：Phase 3 補上（react-i18next）

9. **自動更新**
   - oMLX 有 Sparkle 自動更新
   - dllm 未規劃
   - **建議**：Phase 4 補上

### 3.3 🟢 dllm 的差異化優勢（無需對標）

| 優勢 | 說明 |
|------|------|
| **跨平台** | Mac + NVIDIA + 未來消費級，oMLX 僅 Mac |
| **Docker 化** | 完整容器化部署，oMLX 無 Docker |
| **RAG 內建** | 本地知識庫是核心功能，oMLX 無此規劃 |
| **資料庫 Agent** | NL2SQL 連接企業資料庫，oMLX 無 |
| **混合雲** | 本地優先 + 雲端 fallback，oMLX 純本地 |
| **企業級部署** | systemd/OEM/監控/審計，oMLX 面向個人 |
| **統一端口 11400** | 與 oMLX 8000 不同，但與你的其他服務一致 |

---

## 四、建議優先順序

### Phase 0（立即）：基礎可用（2-3 週）

目標：讓 `dllm serve` 能實際回答問題

1. **修復編譯錯誤**：讓 `cargo build --workspace` 成功
2. **實現 SSE Streaming**：`ChatChunk` 串流輸出
3. **接 vLLM 端到端**：`VLLMClient.chat_completion` 真正調用 vLLM
4. **EnginePool 引擎引用**：解決 `DashMap` 生命週期問題（改用 `Arc`）

### Phase 1：功能對齊 oMLX 核心（4-6 週）

目標：達到 oMLX 80% 核心功能

5. **Anthropic API**：`/v1/messages` 請求轉換
6. **Embedding API**：接入 BGE-M3 服務
7. **Tool Calling**：至少支援 JSON `<tool_call>` 格式
8. **Admin Dashboard MVP**：模型列表、載入/卸載、基本監控
9. **Memory Enforcer 完善**：系統級 OOM 防護

### Phase 2：企業級功能（6-8 週）

目標：發揮 dllm 差異化優勢

10. **RAG Pipeline**：文件上傳、解析、Embedding、檢索
11. **Agent ReAct Loop**：真正能查資料庫、發郵件
12. **混合雲路由**：本地失敗自動轉雲端
13. **Reranker / OCR**：補強 RAG 準確率

### Phase 3：產品化（4-6 週）

14. **Admin Dashboard 完整版**：Chat UI、Benchmark、Integrations
15. **模型下載器**：HuggingFace 整合
16. **多語言 / 自動更新**
17. **效能優化**：請求批次合併、KV Cache 共享

---

## 五、風險提示

| 風險 | 等級 | 說明 |
|------|------|------|
| **SSD KV Cache 缺失** | 🔴 高 | 這是 oMLX 的核心競爭力，dllm 若無此功能，在長上下文場景（Agent）會明顯落後。但 vLLM 社群已有相關討論，可追蹤 |
| **Admin UI 開發量** | 🟡 中 | 前端工作量可能佔總開發 30-40%，需評估是否精簡 MVP |
| **vLLM 多實例記憶體碎片** | 🟡 中 | 每個模型一個 vLLM 進程，GPU VRAM 無法動態共享，這是硬體限制，dllm 只能緩解 |
| **Tool Calling 格式爆炸** | 🟡 中 | 12+ 種格式維護成本高，建議先支援最常見的 3-4 種 |
| **跨平台編譯複雜度** | 🟢 低 | Rust 條件編譯成熟，Mac/NVIDIA 分開編譯即可 |

---

## 六、總結

### 6.1 誠實評估

**dllm 目前 vs oMLX：約 20-25% 功能實現度**

- 架構設計和文件品質高於 oMLX 初期（因為有 oMLX 作為參考）
- 但實際可運作功能幾乎為零（僅能啟動 HTTP server 回傳假資料）
- **預估達到 oMLX v0.4.4 功能水準需要 4-6 個月全職開發**

### 6.2 策略建議

1. **不要試圖 100% 複製 oMLX**：oMLX 是 Mac 個人工具，dllm 是企業伺服器，定位不同
2. **優先補上「企業用戶刚需」**：RAG、Agent、混合雲這些 oMLX 沒有的功能
3. **Admin UI 可先用成熟框架**：考慮用 Gradio / Streamlit 做 MVP Admin，而非從頭寫 React
4. **SSD Cache 可暫時擱置**：vLLM PagedAttention 效能已經很好，SSD Cache 是錦上添花
5. **盡快讓「一個完整場景」跑通**：例如「上傳 PDF → 問答」，比 100 個功能都半吊子更有價值

---

*本文件應每兩週更新一次，追蹤進度變化。*
