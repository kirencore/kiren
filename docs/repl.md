# REPL Mode

Kiren includes an interactive REPL (Read-Eval-Print Loop) for testing JavaScript code in real-time.

## Starting REPL

```bash
kiren --repl
# or
kiren -r
```

## Features

### Interactive JavaScript Execution
```
kiren> 2 + 2
4
kiren> const name = "World"
kiren> `Hello, ${name}!`
Hello, World!
```

### Multi-line Support
```
kiren> function fibonacci(n) {
...      if (n <= 1) return n;
...      return fibonacci(n - 1) + fibonacci(n - 2);
...    }
kiren> fibonacci(10)
55
```

### Built-in APIs
All Kiren APIs are available in REPL mode:

```
kiren> console.log("Testing console API")
Testing console API

kiren> setTimeout(() => console.log("Timer works!"), 1000)
8f4a2b1c-... (timer ID)
Timer works!

kiren> fetch("https://api.github.com")
Promise { <pending> }
Fetch success: ...
```

## Commands

### `.exit`
Exit the REPL:
```
kiren> .exit
Goodbye!
```

## Tips and Tricks

### Variable Persistence
Variables and functions persist across REPL sessions:
```
kiren> let counter = 0
kiren> counter++
0
kiren> counter
1
```

### Inspecting Objects
```
kiren> const obj = { a: 1, b: 2 }
kiren> obj
[object Object]
```

### Testing Async Code
```
kiren> async function test() { return "async result"; }
kiren> test()
Promise { <pending> }
```

### Error Handling
```
kiren> invalidFunction()
Error: invalidFunction is not defined
```

## Performance Notes

- Each input creates a new V8 execution context
- Variables and functions persist in the global scope
- Long-running operations won't block the REPL prompt
- Memory usage is optimized for interactive development

## Limitations

Current limitations of the REPL:
- No syntax highlighting
- No auto-completion
- No history navigation (coming soon)
- No bracket matching

## Advanced Usage

### Testing APIs
```
kiren> // Test timer functions
kiren> let count = 0
kiren> const timer = setInterval(() => { count++; console.log(count); }, 1000)
1
2
3
kiren> clearInterval(timer)
```

### Debugging
```
kiren> function debug(x) { console.log("Debug:", x); return x; }
kiren> debug(42)
Debug: 42
42
```

### Code Snippets
```
kiren> // Utility functions
kiren> const range = (n) => Array.from({length: n}, (_, i) => i)
kiren> range(5)
0,1,2,3,4
```