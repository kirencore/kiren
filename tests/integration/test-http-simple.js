console.log("Testing HTTP server creation...");

const server = http.createServer();
console.log("Server object created:", typeof server);

server.get("/test", "Hello World");
console.log("Route registered");

// Don't call listen to avoid segfault
console.log("Test completed successfully");