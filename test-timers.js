// Timer API Test
console.log("Testing Timer API...");

// Test function callbacks with simple outputs
setTimeout(function() {
    console.log("setTimeout function callback executed!");
}, 100);

// Test string callbacks
setTimeout("console.log('setTimeout string callback executed!')", 200);

// Test setInterval function callbacks (simplified)
const intervalId = setInterval(function() {
    console.log("setInterval function callback executed!");
}, 300);

// Clear interval after a short time
setTimeout(function() {
    clearInterval(intervalId);
    console.log("Interval cleared");
}, 1000);

// Test clearTimeout
const timeoutId = setTimeout(function() {
    console.log("This should NOT be executed");
}, 2000);

clearTimeout(timeoutId);
console.log("Timeout cleared before execution");

// Final message
setTimeout(function() {
    console.log("Timer API tests completed!");
}, 1500);