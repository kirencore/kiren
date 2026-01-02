// Kiren Edge API Example
// Demonstrates fetch() + Kiren.serve()

console.log("🚀 Starting Kiren Edge API on port 3000...\n");

Kiren.serve({
  port: 3000,
  fetch: (req) => {
    const url = new URL("http://localhost" + req.url);
    const pathname = url.pathname;

    // Health check
    if (pathname === "/health") {
      return new Response(JSON.stringify({ status: "ok", runtime: "kiren" }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    // Generate UUID
    if (pathname === "/uuid") {
      return new Response(JSON.stringify({ uuid: crypto.randomUUID() }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    // Echo headers
    if (pathname === "/headers") {
      return new Response(JSON.stringify(req.headers), {
        headers: { "Content-Type": "application/json" }
      });
    }

    // Proxy example (fetch from external API)
    if (pathname === "/proxy") {
      const query = url.searchParams.get("url");
      if (query) {
        try {
          const res = fetch(query);
          return new Response(res.text(), {
            status: res.status,
            headers: { "Content-Type": "text/plain" }
          });
        } catch (e) {
          return new Response(JSON.stringify({ error: e.message }), {
            status: 500,
            headers: { "Content-Type": "application/json" }
          });
        }
      }
      return new Response(JSON.stringify({ error: "Missing url parameter" }), {
        status: 400,
        headers: { "Content-Type": "application/json" }
      });
    }

    // 404 for everything else
    return new Response(JSON.stringify({
      error: "Not Found",
      endpoints: ["/health", "/uuid", "/headers", "/proxy?url=..."]
    }), {
      status: 404,
      headers: { "Content-Type": "application/json" }
    });
  }
});

console.log("Endpoints:");
console.log("  GET /health  - Health check");
console.log("  GET /uuid    - Generate UUID");
console.log("  GET /headers - Echo request headers");
console.log("  GET /proxy?url=http://... - Proxy HTTP request");
