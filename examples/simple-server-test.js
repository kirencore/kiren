console.log("Testing simple HTTP server...");

const server = http.createServer();

server.get("/", () => {
    console.log("Route / called!");
    return "Hello World from Kiren!";
});

server.get("/test", () => {
    console.log("Route /test called!");
    return "Test endpoint working!";
});

console.log("Creating server...");
server.listen(3000);
console.log("Server setup complete. Should be accessible at http://localhost:3000");