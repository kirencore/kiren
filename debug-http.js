const http = require("http");
console.log("Available methods on http:", Object.keys(http));

const server = http.createServer();
console.log("Available methods on server:", Object.keys(server));
console.log("server.get:", typeof server.get);
console.log("server.listen:", typeof server.listen);
