# Timer APIs

Kiren provides timer functions for delayed and repeated execution of code.

## Methods

### `setTimeout(callback, delay)`

Executes a function after a specified delay.

**Parameters:**
- `callback` (Function) - Function to execute
- `delay` (Number) - Delay in milliseconds

**Returns:**
- `String` - Timer ID that can be used with `clearTimeout()`

**Example:**
```javascript
const timerId = setTimeout(() => {
    console.log("This runs after 1 second");
}, 1000);

console.log("Timer ID:", timerId);
```

### `clearTimeout(timerId)`

Cancels a timeout previously established by calling `setTimeout()`.

**Parameters:**
- `timerId` (String) - Timer ID returned by `setTimeout()`

**Example:**
```javascript
const timerId = setTimeout(() => {
    console.log("This will not run");
}, 5000);

// Cancel the timeout
clearTimeout(timerId);
```

### `setInterval(callback, delay)`

Repeatedly executes a function with a fixed time delay between each call.

**Parameters:**
- `callback` (Function) - Function to execute repeatedly
- `delay` (Number) - Delay between executions in milliseconds

**Returns:**
- `String` - Timer ID that can be used with `clearInterval()`

**Example:**
```javascript
const intervalId = setInterval(() => {
    console.log("This runs every 500ms");
}, 500);

// Stop after 3 seconds
setTimeout(() => {
    clearInterval(intervalId);
    console.log("Interval stopped");
}, 3000);
```

### `clearInterval(intervalId)`

Cancels an interval previously established by calling `setInterval()`.

**Parameters:**
- `intervalId` (String) - Timer ID returned by `setInterval()`

**Example:**
```javascript
const intervalId = setInterval(() => {
    console.log("Repeating...");
}, 1000);

// Stop the interval after 5 seconds
setTimeout(() => {
    clearInterval(intervalId);
}, 5000);
```

## Advanced Usage

### Combining Timers
```javascript
let counter = 0;

const intervalId = setInterval(() => {
    counter++;
    console.log(`Count: ${counter}`);
    
    if (counter >= 5) {
        clearInterval(intervalId);
        console.log("Counting complete!");
    }
}, 1000);
```

### Timer Chaining
```javascript
setTimeout(() => {
    console.log("Step 1");
    
    setTimeout(() => {
        console.log("Step 2");
        
        setTimeout(() => {
            console.log("Step 3");
        }, 1000);
    }, 1000);
}, 1000);
```

## Performance Notes

- Timers use UUID-based IDs for thread safety
- Timers are managed using Tokio's async runtime
- Clearing a timer that doesn't exist is safe (no error)
- Very short delays (< 1ms) may not be precise due to system limitations

## Implementation Details

- Built on Tokio's `sleep` and `interval` functions
- Timer IDs are UUID v4 strings for uniqueness
- Timers are stored in a concurrent HashMap for thread safety
- Cancelled timers are immediately removed from memory