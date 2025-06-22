  import { createServer } from 'kiren/http';
  const server = createServer();

  server.get("/", () => "Hello World!");           // ✅ Works
  server.get("/api", "Static response");           // ✅ Works  
  server.get("/health", () => "Server OK");       // ✅ Works

  server.listen(3000);