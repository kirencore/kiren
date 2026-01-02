// Test body parsing
Kiren.serve({
  port: 4010,
  fetch: function(req) {
    console.log("URL:", req.url);
    console.log("Method:", req.method);
    console.log("Body:", req.body);
    console.log("Body type:", typeof req.body);
    console.log("Headers:", JSON.stringify(req.headers));

    var bodyData = null;
    if (req.body) {
      try {
        bodyData = JSON.parse(req.body);
      } catch (e) {
        bodyData = req.body;
      }
    }

    return new Response(JSON.stringify({
      receivedBody: bodyData,
      rawBody: req.body
    }), {
      headers: { "Content-Type": "application/json" }
    });
  }
});

console.log("Test server on http://localhost:4010");
