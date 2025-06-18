# HTTP Server API

The HTTP Server API provides a zero-configuration web server for building web applications and APIs.

## Overview

Kiren's HTTP Server API is designed for simplicity and production readiness. No external dependencies or complex configuration required.

## Basic Usage

### Creating a Server

```javascript
const server = http.createServer();
```

### Adding Routes

```javascript
// GET route
server.get("/", () => "Hello World!");

// POST route  
server.post("/api/data", () => ({ message: "Data received" }));

// Route with parameters (planned)
server.get("/users/:id", (req) => ({ userId: req.params.id }));
```

### Starting the Server

```javascript
server.listen(3000);
console.log("Server running on http://localhost:3000");
```

## Complete Example

```javascript
// Production-ready server
const server = http.createServer();

// Health check endpoint
server.get("/health", () => ({
    status: "ok",
    runtime: "kiren",
    timestamp: new Date().toISOString()
}));

// API routes
server.get("/api/users", () => ({
    users: [
        { id: 1, name: "Alice" },
        { id: 2, name: "Bob" }
    ]
}));

server.post("/api/users", () => ({
    message: "User created",
    id: 3
}));

// Start server
const PORT = process.env.PORT || 3000;
server.listen(PORT);
console.log(`🚀 Server running on port ${PORT}`);
```

## Methods

### `http.createServer()`

Creates a new HTTP server instance.

**Returns:**
- `Server` - Server object with routing methods

**Example:**
```javascript
const server = http.createServer();
```

### `server.get(path, handler)`

Registers a GET route handler.

**Parameters:**
- `path` (String) - Route path (e.g., "/", "/api/users")
- `handler` (Function) - Route handler function

**Example:**
```javascript
server.get("/", () => "Hello World!");
server.get("/api/data", () => ({ data: "example" }));
```

### `server.post(path, handler)`

Registers a POST route handler.

**Parameters:**
- `path` (String) - Route path
- `handler` (Function) - Route handler function

**Example:**
```javascript
server.post("/api/users", () => ({ message: "User created" }));
```

### `server.listen(port)`

Starts the HTTP server on the specified port.

**Parameters:**
- `port` (Number) - Port number to listen on

**Example:**
```javascript
server.listen(3000);
```

## Built-in Routes

Kiren provides several built-in routes for common use cases:

### Health Check
- **GET /** - Returns "Hello from Kiren HTTP Server! 🚀"
- **GET /health** - Returns JSON health status

```json
{
    "status": "ok",
    "runtime": "kiren", 
    "version": "0.1.0"
}
```

### API Endpoints
- **GET /api/*** - Returns JSON response for any API path

```json
{
    "message": "API endpoint /api/example",
    "runtime": "kiren"
}
```

## Response Types

### String Response
```javascript
server.get("/text", () => "Plain text response");
```

### JSON Response  
```javascript
server.get("/json", () => ({ message: "JSON response" }));
```

### HTML Response
```javascript
server.get("/html", () => `
    <!DOCTYPE html>
    <html>
        <head><title>Kiren App</title></head>
        <body><h1>Hello from Kiren!</h1></body>
    </html>
`);
```

## Environment Configuration

```javascript
// Use environment variables
const PORT = process.env.PORT || 3000;
const ENV = process.env.NODE_ENV || "development";

server.get("/config", () => ({
    port: PORT,
    environment: ENV,
    runtime: "kiren"
}));

server.listen(PORT);
```

## Production Deployment

### Docker Container

```dockerfile
FROM scratch
COPY kiren /kiren
COPY app.js /app.js
ENV PORT=3000
EXPOSE 3000
CMD ["/kiren", "/app.js"]
```

### Single Binary Deployment

```bash
# Build optimized binary
cargo build --release

# Copy to production server
scp target/release/kiren server:/usr/local/bin/
scp app.js server:/opt/app/

# Run on server
./kiren app.js
```

## Performance Characteristics

- **Memory Usage**: Minimal overhead
- **Startup Time**: <100ms cold start
- **Concurrent Connections**: Handled by Tokio async runtime
- **Binary Size**: ~15MB (single file)

## Current Limitations

- Route parameters not yet implemented
- Request body parsing not yet available
- Static file serving not yet implemented
- Custom headers/status codes not yet supported

## Planned Features

### Request Object
```javascript
server.get("/users/:id", (req) => {
    return {
        userId: req.params.id,
        query: req.query,
        headers: req.headers
    };
});
```

### Response Object
```javascript
server.get("/custom", (req, res) => {
    res.status(201);
    res.header("Content-Type", "application/json");
    return { message: "Created" };
});
```

### Static File Serving
```javascript
server.static("/public", "./static/");
```

### Middleware Support
```javascript
server.use((req, res, next) => {
    console.log(`${req.method} ${req.path}`);
    next();
});
```

## Error Handling

Currently, errors in route handlers will be caught and result in a 500 Internal Server Error response. Better error handling is planned for future releases.

## Comparison with Other Frameworks

| Feature | Kiren | Express.js | Fastify |
|---------|-------|------------|---------|
| **Setup** | Zero config | npm install | npm install |
| **Binary Size** | 15MB | 200MB+ | 150MB+ |
| **Dependencies** | 0 | 100+ | 50+ |
| **Startup** | <100ms | 500ms+ | 300ms+ |

## Browser Compatibility

The HTTP server serves standard HTTP responses compatible with all modern browsers and HTTP clients.

## Examples

See the [`examples/`](../../examples/) directory for more HTTP server examples:

- `simple-server.js` - Basic API testing
- `http-server.js` - Route registration
- `production-demo.js` - Full production example