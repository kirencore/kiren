var express = require("kiren-express");
var app = express();

// Serve static files from test_static/public
app.use(express.static("test_static/public"));

// API route
app.get("/api/hello", function(req, res) {
  res.json({ message: "Hello from API!" });
});

app.listen(3000, function() {
  console.log("Server running on http://localhost:3000");
});
