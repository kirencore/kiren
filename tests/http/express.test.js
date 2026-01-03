// Simple test for kiren-express
var express = require("./lib/kiren-express.js");

var app = express();

app.get("/test", function(req, res) {
  console.log("[handler] /test handler called");
  res.json({ message: "Hello from test" });
});

app.get("/healthcheck", function(req, res) {
  console.log("[handler] /healthcheck handler called");
  res.json({ status: "ok" });
});

app.listen(4005, function() {
  console.log("Test server on http://localhost:4005");
  console.log("Try: curl http://localhost:4005/test");
  console.log("Try: curl http://localhost:4005/healthcheck");
});
