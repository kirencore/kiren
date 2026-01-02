// Test WebSocket server
console.log("Starting WebSocket test server...");

Kiren.ws({
  port: 8080,
  open: function(ws) {
    console.log("Client connected:", ws.id);
    Kiren.wsSend(ws, JSON.stringify({ type: "welcome", message: "Hello!" }));
  },
  message: function(ws, data) {
    console.log("Received from", ws.id, ":", data);

    // Echo back
    Kiren.wsSend(ws, JSON.stringify({ type: "echo", data: data }));

    // Broadcast to all
    Kiren.wsBroadcast(JSON.stringify({ type: "broadcast", from: ws.id, data: data }));
  },
  close: function(ws) {
    console.log("Client disconnected:", ws.id);
  }
});

console.log("WebSocket server running on ws://localhost:8080");
