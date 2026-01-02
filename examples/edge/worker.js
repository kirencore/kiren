// Kiren Edge Worker
// Cloudflare Workers-style API

// This is how you'd write a Cloudflare Worker
// Kiren aims to be compatible with this pattern

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);

    // Router
    switch (url.pathname) {
      case "/":
        return new Response("Welcome to Kiren Edge! 🚀", {
          headers: { "Content-Type": "text/plain" }
        });

      case "/api/time":
        return Response.json({
          timestamp: Date.now(),
          iso: new Date().toISOString()
        });

      case "/api/random":
        return Response.json({
          uuid: crypto.randomUUID(),
          bytes: Array.from(crypto.getRandomValues(new Uint8Array(8)))
        });

      default:
        return new Response("Not Found", { status: 404 });
    }
  }
};

// For now, we use Kiren.serve() until we implement the Worker API
// This is the bridge between Cloudflare Workers style and Kiren's current API

console.log("🌐 Kiren Edge Worker");
console.log("   Binary: 721KB");
console.log("   Cold Start: <1ms");
console.log("");

// Simulate the worker
const worker = {
  fetch: (req) => {
    const url = new URL("http://localhost" + req.url);

    switch (url.pathname) {
      case "/":
        return new Response("Welcome to Kiren Edge! 🚀");

      case "/api/time":
        return new Response(JSON.stringify({
          timestamp: Date.now(),
          iso: new Date().toISOString()
        }), {
          headers: { "Content-Type": "application/json" }
        });

      case "/api/random":
        return new Response(JSON.stringify({
          uuid: crypto.randomUUID(),
          bytes: Array.from(crypto.getRandomValues(new Uint8Array(8)))
        }), {
          headers: { "Content-Type": "application/json" }
        });

      default:
        return new Response("Not Found", { status: 404 });
    }
  }
};

Kiren.serve({
  port: 8787, // Cloudflare Workers default port
  fetch: worker.fetch
});

console.log("Server running on http://localhost:8787");
console.log("");
console.log("Try:");
console.log("  curl http://localhost:8787/");
console.log("  curl http://localhost:8787/api/time");
console.log("  curl http://localhost:8787/api/random");
