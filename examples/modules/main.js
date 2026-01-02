// Test require() module system

console.log("=== Module System Test ===\n");

// Test requiring local .js files
console.log("--- Loading math.js ---");
const math = require("./math");
console.log("math.add(2, 3):", math.add(2, 3));
console.log("math.multiply(4, 5):", math.multiply(4, 5));
console.log("math.PI:", math.PI);

console.log("\n--- Loading greet.js ---");
const greet = require("./greet");
console.log(greet.greet("Kiren"));
console.log(greet.farewell("World"));
console.log("greet.version:", greet.version);

// Test requiring JSON files
console.log("\n--- Loading config.json ---");
const config = require("./config.json");
console.log("config.name:", config.name);
console.log("config.version:", config.version);
console.log("config.settings.port:", config.settings.port);

// Test module caching (should use cached version)
console.log("\n--- Testing module caching ---");
const math2 = require("./math");
console.log("Same module?:", math === math2);

// Test __filename and __dirname (inside a module)
console.log("\n--- __filename and __dirname ---");
const info = require("./info");
console.log("info.filename:", info.filename);
console.log("info.dirname:", info.dirname);

console.log("\n=== All Module Tests Complete ===");
