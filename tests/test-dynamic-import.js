// Test dynamic import functionality

console.log("🧪 Testing Dynamic Import Features");
console.log("=================================");

// Test 1: Built-in modules
console.log("\n1️⃣ Testing Built-in Module Imports:");

import('fs').then(fs => {
    console.log("✅ fs module loaded:", typeof fs);
    console.log("   fs.readFile:", typeof fs.readFile);
    console.log("   fs.writeFile:", typeof fs.writeFile);
}).catch(err => {
    console.log("❌ fs import failed:", err.message);
});

import('path').then(path => {
    console.log("✅ path module loaded:", typeof path);
    console.log("   path.join:", typeof path.join);
    console.log("   path.resolve:", typeof path.resolve);
}).catch(err => {
    console.log("❌ path import failed:", err.message);
});

import('http').then(http => {
    console.log("✅ http module loaded:", typeof http);
    console.log("   http.createServer:", typeof http.createServer);
}).catch(err => {
    console.log("❌ http import failed:", err.message);
});

// Test 2: Non-existent module
console.log("\n2️⃣ Testing Non-existent Module:");
import('./non-existent.js').then(module => {
    console.log("✅ Non-existent module fallback:", typeof module);
    console.log("   module.default:", typeof module.default);
}).catch(err => {
    console.log("❌ Non-existent module failed:", err.message);
});

console.log("\n🎉 Dynamic import tests completed!");