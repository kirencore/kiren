// Kiren HTTP Server Example

Kiren.serve({
  port: 3000,
  fetch(req) {
    console.log(req.method, req.url);

    if (req.url === "/" || req.url === "") {
      return new Response("Welcome to Kiren! 🚀");
    }

    if (req.url === "/json") {
      return Response.json({
        message: "Hello from Kiren!",
        version: Kiren.version,
        timestamp: Date.now()
      });
    }

    if (req.url === "/html") {
      return new Response("<h1>Hello Kiren!</h1><p>This is HTML</p>", {
        headers: { "content-type": "text/html" }
      });
    }

    return new Response("Not Found", { status: 404 });
  }
});
