console.log("🚀 Starting Kiren HTTP Server...");

// Create HTTP server
const server = http.createServer();

// Register routes
server.get("/", (req, res) => {
    console.log("GET / called");
    return "Hello from Kiren!";
});

server.get("/api/users", (req, res) => {
    console.log("GET /api/users called");
    return { users: ["Alice", "Bob", "Charlie"] };
});

server.post("/api/data", (req, res) => {
    console.log("POST /api/data called");
    return { message: "Data received" };
});

// Start server
console.log("Starting server on port 3000...");
server.listen(3000);

console.log("🎉 Server ready! Visit http://localhost:3000");