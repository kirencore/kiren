# Kiren

Rust ile geliştirilmiş yüksek performanslı JavaScript runtime'ı.

## Özellikler

- **Yüksek Performans**: Rust'ın güvenliği ve hızı ile V8 engine'in gücü
- **Modern JavaScript**: ES2022+ özelliklerini destekler
- **Built-in APIs**: Console, Timers, Fetch API desteği
- **REPL Mode**: Interaktif JavaScript geliştirme ortamı
- **Module System**: ES Modules ve CommonJS desteği (geliştirilme aşamasında)
- **Async/Await**: Tam async/await desteği
- **TypeScript**: TypeScript dosya çalıştırma desteği (planlanan)

## Kurulum

### Kaynak Koddan Derleme

```bash
# Gereksinimler: Rust 1.70+, Git
git clone https://github.com/mertcanaltin/kiren.git
cd kiren
cargo build --release
```

### 🛠️ Developer Setup

```bash
# 1. Clone repository
git clone https://github.com/mertcanaltin/kiren.git
cd kiren

# 2. Quick start with Make
make build          # Development build
make dev            # Build + REPL
make test           # Run tests
make server         # HTTP server demo

# 3. Or use Cargo directly
cargo build                    # Development build
cargo run examples/hello.js   # Run example
cargo run -- --repl          # Interactive REPL
cargo test                    # Run tests
cargo build --release        # Optimized build

# 4. Development helpers
make format         # Format code
make clippy         # Run linter
make check          # All quality checks
make examples       # List available examples
```

### 🔧 Build Script

```bash
# Use the build script for more options
./scripts/build.sh release    # Optimized build
./scripts/build.sh test       # Run tests
./scripts/build.sh check      # Full quality check
./scripts/build.sh help       # See all options
```

## Kullanım

### JavaScript Dosyası Çalıştırma

```bash
./target/release/kiren examples/hello.js
```

### REPL Modu

```bash
./target/release/kiren --repl
```

## 📦 Installation

### Quick Install (One-liner)
```bash
curl -fsSL https://raw.githubusercontent.com/mertcanaltin/kiren/main/install.sh | bash
```

### Package Managers
```bash
# Homebrew (macOS/Linux)
brew install kiren

# Cargo (Rust users)
cargo install kiren

# Docker
docker run -it ghcr.io/mertcanaltin/kiren --repl
```

### Manual Download
Download from [GitHub Releases](https://github.com/mertcanaltin/kiren/releases)

**Supported Platforms:**
- ✅ Linux (x64, ARM64)
- ✅ macOS (Intel, Apple Silicon)  
- ✅ Windows (x64)

Detaylı kurulum talimatları: [docs/DISTRIBUTION.md](docs/DISTRIBUTION.md)

## Örnekler

### Temel JavaScript
```javascript
console.log("Hello, Kiren!");

const sum = (a, b) => a + b;
console.log("5 + 3 =", sum(5, 3));
```

### Timer APIs
```javascript
// setTimeout
setTimeout(() => {
    console.log("Bu 1 saniye sonra çalışır");
}, 1000);

// setInterval
const intervalId = setInterval(() => {
    console.log("Bu her saniye tekrarlanır");
}, 1000);

// 5 saniye sonra durdur
setTimeout(() => {
    clearInterval(intervalId);
    console.log("Interval durduruldu");
}, 5000);
```

### File System Operations
```javascript
// Dosya yazma ve okuma
fs.writeFile("test.txt", "Hello from Kiren!");
const content = fs.readFile("test.txt");
console.log("Dosya içeriği:", content);

// Dizin oluşturma
fs.mkdir("yeni-klasor");
console.log("Klasör var mı?", fs.exists("yeni-klasor"));
```

### Process API
```javascript
// Environment variables
console.log("HOME dizini:", process.env.HOME);

// Command line arguments
console.log("Argümanlar:", process.argv);

// Current working directory
console.log("Çalışma dizini:", process.cwd());
```

### HTTP Requests
```javascript
fetch("https://api.github.com/users/mertcanaltin")
    .then(() => console.log("İstek başarılı"))
    .catch(error => console.log("Hata:", error));
```

### 🔥 HTTP Server (NEW!)
```javascript
// Zero-config production server
const server = http.createServer();

server.get("/", () => "Hello from Kiren!");
server.get("/api/users", () => ({ users: ["Alice", "Bob"] }));
server.post("/api/data", () => ({ message: "Data received" }));

server.listen(3000);
console.log("🚀 Server ready at http://localhost:3000");
```

### Docker Deployment
```dockerfile
FROM scratch
COPY kiren /kiren
COPY app.js /app.js
CMD ["/kiren", "/app.js"]
# Result: 15MB container vs Node.js 500MB+
```

## Performans Karşılaştırmaları

**Gerçek Benchmark Sonuçları** (vs Node.js v20.18.1):

| Metric | Kiren v0.1.0 | Node.js | Sonuç |
|--------|--------------|---------|--------|
| **Startup Time** | 72ms | 22ms | Node.js 3.3x daha hızlı |
| **Fibonacci(35)** | 54ms | 46ms | Node.js 1.2x daha hızlı |
| **Loop (10M)** | 37ms | 8ms | Node.js 4.6x daha hızlı |

### 🎯 Kiren'in Gerçek Avantajları:

- **🦀 Memory Safety**: Rust'ın ownership modeli ile güvenlik
- **📦 Single Binary**: Dependency hell yok, kolay deployment  
- **🔧 Simplicity**: Minimal setup, sadece binary copy & run
- **🛠️ Learning**: JavaScript runtime internals'ını anlamak için mükemmel
- **🚀 Potential**: Optimizasyon ve unique features için alan var

> **Not**: v0.1.0'da Kiren henüz Node.js'den hızlı değil. Bu bir öğrenme projesi ve functional runtime olarak değerlendirilmeli. Detaylı benchmark sonuçları için [`benchmarks/BENCHMARK_RESULTS.md`](benchmarks/BENCHMARK_RESULTS.md) dosyasına bakın.

## Geliştirme Durumu

### ✅ Tamamlanan Özellikler
- [x] **Temel V8 entegrasyonu** - Tam JavaScript desteği
- [x] **Console API** - `console.log()`, `console.time()`, `console.timeEnd()`
- [x] **REPL modu** - İnteraktif JavaScript ortamı (.exit, .help)
- [x] **CLI interface** - Dosya çalıştırma ve komut satırı
- [x] **Timer APIs** - `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`
- [x] **Fetch API** - HTTP requests (Promise-based)
- [x] **File System API** - `fs.readFile`, `fs.writeFile`, `fs.exists`, `fs.mkdir`
- [x] **Process API** - `process.env`, `process.argv`, `process.cwd()`, `process.exit()`
- [x] **🔥 HTTP Server API** - Zero-config web server with routing

### 🔄 Geliştirilmekte
- [ ] Error handling & stack traces
- [ ] Timer callback execution
- [ ] ES Modules
- [ ] CommonJS

### 📋 Planlanmış Özellikler
- [ ] TypeScript desteği
- [ ] Package manager entegrasyonu
- [ ] WebAssembly desteği
- [ ] HTTP Server API
- [ ] Worker Threads

## 🤝 Katkıda Bulunma

Kiren'e katkıda bulunmak istiyorsanız:

1. **[CONTRIBUTING.md](CONTRIBUTING.md)** dosyasını okuyun
2. Repository'yi fork edin
3. Development setup yapın: `cargo build`
4. Feature branch oluşturun: `git checkout -b feature/amazing-feature`
5. Değişikliklerinizi test edin: `cargo test`
6. Pull Request oluşturun

### 🎯 Katkı Alanları

- **Performance optimization** - V8 entegrasyonu iyileştirme
- **Stability fixes** - Segfault'ları düzeltme
- **New APIs** - Timer callbacks, ES Modules
- **Documentation** - API guides, examples
- **Testing** - Unit tests, integration tests

Detaylar için: [CONTRIBUTING.md](CONTRIBUTING.md)

## Lisans

MIT lisansı altında dağıtılmaktadır. Detaylar için `LICENSE` dosyasına bakın.

## İletişim

Mert Can Altin - [@mertcanaltin](https://github.com/mertcanaltin)

Proje Linki: [https://github.com/mertcanaltin/kiren](https://github.com/mertcanaltin/kiren)