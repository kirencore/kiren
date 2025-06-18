# Console API

The Console API provides logging functionality for debugging and output display.

## Methods

### `console.log(...args)`

Outputs a message to the console.

**Parameters:**
- `...args` - Any number of values to log

**Example:**
```javascript
console.log("Hello, World!");
console.log("Value:", 42);
console.log("Multiple", "arguments", "are", "supported");
```

**Output:**
```
Hello, World!
Value: 42
Multiple arguments are supported
```

## Advanced Usage

### Logging Objects
```javascript
const user = { name: "John", age: 30 };
console.log("User:", user);
```

### Template Literals
```javascript
const name = "Kiren";
console.log(`Welcome to ${name}!`);
```

### Expressions
```javascript
console.log("2 + 2 =", 2 + 2);
console.log("Current time:", new Date().toISOString());
```

## Performance Notes

- Console.log is synchronous and will block execution
- Avoid excessive logging in production code
- Arguments are converted to strings using JavaScript's `toString()` method