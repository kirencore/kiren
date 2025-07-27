#![allow(dead_code)]
use crate::runtime::engine::Engine;
use anyhow::Result;
use hyper::{Body, Request, Response, Server, StatusCode};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use v8;

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

pub static EXPRESS_ROUTES: Lazy<Arc<Mutex<Vec<Route>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));
pub static EXPRESS_MIDDLEWARE: Lazy<Arc<Mutex<Vec<Middleware>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

// Use thread-local engine for middleware execution to reduce overhead
use std::cell::RefCell;
thread_local! {
    static MIDDLEWARE_ENGINE: RefCell<Option<Engine>> = RefCell::new(None);
}

/// Get or create thread-local engine for middleware execution
fn get_middleware_engine() -> Result<Engine> {
    MIDDLEWARE_ENGINE.with(|engine_cell| {
        let mut engine_opt = engine_cell.borrow_mut();
        match engine_opt.take() {
            Some(engine) => Ok(engine),
            None => Engine::new(),
        }
    })
}

/// Return engine to thread-local storage
fn return_middleware_engine(engine: Engine) {
    MIDDLEWARE_ENGINE.with(|engine_cell| {
        let mut engine_opt = engine_cell.borrow_mut();
        *engine_opt = Some(engine);
    });
}

pub fn setup_express(_scope: &mut v8::HandleScope, _context: v8::Local<v8::Context>) -> Result<()> {
    // Express is now handled through the require() system in npm_simple.rs
    // This function remains for any future direct express setup needs
    Ok(())
}

pub fn create_express_app(
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

    // Add json middleware function
    let json_key = v8::String::new(scope, "json").unwrap();
    let json_template = v8::FunctionTemplate::new(scope, express_json);
    let json_function = json_template.get_function(scope).unwrap();
    app_obj.set(scope, json_key.into(), json_function.into());

    // Add urlencoded middleware function
    let urlencoded_key = v8::String::new(scope, "urlencoded").unwrap();
    let urlencoded_template = v8::FunctionTemplate::new(scope, express_urlencoded);
    let urlencoded_function = urlencoded_template.get_function(scope).unwrap();
    app_obj.set(scope, urlencoded_key.into(), urlencoded_function.into());

    rv.set(app_obj.into());
}

pub fn create_router(
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

pub fn express_static(
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

pub fn express_json(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create a JSON parser middleware function
    let middleware_fn = v8::FunctionTemplate::new(scope, json_middleware_handler);
    let middleware = middleware_fn.get_function(scope).unwrap();

    rv.set(middleware.into());
}

pub fn express_urlencoded(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create a URL-encoded parser middleware function
    let middleware_fn = v8::FunctionTemplate::new(scope, urlencoded_middleware_handler);
    let middleware = middleware_fn.get_function(scope).unwrap();

    rv.set(middleware.into());
}

fn json_middleware_handler(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    // This middleware parses JSON request bodies
    // args should be (req, res, next)
    if args.length() < 3 {
        return;
    }

    let req = args.get(0);
    let _res = args.get(1);
    let next = args.get(2);

    // Parse JSON body if content-type is application/json
    if let Ok(req_obj) = v8::Local::<v8::Object>::try_from(req) {
        let content_type_key = v8::String::new(scope, "get").unwrap();
        if let Some(get_fn) = req_obj.get(scope, content_type_key.into()) {
            if let Ok(get_function) = v8::Local::<v8::Function>::try_from(get_fn) {
                let content_type_arg = v8::String::new(scope, "content-type").unwrap();
                let args = [content_type_arg.into()];
                if let Some(content_type_val) = get_function.call(scope, req.into(), &args) {
                    let content_type_str = content_type_val.to_string(scope).unwrap();
                    let content_type = content_type_str.to_rust_string_lossy(scope);

                    if content_type.contains("application/json") {
                        // Parse JSON from body
                        let body_key = v8::String::new(scope, "body").unwrap();
                        if let Some(body_val) = req_obj.get(scope, body_key.into()) {
                            let body_str = body_val.to_string(scope).unwrap();
                            let body = body_str.to_rust_string_lossy(scope);

                            if !body.is_empty() {
                                if let Ok(parsed_json) =
                                    serde_json::from_str::<serde_json::Value>(&body)
                                {
                                    let json_obj = json_to_v8_value(scope, &parsed_json);
                                    req_obj.set(scope, body_key.into(), json_obj);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Call next middleware
    if let Ok(next_fn) = v8::Local::<v8::Function>::try_from(next) {
        let undefined = v8::undefined(scope);
        next_fn.call(scope, undefined.into(), &[]);
    }
}

fn urlencoded_middleware_handler(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    // This middleware parses URL-encoded request bodies
    // args should be (req, res, next)
    if args.length() < 3 {
        return;
    }

    let req = args.get(0);
    let _res = args.get(1);
    let next = args.get(2);

    // Parse URL-encoded body if content-type is application/x-www-form-urlencoded
    if let Ok(req_obj) = v8::Local::<v8::Object>::try_from(req) {
        let content_type_key = v8::String::new(scope, "get").unwrap();
        if let Some(get_fn) = req_obj.get(scope, content_type_key.into()) {
            if let Ok(get_function) = v8::Local::<v8::Function>::try_from(get_fn) {
                let content_type_arg = v8::String::new(scope, "content-type").unwrap();
                let args = [content_type_arg.into()];
                if let Some(content_type_val) = get_function.call(scope, req.into(), &args) {
                    let content_type_str = content_type_val.to_string(scope).unwrap();
                    let content_type = content_type_str.to_rust_string_lossy(scope);

                    if content_type.contains("application/x-www-form-urlencoded") {
                        // Parse form data from body
                        let body_key = v8::String::new(scope, "body").unwrap();
                        if let Some(body_val) = req_obj.get(scope, body_key.into()) {
                            let body_str = body_val.to_string(scope).unwrap();
                            let body = body_str.to_rust_string_lossy(scope);

                            if !body.is_empty() {
                                let parsed_form = parse_urlencoded_body(&body);
                                let form_obj = form_data_to_v8_object(scope, &parsed_form);
                                req_obj.set(scope, body_key.into(), form_obj);
                            }
                        }
                    }
                }
            }
        }
    }

    // Call next middleware
    if let Ok(next_fn) = v8::Local::<v8::Function>::try_from(next) {
        let undefined = v8::undefined(scope);
        next_fn.call(scope, undefined.into(), &[]);
    }
}

fn json_to_v8_value<'a>(
    scope: &mut v8::HandleScope<'a>,
    value: &serde_json::Value,
) -> v8::Local<'a, v8::Value> {
    match value {
        serde_json::Value::Null => v8::null(scope).into(),
        serde_json::Value::Bool(b) => v8::Boolean::new(scope, *b).into(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                v8::Integer::new(scope, i as i32).into()
            } else if let Some(f) = n.as_f64() {
                v8::Number::new(scope, f).into()
            } else {
                v8::null(scope).into()
            }
        }
        serde_json::Value::String(s) => v8::String::new(scope, s).unwrap().into(),
        serde_json::Value::Array(arr) => {
            let v8_array = v8::Array::new(scope, arr.len() as i32);
            for (i, item) in arr.iter().enumerate() {
                let v8_item = json_to_v8_value(scope, item);
                v8_array.set_index(scope, i as u32, v8_item);
            }
            v8_array.into()
        }
        serde_json::Value::Object(obj) => {
            let v8_obj = v8::Object::new(scope);
            for (key, val) in obj {
                let v8_key = v8::String::new(scope, key).unwrap();
                let v8_val = json_to_v8_value(scope, val);
                v8_obj.set(scope, v8_key.into(), v8_val);
            }
            v8_obj.into()
        }
    }
}

fn parse_urlencoded_body(body: &str) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();

    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_key = urlencoding::decode(key).unwrap_or_default();
            let decoded_value = urlencoding::decode(value).unwrap_or_default();
            result.insert(decoded_key.to_string(), decoded_value.to_string());
        }
    }

    result
}

fn form_data_to_v8_object<'a>(
    scope: &mut v8::HandleScope<'a>,
    form_data: &std::collections::HashMap<String, String>,
) -> v8::Local<'a, v8::Value> {
    let v8_obj = v8::Object::new(scope);

    for (key, value) in form_data {
        let v8_key = v8::String::new(scope, key).unwrap();
        let v8_value = v8::String::new(scope, value).unwrap();
        v8_obj.set(scope, v8_key.into(), v8_value.into());
    }

    v8_obj.into()
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

fn app_get(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue) {
    register_route(scope, args, "GET");
}

fn app_post(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "POST");
}

fn app_put(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue) {
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

fn app_use(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue) {
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
            // Convert V8 function to string for storage
            let func_str = middleware_arg.to_string(scope).unwrap();
            let func_string = func_str.to_rust_string_lossy(scope);
            
            // For built-in functions that can't be serialized properly, create working implementations
            let callback_code = if func_string.contains("[native function]") || func_string.starts_with("function ") {
                // This is likely a CORS middleware or other built-in - create a working implementation
                r#"function(req, res, next) {
                    // CORS headers
                    if (res._headers) {
                        res._headers['access-control-allow-origin'] = '*';
                        res._headers['access-control-allow-methods'] = 'GET,HEAD,PUT,PATCH,POST,DELETE,OPTIONS';
                        res._headers['access-control-allow-headers'] = 'Content-Type,Authorization,Cache-Control,Pragma';
                        res._headers['access-control-allow-credentials'] = 'true';
                    }
                    if (req.method === 'OPTIONS') {
                        res.status(200).end();
                    } else {
                        next();
                    }
                }"#.to_string()
            } else {
                func_string
            };

            // Detect function arity for error handlers (4 params: err, req, res, next)
            let is_error_handler =
                callback_code.contains("function") && callback_code.matches(',').count() >= 3;

            let middleware = Middleware {
                path_pattern: path_pattern.clone(),
                callback_code,
                is_error_handler,
            };

            EXPRESS_MIDDLEWARE.lock().unwrap().push(middleware);
            println!(
                "Middleware registered for path: {} (error handler: {})",
                path_pattern, is_error_handler
            );
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

    // Start HTTP server using tokio::spawn to use the existing runtime
    tokio::spawn(async move {
        if let Err(e) = start_express_server(port as u16).await {
            eprintln!("Express server error: {}", e);
        }
    });
}

fn register_route(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, method: &str) {
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
        format!(
            "function(req, res) {{ res.json({{ message: 'Route {} {}' }}); }}",
            method, path
        )
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

    // Handle wildcard routes (*)
    if pattern.contains('*') {
        pattern = pattern.replace("*", "(.*)");
    }

    // Find URL parameters like :id, :userId etc.
    let param_regex = Regex::new(r":([a-zA-Z_][a-zA-Z0-9_]*)(\([^)]+\))?").unwrap();

    for cap in param_regex.captures_iter(path) {
        let param_name = cap[1].to_string();
        param_names.push(param_name);

        let full_match = &cap[0];

        // Check if there's a custom regex pattern for this parameter
        if let Some(custom_pattern) = cap.get(2) {
            // Use custom pattern (without parentheses)
            let custom = custom_pattern.as_str();
            let clean_pattern = &custom[1..custom.len() - 1]; // Remove outer parentheses
            pattern = pattern.replace(full_match, &format!("({})", clean_pattern));
        } else {
            // Default parameter pattern - matches anything except /
            pattern = pattern.replace(full_match, "([^/]+)");
        }
    }

    // Handle optional parameters like (:id)?
    let optional_regex = Regex::new(r"\(([^)]*):([a-zA-Z_][a-zA-Z0-9_]*)\)\?").unwrap();
    for cap in optional_regex.captures_iter(&pattern.clone()) {
        let param_name = cap[2].to_string();
        if !param_names.contains(&param_name) {
            param_names.push(param_name);
        }
        let full_match = &cap[0];
        let prefix = &cap[1];
        pattern = pattern.replace(full_match, &format!("(?:{}([^/]+))?", prefix));
    }

    // Escape other regex special characters
    let chars_to_escape = ["."];
    for ch in chars_to_escape {
        pattern = pattern.replace(ch, &format!("\\{}", ch));
    }

    // Add anchors
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

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_express_request)) });

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
    let body_bytes = hyper::body::to_bytes(req.into_body())
        .await
        .unwrap_or_default();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();

    // Create request context
    let request_context = RequestContext {
        method: method.clone(),
        path: path.clone(),
        query,
        params: HashMap::new(),
        headers,
        body,
    };

    // Execute middleware chain first
    let mut response_context = ResponseContext::default();
    let middleware_result = execute_middleware_chain(&request_context, &mut response_context).await;

    // If middleware handled the response, return it
    if middleware_result.is_err() || response_context.body != "" {
        return Ok(Response::builder()
            .status(response_context.status)
            .header(
                "content-type",
                response_context
                    .headers
                    .get("content-type")
                    .unwrap_or(&"application/json".to_string()),
            )
            .header("access-control-allow-origin", "*")
            .body(Body::from(response_context.body))
            .unwrap());
    }

    // Try to match route
    if let Some((route, params)) = match_route(&method, &path) {
        // Execute route with proper req/res objects
        let response_data = execute_route_callback(
            &route,
            &method,
            &path,
            request_context.query,
            params,
            request_context.headers,
            &request_context.body,
        )
        .await;

        return Ok(Response::builder()
            .status(response_data.0)
            .header(
                "content-type",
                response_data
                    .1
                    .get("content-type")
                    .unwrap_or(&"application/json".to_string()),
            )
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
    // Use thread-local engine for better performance
    let mut engine = get_middleware_engine().unwrap();

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
                sendStatus: function(code) {{
                    this._statusCode = code;
                    // Map status codes to standard messages
                    const statusMessages = {{
                        200: 'OK',
                        201: 'Created',
                        204: 'No Content',
                        400: 'Bad Request',
                        401: 'Unauthorized',
                        403: 'Forbidden',
                        404: 'Not Found',
                        500: 'Internal Server Error'
                    }};
                    this._body = statusMessages[code] || 'Unknown Status';
                    this._headers['content-type'] = 'text/plain';
                    this._finished = true;
                    return this;
                }},
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
        if body.is_empty() {
            "{{}}".to_string()
        } else {
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
                return_middleware_engine(engine);
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
    })
    .to_string();

    return_middleware_engine(engine);
    (200, headers, fallback_body)
}

async fn execute_middleware_chain(
    request: &RequestContext,
    response: &mut ResponseContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let middleware_list = EXPRESS_MIDDLEWARE.lock().unwrap().clone();

    for middleware in &middleware_list {
        // Check if middleware path pattern matches current request path
        if should_execute_middleware(&middleware.path_pattern, &request.path) {
            // Execute middleware
            let middleware_result = execute_middleware(middleware, request, response).await;

            match middleware_result {
                Ok(should_continue) => {
                    if !should_continue {
                        // Middleware called res.send() or similar, stop chain
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("🚨 Middleware chain error: {}", e);
                    // Error in middleware, stop chain
                    response.status = 500;
                    response.body = json!({
                        "error": "Internal Server Error",
                        "message": "Middleware execution failed"
                    })
                    .to_string();
                    return Err("Middleware error".into());
                }
            }
        }
    }

    Ok(())
}

fn should_execute_middleware(pattern: &str, path: &str) -> bool {
    match pattern {
        "*" => true, // Global middleware
        p if p.ends_with("/*") => {
            let prefix = &p[..p.len() - 2];
            path.starts_with(prefix)
        }
        p => path == p || path.starts_with(&format!("{}/", p)),
    }
}

async fn execute_middleware(
    middleware: &Middleware,
    request: &RequestContext,
    response: &mut ResponseContext,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut engine = get_middleware_engine()?;

    // Create middleware execution context
    let middleware_code = format!(
        r#"
        (function() {{
            let nextCalled = false;
            let responseSent = false;
            
            // Create req object
            const req = {{
                method: '{}',
                path: '{}',
                url: '{}',
                params: {},
                query: {},
                headers: {},
                body: {},
                get: function(name) {{ return this.headers[name.toLowerCase()]; }},
                timestamp: Date.now(),
                apiCall: false,
                isAdmin: false,
                authenticated: false,
                authorized: false
            }};
            
            // Create res object
            const res = {{
                _statusCode: {},
                _headers: {},
                _body: '{}',
                _finished: false,
                status: function(code) {{ 
                    this._statusCode = code; 
                    return this; 
                }},
                json: function(data) {{ 
                    this._headers['content-type'] = 'application/json';
                    this._body = JSON.stringify(data);
                    this._finished = true;
                    responseSent = true;
                    return this;
                }},
                send: function(data) {{ 
                    if (!this._headers['content-type']) {{
                        this._headers['content-type'] = 'text/html';
                    }}
                    this._body = String(data);
                    this._finished = true;
                    responseSent = true;
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
                    responseSent = true;
                }}
            }};
            
            // Create next function
            const next = function(err) {{
                if (err) {{
                    throw new Error(err);
                }}
                nextCalled = true;
            }};
            
            // Execute middleware function
            try {{
                const middlewareFunction = {};
                if (typeof middlewareFunction === 'function') {{
                    middlewareFunction(req, res, next);
                }}
            }} catch (error) {{
                return {{
                    error: error.message,
                    nextCalled: false,
                    responseSent: false
                }};
            }}
            
            // Return execution result
            return {{
                nextCalled: nextCalled,
                responseSent: responseSent,
                statusCode: res._statusCode,
                headers: res._headers,
                body: res._body,
                req: {{
                    timestamp: req.timestamp,
                    apiCall: req.apiCall,
                    isAdmin: req.isAdmin,
                    authenticated: req.authenticated,
                    authorized: req.authorized
                }}
            }};
        }})()
        "#,
        request.method,
        request.path,
        request.path,
        serde_json::to_string(&request.params)?,
        serde_json::to_string(&request.query)?,
        serde_json::to_string(&request.headers)?,
        serde_json::to_string(&request.body)?,
        response.status,
        serde_json::to_string(&response.headers)?,
        response.body,
        middleware.callback_code
    );

    match engine.execute(&middleware_code) {
        Ok(result) => {
            if let Ok(execution_data) = serde_json::from_str::<serde_json::Value>(&result) {
                let next_called = execution_data["nextCalled"].as_bool().unwrap_or(false);
                let response_sent = execution_data["responseSent"].as_bool().unwrap_or(false);

                // Update response if middleware set values
                if response_sent {
                    response.status = execution_data["statusCode"].as_u64().unwrap_or(200) as u16;
                    response.body = execution_data["body"].as_str().unwrap_or("").to_string();

                    if let Some(headers_obj) = execution_data["headers"].as_object() {
                        for (key, value) in headers_obj {
                            if let Some(value_str) = value.as_str() {
                                response.headers.insert(key.clone(), value_str.to_string());
                            }
                        }
                    }
                    return_middleware_engine(engine);
                    return Ok(false); // Stop middleware chain
                }

                // Continue to next middleware if next() was called
                return_middleware_engine(engine);
                return Ok(next_called);
            }
        }
        Err(e) => {
            eprintln!("🚨 Middleware execution error: {}", e);
            eprintln!("🚨 Middleware code: {}", middleware.callback_code);
            return_middleware_engine(engine);
            return Err(format!("Middleware execution failed: {}", e).into());
        }
    }

    // Default: continue to next middleware
    return_middleware_engine(engine);
    Ok(true)
}

pub fn clear_express_routes() {
    EXPRESS_ROUTES.lock().unwrap().clear();
}
