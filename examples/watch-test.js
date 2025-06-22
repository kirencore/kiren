import { createServer } from 'kiren/http';

const server = createServer();

// Test message  
console.log("Server starting... v3.0 - LIGHTNING FAST!");

server.get("/", () => "Hello from Watch Mode! v3.0 - INSTANT RELOAD!");
server.get("/api", () => ({ message: "API v3.0 - BLAZING FAST PERFORMANCE", timestamp: Date.now() }));

server.listen(3002);