// Test Gurubu Backend Compatibility
console.log("🚀 Testing Kiren compatibility with Gurubu backend requirements...");

console.log("\n=== Environment Variables ===");
try {
    console.log("NODE_ENV:", process.env.NODE_ENV);
    console.log("PORT:", process.env.PORT);
    console.log("process.platform:", process.platform);
    console.log("process.arch:", process.arch);
    console.log("process.version:", process.version);
    console.log("✅ Process object working");
} catch (e) {
    console.log("❌ Process object failed:", e.message);
}

console.log("\n=== Crypto Operations ===");
try {
    const crypto = require('crypto');
    
    // Test createHash
    const hash = crypto.createHash('sha256');
    hash.update('test data');
    const digest = hash.digest('hex');
    console.log("SHA256 hash:", digest);
    
    // Test randomBytes
    const randomBytes = crypto.randomBytes(16);
    console.log("Random bytes (hex):", randomBytes.toString('hex'));
    
    // Test randomUUID
    const uuid = crypto.randomUUID();
    console.log("Random UUID:", uuid);
    
    console.log("✅ Crypto module working");
} catch (e) {
    console.log("❌ Crypto module failed:", e.message);
}

console.log("\n=== File System Operations ===");
try {
    const fs = require('fs');
    
    // Test writeFileSync
    fs.writeFileSync('./test-file.txt', 'Hello from Kiren!', 'utf8');
    console.log("✅ File written successfully");
    
    // Test readFileSync
    const content = fs.readFileSync('./test-file.txt', 'utf8');
    console.log("File content:", content);
    
    // Test existsSync
    const exists = fs.existsSync('./test-file.txt');
    console.log("File exists:", exists);
    
    // Test statSync
    const stats = fs.statSync('./test-file.txt');
    console.log("File size:", stats.size);
    console.log("Is file:", stats.isFile());
    
    console.log("✅ File system operations working");
} catch (e) {
    console.log("❌ File system failed:", e.message);
}

console.log("\n=== Timer Operations (Already Working) ===");
try {
    setTimeout(() => {
        console.log("✅ setTimeout working");
    }, 100);
    
    console.log("✅ Timer functions available");
} catch (e) {
    console.log("❌ Timers failed:", e.message);
}

console.log("\n=== Module System ===");
try {
    // Test built-in modules
    const path = require('path');
    const joined = path.join('src', 'api', 'test.ts');
    console.log("Path join:", joined);
    
    const events = require('events');
    const emitter = new events.EventEmitter();
    emitter.on('test', () => console.log("✅ EventEmitter working"));
    emitter.emit('test');
    
    console.log("✅ Module system working");
} catch (e) {
    console.log("❌ Module system failed:", e.message);
}

console.log("\n=== Express Framework ===");
try {
    const express = require('express');
    const app = express();
    
    app.get('/test', (req, res) => {
        res.json({ message: 'Hello from Kiren!' });
    });
    
    console.log("✅ Express app created successfully");
} catch (e) {
    console.log("❌ Express failed:", e.message);
}

console.log("\n🎯 Kiren Gurubu Compatibility Test Completed!");
console.log("Ready to run Node.js backends with enhanced features!");