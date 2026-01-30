.PHONY: all build dev test clean install release help

# Default target
all: build

# Development build
dev:
	cargo build

# Production build
build:
	cargo build --release

# Run tests
test:
	cargo test

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Format code
fmt:
	cargo fmt

# Check code
check:
	cargo check
	cargo clippy

# Clean build artifacts
clean:
	cargo clean

# Install locally
install: build
	cargo install --path .

# Run with debug output
run-dev:
	DEBUG=1 cargo run -- $(ARGS)

# Show help
help:
	@echo "CCM Rust - Build Commands"
	@echo ""
	@echo "make dev          - Development build"
	@echo "make build        - Production build"
	@echo "make test         - Run tests"
	@echo "make test-verbose - Run tests with output"
	@echo "make fmt          - Format code"
	@echo "make check        - Check code (clippy)"
	@echo "make clean        - Clean build artifacts"
	@echo "make install      - Install locally"
	@echo "make run-dev      - Run with debug logging"
