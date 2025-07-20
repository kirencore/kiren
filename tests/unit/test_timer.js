console.log("Starting timer test...");

setTimeout(() => {
    console.log("Timer 1: Hello from setTimeout after 1000ms!");
}, 1000);

setTimeout(() => {
    console.log("Timer 2: Hello from setTimeout after 2000ms!");
}, 2000);

console.log("Timer test script finished, waiting for callbacks...");