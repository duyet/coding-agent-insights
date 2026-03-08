#!/usr/bin/env bash
# CAI Test Script
# Runs all tests with coverage reporting

set -eEuo pipefail
trap 'echo "::error::Tests failed at line $LINENO"' ERR

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
CARGO="${CARGO:-cargo}"
COVERAGE_TOOL="${COVERAGE_TOOL:-llvm-cov}"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

check_tooling() {
    log_info "Checking test tooling..."

    # Check cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Install Rust from https://rustup.rs/"
        exit 1
    fi

    # Check coverage tool
    if [ "$COVERAGE" = "true" ]; then
        if ! cargo llvm-cov --help &> /dev/null; then
            log_warn "cargo-llvm-cov not found. Install with: cargo install cargo-llvm-cov"
            log_warn "Disabling coverage..."
            COVERAGE=false
        fi
    fi
}

run_unit_tests() {
    log_test "Running unit tests..."
    pushd "$PROJECT_ROOT" > /dev/null

    $CARGO test --workspace --lib --no-fail-fast "$@"

    popd > /dev/null
    ((TESTS_PASSED++))
}

run_integration_tests() {
    log_test "Running integration tests..."
    pushd "$PROJECT_ROOT" > /dev/null

    $CARGO test --workspace --test integration_tests --no-fail-fast "$@"

    popd > /dev/null
    ((TESTS_PASSED++))
}

run_doc_tests() {
    log_test "Running documentation tests..."
    pushd "$PROJECT_ROOT" > /dev/null

    $CARGO test --doc --workspace --no-fail-fast

    popd > /dev/null
    ((TESTS_PASSED++))
}

run_all_tests() {
    log_test "Running all tests..."
    pushd "$PROJECT_ROOT" > /dev/null

    $CARGO test --workspace --all-features --no-fail-fast "$@"

    popd > /dev/null
    ((TESTS_PASSED++))
}

run_coverage() {
    if [ "$COVERAGE" != "true" ]; then
        return
    fi

    log_info "Running tests with coverage..."
    pushd "$PROJECT_ROOT" > /dev/null

    local COVERAGE_DIR="$PROJECT_ROOT/target/coverage"

    # Clean previous coverage
    cargo llvm-cov clean --workspace

    # Run tests with coverage
    if [ "$HTML" = "true" ]; then
        cargo llvm-cov --all-features --workspace --html --output-dir "$COVERAGE_DIR"
        log_info "HTML coverage report: $COVERAGE_DIR/index.html"
    else
        cargo llvm-cov --all-features --workspace --lcov --output-path "$COVERAGE_DIR/lcov.info"
        log_info "LCOV coverage report: $COVERAGE_DIR/lcov.info"
    fi

    # Show summary
    cargo llvm-cov --all-features --workspace --summary

    popd > /dev/null
}

run_benchmarks() {
    if [ "$BENCHMARKS" != "true" ]; then
        return
    fi

    log_info "Running benchmarks..."
    pushd "$PROJECT_ROOT" > /dev/null

    if ! cargo divan --help &> /dev/null; then
        log_warn "cargo-divan not installed. Install with: cargo install cargo-divan"
        return
    fi

    cargo divan --workspace --bench

    popd > /dev/null
}

print_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] [TARGET]

Run CAI tests with various options.

OPTIONS:
    -h, --help              Show this help message
    -c, --coverage          Generate coverage report
    --html                  Generate HTML coverage (requires --coverage)
    -b, --benchmarks        Run benchmarks
    -v, --verbose           Enable verbose output
    --no-fail-fast          Continue running tests after failure

TARGETS:
    all                     Run all tests (default)
    unit                    Run unit tests only
    integration             Run integration tests only
    doc                     Run documentation tests only

EXAMPLES:
    $(basename "$0")                    # Run all tests
    $(basename "$0") -c                 # Run all tests with coverage
    $(basename "$0") -c --html          # Run all tests with HTML coverage
    $(basename "$0") unit               # Run unit tests only
    $(basename "$0") -b                 # Run all tests + benchmarks

EOF
}

# Parse arguments
COVERAGE=false
HTML=false
BENCHMARKS=false
VERBOSE=false
FAIL_FAST="--no-fail-fast"
TARGET="all"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            print_usage
            exit 0
            ;;
        -c|--coverage)
            COVERAGE=true
            shift
            ;;
        --html)
            HTML=true
            shift
            ;;
        -b|--benchmarks)
            BENCHMARKS=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --no-fail-fast)
            FAIL_FAST=""
            shift
            ;;
        all|unit|integration|doc)
            TARGET="$1"
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
        *)
            log_error "Unknown target: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Main execution
log_info "Starting test suite..."
check_tooling

# Run tests based on target
case $TARGET in
    all)
        run_all_tests $FAIL_FAST
        ;;
    unit)
        run_unit_tests $FAIL_FAST
        ;;
    integration)
        run_integration_tests $FAIL_FAST
        ;;
    doc)
        run_doc_tests
        ;;
esac

# Run coverage
run_coverage

# Run benchmarks
run_benchmarks

# Summary
log_info "Test suite completed!"
log_info "Tests passed: $TESTS_PASSED"
if [ $TESTS_FAILED -gt 0 ]; then
    log_warn "Tests failed: $TESTS_FAILED"
fi
