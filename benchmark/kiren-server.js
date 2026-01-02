// Kiren HTTP Server for benchmark comparison

Kiren.serve({
  port: 3000,
  fetch(req) {
    if (req.url === "/" || req.url === "") {
      return new Response("Welcome to Kiren!");
    }

    if (req.url === "/json") {
      return Response.json({
        message: "Hello from Kiren!",
        timestamp: Date.now()
      });
    }

    return new Response("Not Found", { status: 404 });
  }
});
