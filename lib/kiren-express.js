// Kiren Express - Lightweight Express.js compatibility layer

var _routes = [];
var _middlewares = [];

function createApp() {
  _routes = [];
  _middlewares = [];

  return {
    use: function(fn) {
      if (typeof fn === "function") {
        _middlewares.push(fn);
      }
    },

    get: function(path, handler) {
      _routes.push({ method: "GET", path: path, handler: handler });
    },

    post: function(path, handler) {
      _routes.push({ method: "POST", path: path, handler: handler });
    },

    put: function(path, handler) {
      _routes.push({ method: "PUT", path: path, handler: handler });
    },

    delete: function(path, handler) {
      _routes.push({ method: "DELETE", path: path, handler: handler });
    },

    listen: function(port, cb) {
      console.log("Starting Kiren Express on port " + port);
      console.log("Routes: " + _routes.length);

      Kiren.serve({
        port: port,
        fetch: handleRequest
      });

      if (cb) cb();
    }
  };
}

function handleRequest(raw) {
  var url = raw.url || "/";
  var method = raw.method || "GET";

  // Parse path and query
  var path = url;
  var queryIdx = url.indexOf("?");
  if (queryIdx !== -1) {
    path = url.substring(0, queryIdx);
  }

  // Build request object
  var req = {
    method: method,
    url: url,
    path: path,
    params: {},
    query: {},
    body: null,
    headers: raw.headers || {}
  };

  // Parse query string
  if (queryIdx !== -1) {
    var qs = url.substring(queryIdx + 1);
    var pairs = qs.split("&");
    for (var i = 0; i < pairs.length; i++) {
      var kv = pairs[i].split("=");
      if (kv[0]) {
        req.query[kv[0]] = kv[1] || "";
      }
    }
  }

  // Parse body
  if (raw.body) {
    try {
      req.body = JSON.parse(raw.body);
    } catch (e) {
      req.body = raw.body;
    }
  }

  // Build response object
  var resBody = "";
  var resStatus = 200;
  var resHeaders = {};

  var res = {
    status: function(code) {
      resStatus = code;
      return this;
    },
    set: function(k, v) {
      resHeaders[k] = v;
      return this;
    },
    header: function(k, v) {
      resHeaders[k] = v;
      return this;
    },
    json: function(data) {
      resHeaders["Content-Type"] = "application/json";
      resBody = JSON.stringify(data);
      return this;
    },
    send: function(data) {
      if (typeof data === "object") {
        return this.json(data);
      }
      resBody = String(data);
      return this;
    },
    end: function(data) {
      if (data) resBody = String(data);
      return this;
    }
  };

  // Run middlewares
  for (var i = 0; i < _middlewares.length; i++) {
    _middlewares[i](req, res, function() {});
  }

  // Find matching route
  var matched = null;
  for (var i = 0; i < _routes.length; i++) {
    var route = _routes[i];
    if (route.method !== method) continue;

    var m = matchRoute(route.path, path);
    if (m !== null) {
      matched = route;
      req.params = m;
      break;
    }
  }

  // Execute handler
  if (matched) {
    try {
      matched.handler(req, res);
    } catch (e) {
      resStatus = 500;
      resBody = JSON.stringify({ error: e.message || "Server error" });
    }
  } else {
    resStatus = 404;
    resBody = JSON.stringify({ error: "Not Found" });
  }

  return new Response(resBody, { status: resStatus, headers: resHeaders });
}

function matchRoute(pattern, path) {
  var patternParts = pattern.split("/").filter(function(x) { return x !== ""; });
  var pathParts = path.split("/").filter(function(x) { return x !== ""; });

  if (patternParts.length !== pathParts.length) {
    return null;
  }

  var params = {};
  for (var i = 0; i < patternParts.length; i++) {
    var pp = patternParts[i];
    var pa = pathParts[i];

    if (pp.charAt(0) === ":") {
      params[pp.substring(1)] = pa;
    } else if (pp !== pa) {
      return null;
    }
  }

  return params;
}

function cors(opts) {
  opts = opts || {};
  var origin = opts.origin || "*";
  var methods = opts.methods || ["GET", "POST", "PUT", "DELETE"];
  var headers = opts.allowedHeaders || ["Content-Type"];

  return function(req, res, next) {
    res.set("Access-Control-Allow-Origin", origin);
    res.set("Access-Control-Allow-Methods", methods.join(","));
    res.set("Access-Control-Allow-Headers", headers.join(","));
    if (opts.credentials) {
      res.set("Access-Control-Allow-Credentials", "true");
    }
    if (req.method === "OPTIONS") {
      res.status(204).end();
      return;
    }
    next();
  };
}

function json() {
  return function(req, res, next) { next(); };
}

function Router() {
  return {
    _routes: [],
    get: function(p, h) { this._routes.push({method: "GET", path: p, handler: h}); },
    post: function(p, h) { this._routes.push({method: "POST", path: p, handler: h}); }
  };
}

module.exports = createApp;
module.exports.cors = cors;
module.exports.json = json;
module.exports.Router = Router;
