// Simple timer test without variables

console.log("⏰ Simple Timer Test");
console.log("===================");

// Test with simple string callback
setTimeout("console.log('✅ Simple string callback works!')", 100);

// Test with math operation
setTimeout("console.log('✅ Math result:', 5 + 3)", 200);

// Test function callback
setTimeout(function() {
    console.log("✅ Function callback works!");
}, 300);

console.log("Timer tests set up. Waiting for callbacks...");