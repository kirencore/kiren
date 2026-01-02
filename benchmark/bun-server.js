// Bun HTTP Server for benchmark comparison

Bun.serve({
  port: 3001,
  fetch(req) {
    const url = new URL(req.url);

    if (url.pathname === "/" || url.pathname === "") {
      return new Response("Welcome to Bun!");
    }

    if (url.pathname === "/json") {
      return Response.json({
        message: "Hello from Bun!",
        timestamp: Date.now()
      });
    }

    return new Response("Not Found", { status: 404 });
  }
});

console.log("Bun server listening on http://localhost:3001");
