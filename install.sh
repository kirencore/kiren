#!/bin/bash

# Kiren Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/mertcanaltin/kiren/main/install.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Constants
GITHUB_REPO="mertcanaltin/kiren"
INSTALL_DIR="${KIREN_INSTALL_DIR:-/usr/local/bin}"
TMP_DIR="/tmp/kiren-install"

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
    local version=${KIREN_VERSION:-$(get_latest_version)}
    
    if [ -z "$version" ]; then
        error "Could not determine version to install"
    fi
    
    log "Installing Kiren v$version for $platform..."
    
    # Create temp directory
    mkdir -p "$TMP_DIR"
    cd "$TMP_DIR"
    
    # Download URL
    local file_name="kiren-$platform"
    local download_url="https://github.com/$GITHUB_REPO/releases/download/v$version/$file_name.tar.gz"
    
    if [[ $platform == "windows-x64" ]]; then
        download_url="https://github.com/$GITHUB_REPO/releases/download/v$version/$file_name.zip"
    fi
    
    log "Downloading from: $download_url"
    
    # Download binary
    if command -v curl &> /dev/null; then
        curl -fsSL "$download_url" -o "kiren-archive"
    elif command -v wget &> /dev/null; then
        wget -q "$download_url" -O "kiren-archive"
    else
        error "Neither curl nor wget is available"
    fi
    
    # Extract
    if [[ $platform == "windows-x64" ]]; then
        unzip -q kiren-archive
        chmod +x kiren.exe
        binary_name="kiren.exe"
    else
        tar -xzf kiren-archive
        chmod +x kiren
        binary_name="kiren"
    fi
    
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
    rm -rf "$TMP_DIR"
    
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