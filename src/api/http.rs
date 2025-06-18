use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use once_cell::sync::Lazy;
use serde_json::json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use v8;

// Safe route storage without V8 handles
#[derive(Clone)]
struct RouteInfo {
    method: String,
    path: String,
    response: String,
}

static ROUTES: Lazy<Arc<Mutex<Vec<RouteInfo>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

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

fn create_server(
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

    // SAFETY: Don't spawn server in background to avoid scope issues
    // Instead, use a controlled approach
    println!("🚀 Kiren HTTP server configured for port {}", port);
    println!("✅ Server running at http://localhost:{}", port);
    println!("🔗 Health check: http://localhost:{}/health", port);
    println!("📊 API stats: http://localhost:{}/api/stats", port);

    // Start server in a controlled way using current thread
    std::thread::spawn(move || {
        println!("🔧 Starting HTTP server thread on port {}", port);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            println!("🚀 HTTP server runtime created, binding to address...");
            if let Err(e) = start_http_server(port).await {
                eprintln!("❌ Server error: {}", e);
            } else {
                println!("✅ HTTP server started successfully on port {}", port);
            }
        });
        println!("🔚 HTTP server thread ending");
    });
}

fn server_get(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    // SAFETY: Store route info without V8 handles
    let callback_arg = args.get(1);
    let response_content = if callback_arg.is_string() {
        // If callback is a string, use it as response
        let response_str = callback_arg.to_string(scope).unwrap();
        response_str.to_rust_string_lossy(scope)
    } else {
        // Default response for function callbacks (not implemented yet)
        format!("Route handler for GET {}", path)
    };

    let route = RouteInfo {
        method: "GET".to_string(),
        path: path.clone(),
        response: response_content,
    };

    ROUTES.lock().unwrap().push(route);
    println!("GET route registered: {}", path);
}

fn server_post(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    // SAFETY: Store route info without V8 handles
    let callback_arg = args.get(1);
    let response_content = if callback_arg.is_string() {
        // If callback is a string, use it as response
        let response_str = callback_arg.to_string(scope).unwrap();
        response_str.to_rust_string_lossy(scope)
    } else {
        // Default response for function callbacks (not implemented yet)
        format!("Route handler for POST {}", path)
    };

    let route = RouteInfo {
        method: "POST".to_string(),
        path: path.clone(),
        response: response_content,
    };

    ROUTES.lock().unwrap().push(route);
    println!("POST route registered: {}", path);
}

fn server_put(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    // SAFETY: Store route info without V8 handles
    let callback_arg = args.get(1);
    let response_content = if callback_arg.is_string() {
        // If callback is a string, use it as response
        let response_str = callback_arg.to_string(scope).unwrap();
        response_str.to_rust_string_lossy(scope)
    } else {
        // Default response for function callbacks (not implemented yet)
        format!("Route handler for PUT {}", path)
    };

    let route = RouteInfo {
        method: "PUT".to_string(),
        path: path.clone(),
        response: response_content,
    };

    ROUTES.lock().unwrap().push(route);
    println!("PUT route registered: {}", path);
}

fn server_delete(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    // SAFETY: Store route info without V8 handles
    let callback_arg = args.get(1);
    let response_content = if callback_arg.is_string() {
        // If callback is a string, use it as response
        let response_str = callback_arg.to_string(scope).unwrap();
        response_str.to_rust_string_lossy(scope)
    } else {
        // Default response for function callbacks (not implemented yet)
        format!("Route handler for DELETE {}", path)
    };

    let route = RouteInfo {
        method: "DELETE".to_string(),
        path: path.clone(),
        response: response_content,
    };

    ROUTES.lock().unwrap().push(route);
    println!("DELETE route registered: {}", path);
}

async fn start_http_server(port: u16) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🌐 Binding HTTP server to address: {}", addr);

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);
    println!("🎯 HTTP server bound successfully, starting to serve requests...");

    if let Err(e) = server.await {
        eprintln!("❌ HTTP server failed: {}", e);
        return Err(e.into());
    }

    println!("✅ HTTP server completed successfully");
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
                return Ok(Response::builder()
                    .header("content-type", "text/html")
                    .body(Body::from(route.response.clone()))
                    .unwrap());
            }
        }
    }

    // Built-in routes
    let response = match (req.method(), path) {
        (&Method::GET, "/") => Response::new(Body::from("Hello from Kiren HTTP Server! 🚀")),
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
