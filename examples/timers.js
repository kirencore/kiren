console.log("Testing Timer APIs...");

// Test setTimeout
console.log("Setting timeout for 1 second...");
const timer1 = setTimeout(() => {
    console.log("Timer 1 executed!");
}, 1000);

// Test setInterval  
console.log("Setting interval for 500ms...");
const timer2 = setInterval(() => {
    console.log("Interval executed!");
}, 500);

// Clear interval after 3 seconds
setTimeout(() => {
    console.log("Clearing interval...");
    clearInterval(timer2);
}, 3000);

console.log("Timer IDs:", timer1, timer2);