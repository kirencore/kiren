// Promise test for Kiren

console.log("1. Starting promise test...");

// Simple promise
const promise1 = new Promise((resolve) => {
    console.log("2. Promise executor running");
    resolve("Hello from Promise!");
});

promise1.then((result) => {
    console.log("3. Promise resolved:", result);
});

// Promise with setTimeout
const delay = (ms) => new Promise((resolve) => {
    setTimeout(() => resolve(ms), ms);
});

delay(100).then((ms) => {
    console.log(`4. Delayed promise resolved after ${ms}ms`);
});

// Promise chain
Promise.resolve(1)
    .then(x => x + 1)
    .then(x => x * 2)
    .then(x => console.log("5. Promise chain result:", x));

// Async/await style (using promises)
const asyncTest = () => {
    return delay(50).then(() => {
        console.log("6. Async operation completed");
        return "done";
    });
};

asyncTest().then(result => {
    console.log("7. Final result:", result);
});

console.log("8. All promises scheduled (this runs before resolutions)");
