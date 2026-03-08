# CAI Makefile - Common development and testing commands

.PHONY: help build test clean lint fmt check-coverage benchmark all release-bump release-dry-run

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
	@echo "Release:"
	@echo "  make release-patch  - Bump patch version and prepare release"
	@echo "  make release-minor  - Bump minor version and prepare release"
	@echo "  make release-major  - Bump major version and prepare release"
	@echo "  make release-dry-run - Preview release changes"
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

# Release targets
release-patch:
	./scripts/release.sh -b patch

release-minor:
	./scripts/release.sh -b minor

release-major:
	./scripts/release.sh -b major

release-dry-run:
	./scripts/release.sh -b patch -d

release-push:
	./scripts/release.sh -b patch -p
