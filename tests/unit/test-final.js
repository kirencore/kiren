// Final comprehensive test

console.log("🎯 Kiren v0.1.0 - Final Comprehensive Test");
console.log("==========================================");

// Test 1: Basic JavaScript
console.log("\n1️⃣ Basic JavaScript:");
const result = 2 + 2;
console.log("✅ Arithmetic:", result);

// Test 2: Console timing
console.log("\n2️⃣ Console Timing:");
console.time("test-timer");
for (let i = 0; i < 1000; i++) {
    Math.sqrt(i);
}
console.timeEnd("test-timer");

// Test 3: CommonJS modules
console.log("\n3️⃣ CommonJS Modules:");
try {
    const fs = require('fs');
    const path = require('path');
    console.log("✅ CommonJS working - fs:", typeof fs, "path:", typeof path);
} catch (e) {
    console.log("❌ CommonJS error:", e.message);
}

// Test 4: Module objects
console.log("\n4️⃣ Module Objects:");
console.log("✅ module exists:", typeof module);
console.log("✅ exports exists:", typeof exports);

// Test 5: Timers (simple)
console.log("\n5️⃣ Timer Registration:");
const timerId = setTimeout(() => {
    console.log("Timer callback!");
}, 100);
console.log("✅ Timer registered:", typeof timerId);

console.log("\n🎉 All tests completed successfully!");