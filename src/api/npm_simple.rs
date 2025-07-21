use anyhow::Result;
use std::fs;
use std::path::Path;
use v8;

pub fn setup_npm_compatibility(
    scope: &mut v8::HandleScope,
    context: v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Simple require() function for basic CommonJS compatibility
    let require_key = v8::String::new(scope, "require").unwrap();
    let require_template = v8::FunctionTemplate::new(scope, simple_require);
    let require_function_obj = require_template.get_function(scope).unwrap();
    global.set(scope, require_key.into(), require_function_obj.into());

    // __dirname and __filename globals
    let dirname_key = v8::String::new(scope, "__dirname").unwrap();
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let dirname_value = v8::String::new(scope, &current_dir.to_string_lossy()).unwrap();
    global.set(scope, dirname_key.into(), dirname_value.into());

    let filename_key = v8::String::new(scope, "__filename").unwrap();
    let filename_value = v8::String::new(scope, "main.js").unwrap();
    global.set(scope, filename_key.into(), filename_value.into());

    // module.exports object for CommonJS
    let module_key = v8::String::new(scope, "module").unwrap();
    let module_obj = v8::Object::new(scope);

    let exports_key = v8::String::new(scope, "exports").unwrap();
    let exports_obj = v8::Object::new(scope);
    module_obj.set(scope, exports_key.into(), exports_obj.into());

    global.set(scope, module_key.into(), module_obj.into());

    // exports shorthand
    let exports_global_key = v8::String::new(scope, "exports").unwrap();
    global.set(scope, exports_global_key.into(), exports_obj.into());

    Ok(())
}

fn simple_require(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error_str = v8::String::new(scope, "require() missing path argument").unwrap();
        let error = v8::Exception::error(scope, error_str);
        scope.throw_exception(error);
        return;
    }

    let module_path_arg = args.get(0);
    let module_path_str = module_path_arg.to_string(scope).unwrap();
    let module_path = module_path_str.to_rust_string_lossy(scope);

    println!("Requiring module: {}", module_path);

    // Try to load file-based modules first (like Node.js)
    match load_module_with_resolution(scope, &module_path) {
        Ok(module_exports) => {
            println!("✓ Module loaded from file system: {}", module_path);
            rv.set(module_exports);
            return;
        }
        Err(e) => {
            println!("✗ File system load failed: {}", e);
            // Fallback to built-in modules for core modules only
            if let Some(builtin) = get_builtin_module(scope, &module_path) {
                println!("✓ Using built-in module: {}", module_path);
                rv.set(builtin);
                return;
            }
        }
    }

    // If neither worked, throw error
    let error_msg = format!("Cannot resolve module '{}'", module_path);
    let error_str = v8::String::new(scope, &error_msg).unwrap();
    let error = v8::Exception::error(scope, error_str);
    scope.throw_exception(error);
}

fn load_module_with_resolution<'a>(
    scope: &mut v8::HandleScope<'a>,
    module_path: &str,
) -> Result<v8::Local<'a, v8::Value>> {
    let current_dir = std::env::current_dir()?;
    println!("Current directory: {}", current_dir.display());

    // Node.js style module resolution
    if !module_path.starts_with("./")
        && !module_path.starts_with("../")
        && !module_path.starts_with("/")
    {
        println!("Trying node_modules resolution for: {}", module_path);
        // Try node_modules
        if let Ok(result) = try_node_modules_resolution(scope, &current_dir, module_path) {
            println!("✓ Found in node_modules");
            return Ok(result);
        }
        println!("✗ Not found in node_modules");
    }

    println!("Trying relative/absolute path resolution");
    // Relative/absolute path resolution
    load_simple_module(scope, module_path)
}

fn try_node_modules_resolution<'a>(
    scope: &mut v8::HandleScope<'a>,
    current_dir: &std::path::Path,
    module_name: &str,
) -> Result<v8::Local<'a, v8::Value>> {
    let node_modules_path = current_dir.join("node_modules").join(module_name);
    println!("Looking for module at: {}", node_modules_path.display());

    // Try package.json main field
    let package_json_path = node_modules_path.join("package.json");
    println!("Checking package.json at: {}", package_json_path.display());

    if package_json_path.exists() {
        println!("✓ package.json exists");
        if let Ok(package_content) = fs::read_to_string(&package_json_path) {
            println!("✓ package.json read successfully");
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&package_content) {
                println!("✓ package.json parsed successfully");
                if let Some(main) = package_json.get("main").and_then(|v| v.as_str()) {
                    println!("✓ Found main field: {}", main);
                    let main_path = node_modules_path.join(main);
                    println!("Checking main file at: {}", main_path.display());
                    if main_path.exists() {
                        println!("✓ Main file exists, reading...");
                        let content = fs::read_to_string(&main_path)?;
                        println!("✓ Main file content read, executing...");
                        return execute_module_content(scope, &content);
                    } else {
                        println!("✗ Main file does not exist");
                    }
                } else {
                    println!("✗ No main field in package.json");
                }
            } else {
                println!("✗ Failed to parse package.json");
            }
        } else {
            println!("✗ Failed to read package.json");
        }
    } else {
        println!("✗ package.json does not exist");
    }

    // Try index.js
    let index_path = node_modules_path.join("index.js");
    println!("Trying index.js at: {}", index_path.display());
    if index_path.exists() {
        let content = fs::read_to_string(&index_path)?;
        return execute_module_content(scope, &content);
    }

    // Try direct file
    if node_modules_path.exists() && node_modules_path.is_file() {
        let content = fs::read_to_string(&node_modules_path)?;
        return execute_module_content(scope, &content);
    }

    Err(anyhow::anyhow!("Module not found: {}", module_name))
}

fn get_builtin_module<'a>(
    scope: &mut v8::HandleScope<'a>,
    module_name: &str,
) -> Option<v8::Local<'a, v8::Value>> {
    match module_name {
        "fs" => Some(create_simple_fs_module(scope)),
        "path" => Some(create_simple_path_module(scope)),
        "os" => Some(create_simple_os_module(scope)),
        "http" => Some(create_simple_http_module(scope)),
        "express" => Some(create_express_module(scope)),
        "socket.io" => Some(create_socketio_module(scope)),
        "redis" => Some(create_redis_module(scope)),
        "cors" => Some(create_cors_module(scope)),
        "body-parser" => Some(create_body_parser_module(scope)),
        "cookie-parser" => Some(create_cookie_parser_module(scope)),
        "dotenv" => Some(create_dotenv_module(scope)),
        "uuid" => Some(create_uuid_module(scope)),
        "jsonwebtoken" => Some(create_jsonwebtoken_module(scope)),
        "axios" => Some(create_axios_module(scope)),
        _ => None,
    }
}

fn create_simple_fs_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let fs_obj = v8::Object::new(scope);

    // readFileSync
    let read_file_sync_key = v8::String::new(scope, "readFileSync").unwrap();
    let read_file_sync_template = v8::FunctionTemplate::new(scope, simple_read_file_sync);
    let read_file_sync_fn = read_file_sync_template.get_function(scope).unwrap();
    fs_obj.set(scope, read_file_sync_key.into(), read_file_sync_fn.into());

    // writeFileSync
    let write_file_sync_key = v8::String::new(scope, "writeFileSync").unwrap();
    let write_file_sync_template = v8::FunctionTemplate::new(scope, simple_write_file_sync);
    let write_file_sync_fn = write_file_sync_template.get_function(scope).unwrap();
    fs_obj.set(scope, write_file_sync_key.into(), write_file_sync_fn.into());

    // existsSync
    let exists_sync_key = v8::String::new(scope, "existsSync").unwrap();
    let exists_sync_template = v8::FunctionTemplate::new(scope, simple_exists_sync);
    let exists_sync_fn = exists_sync_template.get_function(scope).unwrap();
    fs_obj.set(scope, exists_sync_key.into(), exists_sync_fn.into());

    fs_obj.into()
}

fn create_simple_path_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let path_obj = v8::Object::new(scope);

    // join
    let join_key = v8::String::new(scope, "join").unwrap();
    let join_template = v8::FunctionTemplate::new(scope, simple_path_join);
    let join_fn = join_template.get_function(scope).unwrap();
    path_obj.set(scope, join_key.into(), join_fn.into());

    path_obj.into()
}

fn create_simple_os_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let os_obj = v8::Object::new(scope);

    // platform
    let platform_key = v8::String::new(scope, "platform").unwrap();
    let platform_template = v8::FunctionTemplate::new(scope, simple_os_platform);
    let platform_fn = platform_template.get_function(scope).unwrap();
    os_obj.set(scope, platform_key.into(), platform_fn.into());

    os_obj.into()
}

fn create_simple_http_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let http_obj = v8::Object::new(scope);

    // createServer
    let create_server_key = v8::String::new(scope, "createServer").unwrap();
    let create_server_template = v8::FunctionTemplate::new(scope, simple_create_server);
    let create_server_fn = create_server_template.get_function(scope).unwrap();
    http_obj.set(scope, create_server_key.into(), create_server_fn.into());

    http_obj.into()
}

fn load_simple_module<'a>(
    scope: &mut v8::HandleScope<'a>,
    module_path: &str,
) -> Result<v8::Local<'a, v8::Value>> {
    let current_dir = std::env::current_dir()?;
    let full_path = if module_path.starts_with("./") || module_path.starts_with("../") {
        current_dir.join(module_path)
    } else {
        std::path::PathBuf::from(module_path)
    };

    // Try the path as-is
    if let Ok(content) = fs::read_to_string(&full_path) {
        return execute_module_content(scope, &content);
    }

    // Try with .js extension
    let js_path = full_path.with_extension("js");
    if let Ok(content) = fs::read_to_string(&js_path) {
        return execute_module_content(scope, &content);
    }

    // Try with .json extension
    let json_path = full_path.with_extension("json");
    if let Ok(content) = fs::read_to_string(&json_path) {
        return parse_json_module(scope, &content);
    }

    Err(anyhow::anyhow!("Module not found: {}", module_path))
}

fn execute_module_content<'a>(
    scope: &mut v8::HandleScope<'a>,
    content: &str,
) -> Result<v8::Local<'a, v8::Value>> {
    // Create a simple module execution context
    let _exports_obj = v8::Object::new(scope);

    // Wrap the module code in a simple IIFE
    let wrapped_code = format!(
        "(function() {{\n  var exports = {{}}, module = {{ exports: exports }};\n  {}\n  return module.exports;\n}})()",
        content
    );

    let source_string = v8::String::new(scope, &wrapped_code).unwrap();
    let script = v8::Script::compile(scope, source_string, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to compile module"))?;

    let result = script
        .run(scope)
        .ok_or_else(|| anyhow::anyhow!("Failed to run module"))?;

    Ok(result)
}

fn parse_json_module<'a>(
    scope: &mut v8::HandleScope<'a>,
    content: &str,
) -> Result<v8::Local<'a, v8::Value>> {
    let json_value: serde_json::Value = serde_json::from_str(content)?;
    Ok(json_to_v8_simple(scope, &json_value))
}

fn json_to_v8_simple<'a>(
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
                let v8_item = json_to_v8_simple(scope, item);
                v8_array.set_index(scope, i as u32, v8_item);
            }
            v8_array.into()
        }
        serde_json::Value::Object(obj) => {
            let v8_obj = v8::Object::new(scope);
            for (key, val) in obj {
                let v8_key = v8::String::new(scope, key).unwrap();
                let v8_val = json_to_v8_simple(scope, val);
                v8_obj.set(scope, v8_key.into(), v8_val);
            }
            v8_obj.into()
        }
    }
}

// Simple builtin function implementations
fn simple_read_file_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error_str = v8::String::new(scope, "readFileSync requires a path").unwrap();
        let error = v8::Exception::error(scope, error_str);
        scope.throw_exception(error);
        return;
    }

    let path_arg = args.get(0);
    let path_string = path_arg.to_string(scope).unwrap();
    let path = path_string.to_rust_string_lossy(scope);

    match fs::read_to_string(&path) {
        Ok(content) => {
            let result = v8::String::new(scope, &content).unwrap();
            rv.set(result.into());
        }
        Err(e) => {
            let error_msg = format!("Failed to read file '{}': {}", path, e);
            let error_str = v8::String::new(scope, &error_msg).unwrap();
            let error = v8::Exception::error(scope, error_str);
            scope.throw_exception(error);
        }
    }
}

fn simple_write_file_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error_str = v8::String::new(scope, "writeFileSync requires path and data").unwrap();
        let error = v8::Exception::error(scope, error_str);
        scope.throw_exception(error);
        return;
    }

    let path_arg = args.get(0);
    let path_string = path_arg.to_string(scope).unwrap();
    let path = path_string.to_rust_string_lossy(scope);

    let data_arg = args.get(1);
    let data_string = data_arg.to_string(scope).unwrap();
    let data = data_string.to_rust_string_lossy(scope);

    if let Err(e) = fs::write(&path, data) {
        let error_msg = format!("Failed to write file '{}': {}", path, e);
        let error_str = v8::String::new(scope, &error_msg).unwrap();
        let error = v8::Exception::error(scope, error_str);
        scope.throw_exception(error);
    }
}

fn simple_exists_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    }

    let path_arg = args.get(0);
    let path_string = path_arg.to_string(scope).unwrap();
    let path = path_string.to_rust_string_lossy(scope);

    let exists = Path::new(&path).exists();
    rv.set(v8::Boolean::new(scope, exists).into());
}

fn simple_path_join(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut path_buf = std::path::PathBuf::new();

    for i in 0..args.length() {
        let arg = args.get(i);
        let str_string = arg.to_string(scope).unwrap();
        let part = str_string.to_rust_string_lossy(scope);
        path_buf.push(part);
    }

    let result = v8::String::new(scope, &path_buf.to_string_lossy()).unwrap();
    rv.set(result.into());
}

fn simple_os_platform(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let platform = std::env::consts::OS;
    let result = v8::String::new(scope, platform).unwrap();
    rv.set(result.into());
}

fn simple_create_server(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create a simple server object
    let server_obj = v8::Object::new(scope);

    // listen method
    let listen_key = v8::String::new(scope, "listen").unwrap();
    let listen_template = v8::FunctionTemplate::new(scope, simple_server_listen);
    let listen_fn = listen_template.get_function(scope).unwrap();
    server_obj.set(scope, listen_key.into(), listen_fn.into());

    rv.set(server_obj.into());
}

fn simple_server_listen(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let port = if args.length() > 0 {
        args.get(0).uint32_value(scope).unwrap_or(3000)
    } else {
        3000
    };

    println!("HTTP server listening on port {}", port);

    // Execute callback if provided
    if args.length() > 1 && args.get(1).is_function() {
        let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
        let undefined = v8::undefined(scope);
        let callback_args = [];
        callback.call(scope, undefined.into(), &callback_args);
    }
}

fn create_express_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    // Create the main express function
    let express_template = v8::FunctionTemplate::new(scope, create_express_app_from_require);
    let express_function = express_template.get_function(scope).unwrap();

    // Add static method (express.static)
    let static_key = v8::String::new(scope, "static").unwrap();
    let static_template = v8::FunctionTemplate::new(scope, crate::api::express::express_static);
    let static_function = static_template.get_function(scope).unwrap();
    express_function.set(scope, static_key.into(), static_function.into());

    // Add json method (express.json)
    let json_key = v8::String::new(scope, "json").unwrap();
    let json_template = v8::FunctionTemplate::new(scope, crate::api::express::express_json);
    let json_function = json_template.get_function(scope).unwrap();
    express_function.set(scope, json_key.into(), json_function.into());

    // Add urlencoded method (express.urlencoded)
    let urlencoded_key = v8::String::new(scope, "urlencoded").unwrap();
    let urlencoded_template =
        v8::FunctionTemplate::new(scope, crate::api::express::express_urlencoded);
    let urlencoded_function = urlencoded_template.get_function(scope).unwrap();
    express_function.set(scope, urlencoded_key.into(), urlencoded_function.into());

    // Add Router method (express.Router)
    let router_key = v8::String::new(scope, "Router").unwrap();
    let router_template = v8::FunctionTemplate::new(scope, crate::api::express::create_router);
    let router_function = router_template.get_function(scope).unwrap();
    express_function.set(scope, router_key.into(), router_function.into());

    express_function.into()
}

// Wrapper function that creates an express app when called from require('express')()
fn create_express_app_from_require(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // This is equivalent to calling the original create_express_app
    crate::api::express::create_express_app(scope, args, rv);
}

fn create_socketio_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    // Create Socket.IO module object
    let socketio_obj = v8::Object::new(scope);

    // Server constructor
    let server_key = v8::String::new(scope, "Server").unwrap();
    let server_template = v8::FunctionTemplate::new(scope, socketio_server_constructor);
    let server_function = server_template.get_function(scope).unwrap();
    socketio_obj.set(scope, server_key.into(), server_function.into());

    // Also add default function export that creates a server
    let default_template = v8::FunctionTemplate::new(scope, socketio_server_constructor);
    let default_function = default_template.get_function(scope).unwrap();

    // Set Server as constructor property
    default_function.set(scope, server_key.into(), server_function.into());

    socketio_obj.into()
}

fn create_redis_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let redis_obj = v8::Object::new(scope);

    // createClient method
    let create_client_key = v8::String::new(scope, "createClient").unwrap();
    let create_client_template = v8::FunctionTemplate::new(scope, redis_create_client);
    let create_client_function = create_client_template.get_function(scope).unwrap();
    redis_obj.set(
        scope,
        create_client_key.into(),
        create_client_function.into(),
    );

    redis_obj.into()
}

fn create_cors_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    // CORS is typically used as a function that returns middleware
    let cors_template = v8::FunctionTemplate::new(scope, cors_middleware_factory);
    let cors_function = cors_template.get_function(scope).unwrap();

    cors_function.into()
}

fn create_body_parser_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let body_parser_obj = v8::Object::new(scope);

    // json method
    let json_key = v8::String::new(scope, "json").unwrap();
    let json_template = v8::FunctionTemplate::new(scope, crate::api::express::express_json);
    let json_function = json_template.get_function(scope).unwrap();
    body_parser_obj.set(scope, json_key.into(), json_function.into());

    // urlencoded method
    let urlencoded_key = v8::String::new(scope, "urlencoded").unwrap();
    let urlencoded_template =
        v8::FunctionTemplate::new(scope, crate::api::express::express_urlencoded);
    let urlencoded_function = urlencoded_template.get_function(scope).unwrap();
    body_parser_obj.set(scope, urlencoded_key.into(), urlencoded_function.into());

    body_parser_obj.into()
}

fn socketio_server_constructor(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let server_obj = v8::Object::new(scope);

    // on method for event listeners
    let on_key = v8::String::new(scope, "on").unwrap();
    let on_template = v8::FunctionTemplate::new(scope, socketio_on);
    let on_function = on_template.get_function(scope).unwrap();
    server_obj.set(scope, on_key.into(), on_function.into());

    // emit method
    let emit_key = v8::String::new(scope, "emit").unwrap();
    let emit_template = v8::FunctionTemplate::new(scope, socketio_emit);
    let emit_function = emit_template.get_function(scope).unwrap();
    server_obj.set(scope, emit_key.into(), emit_function.into());

    println!("Socket.IO Server created (mock implementation)");
    rv.set(server_obj.into());
}

fn socketio_on(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() >= 2 {
        let event_arg = args.get(0);
        let event_str = event_arg.to_string(scope).unwrap();
        let event = event_str.to_rust_string_lossy(scope);

        println!("Socket.IO: Registered listener for '{}' event", event);

        // Simulate immediate connection for 'connection' event
        if event == "connection" && args.get(1).is_function() {
            let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();

            // Create mock socket object
            let socket_obj = v8::Object::new(scope);

            // Add socket.emit method
            let emit_key = v8::String::new(scope, "emit").unwrap();
            let emit_template = v8::FunctionTemplate::new(scope, socketio_emit);
            let emit_function = emit_template.get_function(scope).unwrap();
            socket_obj.set(scope, emit_key.into(), emit_function.into());

            // Add socket.on method
            let on_key = v8::String::new(scope, "on").unwrap();
            let on_template = v8::FunctionTemplate::new(scope, socketio_on);
            let on_function = on_template.get_function(scope).unwrap();
            socket_obj.set(scope, on_key.into(), on_function.into());

            // Call the connection callback with mock socket
            let undefined = v8::undefined(scope);
            let callback_args = [socket_obj.into()];
            callback.call(scope, undefined.into(), &callback_args);
            println!("Socket.IO: Simulated connection event");
        }
    }
}

fn socketio_emit(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() >= 1 {
        let event_arg = args.get(0);
        let event_str = event_arg.to_string(scope).unwrap();
        let event = event_str.to_rust_string_lossy(scope);

        let data = if args.length() > 1 {
            let data_arg = args.get(1);
            let data_str = data_arg.to_string(scope).unwrap();
            data_str.to_rust_string_lossy(scope)
        } else {
            "no data".to_string()
        };

        println!("Socket.IO: Emitted '{}' event with data: {}", event, data);
    }
}

fn redis_create_client(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let client_obj = v8::Object::new(scope);

    // connect method
    let connect_key = v8::String::new(scope, "connect").unwrap();
    let connect_template = v8::FunctionTemplate::new(scope, redis_connect);
    let connect_function = connect_template.get_function(scope).unwrap();
    client_obj.set(scope, connect_key.into(), connect_function.into());

    // get method
    let get_key = v8::String::new(scope, "get").unwrap();
    let get_template = v8::FunctionTemplate::new(scope, redis_get);
    let get_function = get_template.get_function(scope).unwrap();
    client_obj.set(scope, get_key.into(), get_function.into());

    // set method
    let set_key = v8::String::new(scope, "set").unwrap();
    let set_template = v8::FunctionTemplate::new(scope, redis_set);
    let set_function = set_template.get_function(scope).unwrap();
    client_obj.set(scope, set_key.into(), set_function.into());

    // del method
    let del_key = v8::String::new(scope, "del").unwrap();
    let del_template = v8::FunctionTemplate::new(scope, redis_del);
    let del_function = del_template.get_function(scope).unwrap();
    client_obj.set(scope, del_key.into(), del_function.into());

    println!("Redis client created (mock implementation)");
    rv.set(client_obj.into());
}

fn redis_connect(
    _scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    println!("Redis: Connected to mock Redis server");
}

fn redis_get(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() >= 2 {
        let key_arg = args.get(0);
        let key_str = key_arg.to_string(scope).unwrap();
        let key = key_str.to_rust_string_lossy(scope);

        println!("Redis GET: {}", key);

        // Mock response - call callback with null value
        if args.get(1).is_function() {
            let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
            let undefined = v8::undefined(scope);
            let null_val = v8::null(scope);
            let callback_args = [null_val.into(), null_val.into()]; // err, result
            callback.call(scope, undefined.into(), &callback_args);
        }
    }
}

fn redis_set(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() >= 3 {
        let key_arg = args.get(0);
        let key_str = key_arg.to_string(scope).unwrap();
        let key = key_str.to_rust_string_lossy(scope);

        let value_arg = args.get(1);
        let value_str = value_arg.to_string(scope).unwrap();
        let value = value_str.to_rust_string_lossy(scope);

        println!("Redis SET: {} = {}", key, value);

        // Mock response - call callback with success
        if args.get(2).is_function() {
            let callback = v8::Local::<v8::Function>::try_from(args.get(2)).unwrap();
            let undefined = v8::undefined(scope);
            let null_val = v8::null(scope);
            let ok_val = v8::String::new(scope, "OK").unwrap();
            let callback_args = [null_val.into(), ok_val.into()]; // err, result
            callback.call(scope, undefined.into(), &callback_args);
        }
    }
}

fn redis_del(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() >= 2 {
        let key_arg = args.get(0);
        let key_str = key_arg.to_string(scope).unwrap();
        let key = key_str.to_rust_string_lossy(scope);

        println!("Redis DEL: {}", key);

        // Mock response - call callback with success
        if args.get(1).is_function() {
            let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
            let undefined = v8::undefined(scope);
            let null_val = v8::null(scope);
            let count_val = v8::Integer::new(scope, 1);
            let callback_args = [null_val.into(), count_val.into()]; // err, result
            callback.call(scope, undefined.into(), &callback_args);
        }
    }
}

fn cors_middleware_factory(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Return a middleware function that adds CORS headers
    let middleware_template = v8::FunctionTemplate::new(scope, cors_middleware);
    let middleware_function = middleware_template.get_function(scope).unwrap();

    rv.set(middleware_function.into());
}

fn cors_middleware(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    // CORS middleware implementation
    if args.length() >= 3 {
        let _req = args.get(0);
        let res = args.get(1);
        let next = args.get(2);

        // Add CORS headers to response
        if let Ok(res_obj) = v8::Local::<v8::Object>::try_from(res) {
            let set_header_key = v8::String::new(scope, "set").unwrap();
            if let Some(set_header_fn) = res_obj.get(scope, set_header_key.into()) {
                if let Ok(set_function) = v8::Local::<v8::Function>::try_from(set_header_fn) {
                    // Set CORS headers
                    let headers = [
                        ("Access-Control-Allow-Origin", "*"),
                        (
                            "Access-Control-Allow-Methods",
                            "GET, POST, PUT, DELETE, OPTIONS",
                        ),
                        (
                            "Access-Control-Allow-Headers",
                            "Content-Type, Authorization",
                        ),
                    ];

                    for (name, value) in headers {
                        let name_val = v8::String::new(scope, name).unwrap();
                        let value_val = v8::String::new(scope, value).unwrap();
                        let set_args = [name_val.into(), value_val.into()];
                        set_function.call(scope, res.into(), &set_args);
                    }
                }
            }
        }

        // Call next middleware
        if let Ok(next_fn) = v8::Local::<v8::Function>::try_from(next) {
            let undefined = v8::undefined(scope);
            next_fn.call(scope, undefined.into(), &[]);
        }

        println!("CORS: Headers set");
    }
}

fn create_cookie_parser_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    // Cookie parser returns a middleware function
    let cookie_parser_template = v8::FunctionTemplate::new(scope, cookie_parser_middleware_factory);
    let cookie_parser_function = cookie_parser_template.get_function(scope).unwrap();

    cookie_parser_function.into()
}

fn create_dotenv_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let dotenv_obj = v8::Object::new(scope);

    // config method
    let config_key = v8::String::new(scope, "config").unwrap();
    let config_template = v8::FunctionTemplate::new(scope, dotenv_config);
    let config_function = config_template.get_function(scope).unwrap();
    dotenv_obj.set(scope, config_key.into(), config_function.into());

    dotenv_obj.into()
}

fn create_uuid_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let uuid_obj = v8::Object::new(scope);

    // v4 method for random UUID
    let v4_key = v8::String::new(scope, "v4").unwrap();
    let v4_template = v8::FunctionTemplate::new(scope, uuid_v4);
    let v4_function = v4_template.get_function(scope).unwrap();
    uuid_obj.set(scope, v4_key.into(), v4_function.into());

    uuid_obj.into()
}

fn create_jsonwebtoken_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let jwt_obj = v8::Object::new(scope);

    // sign method
    let sign_key = v8::String::new(scope, "sign").unwrap();
    let sign_template = v8::FunctionTemplate::new(scope, jwt_sign);
    let sign_function = sign_template.get_function(scope).unwrap();
    jwt_obj.set(scope, sign_key.into(), sign_function.into());

    // verify method
    let verify_key = v8::String::new(scope, "verify").unwrap();
    let verify_template = v8::FunctionTemplate::new(scope, jwt_verify);
    let verify_function = verify_template.get_function(scope).unwrap();
    jwt_obj.set(scope, verify_key.into(), verify_function.into());

    jwt_obj.into()
}

fn create_axios_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    // Axios can be used as a function and has method properties
    let axios_template = v8::FunctionTemplate::new(scope, axios_request);
    let axios_function = axios_template.get_function(scope).unwrap();

    // Add HTTP method shortcuts
    let get_key = v8::String::new(scope, "get").unwrap();
    let get_template = v8::FunctionTemplate::new(scope, axios_get);
    let get_function = get_template.get_function(scope).unwrap();
    axios_function.set(scope, get_key.into(), get_function.into());

    let post_key = v8::String::new(scope, "post").unwrap();
    let post_template = v8::FunctionTemplate::new(scope, axios_post);
    let post_function = post_template.get_function(scope).unwrap();
    axios_function.set(scope, post_key.into(), post_function.into());

    let put_key = v8::String::new(scope, "put").unwrap();
    let put_template = v8::FunctionTemplate::new(scope, axios_put);
    let put_function = put_template.get_function(scope).unwrap();
    axios_function.set(scope, put_key.into(), put_function.into());

    let delete_key = v8::String::new(scope, "delete").unwrap();
    let delete_template = v8::FunctionTemplate::new(scope, axios_delete);
    let delete_function = delete_template.get_function(scope).unwrap();
    axios_function.set(scope, delete_key.into(), delete_function.into());

    axios_function.into()
}

fn cookie_parser_middleware_factory(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Return a middleware function that parses cookies
    let middleware_template = v8::FunctionTemplate::new(scope, cookie_parser_middleware);
    let middleware_function = middleware_template.get_function(scope).unwrap();

    rv.set(middleware_function.into());
}

fn cookie_parser_middleware(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    // Cookie parser middleware implementation
    if args.length() >= 3 {
        let req = args.get(0);
        let _res = args.get(1);
        let next = args.get(2);

        // Add cookies property to request
        if let Ok(req_obj) = v8::Local::<v8::Object>::try_from(req) {
            let cookies_key = v8::String::new(scope, "cookies").unwrap();
            let cookies_obj = v8::Object::new(scope);
            req_obj.set(scope, cookies_key.into(), cookies_obj.into());
        }

        // Call next middleware
        if let Ok(next_fn) = v8::Local::<v8::Function>::try_from(next) {
            let undefined = v8::undefined(scope);
            next_fn.call(scope, undefined.into(), &[]);
        }

        println!("Cookie Parser: Cookies parsed");
    }
}

fn dotenv_config(
    _scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    println!("dotenv.config(): Environment variables loaded (mock)");
}

fn uuid_v4(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Generate a mock UUID
    let uuid = "12345678-1234-4000-8000-123456789abc";
    let uuid_str = v8::String::new(scope, uuid).unwrap();
    rv.set(uuid_str.into());
}

fn jwt_sign(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() >= 2 {
        let payload_arg = args.get(0);
        let secret_arg = args.get(1);

        let payload_str = payload_arg.to_string(scope).unwrap();
        let payload = payload_str.to_rust_string_lossy(scope);
        let secret_str = secret_arg.to_string(scope).unwrap();
        let secret = secret_str.to_rust_string_lossy(scope);

        println!("JWT Sign: payload={}, secret={}", payload, secret);

        // Return mock JWT token
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.mock.token";
        let token_str = v8::String::new(scope, token).unwrap();
        rv.set(token_str.into());
    }
}

fn jwt_verify(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() >= 3 {
        let token_arg = args.get(0);
        let secret_arg = args.get(1);
        let callback_arg = args.get(2);

        let token_str = token_arg.to_string(scope).unwrap();
        let token = token_str.to_rust_string_lossy(scope);
        let secret_str = secret_arg.to_string(scope).unwrap();
        let secret = secret_str.to_rust_string_lossy(scope);

        println!("JWT Verify: token={}, secret={}", token, secret);

        // Mock verification result
        if callback_arg.is_function() {
            let callback = v8::Local::<v8::Function>::try_from(callback_arg).unwrap();
            let undefined = v8::undefined(scope);
            let null_val = v8::null(scope);

            // Create mock decoded payload
            let decoded_obj = v8::Object::new(scope);
            let user_id_key = v8::String::new(scope, "userId").unwrap();
            let user_id_val = v8::String::new(scope, "12345").unwrap();
            decoded_obj.set(scope, user_id_key.into(), user_id_val.into());

            let callback_args = [null_val.into(), decoded_obj.into()]; // err, decoded
            callback.call(scope, undefined.into(), &callback_args);
        }
    }
}

fn axios_request(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() >= 1 {
        let config_arg = args.get(0);
        let config_str = config_arg.to_string(scope).unwrap();
        let config = config_str.to_rust_string_lossy(scope);

        println!("Axios Request: {}", config);

        // Return a simple response object instead of promise
        let response_obj = v8::Object::new(scope);
        let status_key = v8::String::new(scope, "status").unwrap();
        let status_val = v8::Integer::new(scope, 200);
        response_obj.set(scope, status_key.into(), status_val.into());

        let data_key = v8::String::new(scope, "data").unwrap();
        let data_obj = v8::Object::new(scope);
        response_obj.set(scope, data_key.into(), data_obj.into());

        rv.set(response_obj.into());
    }
}

fn axios_get(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() >= 1 {
        let url_arg = args.get(0);
        let url_str = url_arg.to_string(scope).unwrap();
        let url = url_str.to_rust_string_lossy(scope);

        println!("Axios GET: {}", url);

        // Return a simple response object
        let response_obj = v8::Object::new(scope);
        let status_key = v8::String::new(scope, "status").unwrap();
        let status_val = v8::Integer::new(scope, 200);
        response_obj.set(scope, status_key.into(), status_val.into());

        rv.set(response_obj.into());
    }
}

fn axios_post(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() >= 1 {
        let url_arg = args.get(0);
        let url_str = url_arg.to_string(scope).unwrap();
        let url = url_str.to_rust_string_lossy(scope);

        println!("Axios POST: {}", url);

        let response_obj = v8::Object::new(scope);
        let status_key = v8::String::new(scope, "status").unwrap();
        let status_val = v8::Integer::new(scope, 200);
        response_obj.set(scope, status_key.into(), status_val.into());

        rv.set(response_obj.into());
    }
}

fn axios_put(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() >= 1 {
        let url_arg = args.get(0);
        let url_str = url_arg.to_string(scope).unwrap();
        let url = url_str.to_rust_string_lossy(scope);

        println!("Axios PUT: {}", url);

        let response_obj = v8::Object::new(scope);
        let status_key = v8::String::new(scope, "status").unwrap();
        let status_val = v8::Integer::new(scope, 200);
        response_obj.set(scope, status_key.into(), status_val.into());

        rv.set(response_obj.into());
    }
}

fn axios_delete(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() >= 1 {
        let url_arg = args.get(0);
        let url_str = url_arg.to_string(scope).unwrap();
        let url = url_str.to_rust_string_lossy(scope);

        println!("Axios DELETE: {}", url);

        let response_obj = v8::Object::new(scope);
        let status_key = v8::String::new(scope, "status").unwrap();
        let status_val = v8::Integer::new(scope, 200);
        response_obj.set(scope, status_key.into(), status_val.into());

        rv.set(response_obj.into());
    }
}
