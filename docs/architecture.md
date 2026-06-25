# 架構概述

## 系統架構

```
Client → Port 11400
         │
         ├── dllm-core（Rust, 3.1MB）
         │     ├── OpenAI API 路由
         │     ├── Engine Pool（多模型 LRU）
         │     ├── API Key 驗證
         │     └── 審計日誌
         │
         └── vLLM（推理引擎）
               ├── PagedAttention
               ├── Continuous Batching
               └── GPU 後端
```

## 分層說明

| 層級 | 技術 | 說明 |
|------|------|------|
| **API 層** | Axum (Rust) | OpenAI-compatible HTTP 伺服器，Port 11400 |
| **引擎池** | Engine Pool | 多模型載入、LRU eviction、pin/unpin |
| **載入策略** | Pinned / Hot / Standby | 常駐、熱載入、備援降級 |
| **推理引擎** | vLLM | PagedAttention、Continuous Batching |
| **GPU 後端** | CUDA / ROCm / XPU | 執行時期自動偵測 |

## 核心模組

| 模組 | 說明 |
|------|------|
| `dllm-core` | API 伺服器、Engine Pool、CLI、模型管理 |
| `dllm-shared` | 共享類型、trait、License、Token 計算 |
| `dllm-nvidia` | NVIDIA vLLM 後端（條件編譯） |
| `dllm-mac` | Mac MLX 後端（條件編譯） |

## 硬體支援

dllm 執行時期自動偵測 GPU 後端：

| 後端 | 偵測方式 | pip 套件 |
|------|---------|---------|
| NVIDIA CUDA | `nvidia-smi` | `vllm` |
| AMD ROCm | `rocm-smi` | `vllm-rocm` |
| Intel XPU | `xpu-smi` | `vllm-intel` |
| CPU only | 無 GPU | `vllm` |

可透過 `dllm config set backend <auto|nvidia|amd|intel>` 手動指定。
