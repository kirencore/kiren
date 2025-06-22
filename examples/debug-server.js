console.log("Debug: Testing HTTP responses...");

const server = http.createServer();

// String response test
server.get("/string", "This is a direct string response");

// Function response test  
server.get("/", () => {
    console.log("Debug: / route callback called");
    return "Hello from function callback!";
});

server.get("/test", () => {
    console.log("Debug: /test route callback called");
    return { message: "JSON response from callback", working: true };
});

server.listen(3001);
console.log("Debug server ready on port 3001");