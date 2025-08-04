# Kiren

A high-performance JavaScript runtime built with Rust, powered by the V8 engine.

## Features

- **🚀 High Performance**: Leveraging Rust's safety and speed with V8's power
- **🔧 Modern JavaScript**: Support for ES2022+ features
- **🛠️ Built-in APIs**: Console, Timers, Fetch, File System, HTTP Server
- **💻 REPL Mode**: Interactive JavaScript development environment
- **📦 Module System**: ES Modules and CommonJS support (in development)
- **⚡ Async/Await**: Full async/await support with proper event loop
- **🎯 TypeScript**: TypeScript file execution support (planned)
- **🐳 Docker Ready**: Single binary deployment, 15MB containers

## Installation

### 🚀 Quick Install (Recommended)
```bash
# One-line installer (macOS/Linux)
curl -fsSL https://raw.githubusercontent.com/kirencore/kiren/main/install.sh | bash
```

### 🍺 Package Managers
```bash
# Homebrew (macOS/Linux) - Coming Soon
brew install kiren

# Cargo (Rust users)
cargo install --git https://github.com/kirencore/kiren

# NPM (Node.js users) - Coming Soon  
npm install -g kiren-runtime

# Docker
docker run -it ghcr.io/kirencore/kiren --repl
```

### 📥 Manual Download
Download pre-built binaries from [GitHub Releases](https://github.com/kirencore/kiren/releases)

**Available Platforms:**
- ✅ **Linux** (x64, ARM64) - `kiren-linux-x64.tar.gz`, `kiren-linux-arm64.tar.gz`
- ✅ **macOS** (Intel, Apple Silicon) - `kiren-macos-x64.tar.gz`, `kiren-macos-arm64.tar.gz`
- ✅ **Windows** (x64) - `kiren-windows-x64.zip`

### 🛠️ Build from Source
```bash
# Prerequisites: Rust 1.70+, Git
git clone https://github.com/kirencore/kiren.git
cd kiren

# Quick start with Make
make build          # Development build
make dev            # Build + REPL
make test           # Run tests
make server         # HTTP server demo

# Or use Cargo directly
cargo build --release        # Optimized build
cargo run examples/hello.js  # Run example
cargo run -- --repl         # Interactive REPL

# Install locally
cp target/release/kiren /usr/local/bin/
```

## Usage

### Execute JavaScript Files
```bash
kiren examples/hello.js
```

### REPL Mode
```bash
kiren --repl
```

### Configuration
Kiren uses `kiren.toml` for configuration:
```toml
[runtime]
version = "3.0.0"
memory_limit = 512

[environment]
NODE_ENV = "development"
LOG_LEVEL = "info"  # "debug", "info", "warn", "error", "silent"

[server]
default_port = 3000
cors_enabled = true
```

## Examples

### Basic JavaScript
```javascript
console.log("Hello, Kiren!");

const sum = (a, b) => a + b;
console.log("5 + 3 =", sum(5, 3));

// Modern JavaScript features
const users = [
    { name: "Alice", age: 30 },
    { name: "Bob", age: 25 }
];

const adults = users.filter(user => user.age >= 18);
console.log("Adults:", adults);
```

### Timer APIs
```javascript
// setTimeout
setTimeout(() => {
    console.log("This runs after 1 second");
}, 1000);

// setInterval
const intervalId = setInterval(() => {
    console.log("This repeats every second");
}, 1000);

// Clear after 5 seconds
setTimeout(() => {
    clearInterval(intervalId);
    console.log("Interval cleared");
}, 5000);
```

### File System Operations
```javascript
// Write and read files
fs.writeFile("test.txt", "Hello from Kiren!");
const content = fs.readFile("test.txt");
console.log("File content:", content);

// Directory operations
fs.mkdir("new-folder");
console.log("Folder exists?", fs.exists("new-folder"));
```

### Process API
```javascript
// Environment variables
console.log("HOME directory:", process.env.HOME);

// Command line arguments
console.log("Arguments:", process.argv);

// Current working directory
console.log("Working directory:", process.cwd());

// Exit with code
process.exit(0);
```

### HTTP Requests
```javascript
// Fetch API
fetch("https://api.github.com/users/mertcanaltin")
    .then(response => response.json())
    .then(data => console.log("User:", data.name))
    .catch(error => console.log("Error:", error));

// Async/await
async function getUser() {
    try {
        const response = await fetch("https://api.github.com/users/mertcanaltin");
        const user = await response.json();
        console.log("User:", user.name);
    } catch (error) {
        console.log("Error:", error);
    }
}

getUser();
```

### 🔥 HTTP Server (Production Ready)
```javascript
// Zero-config production server
const server = http.createServer();

// Routes
server.get("/", () => "Hello from Kiren!");
server.get("/api/users", () => ({ users: ["Alice", "Bob"] }));
server.post("/api/data", (req) => ({ 
    message: "Data received", 
    data: req.body 
}));

// Middleware
server.use((req, res, next) => {
    console.log(`${req.method} ${req.url}`);
    next();
});

// Start server
server.listen(3000);
console.log("🚀 Server ready at http://localhost:3000");
```

### Docker Deployment
```dockerfile
FROM scratch
COPY kiren /kiren
COPY app.js /app.js
EXPOSE 3000
CMD ["/kiren", "/app.js"]
# Result: 15MB container vs Node.js 500MB+
```

## Performance Benchmarks

**Real Benchmark Results** (vs Node.js v20.18.1):

| Metric | Kiren v3.0.0 | Node.js | Result |
|--------|--------------|---------|--------|
| **Startup Time** | 72ms | 22ms | Node.js 3.3x faster |
| **Fibonacci(35)** | 54ms | 46ms | Node.js 1.2x faster |
| **Loop (10M)** | 37ms | 8ms | Node.js 4.6x faster |
| **Memory Usage** | 45MB | 89MB | Kiren 50% less memory |
| **Binary Size** | 15MB | 500MB+ | Kiren 97% smaller |

### 🎯 Kiren's Real Advantages:

- **🦀 Memory Safety**: Rust's ownership model prevents crashes
- **📦 Single Binary**: No dependency hell, easy deployment  
- **🔧 Simplicity**: Minimal setup, just copy & run
- **🐳 Container Friendly**: 15MB Docker images vs 500MB+ Node.js
- **🛠️ Learning**: Perfect for understanding JavaScript runtime internals
- **🚀 Potential**: Room for optimization and unique features

> **Note**: At v3.0.0, Kiren is not faster than Node.js yet. This is a learning project and should be evaluated as a functional runtime. For detailed benchmark results, see [`benchmarks/BENCHMARK_RESULTS.md`](benchmarks/BENCHMARK_RESULTS.md).

## Development Status

### ✅ Completed Features
- [x] **V8 Integration** - Full JavaScript support with modern features
- [x] **Console API** - `console.log()`, `console.time()`, `console.timeEnd()`, etc.
- [x] **REPL Mode** - Interactive JavaScript environment (`.exit`, `.help`)
- [x] **CLI Interface** - File execution and command line options
- [x] **Timer APIs** - `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`
- [x] **Fetch API** - HTTP requests with Promise support
- [x] **File System API** - `fs.readFile`, `fs.writeFile`, `fs.exists`, `fs.mkdir`
- [x] **Process API** - `process.env`, `process.argv`, `process.cwd()`, `process.exit()`
- [x] **🔥 HTTP Server API** - Production-ready web server with routing

### 🔄 In Development
- [ ] Enhanced error handling & stack traces
- [ ] Timer callback execution improvements
- [ ] ES Modules system
- [ ] CommonJS compatibility layer

### 📋 Planned Features
- [ ] TypeScript support (.ts, .tsx files)
- [ ] Package manager integration (npm-compatible)
- [ ] WebAssembly support
- [ ] Worker Threads
- [ ] Streaming APIs
- [ ] Crypto APIs

## 🤝 Contributing

We welcome contributions to Kiren! Here's how to get started:

### Getting Started
1. Read our **[CONTRIBUTING.md](CONTRIBUTING.md)** guide
2. Fork the repository
3. Set up development environment: `cargo build`
4. Create a feature branch: `git checkout -b feature/amazing-feature`
5. Test your changes: `cargo test`
6. Submit a Pull Request

### 🎯 Contribution Areas

- **Performance Optimization** - Improve V8 integration and runtime speed
- **Stability Fixes** - Fix crashes and memory issues
- **New APIs** - Implement missing JavaScript APIs
- **Documentation** - API guides, tutorials, examples
- **Testing** - Unit tests, integration tests, benchmarks
- **Platform Support** - Windows, ARM64, WebAssembly

### Development Setup
```bash
# Clone and build
git clone https://github.com/kirencore/kiren.git
cd kiren

# Development helpers
make format         # Format code with rustfmt
make clippy         # Run Clippy linter
make test           # Run all tests
make check          # All quality checks
make examples       # List available examples

# Build script with options
./scripts/build.sh release    # Optimized build
./scripts/build.sh test       # Run tests
./scripts/build.sh check      # Full quality check
```

For details, see: [CONTRIBUTING.md](CONTRIBUTING.md)

## License

Distributed under the MIT License. See `LICENSE` file for details.

## Contact & Community

**Maintainer**: Mert Can Altin - [@mertcanaltin](https://github.com/mertcanaltin)

**Project Link**: [https://github.com/kirencore/kiren](https://github.com/kirencore/kiren)

**Community**:
- 🐛 [Report Issues](https://github.com/kirencore/kiren/issues)
- 💡 [Feature Requests](https://github.com/kirencore/kiren/discussions)
- 📖 [Documentation](https://docs.kiren.dev) (Coming Soon)
- 💬 [Discord Server](https://discord.gg/kiren) (Coming Soon)

---

**⭐ If you find Kiren useful, please consider giving it a star on GitHub!**