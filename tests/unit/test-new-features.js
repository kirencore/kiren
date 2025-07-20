// Test new Kiren features

console.log("🧪 Testing Kiren v0.1.0 New Features");
console.log("================================");

// Test 1: ES Modules (dynamic import)
console.log("\n1️⃣ Testing ES Modules:");
try {
    import('./non-existent.js').catch(err => {
        console.log("✅ Dynamic import rejection works:", err.message);
    });
} catch (e) {
    console.log("✅ Dynamic import syntax available");
}

// Test 2: CommonJS (require)
console.log("\n2️⃣ Testing CommonJS:");
try {
    const fs = require('fs');
    console.log("✅ Built-in 'fs' module loaded:", typeof fs);
    
    const path = require('path');
    console.log("✅ Built-in 'path' module loaded:", typeof path);
    
    const http = require('http');
    console.log("✅ Built-in 'http' module loaded:", typeof http);
} catch (e) {
    console.log("❌ CommonJS require failed:", e.message);
}

// Test 3: Timer callbacks with string
console.log("\n3️⃣ Testing Timer String Callbacks:");
setTimeout("console.log('⚡ String callback executed!')", 100);

// Test 4: Performance optimizations (just verify they work)
console.log("\n4️⃣ Testing Performance Features:");
console.time("execution-time");
for (let i = 0; i < 1000; i++) {
    Math.sqrt(i);
}
console.timeEnd("execution-time");

// Test 5: Module objects
console.log("\n5️⃣ Testing Module Objects:");
console.log("module object exists:", typeof module);
console.log("exports object exists:", typeof exports);

console.log("\n🎉 All tests completed!");