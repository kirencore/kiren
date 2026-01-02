// Async test for Kiren event loop

console.log("Starting async test...");

// Test setTimeout
setTimeout(() => {
    console.log("1. Timeout after 100ms");
}, 100);

setTimeout(() => {
    console.log("2. Timeout after 200ms");
}, 200);

setTimeout(() => {
    console.log("3. Timeout after 50ms (should run first)");
}, 50);

// Test setInterval
let count = 0;
const intervalId = setInterval(() => {
    count++;
    console.log(`4. Interval tick #${count}`);

    if (count >= 3) {
        clearInterval(intervalId);
        console.log("5. Interval cleared!");
    }
}, 80);

// Test clearTimeout
const cancelMe = setTimeout(() => {
    console.log("This should NOT appear!");
}, 150);
clearTimeout(cancelMe);

console.log("Scheduled all timers, waiting for execution...");
