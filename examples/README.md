# Kiren Examples

This directory contains example JavaScript applications demonstrating Kiren's capabilities.

## 🚀 HTTP Server Demo

### Quick Start
```bash
# Run the production demo
./target/release/kiren examples/production-demo.js

# Visit http://localhost:3000 in your browser
```

### What You'll See
- ✅ Zero-config HTTP server
- ✅ Built-in routing
- ✅ JSON API endpoints
- ✅ Environment variable support
- ✅ Production-ready HTML responses

## 📊 Available Examples

### Core Runtime
- `hello.js` - Basic JavaScript execution
- `fibonacci-test.js` - CPU performance test with console.time
- `loop-test.js` - Loop performance benchmark

### APIs
- `timers.js` - setTimeout/setInterval examples
- `filesystem.js` - File operations (read/write/mkdir)
- `process.js` - Environment variables and command line args
- `fetch.js` - HTTP requests (Promise-based)

### HTTP Server
- `simple-server.js` - Basic HTTP API test
- `http-server.js` - Route registration demo
- `production-demo.js` - **Full production example**

## 🐳 Docker Deployment

```bash
# Build container
docker build -t kiren-app .

# Run in production
docker run -p 3000:3000 -e NODE_ENV=production kiren-app

# Container size: ~20MB (vs Node.js ~500MB)
```

## 🔧 Development Workflow

```bash
# Interactive development
./target/release/kiren --repl

# Run any example
./target/release/kiren examples/[filename].js

# Check performance
cd benchmarks && ./run-benchmarks.sh
```

## 💡 Use Cases

### 1. Microservices
```javascript
// Zero-dependency API server
const server = http.createServer();
server.get('/api/health', () => ({ status: 'ok' }));
server.listen(process.env.PORT || 3000);
```

### 2. Automation Scripts
```javascript
// System administration
const logs = fs.readFile('/var/log/app.log');
const errors = logs.split('\n').filter(line => line.includes('ERROR'));
console.log(`Found ${errors.length} errors`);
```

### 3. Edge Computing
```javascript
// Minimal resource usage
const response = await fetch(process.env.API_ENDPOINT);
const data = await response.text();
fs.writeFile('/tmp/cache.json', data);
```

## 🎯 Performance Notes

- **Binary Size**: ~15MB (single file)
- **Memory Usage**: Minimal overhead
- **Startup Time**: <100ms cold start
- **Dependencies**: Zero external requirements

## 🔍 Debugging

```bash
# Enable verbose logging
RUST_LOG=debug ./target/release/kiren app.js

# Check binary info
file target/release/kiren
ls -lh target/release/kiren
```