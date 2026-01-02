// Kiren Express Compatibility Layer
// Drop-in replacement for Express.js

function createApp() {
  var routes = [];
  var middlewares = [];

  var app = {
    // Middleware registration
    use: function(pathOrMiddleware) {
      var handlers = Array.prototype.slice.call(arguments, 1);

      if (typeof pathOrMiddleware === 'function') {
        middlewares.push({ path: null, handler: pathOrMiddleware });
      } else if (typeof pathOrMiddleware === 'string') {
        for (var i = 0; i < handlers.length; i++) {
          var handler = handlers[i];
          if (handler && handler.routes) {
            // It's a router - mount its routes
            for (var j = 0; j < handler.routes.length; j++) {
              var route = handler.routes[j];
              routes.push({
                method: route.method,
                path: pathOrMiddleware + route.path,
                handlers: route.handlers
              });
            }
          } else if (typeof handler === 'function') {
            middlewares.push({ path: pathOrMiddleware, handler: handler });
          }
        }
      }
      return app;
    },

    get: function(path) {
      var handlers = Array.prototype.slice.call(arguments, 1);
      routes.push({ method: 'GET', path: path, handlers: handlers });
      return app;
    },
    post: function(path) {
      var handlers = Array.prototype.slice.call(arguments, 1);
      routes.push({ method: 'POST', path: path, handlers: handlers });
      return app;
    },
    put: function(path) {
      var handlers = Array.prototype.slice.call(arguments, 1);
      routes.push({ method: 'PUT', path: path, handlers: handlers });
      return app;
    },
    delete: function(path) {
      var handlers = Array.prototype.slice.call(arguments, 1);
      routes.push({ method: 'DELETE', path: path, handlers: handlers });
      return app;
    },

    listen: function(port, callback) {
      var server = { port: port };

      function fetchHandler(req) {
        var urlObj = new URL("http://localhost" + req.url);
        var pathname = urlObj.pathname;
        var method = req.method || 'GET';

        // Build request object
        var expressReq = {
          method: method,
          url: req.url,
          path: pathname,
          query: {},
          params: {},
          headers: req.headers || {},
          cookies: {},
          body: null
        };

        expressReq.get = function(name) {
          return expressReq.headers[name.toLowerCase()];
        };

        // Parse query string
        urlObj.searchParams.forEach(function(value, key) {
          expressReq.query[key] = value;
        });

        // Parse cookies
        var cookieHeader = expressReq.headers['cookie'] || '';
        var cookieParts = cookieHeader.split(';');
        for (var i = 0; i < cookieParts.length; i++) {
          var parts = cookieParts[i].trim().split('=');
          if (parts[0]) expressReq.cookies[parts[0]] = parts[1];
        }

        // Parse body
        if (req.body && (method === 'POST' || method === 'PUT')) {
          try {
            expressReq.body = JSON.parse(req.body);
          } catch (e) {
            expressReq.body = req.body;
          }
        }

        // Build response object
        var responseBody = '';
        var responseStatus = 200;
        var responseHeaders = { 'Content-Type': 'application/json' };
        var responseSent = false;

        var expressRes = {
          statusCode: 200,
          status: function(code) {
            responseStatus = code;
            this.statusCode = code;
            return this;
          },
          set: function(name, value) {
            responseHeaders[name] = value;
            return this;
          },
          header: function(name, value) { return this.set(name, value); },
          setHeader: function(name, value) { responseHeaders[name] = value; },
          json: function(data) {
            responseHeaders['Content-Type'] = 'application/json';
            responseBody = JSON.stringify(data);
            responseSent = true;
            return this;
          },
          send: function(data) {
            if (typeof data === 'object') return this.json(data);
            responseBody = String(data);
            responseSent = true;
            return this;
          },
          end: function(data) {
            if (data) responseBody = data;
            responseSent = true;
            return this;
          }
        };

        // Find matching route
        var matchedRoute = null;
        var params = {};

        for (var i = 0; i < routes.length; i++) {
          var route = routes[i];
          if (route.method !== method) continue;

          var match = matchPath(route.path, pathname);
          if (match) {
            matchedRoute = route;
            params = match.params;
            break;
          }
        }

        expressReq.params = params;

        // Run global middlewares synchronously
        for (var i = 0; i < middlewares.length; i++) {
          var mw = middlewares[i];
          if (mw.path === null || pathname.indexOf(mw.path) === 0) {
            var nextCalled = false;
            var nextFn = function() { nextCalled = true; };
            try {
              mw.handler(expressReq, expressRes, nextFn);
              if (responseSent) break;
            } catch (e) {
              console.error("Middleware error:", e);
            }
          }
        }

        // Run route handlers
        if (matchedRoute && !responseSent) {
          for (var i = 0; i < matchedRoute.handlers.length; i++) {
            var handler = matchedRoute.handlers[i];
            var nextCalled = false;
            var nextFn = function() { nextCalled = true; };
            try {
              handler(expressReq, expressRes, nextFn);
              if (responseSent) break;
            } catch (e) {
              console.error("Route error:", e);
              responseStatus = 500;
              responseBody = JSON.stringify({ error: e.message });
              break;
            }
          }
        }

        // 404 if no route matched
        if (!matchedRoute && !responseSent) {
          responseStatus = 404;
          responseBody = JSON.stringify({ error: 'Not Found' });
        }

        return new Response(responseBody, {
          status: responseStatus,
          headers: responseHeaders
        });
      }

      // Start Kiren server
      Kiren.serve({
        port: port,
        fetch: fetchHandler
      });

      if (callback) callback();
      return server;
    }
  };

  return app;
}

// Path matching
function matchPath(pattern, pathname) {
  var patternParts = pattern.split('/').filter(function(p) { return p; });
  var pathParts = pathname.split('/').filter(function(p) { return p; });

  if (patternParts.length !== pathParts.length) return null;

  var params = {};
  for (var i = 0; i < patternParts.length; i++) {
    var pp = patternParts[i];
    var pathPart = pathParts[i];

    if (pp.charAt(0) === ':') {
      params[pp.slice(1)] = pathPart;
    } else if (pp !== pathPart) {
      return null;
    }
  }

  return { params: params };
}

// Router factory
function Router() {
  return {
    routes: [],
    get: function(path) {
      var handlers = Array.prototype.slice.call(arguments, 1);
      this.routes.push({ method: 'GET', path: path, handlers: handlers });
      return this;
    },
    post: function(path) {
      var handlers = Array.prototype.slice.call(arguments, 1);
      this.routes.push({ method: 'POST', path: path, handlers: handlers });
      return this;
    },
    put: function(path) {
      var handlers = Array.prototype.slice.call(arguments, 1);
      this.routes.push({ method: 'PUT', path: path, handlers: handlers });
      return this;
    },
    delete: function(path) {
      var handlers = Array.prototype.slice.call(arguments, 1);
      this.routes.push({ method: 'DELETE', path: path, handlers: handlers });
      return this;
    }
  };
}

// CORS middleware
function cors(options) {
  options = options || {};
  return function(req, res, next) {
    var origin = options.origin || '*';
    var methods = (options.methods || ['GET', 'POST', 'PUT', 'DELETE']).join(', ');
    var headers = (options.allowedHeaders || ['Content-Type', 'Authorization']).join(', ');

    res.set('Access-Control-Allow-Origin', origin);
    res.set('Access-Control-Allow-Methods', methods);
    res.set('Access-Control-Allow-Headers', headers);

    if (options.credentials) {
      res.set('Access-Control-Allow-Credentials', 'true');
    }

    if (req.method === 'OPTIONS') {
      res.status(204).end();
      return;
    }

    next();
  };
}

// Body parser
function json() {
  return function(req, res, next) { next(); };
}

// Cookie parser
function cookieParser() {
  return function(req, res, next) { next(); };
}

// Exports
module.exports = createApp;
module.exports.Router = Router;
module.exports.json = json;
module.exports.cors = cors;
module.exports.cookieParser = cookieParser;
