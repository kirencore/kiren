// Clean ES Module static import example
console.log("Starting Real Static Import Example...");

// Static imports - real ES module syntax
import fs from 'kiren/fs';
import { createServer } from 'kiren/http';
import { readFile, writeFile } from 'kiren/fs';

console.log("Static imports completed successfully!");

// Test FS operations with named imports
console.log("Testing named import FS operations...");
writeFile("static-import-test.txt", "Hello from real static imports!");
const content = readFile("static-import-test.txt");
console.log("File content:", content);

// Test FS operations with default import
console.log("Testing default import FS operations...");
const dirExists = fs.exists(".");
console.log("Current directory exists:", dirExists);

// Test HTTP server with named import
console.log("Creating server with named import...");
const server = createServer();

server.get("/", () => {
    return "Hello from Real Static Import Server!";
});

server.get("/api/info", () => {
    return {
        message: "Real static imports working!",
        importType: "ES Modules static import",
        syntax: "import { createServer } from 'kiren/http'",
        runtime: "kiren",
        timestamp: new Date().toISOString()
    };
});

server.listen(3004);

console.log("Real static import example ready!");
console.log("Server running on http://localhost:3004");
console.log("Available endpoints:");
console.log("  - GET http://localhost:3004/");
console.log("  - GET http://localhost:3004/api/info");

export { server };
export default { fs, createServer, readFile, writeFile };