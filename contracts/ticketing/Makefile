.PHONY: help test build fmt lint clean optimize check all

# Default target
default: help

# Help target - lists all available targets
help:
	@echo "Available targets (ticketing contract):"
	@echo "  help       - Show this help message"
	@echo "  all        - Run all checks (fmt, lint, test, build)"
	@echo "  test       - Run test suite"
	@echo "  build      - Build contract WASM"
	@echo "  optimize   - Build and optimize contract WASM"
	@echo "  fmt        - Format code with rustfmt"
	@echo "  lint       - Run clippy lints"
	@echo "  check      - Run fmt, lint, and test"
	@echo "  clean      - Clean build artifacts"

test: build
	cargo test

build:
	stellar contract build
	@ls -l target/wasm32v1-none/release/*.wasm 2>/dev/null || ls -l target/wasm32-unknown-unknown/release/*.wasm 2>/dev/null || true

optimize: build
	@WASM="target/wasm32v1-none/release/stellar_tickets_ticketing.wasm"; \
	if [ ! -f "$$WASM" ]; then \
		WASM="target/wasm32-unknown-unknown/release/stellar_tickets_ticketing.wasm"; \
	fi; \
	if [ ! -f "$$WASM" ]; then \
		echo "Error: Built WASM file not found"; exit 1; \
	fi; \
	echo "==> Optimizing contract WASM..."; \
	stellar contract optimize --wasm "$$WASM"; \
	OPT_WASM="$${WASM%.wasm}.optimized.wasm"; \
	if [ -f "$$OPT_WASM" ]; then \
		FINAL_WASM="$$OPT_WASM"; \
	else \
		FINAL_WASM="$$WASM"; \
	fi; \
	WASM_HASH=$$(sha256sum "$$FINAL_WASM" | awk '{print $$1}'); \
	echo "=================================================="; \
	echo "Contract build & optimization successful!"; \
	echo "WASM File: $$FINAL_WASM"; \
	echo "WASM Hash: $$WASM_HASH"; \
	echo "=================================================="

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets -- -D warnings -W clippy::pedantic

check: fmt lint test

clean:
	cargo clean

all: check build