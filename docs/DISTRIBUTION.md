# Kiren Distribution Guide

This document explains how to install and distribute Kiren across different platforms.

## 🚀 Quick Installation

### One-liner Install (Recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/mertcanaltin/kiren/main/install.sh | bash
```

This script automatically detects your platform and installs the latest version.

## 📦 Installation Methods

### 1. Binary Releases (All Platforms)

Download pre-built binaries from [GitHub Releases](https://github.com/mertcanaltin/kiren/releases):

**Linux (x64)**
```bash
curl -LO https://github.com/mertcanaltin/kiren/releases/latest/download/kiren-linux-x64.tar.gz
tar -xzf kiren-linux-x64.tar.gz
sudo mv kiren /usr/local/bin/
```

**macOS (Intel)**
```bash
curl -LO https://github.com/mertcanaltin/kiren/releases/latest/download/kiren-macos-x64.tar.gz
tar -xzf kiren-macos-x64.tar.gz
sudo mv kiren /usr/local/bin/
```

**macOS (Apple Silicon)**
```bash
curl -LO https://github.com/mertcanaltin/kiren/releases/latest/download/kiren-macos-arm64.tar.gz
tar -xzf kiren-macos-arm64.tar.gz
sudo mv kiren /usr/local/bin/
```

**Windows**
```powershell
# Download from: https://github.com/mertcanaltin/kiren/releases/latest/download/kiren-windows-x64.zip
# Extract and add to PATH
```

### 2. Package Managers

#### Homebrew (macOS/Linux)
```bash
# Add tap (once available)
brew tap mertcanaltin/kiren
brew install kiren
```

#### Cargo (Rust users)
```bash
cargo install kiren
```

#### Snap (Linux)
```bash
# Coming soon
sudo snap install kiren
```

#### Chocolatey (Windows)
```powershell
# Coming soon
choco install kiren
```

### 3. Docker

#### Official Image
```bash
# Run interactive REPL
docker run -it ghcr.io/mertcanaltin/kiren --repl

# Run JavaScript file
docker run -v $PWD:/app ghcr.io/mertcanaltin/kiren /app/script.js

# HTTP server
docker run -p 3000:3000 -v $PWD:/app ghcr.io/mertcanaltin/kiren /app/server.js
```

#### Build Custom Image
```dockerfile
FROM ghcr.io/mertcanaltin/kiren:latest
COPY . /app
WORKDIR /app
CMD ["kiren", "index.js"]
```

### 4. Build from Source

#### Prerequisites
- Rust 1.70+
- Git

#### Steps
```bash
git clone https://github.com/mertcanaltin/kiren.git
cd kiren
cargo build --release
sudo cp target/release/kiren /usr/local/bin/
```

## 🌍 Platform Support

| Platform | Architecture | Status | Binary Name |
|----------|--------------|--------|-------------|
| **Linux** | x86_64 | ✅ Supported | kiren-linux-x64 |
| **Linux** | aarch64 | 🔄 Planned | kiren-linux-arm64 |
| **macOS** | x86_64 | ✅ Supported | kiren-macos-x64 |
| **macOS** | arm64 | ✅ Supported | kiren-macos-arm64 |
| **Windows** | x86_64 | ✅ Supported | kiren-windows-x64 |
| **FreeBSD** | x86_64 | 🔄 Planned | kiren-freebsd-x64 |

## 📁 File Structure

After installation, Kiren provides:
- **Binary**: `/usr/local/bin/kiren` (Unix) or `kiren.exe` (Windows)
- **Size**: ~15MB (single file, zero dependencies)
- **Permissions**: Executable

## 🔧 Verification

Verify your installation:
```bash
# Check version
kiren --version

# Test basic functionality
echo 'console.log("Hello Kiren!")' > test.js
kiren test.js

# Start REPL
kiren --repl
```

## 🚀 Quick Start

```bash
# 1. Create a simple server
cat > server.js << 'EOF'
const server = http.createServer();
server.get("/", () => "Hello from Kiren!");
server.listen(3000);
console.log("🚀 Server running on http://localhost:3000");
EOF

# 2. Run it
kiren server.js

# 3. Test it
curl http://localhost:3000
```

## 🔄 Updates

### Manual Update
```bash
# Re-run installer
curl -fsSL https://raw.githubusercontent.com/mertcanaltin/kiren/main/install.sh | bash
```

### Package Manager Update
```bash
# Homebrew
brew upgrade kiren

# Cargo
cargo install kiren --force
```

## 🗑️ Uninstallation

### Manual Removal
```bash
# Remove binary
sudo rm /usr/local/bin/kiren

# Remove any config (none currently)
```

### Package Manager Removal
```bash
# Homebrew
brew uninstall kiren

# Cargo
cargo uninstall kiren
```

## 🐛 Troubleshooting

### Permission Issues
```bash
# If you can't write to /usr/local/bin
KIREN_INSTALL_DIR=$HOME/.local/bin curl -fsSL install.sh | bash
export PATH="$HOME/.local/bin:$PATH"
```

### Network Issues
```bash
# Download manually and install
wget https://github.com/mertcanaltin/kiren/releases/latest/download/kiren-linux-x64.tar.gz
tar -xzf kiren-linux-x64.tar.gz
sudo mv kiren /usr/local/bin/
```

### Platform Not Supported
```bash
# Build from source
git clone https://github.com/mertcanaltin/kiren.git
cd kiren
cargo build --release
```

## 📊 Distribution Stats

**Binary Sizes:**
- Linux: ~15MB
- macOS: ~15MB  
- Windows: ~16MB

**Dependencies:**
- Zero runtime dependencies
- Self-contained binary
- No external libraries required

## 🤝 Package Maintainers

Interested in maintaining Kiren packages for your platform?

**We need maintainers for:**
- Debian/Ubuntu (apt)
- Fedora/CentOS (yum/dnf)
- Arch Linux (AUR)
- Windows (Chocolatey)
- NixOS (nixpkgs)

Contact us through GitHub Issues!