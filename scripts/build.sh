#!/bin/bash

# Kiren Build Script
# Usage: ./scripts/build.sh [dev|release|test|clean]

set -e

echo "🚀 Kiren Build Script"
echo "===================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_status() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install from https://rustup.rs/"
        exit 1
    fi
    
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    print_status "Rust version: $RUST_VERSION"
}

# Development build
build_dev() {
    print_status "Building in development mode..."
    cargo build
    print_success "Development build completed"
    
    if [ -f "target/debug/kiren" ]; then
        SIZE=$(ls -lh target/debug/kiren | awk '{print $5}')
        print_status "Binary size: $SIZE"
    fi
}

# Release build
build_release() {
    print_status "Building optimized release..."
    cargo build --release
    print_success "Release build completed"
    
    if [ -f "target/release/kiren" ]; then
        SIZE=$(ls -lh target/release/kiren | awk '{print $5}')
        print_success "Binary size: $SIZE"
        print_status "Binary location: ./target/release/kiren"
    fi
}

# Run tests
run_tests() {
    print_status "Running tests..."
    cargo test
    print_success "All tests passed"
}

# Clean build artifacts
clean_build() {
    print_status "Cleaning build artifacts..."
    cargo clean
    print_success "Build artifacts cleaned"
}

# Format code
format_code() {
    print_status "Formatting code..."
    cargo fmt
    print_success "Code formatted"
}

# Run clippy
run_clippy() {
    print_status "Running clippy..."
    cargo clippy -- -D warnings
    print_success "Clippy checks passed"
}

# Check everything
check_all() {
    print_status "Running comprehensive checks..."
    format_code
    run_clippy
    run_tests
    build_release
    print_success "All checks completed successfully"
}

# Main logic
check_rust

case "${1:-dev}" in
    "dev")
        build_dev
        ;;
    "release")
        build_release
        ;;
    "test")
        run_tests
        ;;
    "clean")
        clean_build
        ;;
    "format")
        format_code
        ;;
    "clippy")
        run_clippy
        ;;
    "check")
        check_all
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  dev      Build in development mode (default)"
        echo "  release  Build optimized release binary"
        echo "  test     Run all tests"
        echo "  clean    Clean build artifacts"
        echo "  format   Format code with cargo fmt"
        echo "  clippy   Run clippy linter"
        echo "  check    Run all checks (format, clippy, test, build)"
        echo "  help     Show this help message"
        ;;
    *)
        print_error "Unknown command: $1"
        print_status "Use '$0 help' for available commands"
        exit 1
        ;;
esac