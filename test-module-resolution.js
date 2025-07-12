// Test current module resolution capabilities
console.log("Testing Module Resolution...");

// Test CommonJS built-in modules
console.log("--- Built-in Modules ---");
try {
    const fs = require('fs');
    console.log("✓ fs module loaded:", typeof fs);
    console.log("  - readFileSync:", typeof fs.readFileSync);
    console.log("  - writeFileSync:", typeof fs.writeFileSync);
    console.log("  - existsSync:", typeof fs.existsSync);
} catch (e) {
    console.log("✗ fs module failed:", e.message);
}

try {
    const path = require('path');
    console.log("✓ path module loaded:", typeof path);
    console.log("  - join:", typeof path.join);
} catch (e) {
    console.log("✗ path module failed:", e.message);
}

try {
    const http = require('http');
    console.log("✓ http module loaded:", typeof http);
    console.log("  - createServer:", typeof http.createServer);
} catch (e) {
    console.log("✗ http module failed:", e.message);
}

try {
    const express = require('express');
    console.log("✓ express module loaded:", typeof express);
    console.log("  - Router:", typeof express.Router);
    console.log("  - static:", typeof express.static);
} catch (e) {
    console.log("✗ express module failed:", e.message);
}

// Test unknown modules (should return empty objects)
console.log("--- Unknown Modules ---");
try {
    const unknown = require('unknown-module');
    console.log("✓ unknown module returns empty object:", Object.keys(unknown).length === 0);
} catch (e) {
    console.log("✗ unknown module failed:", e.message);
}

console.log("Module resolution tests completed!");