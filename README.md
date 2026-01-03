# Kiren

A lightweight JavaScript runtime built with Zig and QuickJS.

Kiren is designed for building fast, standalone server applications with minimal footprint. It provides Node.js-compatible APIs while maintaining a small binary size and fast startup time.

## Features

- **Fast startup** - QuickJS-based engine with minimal overhead
- **Small binary** - Single executable under 4MB
- **HTTP server** - Native Zig HTTP server with high throughput
- **WebSocket support** - Built-in WebSocket server with rooms
- **SQLite database** - Native SQLite for embedded data storage
- **Node.js compatibility** - Familiar APIs (fs, path, Buffer, etc.)
- **Module system** - CommonJS require/exports
- **No dependencies** - Self-contained runtime, no npm required

## Quick Start

```javascript
// server.js
const express = require("express");
const app = express();

app.get("/", function(req, res) {
  res.json({ message: "Hello from Kiren!" });
});

app.listen(3000, function() {
  console.log("Server running on http://localhost:3000");
});
```

```bash
kiren server.js
```

## Installation

### From Source

Requires Zig 0.14.0 or later.

```bash
git clone https://github.com/user/kiren.git
cd kiren
zig build -Doptimize=ReleaseFast
```

The binary will be available at `zig-out/bin/kiren`.

### Pre-built Binaries

Coming soon.

## Usage

```bash
# Run a script
kiren script.js

# Run with arguments
kiren server.js --port 8080
```

## API Overview

### HTTP Server

```javascript
Kiren.serve({
  port: 3000,
  fetch: function(req) {
    return new Response(JSON.stringify({ ok: true }), {
      headers: { "Content-Type": "application/json" }
    });
  }
});
```

### WebSocket Server

```javascript
Kiren.ws({
  port: 8080,
  open: function(ws) {
    Kiren.wsSend(ws, "Welcome!");
  },
  message: function(ws, data) {
    Kiren.wsBroadcast(data);
  }
});
```

### File System

```javascript
const content = fs.readFileSync("config.json", "utf8");
fs.writeFileSync("output.txt", "Hello World");
```

### Path Utilities

```javascript
const fullPath = path.join(__dirname, "data", "file.txt");
const ext = path.extname("script.js"); // ".js"
```

### Buffer

```javascript
const buf = Buffer.from("Hello");
const encoded = buf.toString();
```

### HTTP Client

```javascript
const response = fetch("https://api.example.com/data");
const data = response.json();
```

### SQLite Database

```javascript
const db = Kiren.sqlite(':memory:');

db.exec('CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)');
db.run('INSERT INTO users (name) VALUES (?)', ['Mert']);

const users = db.query('SELECT * FROM users');
console.log(users); // [{ id: 1, name: 'Mert' }]

db.close();
```

For complete API documentation, see [docs/API.md](docs/API.md).

## Compatibility Libraries

Kiren includes drop-in replacements for popular npm packages:

| Package | Status |
|---------|--------|
| express | Supported |
| axios | Supported |
| jsonwebtoken | Supported |
| dotenv | Supported |
| uuid | Supported |

```javascript
const express = require("express");
const axios = require("axios");
const jwt = require("jsonwebtoken");
require("dotenv").config();
```

## Project Structure

```
kiren/
├── src/           # Zig source code
│   ├── main.zig   # Entry point
│   ├── engine.zig # QuickJS bindings
│   └── api/       # JavaScript API implementations
├── lib/           # JavaScript compatibility libraries
├── deps/          # Dependencies (QuickJS, SQLite)
├── tests/         # Test files
├── examples/      # Example applications
└── docs/          # Documentation
```

## Performance

Kiren achieves approximately 20,000 requests/second on a single core for simple JSON responses, comparable to other lightweight runtimes.

## Documentation

- [API Reference](docs/API.md)
- [Building from Source](docs/BUILDING.md)
- [Contributing](CONTRIBUTING.md)

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a pull request.

## License

MIT License
