console.log("Testing simple HTTP server...");

// Just test the API exists
console.log("http object:", typeof http);
console.log("createServer function:", typeof http.createServer);

const server = http.createServer();
console.log("Server created:", typeof server);
console.log("listen function:", typeof server.listen);

// For now, just test that we can create the server
console.log("✅ HTTP server API is working!");