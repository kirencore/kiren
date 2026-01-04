# Kiren API Reference

Kiren is a lightweight JavaScript runtime built with Zig and QuickJS. This document describes the available APIs.

## Table of Contents

- [Bundle Command](#bundle-command)
- [Kiren Global Object](#kiren-global-object)
- [HTTP Server](#http-server)
- [WebSocket Server](#websocket-server)
- [SQLite Database](#sqlite-database)
- [File System](#file-system)
- [Path](#path)
- [Process](#process)
- [Buffer](#buffer)
- [URL](#url)
- [Encoding](#encoding)
- [Crypto](#crypto)
- [Timers](#timers)
- [Console](#console)
- [Module System](#module-system)
- [Compatibility Libraries](#compatibility-libraries)

---

## Bundle Command

Create standalone executables from your JavaScript applications.

### Basic Usage

```bash
# Create standalone executable
kiren bundle app.js

# Specify output name
kiren bundle app.js -o myapp

# Create JS bundle only (no executable)
kiren bundle app.js --js-only
```

### How It Works

The bundle command:
1. Parses your entry file and finds all `require()` calls
2. Recursively bundles all local dependencies
3. Creates a self-contained executable with embedded JavaScript

### Example

```javascript
// app.js
const utils = require("./utils.js");
console.log("Sum:", utils.add(2, 3));

// utils.js
module.exports = {
  add: function(a, b) { return a + b; }
};
```

```bash
kiren bundle app.js -o myapp
./myapp
# Output: Sum: 5
```

### Output Options

| Option | Description |
|--------|-------------|
| `-o <name>` | Output file name (default: input name without .js) |
| `--js-only` | Create bundled JS file instead of executable |

### Notes

- Only local dependencies (`./` or `../` paths) are bundled
- Built-in modules (express, axios, etc.) are available at runtime
- The output executable is self-contained and requires no external files

---

## Kiren Global Object

The `Kiren` object is available globally and provides core runtime functionality.

### Properties

#### `Kiren.version`

Returns the current Kiren version string.

```javascript
console.log(Kiren.version); // "0.1.0"
```

---

## HTTP Server

### `Kiren.serve(options)`

Starts an HTTP server.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `options.port` | `number` | Port to listen on |
| `options.fetch` | `function` | Request handler function |

**Request object properties:**

| Property | Type | Description |
|----------|------|-------------|
| `method` | `string` | HTTP method (GET, POST, etc.) |
| `url` | `string` | Request URL path |
| `headers` | `object` | Request headers |
| `body` | `string` | Request body (for POST/PUT) |

**Example:**

```javascript
Kiren.serve({
  port: 3000,
  fetch: function(req) {
    if (req.url === "/") {
      return new Response(JSON.stringify({ message: "Hello" }), {
        status: 200,
        headers: { "Content-Type": "application/json" }
      });
    }
    return new Response("Not Found", { status: 404 });
  }
});
```

### `Response`

Creates an HTTP response.

**Constructor:**

```javascript
new Response(body, options)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `body` | `string` | Response body |
| `options.status` | `number` | HTTP status code |
| `options.headers` | `object` | Response headers |

---

## WebSocket Server

### `Kiren.ws(options)`

Starts a WebSocket server.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `options.port` | `number` | Port to listen on |
| `options.open` | `function` | Called when client connects |
| `options.message` | `function` | Called when message received |
| `options.close` | `function` | Called when client disconnects |

**WebSocket object properties:**

| Property | Type | Description |
|----------|------|-------------|
| `ws.id` | `string` | Unique client identifier |

**Example:**

```javascript
Kiren.ws({
  port: 8080,
  open: function(ws) {
    console.log("Connected:", ws.id);
  },
  message: function(ws, data) {
    console.log("Received:", data);
  },
  close: function(ws) {
    console.log("Disconnected:", ws.id);
  }
});
```

### `Kiren.wsSend(ws, message)`

Sends a message to a specific client.

| Parameter | Type | Description |
|-----------|------|-------------|
| `ws` | `object` | WebSocket client object |
| `message` | `string` | Message to send |

### `Kiren.wsBroadcast(message)`

Sends a message to all connected clients.

| Parameter | Type | Description |
|-----------|------|-------------|
| `message` | `string` | Message to broadcast |

### `Kiren.wsJoinRoom(ws, roomId)`

Adds a client to a room.

| Parameter | Type | Description |
|-----------|------|-------------|
| `ws` | `object` | WebSocket client object |
| `roomId` | `string` | Room identifier |

### `Kiren.wsBroadcastRoom(roomId, message)`

Sends a message to all clients in a room.

| Parameter | Type | Description |
|-----------|------|-------------|
| `roomId` | `string` | Room identifier |
| `message` | `string` | Message to broadcast |

---

## SQLite Database

Kiren includes native SQLite support for embedded database operations. No external dependencies required.

### `Kiren.sqlite(path)`

Opens or creates a SQLite database.

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `string` | Database file path, or `:memory:` for in-memory database |

**Returns:** `Database` object

```javascript
// In-memory database
const db = Kiren.sqlite(':memory:');

// File-based database
const db = Kiren.sqlite('app.db');
```

### Database Methods

#### `db.exec(sql)`

Executes one or more SQL statements. Use for DDL commands (CREATE, DROP, etc.) and statements that don't return data.

| Parameter | Type | Description |
|-----------|------|-------------|
| `sql` | `string` | SQL statement(s) to execute |

```javascript
db.exec('CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)');
db.exec('CREATE INDEX idx_email ON users(email)');
```

#### `db.run(sql, params)`

Executes a single SQL statement with optional parameters. Use for INSERT, UPDATE, DELETE operations.

| Parameter | Type | Description |
|-----------|------|-------------|
| `sql` | `string` | SQL statement with `?` placeholders |
| `params` | `array` | Parameter values (optional) |

**Returns:** `object` with:
- `changes` - Number of rows affected
- `lastInsertRowid` - Last inserted row ID

```javascript
const result = db.run('INSERT INTO users (name, email) VALUES (?, ?)', ['Mert', 'mert@example.com']);
console.log(result.lastInsertRowid); // 1
console.log(result.changes); // 1

db.run('UPDATE users SET name = ? WHERE id = ?', ['Mert K.', 1]);
db.run('DELETE FROM users WHERE id = ?', [1]);
```

#### `db.query(sql, params)`

Executes a SELECT query and returns all matching rows.

| Parameter | Type | Description |
|-----------|------|-------------|
| `sql` | `string` | SELECT statement with `?` placeholders |
| `params` | `array` | Parameter values (optional) |

**Returns:** `array` of row objects

```javascript
// Get all users
const users = db.query('SELECT * FROM users');
// [{ id: 1, name: 'Mert', email: 'mert@example.com' }, ...]

// With parameters
const user = db.query('SELECT * FROM users WHERE id = ?', [1]);

// With conditions
const active = db.query('SELECT * FROM users WHERE age > ? AND status = ?', [18, 'active']);
```

#### `db.close()`

Closes the database connection. Always close when done to free resources.

```javascript
db.close();
```

### Supported Data Types

| JavaScript Type | SQLite Type |
|-----------------|-------------|
| `number` (integer) | INTEGER |
| `number` (float) | REAL |
| `string` | TEXT |
| `null` / `undefined` | NULL |
| `boolean` | INTEGER (0 or 1) |

### Complete Example

```javascript
const db = Kiren.sqlite(':memory:');

// Create tables
db.exec(`
  CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    price REAL,
    stock INTEGER DEFAULT 0
  )
`);

// Insert data
db.run('INSERT INTO products (name, price, stock) VALUES (?, ?, ?)', ['Laptop', 999.99, 10]);
db.run('INSERT INTO products (name, price, stock) VALUES (?, ?, ?)', ['Mouse', 29.99, 50]);
db.run('INSERT INTO products (name, price, stock) VALUES (?, ?, ?)', ['Keyboard', 79.99, 25]);

// Query data
const expensive = db.query('SELECT * FROM products WHERE price > ?', [50]);
console.log(expensive);
// [{ id: 1, name: 'Laptop', price: 999.99, stock: 10 },
//  { id: 3, name: 'Keyboard', price: 79.99, stock: 25 }]

// Aggregate query
const total = db.query('SELECT COUNT(*) as count, SUM(price) as total FROM products');
console.log(total[0]); // { count: 3, total: 1109.97 }

db.close();
```

---

## File System

The `fs` module provides synchronous file system operations. Available globally without require.

### `fs.readFileSync(path, encoding)`

Reads a file synchronously.

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `string` | File path |
| `encoding` | `string` | Encoding (e.g., "utf8") |

**Returns:** `string` - File contents

```javascript
const content = fs.readFileSync("config.json", "utf8");
```

### `fs.writeFileSync(path, data)`

Writes data to a file.

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `string` | File path |
| `data` | `string` | Data to write |

```javascript
fs.writeFileSync("output.txt", "Hello World");
```

### `fs.existsSync(path)`

Checks if a file or directory exists.

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `string` | Path to check |

**Returns:** `boolean`

### `fs.statSync(path)`

Returns file statistics.

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `string` | File path |

**Returns:** `object` with properties:
- `size` - File size in bytes
- `isFile` - Whether path is a file
- `isDirectory` - Whether path is a directory

### `fs.readdirSync(path)`

Reads directory contents.

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `string` | Directory path |

**Returns:** `array` - Array of filenames

### `fs.mkdirSync(path)`

Creates a directory.

### `fs.unlinkSync(path)`

Deletes a file.

### `fs.rmdirSync(path)`

Removes a directory.

---

## Path

The `path` module provides path manipulation utilities. Available globally without require.

### Properties

| Property | Description |
|----------|-------------|
| `path.sep` | Path separator ("/" on POSIX) |
| `path.delimiter` | Path delimiter (":" on POSIX) |

### Methods

### `path.join(...paths)`

Joins path segments.

```javascript
path.join("/foo", "bar", "baz"); // "/foo/bar/baz"
```

### `path.dirname(path)`

Returns the directory name.

```javascript
path.dirname("/foo/bar/file.txt"); // "/foo/bar"
```

### `path.basename(path, ext)`

Returns the last portion of a path.

```javascript
path.basename("/foo/bar/file.txt");        // "file.txt"
path.basename("/foo/bar/file.txt", ".txt"); // "file"
```

### `path.extname(path)`

Returns the file extension.

```javascript
path.extname("index.html"); // ".html"
```

### `path.normalize(path)`

Normalizes a path.

```javascript
path.normalize("/foo/bar//baz/../qux"); // "/foo/bar/qux"
```

### `path.resolve(...paths)`

Resolves a sequence of paths to an absolute path.

```javascript
path.resolve("foo", "bar"); // "/current/working/dir/foo/bar"
```

### `path.isAbsolute(path)`

Determines if a path is absolute.

```javascript
path.isAbsolute("/foo/bar"); // true
path.isAbsolute("foo/bar");  // false
```

### `path.parse(path)`

Returns an object with path components.

```javascript
path.parse("/home/user/file.txt");
// { root: "/", dir: "/home/user", base: "file.txt", ext: ".txt", name: "file" }
```

### `path.format(pathObject)`

Returns a path string from an object.

---

## Process

The `process` object provides information about the current process. Available globally.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `process.platform` | `string` | Operating system platform |
| `process.arch` | `string` | CPU architecture |
| `process.pid` | `number` | Process ID |
| `process.env` | `object` | Environment variables |
| `process.argv` | `array` | Command line arguments |

### Methods

### `process.cwd()`

Returns the current working directory.

```javascript
console.log(process.cwd()); // "/Users/user/project"
```

### `process.exit(code)`

Exits the process with the specified code.

```javascript
process.exit(0); // Success
process.exit(1); // Error
```

---

## Buffer

The `Buffer` class handles binary data. Available globally.

### Static Methods

### `Buffer.alloc(size)`

Creates a zero-filled buffer of specified size.

```javascript
const buf = Buffer.alloc(10);
```

### `Buffer.from(data)`

Creates a buffer from a string or array.

```javascript
const buf1 = Buffer.from("Hello");
const buf2 = Buffer.from([72, 101, 108, 108, 111]);
```

### `Buffer.concat(buffers)`

Concatenates an array of buffers.

```javascript
const combined = Buffer.concat([buf1, buf2]);
```

### `Buffer.isBuffer(obj)`

Tests if an object is a Buffer.

### `Buffer.byteLength(data)`

Returns byte length of a string or buffer.

### Instance Methods

### `buffer.toString()`

Converts buffer to string.

### `buffer.slice(start, end)`

Returns a new buffer that references the same memory.

### `buffer.copy(target, targetStart)`

Copies data to target buffer.

### `buffer.write(string, offset)`

Writes a string to the buffer.

### `buffer.fill(value)`

Fills the buffer with a value.

### `buffer.equals(otherBuffer)`

Compares two buffers.

### `buffer.readUInt8(offset)` / `buffer.writeUInt8(value, offset)`

Reads/writes unsigned 8-bit integer.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `buffer.length` | `number` | Buffer size in bytes |

---

## URL

The `URL` class provides URL parsing. Available globally.

### Constructor

```javascript
const url = new URL("https://example.com:8080/path?query=value#hash");
```

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `href` | `string` | Full URL |
| `protocol` | `string` | Protocol (e.g., "https:") |
| `hostname` | `string` | Host name |
| `port` | `string` | Port number |
| `pathname` | `string` | Path |
| `search` | `string` | Query string with "?" |
| `hash` | `string` | Fragment with "#" |
| `origin` | `string` | Origin (protocol + host) |
| `searchParams` | `URLSearchParams` | Query parameters |

### Methods

### `url.toString()`

Returns the full URL string.

---

## URLSearchParams

Handles URL query parameters. Available globally.

### Constructor

```javascript
const params = URLSearchParams("foo=1&bar=2");
```

### Methods

### `params.get(name)`

Returns the first value for a parameter.

### `params.getAll(name)`

Returns all values for a parameter.

### `params.has(name)`

Checks if a parameter exists.

### `params.set(name, value)`

Sets a parameter value.

### `params.append(name, value)`

Appends a value to a parameter.

### `params.delete(name)`

Removes a parameter.

### `params.toString()`

Returns the query string.

---

## Encoding

### TextEncoder

Encodes strings to UTF-8 bytes.

```javascript
const encoder = new TextEncoder();
const bytes = encoder.encode("Hello");
```

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `encoding` | `string` | Always "utf-8" |

#### Methods

- `encode(string)` - Returns `Uint8Array`

### TextDecoder

Decodes UTF-8 bytes to strings.

```javascript
const decoder = new TextDecoder();
const text = decoder.decode(bytes);
```

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `encoding` | `string` | Always "utf-8" |

#### Methods

- `decode(buffer)` - Returns `string`

---

## Crypto

The `crypto` object provides cryptographic functionality. Available globally.

### `crypto.randomUUID()`

Generates a random UUID v4.

```javascript
const id = crypto.randomUUID(); // "550e8400-e29b-41d4-a716-446655440000"
```

### `crypto.getRandomValues(typedArray)`

Fills a typed array with random values.

```javascript
const arr = new Uint8Array(16);
crypto.getRandomValues(arr);
```

---

## Timers

Timer functions are available globally.

### `setTimeout(callback, delay)`

Executes a callback after a delay.

| Parameter | Type | Description |
|-----------|------|-------------|
| `callback` | `function` | Function to execute |
| `delay` | `number` | Delay in milliseconds |

**Returns:** `number` - Timer ID

### `clearTimeout(id)`

Cancels a timeout.

### `setInterval(callback, interval)`

Executes a callback repeatedly.

| Parameter | Type | Description |
|-----------|------|-------------|
| `callback` | `function` | Function to execute |
| `interval` | `number` | Interval in milliseconds |

**Returns:** `number` - Timer ID

### `clearInterval(id)`

Cancels an interval.

---

## Console

The `console` object provides logging functions.

### `console.log(...args)`

Outputs a message to stdout.

### `console.warn(...args)`

Outputs a warning message.

### `console.error(...args)`

Outputs an error message to stderr.

---

## Module System

Kiren supports CommonJS modules.

### `require(path)`

Loads a module.

```javascript
const myModule = require("./mymodule.js");
const config = require("./config.json");
```

### `module.exports`

Exports values from a module.

```javascript
// mymodule.js
module.exports = {
  hello: function() { return "Hello"; }
};

// or
module.exports = function() { return "Hello"; };
```

### Module Resolution

1. Relative paths: `./module.js`, `../module.js`
2. JSON files: `./config.json`
3. lib/ directory: `require("express")` loads `lib/express.js`

---

## Compatibility Libraries

Kiren includes drop-in replacements for popular npm packages in the `lib/` directory.

### express

Express.js compatible HTTP framework.

```javascript
const express = require("express");
const app = express();

app.get("/", function(req, res) {
  res.json({ message: "Hello" });
});

app.post("/users", function(req, res) {
  res.status(201).json(req.body);
});

app.listen(3000, function() {
  console.log("Server running on port 3000");
});
```

**Supported features:**
- `app.get()`, `app.post()`, `app.put()`, `app.delete()`
- `app.use()` for middleware
- `express.Router()` for modular routes
- `express.json()` body parser
- `express.cors()` CORS middleware
- `express.static()` static file serving
- Route parameters (`:id`)
- Query string parsing
- Cookie parsing
- `req.params`, `req.query`, `req.body`, `req.headers`
- `res.json()`, `res.send()`, `res.status()`, `res.set()`

#### Static File Serving

Serve static files from a directory:

```javascript
const express = require("express");
const app = express();

// Serve files from ./public directory
app.use(express.static("./public"));

// With options
app.use(express.static("./assets", {
  index: "index.html"  // Default index file (set to false to disable)
}));

app.listen(3000);
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `index` | `string\|false` | `"index.html"` | Index file for directories |

**Supported MIME types:**
- `.html` → `text/html`
- `.css` → `text/css`
- `.js` → `text/javascript`
- `.json` → `application/json`
- `.png` → `image/png`
- `.jpg` → `image/jpeg`
- Other files → `application/octet-stream`

**Security features:**
- Path traversal protection (`..` blocked)
- URL decoding with error handling

### axios

HTTP client for making requests.

```javascript
const axios = require("axios");

// GET request
const response = axios.get("https://api.example.com/users");
console.log(response.data);

// POST request
axios.post("https://api.example.com/users", {
  name: "John"
});

// With config
axios.request({
  method: "GET",
  url: "https://api.example.com/data",
  params: { page: 1 },
  headers: { "Authorization": "Bearer token" }
});
```

**Supported features:**
- `axios.get()`, `axios.post()`, `axios.put()`, `axios.delete()`, `axios.patch()`
- `axios.create()` for instances with defaults
- Request config: `baseURL`, `headers`, `params`, `data`, `auth`
- Response object: `data`, `status`, `statusText`, `headers`

### jsonwebtoken

JWT token handling.

```javascript
const jwt = require("jsonwebtoken");

// Create token
const token = jwt.sign({ userId: 123 }, "secret", { expiresIn: "1h" });

// Decode token (without verification)
const decoded = jwt.decode(token);

// Verify token
const payload = jwt.verify(token, "secret");
```

**Supported features:**
- `jwt.sign()` with `expiresIn` option
- `jwt.decode()` with `complete` option
- `jwt.verify()` with expiration check

### dotenv

Environment variable loader.

```javascript
require("dotenv").config();

console.log(process.env.API_KEY);
```

**Supported features:**
- Loads `.env` file into `process.env`
- Custom path via `config({ path: ".env.local" })`
- Comment and quote handling

### uuid

UUID generation.

```javascript
const { v4: uuidv4 } = require("uuid");

const id = uuidv4(); // "550e8400-e29b-41d4-a716-446655440000"
```

---

## fetch

Global fetch function for HTTP requests.

```javascript
const response = fetch("https://api.example.com/data", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ key: "value" })
});

const data = response.json();
```

**Request options:**

| Option | Type | Description |
|--------|------|-------------|
| `method` | `string` | HTTP method |
| `headers` | `object` | Request headers |
| `body` | `string` | Request body |

**Response object:**

| Property/Method | Description |
|-----------------|-------------|
| `status` | HTTP status code |
| `statusText` | Status message |
| `headers` | Response headers |
| `json()` | Parse body as JSON |
| `text()` | Get body as text |
