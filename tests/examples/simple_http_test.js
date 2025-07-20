console.log("Creating HTTP server...");

const server = http.createServer();

server.get('/', 'Hello World!');
server.get('/test', '{"message": "test"}');

console.log("Registering routes complete");

server.listen(3000);

console.log("Server started on port 3000");

// Keep the process alive for testing
setTimeout(() => {
    console.log("Server running for 30 seconds...");
}, 30000);