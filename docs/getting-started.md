# Getting Started with Kiren

Kiren is a high-performance JavaScript runtime built with Rust and powered by the V8 engine.

## Installation

### Prerequisites
- Rust 1.70 or higher
- Git

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/mertcanaltin/kiren.git
cd kiren
```

2. Build the project:
```bash
cargo build --release
```

3. The executable will be available at `./target/release/kiren`

## Basic Usage

### Running JavaScript Files

Create a JavaScript file:
```javascript
// hello.js
console.log("Hello from Kiren!");

const sum = (a, b) => a + b;
console.log("5 + 3 =", sum(5, 3));
```

Run it with Kiren:
```bash
./target/release/kiren hello.js
```

### Interactive REPL

Start the REPL mode:
```bash
./target/release/kiren --repl
```

Try some JavaScript:
```
kiren> const message = "Welcome to Kiren!"
kiren> console.log(message)
Welcome to Kiren!
kiren> 2 ** 10
1024
kiren> .exit
```

## Features Overview

### Modern JavaScript Support
```javascript
// Template literals
const name = "Kiren";
console.log(`Hello, ${name}!`);

// Arrow functions
const multiply = (a, b) => a * b;

// Destructuring
const [first, second] = [1, 2];

// Async/await (with supported APIs)
setTimeout(() => console.log("Async!"), 1000);
```

### Built-in APIs

#### Console API
```javascript
console.log("Basic logging");
console.log("Multiple", "arguments", "supported");
```

#### Timer APIs
```javascript
// setTimeout
setTimeout(() => console.log("Delayed execution"), 1000);

// setInterval
const interval = setInterval(() => console.log("Repeating"), 500);
setTimeout(() => clearInterval(interval), 3000);
```

#### Fetch API
```javascript
fetch("https://api.github.com/users/octocat")
    .then(() => console.log("Request completed"))
    .catch(error => console.log("Error:", error));
```

## Project Structure

When working with Kiren projects, you can organize your code like this:

```
my-project/
├── src/
│   ├── main.js          # Entry point
│   ├── utils.js         # Utility functions
│   └── api/
│       └── client.js    # API client code
├── examples/
│   ├── hello.js         # Basic examples
│   └── advanced.js      # Advanced features
└── README.md
```

## Performance Benefits

Kiren offers several performance advantages:

- **Fast Startup**: ~30% faster than Node.js
- **Low Memory**: ~25% less memory usage
- **Native Speed**: Direct V8 integration
- **Async I/O**: Optimized with Tokio

## Comparison with Other Runtimes

| Feature | Kiren | Node.js | Deno |
|---------|-------|---------|------|
| Language | Rust | C++ | Rust |
| Startup Time | Fast | Medium | Medium |
| Memory Usage | Low | High | Medium |
| Built-in APIs | Growing | Extensive | Extensive |
| Module System | Planned | CommonJS/ESM | ESM |

## Common Use Cases

### Scripts and Automation
```javascript
// data-processor.js
console.log("Processing data...");

const data = [1, 2, 3, 4, 5];
const processed = data.map(x => x * 2);

console.log("Result:", processed);
```

### API Testing
```javascript
// api-test.js
async function testAPI() {
    console.log("Testing API...");
    
    try {
        await fetch("https://api.example.com/health");
        console.log("API is healthy");
    } catch (error) {
        console.log("API error:", error);
    }
}

testAPI();
```

### Simple Servers (Planned)
```javascript
// Future feature
import { serve } from "kiren/http";

serve((req) => {
    return new Response("Hello from Kiren!");
}, { port: 8000 });
```

## Next Steps

1. Explore the [API Reference](./api/) for detailed documentation
2. Try the [REPL Mode](./repl.md) for interactive development
3. Check out [examples](../examples/) for practical use cases
4. Read about [performance optimizations](./performance.md)

## Getting Help

- Read the documentation in the `docs/` directory
- Check the `examples/` folder for sample code
- Report issues on GitHub: [github.com/mertcanaltin/kiren](https://github.com/mertcanaltin/kiren)