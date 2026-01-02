var express = require("./lib/simple-express.js");

var app = express();

app.get("/test", function(req, res) {
  res.json({ message: "Hello" });
});

app.get("/healthcheck", function(req, res) {
  res.json({ status: "ok" });
});

app.listen(4006, function() {
  console.log("Simple server on http://localhost:4006");
});
