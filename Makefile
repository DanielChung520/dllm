# dllm 統一構建系統

.PHONY: all build test clean dev up down lint fmt check docs

# 預設目標
all: build

# 構建
build:
	@echo "=== 構建 Rust 工作區 ==="
	cargo build --workspace

build-release:
	@echo "=== 構建 Release 版本 ==="
	cargo build --workspace --release

# 測試
test:
	@echo "=== 執行測試 ==="
	cargo test --workspace

test-verbose:
	@echo "=== 執行測試（詳細輸出）==="
	cargo test --workspace -- --nocapture

# 程式碼品質
lint:
	@echo "=== Clippy 檢查 ==="
	cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	@echo "=== 格式化程式碼 ==="
	cargo fmt --all

fmt-check:
	@echo "=== 檢查格式 ==="
	cargo fmt --all -- --check

check: fmt-check lint
	@echo "=== 編譯檢查 ==="
	cargo check --workspace

# 開發環境
dev:
	@echo "=== 啟動開發環境 ==="
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d

dev-build:
	@echo "=== 建構並啟動開發環境 ==="
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d --build

up:
	@echo "=== 啟動服務 ==="
	docker-compose up -d

down:
	@echo "=== 停止服務 ==="
	docker-compose down

logs:
	docker-compose logs -f

# 清理
clean:
	@echo "=== 清理建構產物 ==="
	cargo clean
	rm -rf target/
	find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete 2>/dev/null || true

# 文件
docs:
	@echo "=== 產生文件 ==="
	cargo doc --workspace --no-deps --open

# 安裝依賴
install:
	@echo "=== 安裝 Rust 工具 ==="
	rustup component add clippy rustfmt
	@echo "=== 安裝 cargo 工具 ==="
	cargo install cargo-watch cargo-edit cargo-nextest

# 監看模式（開發）
watch:
	cargo watch -x "check --workspace" -x "test --workspace"

watch-core:
	cargo watch -p dllm-core -x "run -p dllm-core -- serve"

# Docker 建構
docker-build:
	@echo "=== 建構 Docker 映像 ==="
	docker build -f deploy/docker/Dockerfile.core -t dllm/core:latest .
	docker build -f deploy/docker/Dockerfile.rag -t dllm/rag:latest ./services/dllm-rag
	docker build -f deploy/docker/Dockerfile.agent -t dllm/agent:latest ./services/dllm-agent
	docker build -f deploy/docker/Dockerfile.admin -t dllm/admin:latest ./admin/dllm-admin

# OEM 預裝
oem-package:
	@echo "=== 建立 OEM 安裝包 ==="
	./deploy/oem/build-package.sh

# 效能測試
bench:
	@echo "=== 執行效能測試 ==="
	cargo bench --workspace

# 安全掃描
security:
	@echo "=== 安全掃描 ==="
	cargo audit
	cargo geiger

# 幫助
help:
	@echo "dllm 統一構建系統"
	@echo ""
	@echo "可用目標："
	@echo "  build         - 構建 Rust 工作區"
	@echo "  build-release - 構建 Release 版本"
	@echo "  test          - 執行測試"
	@echo "  lint          - Clippy 檢查"
	@echo "  fmt           - 格式化程式碼"
	@echo "  check         - 完整檢查（格式 + lint + 編譯）"
	@echo "  dev           - 啟動開發環境（Docker）"
	@echo "  up            - 啟動服務（Docker）"
	@echo "  down          - 停止服務（Docker）"
	@echo "  logs          - 查看日誌"
	@echo "  clean         - 清理建構產物"
	@echo "  docs          - 產生並開啟文件"
	@echo "  install       - 安裝開發工具"
	@echo "  watch         - 監看模式（自動重建）"
	@echo "  docker-build  - 建構所有 Docker 映像"
	@echo "  oem-package   - 建立 OEM 安裝包"
	@echo "  bench         - 效能測試"
	@echo "  security      - 安全掃描"
	@echo "  help          - 顯示此說明"
