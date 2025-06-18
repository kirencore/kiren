# Kiren Makefile
# Simple commands for development workflow

.PHONY: help build dev release test clean format clippy check run examples docker

# Default target
help:
	@echo "🚀 Kiren Development Commands"
	@echo "============================="
	@echo ""
	@echo "Build Commands:"
	@echo "  make build    - Build in development mode"
	@echo "  make release  - Build optimized release binary"
	@echo "  make clean    - Clean build artifacts"
	@echo ""
	@echo "Development:"
	@echo "  make dev      - Build and run REPL"
	@echo "  make test     - Run all tests"
	@echo "  make format   - Format code"
	@echo "  make clippy   - Run linter"
	@echo "  make check    - Run all checks"
	@echo ""
	@echo "Examples:"
	@echo "  make run      - Run hello.js example"
	@echo "  make examples - List all examples"
	@echo "  make server   - Run HTTP server demo"
	@echo ""
	@echo "Docker:"
	@echo "  make docker   - Build Docker container"

# Build commands
build:
	@cargo build

dev: build
	@./target/debug/kiren --repl

release:
	@cargo build --release
	@echo "✅ Release binary: ./target/release/kiren"

# Testing and quality
test:
	@cargo test

format:
	@cargo fmt

clippy:
	@cargo clippy -- -D warnings

check: format clippy test release
	@echo "✅ All checks passed"

# Cleanup
clean:
	@cargo clean

# Examples
run: build
	@./target/debug/kiren examples/hello.js

examples:
	@echo "📁 Available examples:"
	@ls examples/*.js | sed 's/examples\//  /' | sed 's/\.js//'

server: build
	@echo "🚀 Starting HTTP server demo..."
	@./target/debug/kiren examples/production-demo.js

# Docker
docker:
	@echo "🐳 Building Docker container..."
	@docker build -t kiren:latest .
	@echo "✅ Container built: kiren:latest"
	@echo "🚀 Run with: docker run -p 3000:3000 kiren:latest"

# Benchmarks
benchmark: release
	@echo "📊 Running benchmarks..."
	@cd benchmarks && ./run-benchmarks.sh

# Release
release-version:
	@echo "🚀 Creating new release..."
	@read -p "Enter version (e.g., 0.2.0): " version; \
	./scripts/release.sh $$version

release-publish: release
	@echo "📦 Publishing to crates.io..."
	@cargo publish

# Package distribution
package: release
	@echo "📦 Creating distribution packages..."
	@mkdir -p dist
	@tar -czf dist/kiren-$(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -m).tar.gz -C target/release kiren
	@echo "✅ Package created: dist/kiren-$(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -m).tar.gz"