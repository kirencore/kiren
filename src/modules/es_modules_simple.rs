use anyhow::Result;
use v8;

// Simplified ES Modules support
pub fn setup_es_modules(
    scope: &mut v8::HandleScope,
    context: v8::Local<v8::Context>,
) -> Result<()> {
    // Add import() function for dynamic imports
    let global = context.global(scope);
    let import_key = v8::String::new(scope, "import").unwrap();
    let import_template = v8::FunctionTemplate::new(scope, dynamic_import);
    let import_function = import_template.get_function(scope).unwrap();

    global.set(scope, import_key.into(), import_function.into());

    Ok(())
}

// Dynamic import implementation
fn dynamic_import(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "import() requires a module specifier").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let specifier_arg = args.get(0);
    let specifier_str = specifier_arg.to_string(scope).unwrap();
    let specifier = specifier_str.to_rust_string_lossy(scope);

    // Create a promise for async module loading
    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);

    // Try to load common modules
    match specifier.as_str() {
        "fs" | "node:fs" => {
            // Create a working fs module
            let fs_module = v8::Object::new(scope);

            // Add readFileSync function
            let read_file_key = v8::String::new(scope, "readFileSync").unwrap();
            let read_file_template = v8::FunctionTemplate::new(scope, fs_read_file_sync);
            let read_file_fn = read_file_template.get_function(scope).unwrap();
            fs_module.set(scope, read_file_key.into(), read_file_fn.into());

            // Add writeFileSync function
            let write_file_key = v8::String::new(scope, "writeFileSync").unwrap();
            let write_file_template = v8::FunctionTemplate::new(scope, fs_write_file_sync);
            let write_file_fn = write_file_template.get_function(scope).unwrap();
            fs_module.set(scope, write_file_key.into(), write_file_fn.into());

            // Add existsSync function
            let exists_key = v8::String::new(scope, "existsSync").unwrap();
            let exists_template = v8::FunctionTemplate::new(scope, fs_exists_sync);
            let exists_fn = exists_template.get_function(scope).unwrap();
            fs_module.set(scope, exists_key.into(), exists_fn.into());

            resolver.resolve(scope, fs_module.into());
        }
        "path" | "node:path" => {
            // Create a mock path module
            let path_module = v8::Object::new(scope);

            let join_key = v8::String::new(scope, "join").unwrap();
            let join_val =
                v8::String::new(scope, "function join(...args) { return args.join('/'); }")
                    .unwrap();
            path_module.set(scope, join_key.into(), join_val.into());

            let resolve_key = v8::String::new(scope, "resolve").unwrap();
            let resolve_val =
                v8::String::new(scope, "function resolve(path) { return path; }").unwrap();
            path_module.set(scope, resolve_key.into(), resolve_val.into());

            resolver.resolve(scope, path_module.into());
        }
        "http" | "node:http" => {
            // Create a mock http module
            let http_module = v8::Object::new(scope);

            let create_server_key = v8::String::new(scope, "createServer").unwrap();
            let create_server_val =
                v8::String::new(scope, "function createServer() { return {}; }").unwrap();
            http_module.set(scope, create_server_key.into(), create_server_val.into());

            resolver.resolve(scope, http_module.into());
        }
        _ => {
            // Try to load from filesystem
            let base_path =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let module_path = resolve_module_path(&base_path, &specifier);

            match module_path.and_then(|path| std::fs::read_to_string(path).map_err(|e| e.into())) {
                Ok(source) => {
                    // Create a module object with the source
                    let module_obj = v8::Object::new(scope);
                    let default_key = v8::String::new(scope, "default").unwrap();
                    let source_val = v8::String::new(scope, &source).unwrap();
                    module_obj.set(scope, default_key.into(), source_val.into());

                    resolver.resolve(scope, module_obj.into());
                }
                Err(_) => {
                    // Module not found - create empty module
                    let empty_module = v8::Object::new(scope);
                    let default_key = v8::String::new(scope, "default").unwrap();
                    let empty_obj = v8::Object::new(scope);
                    empty_module.set(scope, default_key.into(), empty_obj.into());

                    resolver.resolve(scope, empty_module.into());
                }
            }
        }
    }

    rv.set(promise.into());
}

// Helper function to resolve module paths
fn resolve_module_path(base_path: &std::path::Path, specifier: &str) -> Result<std::path::PathBuf> {
    use std::path::PathBuf;

    // Handle relative imports
    if specifier.starts_with("./") || specifier.starts_with("../") {
        let path = base_path.join(specifier);
        return resolve_file_extension(path);
    }

    // Handle absolute paths
    if specifier.starts_with('/') {
        let path = PathBuf::from(specifier);
        return resolve_file_extension(path);
    }

    // Handle bare imports
    let path = base_path.join(specifier);
    resolve_file_extension(path)
}

fn resolve_file_extension(mut path: std::path::PathBuf) -> Result<std::path::PathBuf> {
    // If path already has extension, use it
    if path.extension().is_some() && path.exists() {
        return Ok(path);
    }

    // Try common JavaScript extensions
    let extensions = [".js", ".mjs", ".ts"];
    for ext in &extensions {
        path.set_extension(&ext[1..]);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try index files
    if path.is_dir() {
        for ext in &extensions {
            let index_path = path.join(format!("index{}", ext));
            if index_path.exists() {
                return Ok(index_path);
            }
        }
    }

    Err(anyhow::anyhow!("Cannot resolve module: {}", path.display()))
}

// File system function implementations for dynamic imports
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
        Err(_e) => {
            let error_msg = v8::String::new(
                scope,
                &format!("ENOENT: no such file or directory, open '{}'", path),
            )
            .unwrap();
            let exception = v8::Exception::error(scope, error_msg);
            scope.throw_exception(exception);
        }
    }
}

fn fs_write_file_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(scope, "writeFileSync requires path and data").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    let data_arg = args.get(1);
    let data_str = data_arg.to_string(scope).unwrap();
    let data = data_str.to_rust_string_lossy(scope);

    match std::fs::write(&path, &data) {
        Ok(_) => {
            rv.set(v8::undefined(scope).into());
        }
        Err(e) => {
            let error_msg =
                v8::String::new(scope, &format!("Failed to write file '{}': {}", path, e)).unwrap();
            let exception = v8::Exception::error(scope, error_msg);
            scope.throw_exception(exception);
        }
    }
}

fn fs_exists_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    let exists = std::path::Path::new(&path).exists();
    rv.set(v8::Boolean::new(scope, exists).into());
}
