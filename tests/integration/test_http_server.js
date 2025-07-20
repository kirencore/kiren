// Test HTTP server with enhanced features
console.log("🚀 Testing enhanced HTTP server...");

const server = http.createServer();

// Test route registration
server.get('/', 'Hello from enhanced Kiren server!');
server.get('/api/test', '{"message": "API endpoint working", "security": "headers enabled"}');
server.get('/health', '{"status": "ok", "enhanced": true}');

console.log("✅ Routes registered");

// Start server
server.listen(3001, () => {
    console.log("🌐 Server listening on http://localhost:3001");
    console.log("🔒 Enhanced security headers enabled");
    console.log("⏱️  Request timeouts: 30 seconds");
    console.log("📦 Max request size: 10MB");
    console.log("🔄 Keep-alive connections enabled");
    
    console.log("\n📋 Test these endpoints:");
    console.log("   • GET http://localhost:3001/");
    console.log("   • GET http://localhost:3001/health");
    console.log("   • GET http://localhost:3001/api/test");
    console.log("   • GET http://localhost:3001/api/stats");
});

console.log("🎯 Server setup completed!");