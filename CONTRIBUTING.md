# Contributing to Kiren

Thank you for your interest in contributing to Kiren. This document provides guidelines and information for contributors.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Set up the development environment (see [Building](docs/BUILDING.md))
4. Create a feature branch

```bash
git clone https://github.com/YOUR_USERNAME/kiren.git
cd kiren
git checkout -b feature/your-feature-name
```

## Development Setup

### Requirements

- Zig 0.14.0 or later
- Git
- A C compiler (for QuickJS compilation)

### Building

```bash
# Debug build
zig build

# Release build
zig build -Doptimize=ReleaseFast

# Run tests
zig build test
```

### Running

```bash
# Run directly
zig build run -- script.js

# Or use the built binary
./zig-out/bin/kiren script.js
```

## Project Structure

```
kiren/
├── src/
│   ├── main.zig       # Entry point, argument parsing
│   ├── engine.zig     # QuickJS engine wrapper
│   └── api/
│       ├── console.zig    # console.log, warn, error
│       ├── fs.zig         # File system operations
│       ├── path.zig       # Path utilities
│       ├── process.zig    # Process information
│       ├── buffer.zig     # Buffer implementation
│       ├── url.zig        # URL parsing
│       ├── encoding.zig   # TextEncoder/TextDecoder
│       ├── crypto.zig     # Crypto APIs
│       ├── http.zig       # HTTP server
│       ├── websocket.zig  # WebSocket server
│       ├── fetch.zig      # HTTP client
│       └── module.zig     # Module system
├── lib/               # JavaScript compatibility libraries
├── deps/quickjs/      # QuickJS source
├── tests/             # Test files
├── examples/          # Example applications
└── docs/              # Documentation
```

## Code Style

### Zig

- Follow the Zig style guide
- Use descriptive variable and function names
- Add comments for complex logic
- Keep functions focused and small

```zig
// Good
pub fn parseHttpRequest(allocator: Allocator, data: []const u8) !HttpRequest {
    // Implementation
}

// Avoid
pub fn parse(a: Allocator, d: []const u8) !HttpRequest {
    // Implementation
}
```

### JavaScript

- Use `var` for compatibility (QuickJS ES2020)
- Use double quotes for strings
- Add JSDoc comments for public APIs

```javascript
// Good
var express = require("express");

function handleRequest(req, res) {
  res.json({ status: "ok" });
}

// Avoid
let express = require('express')
```

## Adding New APIs

### 1. Create the Zig implementation

Add a new file in `src/api/`:

```zig
// src/api/myapi.zig
const std = @import("std");
const engine = @import("../engine.zig");

pub fn register(ctx: engine.JSContext) void {
    // Register global object or functions
}
```

### 2. Register in engine.zig

```zig
const myapi = @import("api/myapi.zig");

pub fn initRuntime() void {
    // ...
    myapi.register(ctx);
}
```

### 3. Add tests

Create a test file in `tests/api/`:

```javascript
// tests/api/myapi.test.js
console.log("=== MyAPI Test ===");

// Test cases
console.log("Feature works:", myapi.feature());

console.log("=== All Tests Complete ===");
```

### 4. Update documentation

Add the new API to `docs/API.md`.

## Adding Compatibility Libraries

Compatibility libraries go in the `lib/` directory and are loaded via `require()`.

### Guidelines

- Match the original API as closely as possible
- Document any differences from the original
- Use only features available in Kiren
- Keep dependencies minimal

### Example

```javascript
// lib/mylib.js

function myFunction() {
  // Implementation using Kiren APIs
}

module.exports = {
  myFunction: myFunction
};
```

## Testing

### Running Tests

```bash
# Run a specific test
./zig-out/bin/kiren tests/api/buffer.test.js

# Run all API tests
for f in tests/api/*.js; do ./zig-out/bin/kiren "$f"; done
```

### Writing Tests

- Test files should be self-contained
- Print clear output for each test case
- Test both success and error cases

```javascript
console.log("=== Feature Test ===");

// Test case 1
console.log("Test 1:", someFunction() === expected);

// Test case 2
try {
  errorFunction();
  console.log("Test 2: FAIL (should have thrown)");
} catch (e) {
  console.log("Test 2: PASS");
}

console.log("=== Complete ===");
```

## Pull Request Process

1. **Create a branch** from `main`
2. **Make your changes** with clear commits
3. **Test your changes** thoroughly
4. **Update documentation** if needed
5. **Submit a pull request**

### Commit Messages

Use clear, descriptive commit messages:

```
add WebSocket room broadcast support

- implement Kiren.wsJoinRoom() for room management
- implement Kiren.wsBroadcastRoom() for targeted broadcasts
- add tests for room functionality
```

### PR Description

Include:
- What the change does
- Why the change is needed
- How to test it
- Any breaking changes

## Reporting Issues

### Bug Reports

Include:
- Kiren version
- Operating system
- Minimal reproduction code
- Expected vs actual behavior
- Error messages if any

### Feature Requests

Include:
- Use case description
- Proposed API design
- Example usage code

## Areas for Contribution

### High Priority

- Async/await support
- Additional Node.js API compatibility
- Windows support improvements
- Performance optimizations

### Good First Issues

- Adding tests for existing APIs
- Documentation improvements
- Example applications
- Error message improvements

## Questions

For questions about contributing, open a GitHub issue with the "question" label.
