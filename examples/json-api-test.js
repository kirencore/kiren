import { createServer } from 'kiren/http';

const server = createServer();

// Test data
const users = [
  { id: 1, name: "Ceren", email: "ceren@example.com" },
  { id: 2, name: "Ali", email: "ali@example.com" },
  { id: 3, name: "Muazzez", email: "muazzez@example.com" }
];

const config = {
  version: "1.0.0",
  environment: "development",
  features: ["auth", "users", "api"]
};

// JSON API endpoints
server.get("/", () => "Kiren JSON API Server");
server.get("/users", () => users);                    // Should return JSON
server.get("/config", () => config);                  // Should return JSON  
server.get("/health", () => ({ status: "ok", uptime: Date.now() }));  // Should return JSON

server.listen(3001);