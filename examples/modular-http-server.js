// Modern ES Modules ile Kiren HTTP Server örneği
console.log("🚀 Starting Modular Kiren HTTP Server...");

// HTTP modülünü import et
import("kiren/http").then(async (http) => {
    console.log("✅ HTTP module loaded successfully");
    
    // Server oluştur
    const server = http.createServer();
    console.log("🔧 HTTP server created");

    // Routes tanımla
    server.get("/", (req, res) => {
        console.log("📝 GET / called");
        return "Hello from Modular Kiren! 🎯";
    });

    server.get("/api/status", (req, res) => {
        console.log("📊 GET /api/status called");
        return {
            status: "ok",
            message: "Modular Kiren API is running!",
            runtime: "kiren",
            moduleSystem: "ES Modules"
        };
    });

    server.post("/api/echo", (req, res) => {
        console.log("🔄 POST /api/echo called");
        return {
            message: "Echo from modular server",
            timestamp: new Date().toISOString()
        };
    });

    // Server'ı başlat
    console.log("🌐 Starting server on port 3001...");
    server.listen(3001);
    
    console.log("🎉 Modular server ready!");
    console.log("📋 Available routes:");
    console.log("  - GET  http://localhost:3001/");
    console.log("  - GET  http://localhost:3001/api/status");
    console.log("  - POST http://localhost:3001/api/echo");
    console.log("  - GET  http://localhost:3001/health (built-in)");
    
}).catch(error => {
    console.error("❌ Failed to load HTTP module:", error);
});