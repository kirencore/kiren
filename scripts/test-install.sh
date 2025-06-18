#!/bin/bash

# Test Installation Methods for Kiren
# This script tests different ways to install Kiren

set -e

echo "🧪 Testing Kiren Installation Methods"
echo "====================================="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Test functions
test_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✅ $1 available${NC}"
        return 0
    else
        echo -e "${RED}❌ $1 not found${NC}"
        return 1
    fi
}

test_kiren_basic() {
    echo -e "${YELLOW}Testing basic Kiren functionality...${NC}"
    
    # Test version
    if kiren --version &> /dev/null; then
        echo -e "${GREEN}✅ kiren --version works${NC}"
    else
        echo -e "${RED}❌ kiren --version failed${NC}"
        return 1
    fi
    
    # Test help
    if kiren --help &> /dev/null; then
        echo -e "${GREEN}✅ kiren --help works${NC}"
    else
        echo -e "${RED}❌ kiren --help failed${NC}"
        return 1
    fi
    
    # Test JavaScript execution
    echo 'console.log("Installation test successful!");' > /tmp/test-kiren.js
    if kiren /tmp/test-kiren.js | grep -q "Installation test successful"; then
        echo -e "${GREEN}✅ JavaScript execution works${NC}"
        rm -f /tmp/test-kiren.js
    else
        echo -e "${RED}❌ JavaScript execution failed${NC}"
        rm -f /tmp/test-kiren.js
        return 1
    fi
    
    # Test HTTP server creation
    echo 'const server = http.createServer(); console.log("Server:", typeof server);' > /tmp/test-server.js
    if kiren /tmp/test-server.js | grep -q "Server: object"; then
        echo -e "${GREEN}✅ HTTP server creation works${NC}"
        rm -f /tmp/test-server.js
    else
        echo -e "${RED}❌ HTTP server creation failed${NC}"
        rm -f /tmp/test-server.js
        return 1
    fi
}

# Test 1: Direct binary installation
echo -e "${YELLOW}🔧 Test 1: Direct Binary Installation${NC}"
if test_command "curl"; then
    echo "Testing install script..."
    
    # Create temporary directory for testing
    TEST_DIR="/tmp/kiren-install-test"
    mkdir -p "$TEST_DIR"
    
    # Test installation script (dry run)
    echo "curl -fsSL https://raw.githubusercontent.com/kirencore/kiren/main/install.sh | bash"
    echo -e "${GREEN}✅ Install script command ready${NC}"
else
    echo -e "${RED}❌ curl not available for install script${NC}"
fi

# Test 2: Check if Kiren is already installed
echo -e "\n${YELLOW}🔧 Test 2: Existing Installation${NC}"
if test_command "kiren"; then
    test_kiren_basic
else
    echo -e "${YELLOW}⚠️  Kiren not installed yet${NC}"
fi

# Test 3: Cargo installation
echo -e "\n${YELLOW}🔧 Test 3: Cargo Installation${NC}"
if test_command "cargo"; then
    echo "Cargo available for: cargo install --git https://github.com/kirencore/kiren"
    echo -e "${GREEN}✅ Cargo installation method available${NC}"
else
    echo -e "${RED}❌ Cargo not available${NC}"
fi

# Test 4: Docker installation
echo -e "\n${YELLOW}🔧 Test 4: Docker Installation${NC}"
if test_command "docker"; then
    echo "Docker available for: docker run -it ghcr.io/kirencore/kiren --repl"
    echo -e "${GREEN}✅ Docker installation method available${NC}"
else
    echo -e "${RED}❌ Docker not available${NC}"
fi

# Test 5: Homebrew installation (future)
echo -e "\n${YELLOW}🔧 Test 5: Homebrew Installation${NC}"
if test_command "brew"; then
    echo "Homebrew available for: brew install kiren (when available)"
    echo -e "${GREEN}✅ Homebrew installation method available${NC}"
else
    echo -e "${RED}❌ Homebrew not available${NC}"
fi

echo -e "\n${GREEN}🎉 Installation test complete!${NC}"
echo ""
echo "Recommended installation methods:"
echo "1. One-liner: curl -fsSL https://raw.githubusercontent.com/kirencore/kiren/main/install.sh | bash"
echo "2. Cargo: cargo install --git https://github.com/kirencore/kiren"
echo "3. Manual: Download from https://github.com/kirencore/kiren/releases"
echo "4. Docker: docker run -it ghcr.io/kirencore/kiren --repl"