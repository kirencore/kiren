// Static import polyfill test
// Bu şimdilik dynamic import kullanarak static import'u simüle ediyor

console.log("🔧 Testing static import simulation...");

// Static import simulasyonu - gerçek ES modules syntax değil ama benzer
const { readFile, writeFile, exists } = await import("kiren/fs");
const { createServer } = await import("kiren/http");

console.log("✅ Modules imported successfully");
console.log("📦 Available FS functions:", Object.keys({ readFile, writeFile, exists }));
console.log("📦 Available HTTP functions:", Object.keys({ createServer }));

// FS kullanımı
console.log("📁 Testing FS operations...");
writeFile("static-test.txt", "Hello from static import simulation!");
const content = readFile("static-test.txt");
console.log("📄 File content:", content);

const fileExists = exists("static-test.txt");
console.log("🔍 File exists:", fileExists);

// HTTP kullanımı
console.log("🌐 Testing HTTP server...");
const server = createServer();

server.get("/static", () => {
    return "Hello from static import simulation server!";
});

server.get("/api/test", () => {
    return {
        message: "Static import simulation working",
        timestamp: new Date().toISOString(),
        importType: "simulated-static"
    };
});

server.listen(3003);

console.log("🎉 Static import simulation test completed!");
console.log("🔗 Server running on http://localhost:3003");
console.log("  - GET /static");
console.log("  - GET /api/test");