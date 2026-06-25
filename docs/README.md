# dllm 文件

> 硬體無關的 LLM 執行層 — 同一套 OpenAI API，管你是 Mac、GB-10 還是 H100。

## 快速連結

| 文件 | 說明 |
|------|------|
| [架構概述](architecture.md) | 系統設計、運作原理、技術選型 |
| [CLI 指令參考](cli.md) | 所有指令的完整用法 |
| [配置說明](config.md) | 設定檔與環境變數 |
| [部署指南](deployment.md) | 從安裝到產品的完整流程 |
| [API 參考](api.md) | OpenAI-compatible API 端點 |
| [合約注意事項](contract-checklist.md) | 商業合約的 AI 撰寫指示 |
| [oMLX 對標分析](OMX_VS_DLLM.md) | dllm 與 oMLX 的功能比較 |

## 快速開始

```bash
# 1. 安裝
git clone https://github.com/DanielChung520/dllm.git
cd dllm
cargo build --release
cp target/release/dllm ~/.local/bin/

# 2. 下載模型
dllm pull Qwen/Qwen3-Coder-30B-A3B-Instruct

# 3. 啟動服務
dllm serve --port 11400

# 4. 對話
dllm run 模型名稱
```

## 專案定位

```
你的應用程式（OpenAI SDK）
         │
         ▼
  base_url = "http://你的設備:11400/v1"
         │
    ┌────┴────┐
    ▼          ▼
  Mac Mini   GB-10 設備
  (oMLX)     (dllm + vLLM)
  2-4 用戶   4-8 用戶
```

dllm 不是要取代 oMLX，而是讓 NVIDIA 設備也有同等級的 LLM 管理體驗。
