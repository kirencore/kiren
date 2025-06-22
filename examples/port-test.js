console.log("Testing port detection...");

const server = http.createServer();

server.get("/", () => {
    return "Hello World!";
});

server.listen(8080);

console.log("Server should be on port 8080");