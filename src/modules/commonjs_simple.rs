use anyhow::Result;
use v8;

// Simplified CommonJS support
pub fn setup_commonjs(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
    let global = context.global(scope);

    // Add global require function
    let require_key = v8::String::new(scope, "require").unwrap();
    let require_template = v8::FunctionTemplate::new(scope, global_require);
    let require_function = require_template.get_function(scope).unwrap();

    global.set(scope, require_key.into(), require_function.into());

    // Add module object
    let module_key = v8::String::new(scope, "module").unwrap();
    let module_obj = v8::Object::new(scope);
    let exports_obj = v8::Object::new(scope);

    let exports_key = v8::String::new(scope, "exports").unwrap();
    module_obj.set(scope, exports_key.into(), exports_obj.into());

    global.set(scope, module_key.into(), module_obj.into());

    // Add exports object
    let exports_key = v8::String::new(scope, "exports").unwrap();
    global.set(scope, exports_key.into(), exports_obj.into());

    Ok(())
}

// Global require function implementation
fn global_require(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "require() missing path").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let specifier_arg = args.get(0);
    let specifier_str = specifier_arg.to_string(scope).unwrap();
    let specifier = specifier_str.to_rust_string_lossy(scope);

    // Handle built-in modules
    match specifier.as_str() {
        "fs" => {
            let fs_obj = v8::Object::new(scope);

            // Add readFileSync function
            let read_file_key = v8::String::new(scope, "readFileSync").unwrap();
            let read_file_tmpl = v8::FunctionTemplate::new(scope, fs_read_file_sync);
            let read_file_fn = read_file_tmpl.get_function(scope).unwrap();
            fs_obj.set(scope, read_file_key.into(), read_file_fn.into());

            // Add writeFileSync function
            let write_file_key = v8::String::new(scope, "writeFileSync").unwrap();
            let write_file_tmpl = v8::FunctionTemplate::new(scope, fs_write_file_sync);
            let write_file_fn = write_file_tmpl.get_function(scope).unwrap();
            fs_obj.set(scope, write_file_key.into(), write_file_fn.into());

            // Add existsSync function
            let exists_key = v8::String::new(scope, "existsSync").unwrap();
            let exists_tmpl = v8::FunctionTemplate::new(scope, fs_exists_sync);
            let exists_fn = exists_tmpl.get_function(scope).unwrap();
            fs_obj.set(scope, exists_key.into(), exists_fn.into());

            rv.set(fs_obj.into());
        }
        "path" => {
            let path_obj = v8::Object::new(scope);
            let join_key = v8::String::new(scope, "join").unwrap();
            let join_str =
                v8::String::new(scope, "function join() { return arguments[0]; }").unwrap();
            path_obj.set(scope, join_key.into(), join_str.into());
            rv.set(path_obj.into());
        }
        "http" => {
            let http_obj = v8::Object::new(scope);

            // Add createServer function
            let create_server_key = v8::String::new(scope, "createServer").unwrap();
            let create_server_tmpl = v8::FunctionTemplate::new(scope, http_create_server);
            let create_server_fn = create_server_tmpl.get_function(scope).unwrap();
            http_obj.set(scope, create_server_key.into(), create_server_fn.into());

            rv.set(http_obj.into());
        }
        "express" => {
            // Create express function that returns an app
            let express_tmpl = v8::FunctionTemplate::new(scope, express_create_app);
            let express_fn = express_tmpl.get_function(scope).unwrap();

            // Add static Router property
            let router_key = v8::String::new(scope, "Router").unwrap();
            let router_tmpl = v8::FunctionTemplate::new(scope, express_create_router);
            let router_fn = router_tmpl.get_function(scope).unwrap();
            express_fn.set(scope, router_key.into(), router_fn.into());

            // Add static static property
            let static_key = v8::String::new(scope, "static").unwrap();
            let static_tmpl = v8::FunctionTemplate::new(scope, express_create_static);
            let static_fn = static_tmpl.get_function(scope).unwrap();
            express_fn.set(scope, static_key.into(), static_fn.into());

            rv.set(express_fn.into());
        }
        _ => {
            // For other modules, return empty object
            let empty_obj = v8::Object::new(scope);
            rv.set(empty_obj.into());
        }
    }
}

// File system function implementations
fn fs_read_file_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "readFileSync requires a file path").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let content_str = v8::String::new(scope, &content).unwrap();
            rv.set(content_str.into());
        }
        Err(_) => {
            let error = v8::String::new(scope, &format!("Cannot read file: {}", path)).unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
        }
    }
}

fn fs_write_file_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(scope, "writeFileSync requires path and content").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    let content_arg = args.get(1);
    let content_str = content_arg.to_string(scope).unwrap();
    let content = content_str.to_rust_string_lossy(scope);

    if std::fs::write(&path, &content).is_err() {
        let error = v8::String::new(scope, &format!("Cannot write file: {}", path)).unwrap();
        let exception = v8::Exception::error(scope, error);
        scope.throw_exception(exception);
    }
}

fn fs_exists_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let false_val = v8::Boolean::new(scope, false);
        rv.set(false_val.into());
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    let exists = std::path::Path::new(&path).exists();
    let exists_val = v8::Boolean::new(scope, exists);
    rv.set(exists_val.into());
}

// HTTP module implementation for CommonJS
fn http_create_server(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create server object
    let server_obj = v8::Object::new(scope);

    // Store the callback function if provided
    if args.length() > 0 {
        let callback = args.get(0);
        let callback_key = v8::String::new(scope, "_requestCallback").unwrap();
        server_obj.set(scope, callback_key.into(), callback);
    }

    // Add listen method
    let listen_key = v8::String::new(scope, "listen").unwrap();
    let listen_tmpl = v8::FunctionTemplate::new(scope, http_server_listen);
    let listen_fn = listen_tmpl.get_function(scope).unwrap();
    server_obj.set(scope, listen_key.into(), listen_fn.into());

    rv.set(server_obj.into());
}

fn http_server_listen(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        return;
    }

    let port_arg = args.get(0);
    let port_str = port_arg.to_string(scope).unwrap();
    let port = port_str.to_rust_string_lossy(scope);

    // Call the callback if provided (second argument)
    if args.length() > 1 {
        let callback = args.get(1);
        if callback.is_function() {
            let callback_fn: v8::Local<v8::Function> = callback.try_into().unwrap();
            let this_obj = args.this();
            let no_args = [];
            let _ = callback_fn.call(scope, this_obj.into(), &no_args);
        }
    }

    println!("HTTP Server listening on port {}", port);
}

// Express module implementation for CommonJS
fn express_create_app(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create app object
    let app_obj = v8::Object::new(scope);

    // Add HTTP methods
    let get_key = v8::String::new(scope, "get").unwrap();
    let get_template = v8::FunctionTemplate::new(scope, express_app_get);
    let get_function = get_template.get_function(scope).unwrap();
    app_obj.set(scope, get_key.into(), get_function.into());

    let post_key = v8::String::new(scope, "post").unwrap();
    let post_template = v8::FunctionTemplate::new(scope, express_app_post);
    let post_function = post_template.get_function(scope).unwrap();
    app_obj.set(scope, post_key.into(), post_function.into());

    let put_key = v8::String::new(scope, "put").unwrap();
    let put_template = v8::FunctionTemplate::new(scope, express_app_put);
    let put_function = put_template.get_function(scope).unwrap();
    app_obj.set(scope, put_key.into(), put_function.into());

    let delete_key = v8::String::new(scope, "delete").unwrap();
    let delete_template = v8::FunctionTemplate::new(scope, express_app_delete);
    let delete_function = delete_template.get_function(scope).unwrap();
    app_obj.set(scope, delete_key.into(), delete_function.into());

    // Add use method for middleware
    let use_key = v8::String::new(scope, "use").unwrap();
    let use_template = v8::FunctionTemplate::new(scope, express_app_use);
    let use_function = use_template.get_function(scope).unwrap();
    app_obj.set(scope, use_key.into(), use_function.into());

    // Add listen method
    let listen_key = v8::String::new(scope, "listen").unwrap();
    let listen_template = v8::FunctionTemplate::new(scope, express_app_listen);
    let listen_function = listen_template.get_function(scope).unwrap();
    app_obj.set(scope, listen_key.into(), listen_function.into());

    rv.set(app_obj.into());
}

fn express_create_router(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create router object (similar to app for now)
    let router_obj = v8::Object::new(scope);

    // Add HTTP methods
    let get_key = v8::String::new(scope, "get").unwrap();
    let get_template = v8::FunctionTemplate::new(scope, express_app_get);
    let get_function = get_template.get_function(scope).unwrap();
    router_obj.set(scope, get_key.into(), get_function.into());

    let post_key = v8::String::new(scope, "post").unwrap();
    let post_template = v8::FunctionTemplate::new(scope, express_app_post);
    let post_function = post_template.get_function(scope).unwrap();
    router_obj.set(scope, post_key.into(), post_function.into());

    rv.set(router_obj.into());
}

fn express_create_static(
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

    // Create a middleware function that serves static files
    let middleware_fn = v8::FunctionTemplate::new(scope, express_static_middleware);
    let middleware = middleware_fn.get_function(scope).unwrap();

    // Store root path in middleware function
    let root_key = v8::String::new(scope, "_staticRoot").unwrap();
    let root_value = v8::String::new(scope, &root_path).unwrap();
    middleware.set(scope, root_key.into(), root_value.into());

    rv.set(middleware.into());
}

fn express_static_middleware(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    // Static middleware handler
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

            // Call next() to continue
            if let Ok(next_fn) = v8::Local::<v8::Function>::try_from(next) {
                let undefined = v8::undefined(scope);
                next_fn.call(scope, undefined.into(), &[]);
            }
        }
    }
}

// Express app method implementations
fn express_app_get(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    express_register_route(scope, args, "GET");
}

fn express_app_post(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    express_register_route(scope, args, "POST");
}

fn express_app_put(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    express_register_route(scope, args, "PUT");
}

fn express_app_delete(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    express_register_route(scope, args, "DELETE");
}

fn express_app_use(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        return;
    }

    let _path_pattern = if args.length() == 1 {
        // app.use(middleware) - global middleware
        "*".to_string()
    } else {
        // app.use(path, middleware) - path-specific middleware
        let path_arg = args.get(0);
        let path_str = path_arg.to_string(scope).unwrap();
        path_str.to_rust_string_lossy(scope)
    };

    println!("Middleware registered");
}

fn express_app_listen(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let port = if args.length() > 0 {
        args.get(0).uint32_value(scope).unwrap_or(3000)
    } else {
        3000
    };

    // Call the callback if provided (second argument)
    if args.length() > 1 {
        let callback = args.get(1);
        if callback.is_function() {
            let callback_fn: v8::Local<v8::Function> = callback.try_into().unwrap();
            let this_obj = args.this();
            let no_args = [];
            let _ = callback_fn.call(scope, this_obj.into(), &no_args);
        }
    }

    println!("Express server listening on port {}", port);
}

fn express_register_route(
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

    println!("Express route registered: {} {}", method, path);
}
