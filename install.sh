#!/bin/bash

# Kiren JavaScript Runtime - One-Line Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/kirencore/kiren/main/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Constants
GITHUB_REPO="kirencore/kiren"
VERSION="v0.1.0"
INSTALL_DIR="/usr/local/bin"

# Functions
log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect platform
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case $os in
        linux*)
            case $arch in
                x86_64) echo "linux-x64" ;;
                aarch64) echo "linux-arm64" ;;
                *) error "Unsupported architecture: $arch" ;;
            esac
            ;;
        darwin*)
            case $arch in
                x86_64) echo "macos-x64" ;;
                arm64) echo "macos-arm64" ;;
                *) error "Unsupported architecture: $arch" ;;
            esac
            ;;
        mingw*|msys*|cygwin*)
            echo "windows-x64"
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac
}

# Get latest release version
get_latest_version() {
    if command -v curl &> /dev/null; then
        curl -fsSL "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | \
            grep '"tag_name":' | \
            sed -E 's/.*"v([^"]+)".*/\1/'
    elif command -v wget &> /dev/null; then
        wget -qO- "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | \
            grep '"tag_name":' | \
            sed -E 's/.*"v([^"]+)".*/\1/'
    else
        error "Neither curl nor wget is available"
    fi
}

# Download and install
install_kiren() {
    local platform=$(detect_platform)
    local version=${KIREN_VERSION:-$VERSION}
    
    log "Installing Kiren $version for $platform..."
    
    # Create temp directory
    local tmp_dir=$(mktemp -d)
    cd "$tmp_dir"
    
    # For now, we only have single binary from release
    # In future releases, we'll have platform-specific binaries
    local download_url="https://github.com/$GITHUB_REPO/releases/download/$version/kiren"
    
    log "Downloading from: $download_url"
    
    # Download binary
    if command -v curl &> /dev/null; then
        curl -fsSL "$download_url" -o "kiren"
    elif command -v wget &> /dev/null; then
        wget -q "$download_url" -O "kiren"
    else
        error "Neither curl nor wget is available"
    fi
    
    # Make executable
    chmod +x kiren
    binary_name="kiren"
    
    # Install to system
    if [ -w "$INSTALL_DIR" ]; then
        mv "$binary_name" "$INSTALL_DIR/"
        success "Kiren installed to $INSTALL_DIR/$binary_name"
    else
        log "Installing to $INSTALL_DIR requires sudo..."
        sudo mv "$binary_name" "$INSTALL_DIR/"
        success "Kiren installed to $INSTALL_DIR/$binary_name"
    fi
    
    # Cleanup
    cd /
    rm -rf "$tmp_dir"
    
    # Verify installation
    if command -v kiren &> /dev/null; then
        success "Installation successful! 🚀"
        echo ""
        echo "Try it out:"
        echo "  kiren --help"
        echo "  kiren --repl"
        echo "  echo 'console.log(\"Hello Kiren!\")' > hello.js && kiren hello.js"
    else
        warn "Kiren installed but not in PATH. Add $INSTALL_DIR to your PATH:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

# Main
main() {
    echo "🚀 Kiren JavaScript Runtime Installer"
    echo "====================================="
    echo ""
    
    # Check dependencies
    if ! command -v tar &> /dev/null && ! command -v unzip &> /dev/null; then
        error "tar or unzip is required for installation"
    fi
    
    install_kiren
}

main "$@"