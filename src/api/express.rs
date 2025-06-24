use anyhow::Result;
use v8;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use serde_json::json;
use regex::Regex;
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;

#[derive(Debug, Clone)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub pattern: String,
    pub callback_code: String,
    pub param_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Middleware {
    pub path_pattern: String,
    pub callback_code: String,
    pub is_error_handler: bool, // true if middleware has 4 parameters (err, req, res, next)
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct ResponseContext {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Default for ResponseContext {
    fn default() -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: String::new(),
        }
    }
}

pub static EXPRESS_ROUTES: Lazy<Arc<Mutex<Vec<Route>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));
pub static EXPRESS_MIDDLEWARE: Lazy<Arc<Mutex<Vec<Middleware>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

pub fn setup_express(_scope: &mut v8::HandleScope, _context: v8::Local<v8::Context>) -> Result<()> {
    // Express is now handled through the require() system in npm_simple.rs
    // This function remains for any future direct express setup needs
    Ok(())
}

fn create_express_app(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create app object
    let app_obj = v8::Object::new(scope);

    // Add HTTP methods
    let get_key = v8::String::new(scope, "get").unwrap();
    let get_template = v8::FunctionTemplate::new(scope, app_get);
    let get_function = get_template.get_function(scope).unwrap();
    app_obj.set(scope, get_key.into(), get_function.into());

    let post_key = v8::String::new(scope, "post").unwrap();
    let post_template = v8::FunctionTemplate::new(scope, app_post);
    let post_function = post_template.get_function(scope).unwrap();
    app_obj.set(scope, post_key.into(), post_function.into());

    let put_key = v8::String::new(scope, "put").unwrap();
    let put_template = v8::FunctionTemplate::new(scope, app_put);
    let put_function = put_template.get_function(scope).unwrap();
    app_obj.set(scope, put_key.into(), put_function.into());

    let delete_key = v8::String::new(scope, "delete").unwrap();
    let delete_template = v8::FunctionTemplate::new(scope, app_delete);
    let delete_function = delete_template.get_function(scope).unwrap();
    app_obj.set(scope, delete_key.into(), delete_function.into());

    // Add use method for middleware
    let use_key = v8::String::new(scope, "use").unwrap();
    let use_template = v8::FunctionTemplate::new(scope, app_use);
    let use_function = use_template.get_function(scope).unwrap();
    app_obj.set(scope, use_key.into(), use_function.into());

    // Add listen method
    let listen_key = v8::String::new(scope, "listen").unwrap();
    let listen_template = v8::FunctionTemplate::new(scope, app_listen);
    let listen_function = listen_template.get_function(scope).unwrap();
    app_obj.set(scope, listen_key.into(), listen_function.into());

    // Add Router function
    let router_key = v8::String::new(scope, "Router").unwrap();
    let router_template = v8::FunctionTemplate::new(scope, create_router);
    let router_function = router_template.get_function(scope).unwrap();
    app_obj.set(scope, router_key.into(), router_function.into());

    // Add static function
    let static_key = v8::String::new(scope, "static").unwrap();
    let static_template = v8::FunctionTemplate::new(scope, express_static);
    let static_function = static_template.get_function(scope).unwrap();
    app_obj.set(scope, static_key.into(), static_function.into());

    rv.set(app_obj.into());
}

fn create_router(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create router object (same structure as app for now)
    let router_obj = v8::Object::new(scope);

    // Add HTTP methods
    let get_key = v8::String::new(scope, "get").unwrap();
    let get_template = v8::FunctionTemplate::new(scope, router_get);
    let get_function = get_template.get_function(scope).unwrap();
    router_obj.set(scope, get_key.into(), get_function.into());

    let post_key = v8::String::new(scope, "post").unwrap();
    let post_template = v8::FunctionTemplate::new(scope, router_post);
    let post_function = post_template.get_function(scope).unwrap();
    router_obj.set(scope, post_key.into(), post_function.into());

    let put_key = v8::String::new(scope, "put").unwrap();
    let put_template = v8::FunctionTemplate::new(scope, router_put);
    let put_function = put_template.get_function(scope).unwrap();
    router_obj.set(scope, put_key.into(), put_function.into());

    let delete_key = v8::String::new(scope, "delete").unwrap();
    let delete_template = v8::FunctionTemplate::new(scope, router_delete);
    let delete_function = delete_template.get_function(scope).unwrap();
    router_obj.set(scope, delete_key.into(), delete_function.into());

    rv.set(router_obj.into());
}

fn express_static(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "express.static() requires a root directory").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let root_arg = args.get(0);
    let root_str = root_arg.to_string(scope).unwrap();
    let root_path = root_str.to_rust_string_lossy(scope);

    // Parse options if provided
    let _options = if args.length() > 1 {
        let options_arg = args.get(1);
        if options_arg.is_object() {
            // TODO: Parse static file options from V8 object
            Some(crate::api::static_files::StaticOptions::default())
        } else {
            None
        }
    } else {
        None
    };

    // Create a middleware function that serves static files
    let middleware_fn = v8::FunctionTemplate::new(scope, static_middleware_handler);
    let middleware = middleware_fn.get_function(scope).unwrap();

    // Store root path in middleware function for later use
    let root_key = v8::String::new(scope, "_staticRoot").unwrap();
    let root_value = v8::String::new(scope, &root_path).unwrap();
    middleware.set(scope, root_key.into(), root_value.into());

    rv.set(middleware.into());
}

fn static_middleware_handler(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    // This is called when the static middleware is executed
    // args should be (req, res, next)
    if args.length() < 3 {
        return;
    }

    let req = args.get(0);
    let _res = args.get(1);
    let next = args.get(2);

    // Extract URL from request
    if let Ok(req_obj) = v8::Local::<v8::Object>::try_from(req) {
        let url_key = v8::String::new(scope, "url").unwrap();
        if let Some(url_val) = req_obj.get(scope, url_key.into()) {
            let url_str = url_val.to_string(scope).unwrap();
            let url = url_str.to_rust_string_lossy(scope);
            
            println!("Static middleware handling: {}", url);
            
            // For now, just call next() to continue to next middleware
            if let Ok(next_fn) = v8::Local::<v8::Function>::try_from(next) {
                let undefined = v8::undefined(scope);
                next_fn.call(scope, undefined.into(), &[]);
            }
        }
    }
}

fn app_get(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "GET");
}

fn app_post(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "POST");
}

fn app_put(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "PUT");
}

fn app_delete(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "DELETE");
}

fn router_get(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "GET");
}

fn router_post(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "POST");
}

fn router_put(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "PUT");
}

fn router_delete(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "DELETE");
}

fn app_use(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        return;
    }
    
    let (path_pattern, middleware_index) = if args.length() == 1 {
        // app.use(middleware) - global middleware
        ("*".to_string(), 0)
    } else {
        // app.use(path, middleware) - path-specific middleware
        let path_arg = args.get(0);
        let path_str = path_arg.to_string(scope).unwrap();
        (path_str.to_rust_string_lossy(scope), 1)
    };
    
    // Handle multiple middleware functions
    let mut start_index = middleware_index;
    while start_index < args.length() {
        let middleware_arg = args.get(start_index);
        
        if middleware_arg.is_function() {
            // Convert function to string representation
            let callback_code = format!(
                "function middleware_{}(req, res, next) {{ \
                    // Middleware logic placeholder \
                    console.log('Middleware executed for: ' + req.url); \
                    next(); \
                }}",
                start_index
            );
            
            // Check if this is an error handler (4 parameters)
            let is_error_handler = false; // TODO: detect function arity
            
            let middleware = Middleware {
                path_pattern: path_pattern.clone(),
                callback_code,
                is_error_handler,
            };
            
            EXPRESS_MIDDLEWARE.lock().unwrap().push(middleware);
            println!("Middleware registered for path: {}", path_pattern);
        }
        
        start_index += 1;
    }
}

fn app_listen(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let port = if args.length() > 0 {
        args.get(0).uint32_value(scope).unwrap_or(3000)
    } else {
        3000
    };

    println!("Express app listening on port {}", port);
    
    // Start HTTP server in background
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = start_express_server(port as u16).await {
                eprintln!("Express server error: {}", e);
            }
        });
    });
}

fn register_route(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    method: &str,
) {
    if args.length() < 2 {
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    let callback_arg = args.get(1);
    
    // Convert function to string for later execution
    let callback_code = if callback_arg.is_function() {
        // For now, we'll create a simple wrapper
        format!("function(req, res) {{ res.json({{ message: 'Route {} {}' }}); }}", method, path)
    } else {
        // String response
        let callback_str = callback_arg.to_string(scope).unwrap();
        let response = callback_str.to_rust_string_lossy(scope);
        format!("function(req, res) {{ res.send('{}'); }}", response)
    };

    // Parse path parameters
    let (pattern, param_names) = parse_route_pattern(&path);

    let route = Route {
        method: method.to_string(),
        path: path.clone(),
        pattern,
        callback_code,
        param_names,
    };

    EXPRESS_ROUTES.lock().unwrap().push(route);
    println!("Route registered: {} {}", method, path);
}

fn parse_route_pattern(path: &str) -> (String, Vec<String>) {
    let mut pattern = path.to_string();
    let mut param_names = Vec::new();

    // Find URL parameters like :id, :userId etc.
    let param_regex = Regex::new(r":([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    
    for cap in param_regex.captures_iter(path) {
        let param_name = cap[1].to_string();
        param_names.push(param_name);
        
        // Replace :param with regex pattern
        pattern = pattern.replace(&cap[0], "([^/]+)");
    }

    // Escape other regex characters
    pattern = pattern.replace(".", "\\.");
    pattern = format!("^{}$", pattern);

    (pattern, param_names)
}

pub fn match_route(method: &str, path: &str) -> Option<(Route, HashMap<String, String>)> {
    let routes = EXPRESS_ROUTES.lock().unwrap();
    
    for route in routes.iter() {
        if route.method != method {
            continue;
        }

        let regex = match Regex::new(&route.pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if let Some(captures) = regex.captures(path) {
            let mut params = HashMap::new();
            
            // Extract URL parameters
            for (i, param_name) in route.param_names.iter().enumerate() {
                if let Some(capture) = captures.get(i + 1) {
                    params.insert(param_name.clone(), capture.as_str().to_string());
                }
            }

            return Some((route.clone(), params));
        }
    }

    None
}

pub async fn start_express_server(port: u16) -> Result<()> {
    use hyper::service::{make_service_fn, service_fn};
    use std::net::SocketAddr;

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_express_request))
    });

    let server = Server::bind(&addr).serve(make_svc);
    println!("Express server running on http://127.0.0.1:{}", port);
    
    if let Err(e) = server.await {
        eprintln!("Express server error: {}", e);
        return Err(e.into());
    }
    
    Ok(())
}

async fn handle_express_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let query = parse_query_string(req.uri().query().unwrap_or(""));
    let headers = extract_headers(&req);
    
    // Get body (simplified for now)
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
    
    // Try to match route
    if let Some((route, params)) = match_route(&method, &path) {
        // Execute route with proper req/res objects
        let response_data = execute_route_callback(&route, &method, &path, query, params, headers, &body).await;
        
        return Ok(Response::builder()
            .status(response_data.0)
            .header("content-type", response_data.1.get("content-type").unwrap_or(&"application/json".to_string()))
            .header("access-control-allow-origin", "*")
            .body(Body::from(response_data.2))
            .unwrap());
    }

    // Default 404 response
    let not_found = json!({
        "error": "Not Found",
        "message": format!("Cannot {} {}", method, path)
    });

    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "application/json")
        .body(Body::from(not_found.to_string()))
        .unwrap())
}

fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if query.is_empty() {
        return map;
    }
    
    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            map.insert(
                urlencoding::decode(key).unwrap_or_default().to_string(),
                urlencoding::decode(value).unwrap_or_default().to_string(),
            );
        }
    }
    map
}

fn extract_headers(req: &Request<Body>) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for (name, value) in req.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }
    headers
}

async fn execute_route_callback(
    route: &Route,
    method: &str,
    path: &str,
    query: HashMap<String, String>,
    params: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: &str,
) -> (u16, HashMap<String, String>, String) {
    // Create a simple V8 context for executing the callback
    use crate::runtime::engine::Engine;
    
    let mut engine = Engine::new().unwrap();
    
    // Create callback execution code
    let callback_code = format!(
        r#"
        (function() {{
            // Create req and res objects
            const req = {{
                method: '{}',
                path: '{}',
                url: '{}',
                params: {},
                query: {},
                headers: {},
                body: {},
                get: function(name) {{ return this.headers[name.toLowerCase()]; }}
            }};
            
            const res = {{
                _statusCode: 200,
                _headers: {{}},
                _body: '',
                _finished: false,
                status: function(code) {{ this._statusCode = code; return this; }},
                json: function(data) {{ 
                    this._headers['content-type'] = 'application/json';
                    this._body = JSON.stringify(data);
                    this._finished = true;
                    return this;
                }},
                send: function(data) {{ 
                    if (!this._headers['content-type']) {{
                        this._headers['content-type'] = 'text/html';
                    }}
                    this._body = String(data);
                    this._finished = true;
                    return this;
                }},
                set: function(name, value) {{ 
                    this._headers[name.toLowerCase()] = value; 
                    return this; 
                }},
                header: function(name, value) {{ return this.set(name, value); }},
                end: function(data) {{ 
                    if (data !== undefined) this._body = String(data);
                    this._finished = true;
                }}
            }};
            
            // Execute the route callback
            const callback = {};
            if (typeof callback === 'function') {{
                callback(req, res);
            }} else {{
                res.send(callback);
            }}
            
            // Return response data
            return {{
                statusCode: res._statusCode,
                headers: res._headers,
                body: res._body
            }};
        }})()
        "#,
        method,
        path,
        path,
        serde_json::to_string(&params).unwrap_or_else(|_| "{{}}".to_string()),
        serde_json::to_string(&query).unwrap_or_else(|_| "{{}}".to_string()),
        serde_json::to_string(&headers).unwrap_or_else(|_| "{{}}".to_string()),
        if body.is_empty() { "{{}}".to_string() } else { 
            serde_json::to_string(body).unwrap_or_else(|_| "{{}}".to_string())
        },
        route.callback_code
    );
    
    // Execute and parse result
    match engine.execute(&callback_code) {
        Ok(result) => {
            if let Ok(response_data) = serde_json::from_str::<serde_json::Value>(&result) {
                let status = response_data["statusCode"].as_u64().unwrap_or(200) as u16;
                let mut headers = HashMap::new();
                
                if let Some(header_obj) = response_data["headers"].as_object() {
                    for (key, value) in header_obj {
                        if let Some(value_str) = value.as_str() {
                            headers.insert(key.clone(), value_str.to_string());
                        }
                    }
                }
                
                let body = response_data["body"].as_str().unwrap_or("").to_string();
                return (status, headers, body);
            }
        }
        Err(_) => {}
    }
    
    // Fallback response
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    let fallback_body = json!({
        "message": format!("Route {} {} executed", method, path),
        "params": params
    }).to_string();
    
    (200, headers, fallback_body)
}

pub fn clear_express_routes() {
    EXPRESS_ROUTES.lock().unwrap().clear();
}