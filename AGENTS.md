# AGENTS.md

## Repository Context

This is a **documentation-only** repository. It contains a single architecture reference document and no executable code, build system, or dependencies.

## Facts

- **Single file**: `DLLM_ARCHITECTURE_REFERENCE.md` — architecture blueprint for an dllm-like service on DGX Spark (NVIDIA Grace Blackwell)
- **No build/test/lint**: There is no `package.json`, `pyproject.toml`, `Makefile`, or any other manifest. Do not invent commands.
- **Not a git repo**: No `.git` directory. Do not run git commands unless initializing.
- **Language**: Traditional Chinese (zh-TW). Preserve existing language when editing.

## Conventions

- Keep markdown formatting consistent with the existing file (line dividers `---`, table syntax, code blocks with `bash` labels)
- The document compares dllm (Apple Silicon/MLX) with vLLM (NVIDIA/CUDA) — maintain this framing when making additions
- Date footer at bottom: update if making substantive changes
