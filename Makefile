# CAI Makefile - Common development and testing commands

.PHONY: help build test clean lint fmt check-coverage benchmark all

# Default target
help:
	@echo "CAI Development Commands:"
	@echo ""
	@echo "Building:"
	@echo "  make build       - Build all crates"
	@echo "  make release     - Build release version"
	@echo ""
	@echo "Testing:"
	@echo "  make test        - Run all tests"
	@echo "  make unit        - Run unit tests only"
	@echo "  make integration - Run integration tests"
	@echo "  make e2e         - Run E2E tests (requires build)"
	@echo "  make test-ignored- Run ignored tests"
	@echo ""
	@echo "Coverage:"
	@echo "  make coverage    - Generate coverage report"
	@echo "  make coverage-html - Generate HTML coverage report"
	@echo ""
	@echo "Linting:"
	@echo "  make lint        - Run clippy"
	@echo "  make fmt         - Format code"
	@echo "  make fmt-check   - Check formatting"
	@echo ""
	@echo "Benchmarks:"
	@echo "  make benchmark   - Run benchmarks"
	@echo ""
	@echo "Other:"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make check       - Run all checks (test + lint)"
	@echo "  make all         - Build, test, and lint"

# Build targets
build:
	cargo build --workspace

release:
	cargo build --workspace --release

# Test targets
test:
	cargo test --workspace

unit:
	cargo test --workspace --lib

integration:
	cargo test --workspace --test integration_tests

e2e: build
	cargo test --workspace --test e2e --ignored

test-ignored:
	cargo test --workspace --ignored

# Coverage targets
coverage:
	cargo llvm-cov --all-features --workspace

coverage-html:
	cargo llvm-cov --all-features --workspace --html

# Linting targets
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

# Benchmark targets
benchmark:
	cargo divan --workspace --bench

# Clean target
clean:
	cargo clean

# Combined targets
check: test lint

all: build test lint
