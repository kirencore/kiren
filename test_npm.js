// Test NPM compatibility features
console.log("Testing Kiren v2.0 NPM Compatibility");

// Test CommonJS globals
console.log("__dirname:", __dirname);
console.log("__filename:", __filename);

// Test require for built-in modules
const fs = require('fs');
const path = require('path');
const os = require('os');

console.log("OS platform:", os.platform);

// Test path operations
const joinedPath = path.join('test', 'path', 'example.js');
console.log("Joined path:", joinedPath);

// Test file operations
try {
    console.log("Creating test file...");
    fs.writeFileSync('test.txt', 'Hello from Kiren!');
    
    console.log("Reading test file...");
    const content = fs.readFileSync('test.txt');
    console.log("File content:", content);
    
    console.log("Checking if file exists...");
    const exists = fs.existsSync('test.txt');
    console.log("File exists:", exists);
} catch (error) {
    console.error("File operation error:", error.message);
}

// Test module.exports
module.exports = { name: "test-module", version: "1.0.0" };
console.log("Module exports:", module.exports);

// Test exports shorthand
exports.hello = function() {
    return "Hello from exports!";
};
console.log("Exports hello:", exports.hello());