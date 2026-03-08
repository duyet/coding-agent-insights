#!/usr/bin/env bash
# CAI Build Script
# Builds all targets with proper error handling

set -eEuo pipefail
trap 'echo "::error::Build failed at line $LINENO"' ERR

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
CARGO="${CARGO:-cargo}"
RUSTFLAGS="${RUSTFLAGS:-}"
TARGET_DIR="${PROJECT_ROOT}/target"

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

check_rust() {
    log_info "Checking Rust installation..."
    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed. Visit https://rustup.rs/ to install."
        exit 1
    fi

    local RUST_VERSION=$(rustc --version)
    log_info "Found $RUST_VERSION"
}

build_crate() {
    local crate=$1
    local profile=${2:-release}

    log_info "Building $crate ($profile)..."

    pushd "$PROJECT_ROOT" > /dev/null

    case $profile in
        dev)
            $CARGO build --package "$crate"
            ;;
        release)
            $CARGO build --release --package "$crate"
            ;;
        *)
            log_error "Unknown profile: $profile"
            exit 1
            ;;
    esac

    popd > /dev/null
}

build_workspace() {
    local profile=${1:-release}

    log_info "Building workspace ($profile)..."

    pushd "$PROJECT_ROOT" > /dev/null

    case $profile in
        dev)
            $CARGO build --workspace
            ;;
        release)
            $CARGO build --workspace --release
            ;;
        *)
            log_error "Unknown profile: $profile"
            exit 1
            ;;
    esac

    popd > /dev/null
}

build_binary() {
    local profile=${1:-release}
    local target_dir="$TARGET_DIR/$profile"

    log_info "Building CAI binary..."

    pushd "$PROJECT_ROOT" > /dev/null

    case $profile in
        dev)
            $CARGO build --package cai-cli
            ;;
        release)
            $CARGO build --release --package cai-cli
            ;;
    esac

    local BINARY="$target_dir/cai"

    if [ ! -f "$BINARY" ]; then
        log_error "Binary not found at $BINARY"
        exit 1
    fi

    # Get binary size
    local SIZE=$(du -h "$BINARY" | cut -f1)
    log_info "Binary size: $SIZE"

    popd > /dev/null
}

build_all_targets() {
    local profile=${1:-release}

    log_info "Building all targets..."

    # Build for current host
    build_workspace "$profile"

    # Build binary
    build_binary "$profile"
}

print_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] [TARGET]

Build CAI project with various options.

OPTIONS:
    -h, --help              Show this help message
    -p, --profile PROFILE   Build profile (dev|release) [default: release]
    -v, --verbose           Enable verbose output

TARGETS:
    all                     Build all targets (default)
    workspace               Build entire workspace
    binary                  Build only the CLI binary
    CRATE                   Build specific crate (e.g., cai-core)

EXAMPLES:
    $(basename "$0")                    # Build all in release mode
    $(basename "$0") -p dev             # Build all in dev mode
    $(basename "$0") -p dev binary      # Build binary in dev mode
    $(basename "$0") cai-core           # Build specific crate
    $(basename "$0") -v workspace       # Build workspace with verbose output

EOF
}

# Parse arguments
PROFILE="release"
VERBOSE=false
TARGET="all"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            print_usage
            exit 0
            ;;
        -p|--profile)
            PROFILE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            export RUSTFLAGS="$RUSTFLAGS -v"
            shift
            ;;
        all|workspace|binary)
            TARGET="$1"
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
        *)
            TARGET="$1"
            shift
            ;;
    esac
done

# Validate profile
if [[ "$PROFILE" != "dev" && "$PROFILE" != "release" ]]; then
    log_error "Invalid profile: $PROFILE (must be 'dev' or 'release')"
    exit 1
fi

# Main execution
log_info "Starting build..."
check_rust

case $TARGET in
    all)
        build_all_targets "$PROFILE"
        ;;
    workspace)
        build_workspace "$PROFILE"
        ;;
    binary)
        build_binary "$PROFILE"
        ;;
    *)
        build_crate "$TARGET" "$PROFILE"
        ;;
esac

log_info "Build completed successfully!"
