// Comprehensive import demonstration - All supported patterns
console.log("🚀 Comprehensive Import Demo - All Patterns");

// 1. Named imports
import { readFile, writeFile, exists } from 'kiren/fs';
import { createServer } from 'kiren/http';

// 2. Default imports  
import fs from 'kiren/fs';

console.log("✅ All imports loaded successfully!");

// Test all import patterns
console.log("\n📁 Testing named imports (FS)...");
writeFile("comprehensive-test.txt", "Hello from comprehensive import demo!");
const content = readFile("comprehensive-test.txt");
console.log("📄 Content from named import:", content);

console.log("\n📂 Testing default import (FS)...");
const fileExists = fs.exists("comprehensive-test.txt");
console.log("🔍 File exists (default import):", fileExists);

// Also test default fs methods
fs.writeFile("default-test.txt", "Hello from default import!");
const defaultContent = fs.readFile("default-test.txt");
console.log("📄 Content from default import:", defaultContent);

console.log("\n🌐 Testing named import (HTTP)...");
const server = createServer();

server.get("/", () => {
    return "Comprehensive Import Demo Server! 🎯";
});

server.get("/api/import-test", () => {
    return {
        message: "All import patterns working!",
        supportedPatterns: {
            namedImports: "import { readFile } from 'kiren/fs'",
            defaultImports: "import fs from 'kiren/fs'",
            mixedUsage: "Both patterns in same file"
        },
        runtime: "kiren",
        moduleSystem: "ES Modules (transformed)",
        timestamp: new Date().toISOString()
    };
});

server.get("/api/fs-demo", () => {
    // Use both named and default imports in API
    const namedExists = exists("comprehensive-test.txt");
    const defaultExists = fs.exists("default-test.txt");
    
    return {
        namedImportResult: namedExists,
        defaultImportResult: defaultExists,
        message: "Both import patterns working in same context"
    };
});

server.listen(3005);

console.log("\n🎉 Comprehensive import demo ready!");
console.log("🔗 Server running on http://localhost:3005");
console.log("📋 Available endpoints:");
console.log("  - GET  http://localhost:3005/");
console.log("  - GET  http://localhost:3005/api/import-test");
console.log("  - GET  http://localhost:3005/api/fs-demo");
console.log("  - GET  http://localhost:3005/health (built-in)");

console.log("\n✨ Summary: All import patterns supported:");
console.log("  ✅ Named imports: import { createServer } from 'kiren/http'");
console.log("  ✅ Default imports: import fs from 'kiren/fs'");  
console.log("  ✅ Mixed usage: Both patterns in same file");
console.log("  ✅ Dynamic imports: import('kiren/fs') (from previous examples)");
console.log("  ✅ Global APIs: Still available for backward compatibility");

// Export example (commented out for now)
// export { server, fs, createServer };