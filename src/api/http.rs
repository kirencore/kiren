use anyhow::Result;
use crossbeam_channel::{bounded, Receiver, Sender};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use once_cell::sync::Lazy;
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;
use v8;
use tokio::sync::broadcast;

// HTTP callback execution system
#[derive(Debug, Clone)]
pub struct HttpCallbackRequest {
    pub id: String,
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct HttpCallbackResponse {
    pub id: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

// Route storage with callback info
#[derive(Clone)]
struct RouteInfo {
    method: String,
    path: String,
    callback_id: Option<String>, // V8 callback function ID
    static_response: Option<String>, // For string responses
}

static ROUTES: Lazy<Arc<Mutex<Vec<RouteInfo>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

// Server shutdown management
static SHUTDOWN_SENDER: Lazy<Arc<Mutex<Option<broadcast::Sender<()>>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

// Simplified approach - removed complex callback system

pub fn shutdown_http_servers() {
    if let Some(sender) = SHUTDOWN_SENDER.lock().unwrap().as_ref() {
        let _ = sender.send(()); // Ignore error if no receivers
    }
    
    // Clear routes for fresh start
    ROUTES.lock().unwrap().clear();
}

pub fn setup_http(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
    let global = context.global(scope);

    // Create http object
    let http_key = v8::String::new(scope, "http").unwrap();
    let http_obj = v8::Object::new(scope);

    // createServer function
    let create_server_key = v8::String::new(scope, "createServer").unwrap();
    let create_server_tmpl = v8::FunctionTemplate::new(scope, create_server);
    let create_server_fn = create_server_tmpl.get_function(scope).unwrap();
    http_obj.set(scope, create_server_key.into(), create_server_fn.into());

    global.set(scope, http_key.into(), http_obj.into());

    Ok(())
}

pub fn create_server(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    
    // Create server object
    let server_obj = v8::Object::new(scope);

    // listen method
    let listen_key = v8::String::new(scope, "listen").unwrap();
    let listen_tmpl = v8::FunctionTemplate::new(scope, server_listen);
    let listen_fn = listen_tmpl.get_function(scope).unwrap();
    server_obj.set(scope, listen_key.into(), listen_fn.into());

    // get method
    let get_key = v8::String::new(scope, "get").unwrap();
    let get_tmpl = v8::FunctionTemplate::new(scope, server_get);
    let get_fn = get_tmpl.get_function(scope).unwrap();
    server_obj.set(scope, get_key.into(), get_fn.into());

    // post method
    let post_key = v8::String::new(scope, "post").unwrap();
    let post_tmpl = v8::FunctionTemplate::new(scope, server_post);
    let post_fn = post_tmpl.get_function(scope).unwrap();
    server_obj.set(scope, post_key.into(), post_fn.into());

    // put method
    let put_key = v8::String::new(scope, "put").unwrap();
    let put_tmpl = v8::FunctionTemplate::new(scope, server_put);
    let put_fn = put_tmpl.get_function(scope).unwrap();
    server_obj.set(scope, put_key.into(), put_fn.into());

    // delete method
    let delete_key = v8::String::new(scope, "delete").unwrap();
    let delete_tmpl = v8::FunctionTemplate::new(scope, server_delete);
    let delete_fn = delete_tmpl.get_function(scope).unwrap();
    server_obj.set(scope, delete_key.into(), delete_fn.into());

    rv.set(server_obj.into());
}

fn server_listen(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let port = if args.length() > 0 {
        args.get(0).number_value(scope).unwrap_or(3000.0) as u16
    } else {
        3000
    };

    // Create new shutdown channel for this server
    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);
    *SHUTDOWN_SENDER.lock().unwrap() = Some(shutdown_tx);
    
    // Start server in background thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = start_http_server(port, shutdown_rx).await {
                eprintln!("Server error: {}", e);
            }
        });
    });
}

fn server_get(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "GET");
}

fn server_post(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "POST");
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
    
    let route = if callback_arg.is_string() {
        // Static string response
        let response_str = callback_arg.to_string(scope).unwrap();
        let response = response_str.to_rust_string_lossy(scope);
        
        RouteInfo {
            method: method.to_string(),
            path: path.clone(),
            callback_id: None,
            static_response: Some(response),
        }
    } else if callback_arg.is_function() {
        // Function callback - execute immediately to get return value
        let callback_fn: v8::Local<v8::Function> = callback_arg.try_into().unwrap();
        
        // Create mock request/response objects for immediate execution
        let undefined = v8::undefined(scope);
        let args = [];
        
        // Execute the callback immediately
        if let Some(result) = callback_fn.call(scope, undefined.into(), &args) {
            let response = if result.is_object() && !result.is_string() {
                // Object result - convert to JSON
                let json_stringify_key = v8::String::new(scope, "JSON").unwrap();
                let global = scope.get_current_context().global(scope);
                
                if let Some(json_obj) = global.get(scope, json_stringify_key.into()) {
                    if let Ok(json_obj) = json_obj.try_into() {
                        let json_obj: v8::Local<v8::Object> = json_obj;
                        let stringify_key = v8::String::new(scope, "stringify").unwrap();
                        
                        if let Some(stringify_fn) = json_obj.get(scope, stringify_key.into()) {
                            if let Ok(stringify_fn) = stringify_fn.try_into() {
                                let stringify_fn: v8::Local<v8::Function> = stringify_fn;
                                let args = [result];
                                
                                if let Some(json_result) = stringify_fn.call(scope, json_obj.into(), &args) {
                                    json_result.to_string(scope).unwrap().to_rust_string_lossy(scope)
                                } else {
                                    result.to_string(scope).unwrap().to_rust_string_lossy(scope)
                                }
                            } else {
                                result.to_string(scope).unwrap().to_rust_string_lossy(scope)
                            }
                        } else {
                            result.to_string(scope).unwrap().to_rust_string_lossy(scope)
                        }
                    } else {
                        result.to_string(scope).unwrap().to_rust_string_lossy(scope)
                    }
                } else {
                    result.to_string(scope).unwrap().to_rust_string_lossy(scope)
                }
            } else {
                // String or primitive result
                result.to_string(scope).unwrap().to_rust_string_lossy(scope)
            };
            
            RouteInfo {
                method: method.to_string(),
                path: path.clone(),
                callback_id: None,
                static_response: Some(response),
            }
        } else {
            // Fallback if execution fails
            RouteInfo {
                method: method.to_string(),
                path: path.clone(),
                callback_id: None,
                static_response: Some(format!("Route handler for {} {}", method, path)),
            }
        }
    } else {
        // Default fallback
        RouteInfo {
            method: method.to_string(),
            path: path.clone(),
            callback_id: None,
            static_response: Some(format!("Route handler for {} {}", method, path)),
        }
    };

    ROUTES.lock().unwrap().push(route);
}

fn server_put(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "PUT");
}

fn server_delete(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    register_route(scope, args, "DELETE");
}

async fn start_http_server(port: u16, mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);
    eprintln!("HTTP server listening on http://127.0.0.1:{}", port);
    
    // Graceful shutdown with broadcast receiver
    let graceful = server.with_graceful_shutdown(async {
        let _ = shutdown_rx.recv().await;
        eprintln!("Shutting down HTTP server on port {}", port);
    });
    
    if let Err(e) = graceful.await {
        eprintln!("Server error: {}", e);
        return Err(e.into());
    }
    
    Ok(())
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let method = req.method().to_string();
    let path = req.uri().path();

    // Check for registered routes first
    {
        let routes = ROUTES.lock().unwrap();
        for route in routes.iter() {
            if route.method == method && route.path == path {
                if let Some(response) = &route.static_response {
                    // Determine content type based on response content
                    let content_type = if response.trim_start().starts_with('{') || response.trim_start().starts_with('[') {
                        "application/json"
                    } else {
                        "text/plain"
                    };
                    
                    return Ok(Response::builder()
                        .header("content-type", content_type)
                        .header("access-control-allow-origin", "*")  // CORS support
                        .body(Body::from(response.clone()))
                        .unwrap());
                }
            }
        }
    }

    // Built-in routes
    let response = match (req.method(), path) {
        (&Method::GET, "/") => Response::new(Body::from("Hello from Kiren HTTP Server")),
        (&Method::GET, "/health") => {
            let health = json!({
                "status": "ok",
                "runtime": "kiren",
                "version": "0.1.0",
                "routes": ROUTES.lock().unwrap().len()
            });
            Response::builder()
                .header("content-type", "application/json")
                .body(Body::from(health.to_string()))
                .unwrap()
        }
        (&Method::GET, "/api/stats") => {
            let routes = ROUTES.lock().unwrap();
            let stats = json!({
                "registered_routes": routes.len(),
                "routes": routes.iter().map(|r| {
                    json!({
                        "method": r.method,
                        "path": r.path
                    })
                }).collect::<Vec<_>>(),
                "runtime": "kiren"
            });
            Response::builder()
                .header("content-type", "application/json")
                .body(Body::from(stats.to_string()))
                .unwrap()
        }
        (&Method::GET, path) if path.starts_with("/api/") => {
            let json_response = json!({
                "message": format!("API endpoint {}", path),
                "runtime": "kiren"
            });
            Response::builder()
                .header("content-type", "application/json")
                .body(Body::from(json_response.to_string()))
                .unwrap()
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap(),
    };

    Ok(response)
}
