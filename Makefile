.PHONY: build run test clean dev help

# Default target
help:
	@echo "Available commands:"
	@echo "  make build    - Build the application"
	@echo "  make run      - Run the server (builds first)"
	@echo "  make dev      - Run the server in development mode with auto-reload"
	@echo "  make test     - Run all tests"
	@echo "  make clean    - Clean build artifacts"
	@echo ""
	@echo "Quick start:"
	@echo "  1. make run"
	@echo "  2. Open http://127.0.0.1:3000 in your browser"

# Build the project
build:
	cargo build --release

# Run the server
run:
	@echo "Starting server on http://127.0.0.1:3000"
	@echo "Press Ctrl+C to stop"
	cargo run --bin server

# Development mode (if cargo-watch is installed)
dev:
	@if command -v cargo-watch >/dev/null 2>&1; then \
		echo "Running in development mode with auto-reload..."; \
		cargo watch -x 'run --bin server'; \
	else \
		echo "cargo-watch not found. Install with: cargo install cargo-watch"; \
		echo "Falling back to regular run mode..."; \
		make run; \
	fi

# Run tests
test:
	cargo test

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Clean build artifacts
clean:
	cargo clean

# Check code without building
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Run clippy lints
lint:
	cargo clippy -- -D warnings

# Build NIST test suite (optional)
nist:
	@echo "Building NIST test suite..."
	cd nist/sts-2.1.2/sts-2.1.2 && make
	@echo "NIST test suite built successfully!"
