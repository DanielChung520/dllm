# dllm: Ollama alternative built on vLLM for NVIDIA GB-10 / DGX Spark

**TL;DR**: dllm is a lightweight LLM runtime that gives you `ollama pull`-like model management on top of **vLLM**. Think "oMLX for NVIDIA". OpenAI-compatible API on port 11400.

## Why I built this

I use oMLX on my Mac (MLX backend, OpenAI API) and wanted the same experience on my **DGX Spark**. Ollama works but uses a custom API format, and raw vLLM has no model management (`dllm pull` / `dllm list`).

So I built a thin Rust layer (3.1MB binary, no Python runtime) around vLLM that provides:

- **`dllm pull` / `dllm list` / `dllm rm`** — like `ollama pull`
- **Multi-model loading** — pinned / hot / standby strategies
- **OpenAI-compatible API** — `/v1/chat/completions`, SSE streaming
- **API key auth + audit logging** — enterprise ready
- **Hardware auto-detection** — 64GB → conservative, 128GB → optimized
- **Only 3.1MB static binary**

## GB-10 ecosystem

GB-10 (Grace Blackwell) is becoming a standard across multiple vendors:

| Vendor | Device | Status |
|--------|--------|--------|
| **NVIDIA** | DGX Spark | ✅ Shipping |
| **ASUS** | GB-10 mini PC | 🔜 |
| **Dell** | GB-10 workstation | 🔜 |
| **HP** | GB-10 workstation | 🔜 |
| **Minisforum** | GB-10 mini PC | 🔜 |

All run the same ARM64 Linux binary.

## Quick start

```bash
git clone https://github.com/DanielChung520/dllm.git
cd dllm
cargo build --release
cp target/release/dllm ~/.local/bin/

dllm pull Qwen/Qwen3-Coder-30B-A3B-Instruct
dllm serve --port 11400
dllm run Qwen3-Coder-30B-A3B-Instruct
```

## Benchmark (DGX Spark, Qwen2.5-0.5B)

| Metric | Result |
|--------|--------|
| Throughput | 151 TPS |
| Avg latency | 0.29s |
| P95 latency | 0.38s |
| 20 requests success rate | 100% |

## Next

- vLLM process management (auto-start engines)
- Token counting  
- Package distribution (.deb / RPM)

**GitHub**: https://github.com/DanielChung520/dllm

Would love feedback from other DGX Spark / GB-10 users!
