// Test timer callbacks specifically

console.log("⏰ Testing Enhanced Timer Callbacks");
console.log("==================================");

// Test string callback
console.log("Setting timeout with string callback...");
setTimeout("console.log('⚡ String timeout callback executed!')", 100);

// Test function callback (will show placeholder)
console.log("Setting timeout with function callback...");
setTimeout(function() {
    console.log("This should work eventually!");
}, 200);

// Test interval with string callback  
console.log("Setting interval with string callback...");
let count = 0;
const intervalId = setInterval("count++; console.log('🔄 Interval #' + count)", 300);

// Clear interval after a few executions
setTimeout("clearInterval('" + intervalId + "')", 1500);

console.log("Timer tests initialized. Wait for callbacks...");