console.log("Testing setTimeout and setInterval...");

// Test setTimeout
setTimeout(() => {
    console.log("✅ setTimeout works! (1000ms)");
}, 1000);

// Test setInterval
let count = 0;
const intervalId = setInterval(() => {
    count++;
    console.log(`✅ setInterval #${count} works! (500ms intervals)`);
    
    if (count >= 3) {
        clearInterval(intervalId);
        console.log("✅ clearInterval works! Stopped after 3 executions");
    }
}, 500);

console.log("Timer tests started - waiting for results...");