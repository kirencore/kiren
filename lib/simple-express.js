// Simple Express compatibility for Kiren

var _routes = [];

function createApp() {
  _routes = [];

  return {
    get: function(path, handler) {
      _routes.push({ method: "GET", path: path, handler: handler });
    },
    post: function(path, handler) {
      _routes.push({ method: "POST", path: path, handler: handler });
    },
    use: function() {},
    listen: function(port, cb) {
      console.log("Starting on port " + port);
      Kiren.serve({
        port: port,
        fetch: handleReq
      });
      if (cb) cb();
    }
  };
}

function handleReq(raw) {
  var url = raw.url || "/";
  var method = raw.method || "GET";

  // Simple path matching
  for (var i = 0; i < _routes.length; i++) {
    var r = _routes[i];
    if (r.method === method && r.path === url) {
      var body = "";
      var status = 200;
      var headers = {};

      var res = {
        json: function(d) {
          body = JSON.stringify(d);
          headers["Content-Type"] = "application/json";
        },
        status: function(s) { status = s; return this; }
      };

      r.handler({}, res);
      return new Response(body, { status: status, headers: headers });
    }
  }

  return new Response(JSON.stringify({error: "Not Found"}), { status: 404 });
}

module.exports = createApp;
