// Combined HTTP + WebSocket Server Test
console.log("Starting combined HTTP + WebSocket server...");

Kiren.serve({
  port: 3000,

  // HTTP handler
  fetch: function(req) {
    console.log("HTTP:", req.method, req.url);

    if (req.url === "/") {
      return new Response(JSON.stringify({
        message: "Welcome to Kiren!",
        endpoints: {
          http: "GET /api/*",
          websocket: "ws://localhost:3000"
        }
      }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    if (req.url === "/api/status") {
      return new Response(JSON.stringify({
        status: "ok",
        runtime: "kiren",
        websocket: "enabled"
      }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    return new Response(JSON.stringify({ error: "Not Found" }), {
      status: 404,
      headers: { "Content-Type": "application/json" }
    });
  },

  // WebSocket handler
  websocket: {
    open: function(ws) {
      console.log("WS client connected:", ws.id);
      Kiren.wsSend(ws, JSON.stringify({
        type: "connected",
        id: ws.id
      }));
    },

    message: function(ws, data) {
      console.log("WS message from", ws.id, ":", data);

      try {
        var msg = JSON.parse(data);

        if (msg.type === "ping") {
          Kiren.wsSend(ws, JSON.stringify({ type: "pong" }));
        } else if (msg.type === "broadcast") {
          Kiren.wsBroadcast(JSON.stringify({
            type: "broadcast",
            from: ws.id,
            message: msg.message
          }));
        } else {
          // Echo
          Kiren.wsSend(ws, JSON.stringify({
            type: "echo",
            data: msg
          }));
        }
      } catch (e) {
        Kiren.wsSend(ws, JSON.stringify({
          type: "echo",
          data: data
        }));
      }
    },

    close: function(ws) {
      console.log("WS client disconnected:", ws.id);
    }
  }
});

console.log("");
console.log("Server running on http://localhost:3000");
console.log("WebSocket on ws://localhost:3000");
console.log("");
console.log("Test HTTP: curl http://localhost:3000/api/status");
console.log("Test WS:   Open test_ws_client.html in browser");
