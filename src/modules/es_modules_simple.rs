use crate::api::http;
use anyhow::Result;
use v8;

// Simplified ES Modules support
pub fn setup_es_modules(
    scope: &mut v8::HandleScope,
    context: v8::Local<v8::Context>,
) -> Result<()> {
    // Add __kiren_import() function for dynamic imports
    let global = context.global(scope);
    let import_key = v8::String::new(scope, "__kiren_import").unwrap();
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
        "fs" | "node:fs" | "kiren/fs" => {
            // Import gerçek FS API'sini - global api'dan alıp modülize et
            let fs_module = v8::Object::new(scope);

            // readFile fonksiyonu (global fs.readFile'dan)
            let read_file_key = v8::String::new(scope, "readFile").unwrap();
            let read_file_template = v8::FunctionTemplate::new(scope, fs_read_file_from_global);
            let read_file_fn = read_file_template.get_function(scope).unwrap();
            fs_module.set(scope, read_file_key.into(), read_file_fn.into());

            // writeFile fonksiyonu (global fs.writeFile'dan)
            let write_file_key = v8::String::new(scope, "writeFile").unwrap();
            let write_file_template = v8::FunctionTemplate::new(scope, fs_write_file_from_global);
            let write_file_fn = write_file_template.get_function(scope).unwrap();
            fs_module.set(scope, write_file_key.into(), write_file_fn.into());

            // exists fonksiyonu (global fs.exists'den)
            let exists_key = v8::String::new(scope, "exists").unwrap();
            let exists_template = v8::FunctionTemplate::new(scope, fs_exists_from_global);
            let exists_fn = exists_template.get_function(scope).unwrap();
            fs_module.set(scope, exists_key.into(), exists_fn.into());

            // mkdir fonksiyonu (global fs.mkdir'den)
            let mkdir_key = v8::String::new(scope, "mkdir").unwrap();
            let mkdir_template = v8::FunctionTemplate::new(scope, fs_mkdir_from_global);
            let mkdir_fn = mkdir_template.get_function(scope).unwrap();
            fs_module.set(scope, mkdir_key.into(), mkdir_fn.into());

            // Node.js uyumluluğu için Sync versiyonları da ekle
            let read_file_sync_key = v8::String::new(scope, "readFileSync").unwrap();
            fs_module.set(scope, read_file_sync_key.into(), read_file_fn.into());

            let write_file_sync_key = v8::String::new(scope, "writeFileSync").unwrap();
            fs_module.set(scope, write_file_sync_key.into(), write_file_fn.into());

            let exists_sync_key = v8::String::new(scope, "existsSync").unwrap();
            fs_module.set(scope, exists_sync_key.into(), exists_fn.into());

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
        "http" | "node:http" | "kiren/http" => {
            // Import gerçek HTTP API'sini - direct implementation
            let http_module = v8::Object::new(scope);

            // createServer fonksiyonunu direkt API'dan al
            let create_server_key = v8::String::new(scope, "createServer").unwrap();
            let create_server_tmpl =
                v8::FunctionTemplate::new(scope, crate::api::http::create_server);
            let create_server_fn = create_server_tmpl.get_function(scope).unwrap();
            http_module.set(scope, create_server_key.into(), create_server_fn.into());

            resolver.resolve(scope, http_module.into());
        }
        _ => {
            // Try to load from filesystem
            let base_path =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let module_path = resolve_module_path(&base_path, &specifier);

            match module_path.and_then(|path| std::fs::read_to_string(path).map_err(|e| e.into())) {
                Ok(source) => {
                    // Simplified module loading - just execute and return the result
                    let transformed_source = transform_module_to_commonjs(&source);

                    // Execute the transformed source
                    let source_str = v8::String::new(scope, &transformed_source).unwrap();
                    let script = v8::Script::compile(scope, source_str, None);

                    if let Some(script) = script {
                        if let Some(result) = script.run(scope) {
                            // Return the executed module result
                            resolver.resolve(scope, result);
                        } else {
                            let empty_module = v8::Object::new(scope);
                            resolver.resolve(scope, empty_module.into());
                        }
                    } else {
                        let empty_module = v8::Object::new(scope);
                        resolver.resolve(scope, empty_module.into());
                    }
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

// Transform ES module to CommonJS-compatible code
fn transform_module_to_commonjs(source: &str) -> String {
    let mut transformed = source.to_string();
    let mut exports = Vec::new();

    // Transform export function declarations
    if let Ok(export_fn_regex) = regex::Regex::new(r"export\s+function\s+(\w+)") {
        for caps in export_fn_regex.captures_iter(source) {
            exports.push(format!("{}: {}", &caps[1], &caps[1]));
        }
        transformed = export_fn_regex
            .replace_all(&transformed, "function $1")
            .to_string();
    }

    // Transform export const/let declarations
    if let Ok(export_const_regex) = regex::Regex::new(r"export\s+(?:const|let)\s+(\w+)") {
        for caps in export_const_regex.captures_iter(source) {
            exports.push(format!("{}: {}", &caps[1], &caps[1]));
        }
        transformed = export_const_regex
            .replace_all(&transformed, "const $1")
            .to_string();
    }

    // Transform export default
    if transformed.contains("export default") {
        transformed = transformed.replace("export default", "const __default_export =");
        exports.push("default: __default_export".to_string());
    }

    // Add module.exports at the end
    if !exports.is_empty() {
        transformed += &format!(
            "\n\n// Return module exports\nreturn {{ {} }};",
            exports.join(", ")
        );
        // Wrap in IIFE to capture return value
        transformed = format!("(function() {{\n{}\n}})()", transformed);
    } else {
        // If no exports, return empty object
        transformed += "\nreturn {};";
        transformed = format!("(function() {{\n{}\n}})()", transformed);
    }

    transformed
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

// HTTP modülü için createServer fonksiyonunu global API'den kullan
fn create_server_from_global(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Global context'den http.createServer'ı çağır
    let global = scope.get_current_context().global(scope);

    // http objesini al
    let http_key = v8::String::new(scope, "http").unwrap();
    if let Some(http_obj) = global.get(scope, http_key.into()) {
        if let Ok(http_obj) = http_obj.try_into() {
            let http_obj: v8::Local<v8::Object> = http_obj;

            // createServer fonksiyonunu al
            let create_server_key = v8::String::new(scope, "createServer").unwrap();
            if let Some(create_server_fn) = http_obj.get(scope, create_server_key.into()) {
                if let Ok(create_server_fn) = create_server_fn.try_into() {
                    let create_server_fn: v8::Local<v8::Function> = create_server_fn;

                    // Fonksiyonu çağır
                    let args = [];
                    if let Some(result) = create_server_fn.call(scope, http_obj.into(), &args) {
                        rv.set(result);
                        return;
                    }
                }
            }
        }
    }

    // Fallback: empty object if global API is not available
    let empty_obj = v8::Object::new(scope);
    rv.set(empty_obj.into());
}

// FS modülü için global API'lerden wrapper fonksiyonları
fn fs_read_file_from_global(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Global fs.readFile'ı çağır
    let global = scope.get_current_context().global(scope);
    let fs_key = v8::String::new(scope, "fs").unwrap();

    if let Some(fs_obj) = global.get(scope, fs_key.into()) {
        if let Ok(fs_obj) = fs_obj.try_into() {
            let fs_obj: v8::Local<v8::Object> = fs_obj;
            let read_file_key = v8::String::new(scope, "readFile").unwrap();

            if let Some(read_file_fn) = fs_obj.get(scope, read_file_key.into()) {
                if let Ok(read_file_fn) = read_file_fn.try_into() {
                    let read_file_fn: v8::Local<v8::Function> = read_file_fn;

                    // Argumentları forward et
                    let mut call_args = Vec::new();
                    for i in 0..args.length() {
                        call_args.push(args.get(i));
                    }

                    if let Some(result) = read_file_fn.call(scope, fs_obj.into(), &call_args) {
                        rv.set(result);
                        return;
                    }
                }
            }
        }
    }

    // Fallback
    let error = v8::String::new(scope, "FS module not available").unwrap();
    let exception = v8::Exception::error(scope, error);
    scope.throw_exception(exception);
}

fn fs_write_file_from_global(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let global = scope.get_current_context().global(scope);
    let fs_key = v8::String::new(scope, "fs").unwrap();

    if let Some(fs_obj) = global.get(scope, fs_key.into()) {
        if let Ok(fs_obj) = fs_obj.try_into() {
            let fs_obj: v8::Local<v8::Object> = fs_obj;
            let write_file_key = v8::String::new(scope, "writeFile").unwrap();

            if let Some(write_file_fn) = fs_obj.get(scope, write_file_key.into()) {
                if let Ok(write_file_fn) = write_file_fn.try_into() {
                    let write_file_fn: v8::Local<v8::Function> = write_file_fn;

                    let mut call_args = Vec::new();
                    for i in 0..args.length() {
                        call_args.push(args.get(i));
                    }

                    if let Some(result) = write_file_fn.call(scope, fs_obj.into(), &call_args) {
                        rv.set(result);
                        return;
                    }
                }
            }
        }
    }

    let error = v8::String::new(scope, "FS module not available").unwrap();
    let exception = v8::Exception::error(scope, error);
    scope.throw_exception(exception);
}

fn fs_exists_from_global(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let global = scope.get_current_context().global(scope);
    let fs_key = v8::String::new(scope, "fs").unwrap();

    if let Some(fs_obj) = global.get(scope, fs_key.into()) {
        if let Ok(fs_obj) = fs_obj.try_into() {
            let fs_obj: v8::Local<v8::Object> = fs_obj;
            let exists_key = v8::String::new(scope, "exists").unwrap();

            if let Some(exists_fn) = fs_obj.get(scope, exists_key.into()) {
                if let Ok(exists_fn) = exists_fn.try_into() {
                    let exists_fn: v8::Local<v8::Function> = exists_fn;

                    let mut call_args = Vec::new();
                    for i in 0..args.length() {
                        call_args.push(args.get(i));
                    }

                    if let Some(result) = exists_fn.call(scope, fs_obj.into(), &call_args) {
                        rv.set(result);
                        return;
                    }
                }
            }
        }
    }

    rv.set(v8::Boolean::new(scope, false).into());
}

fn fs_mkdir_from_global(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let global = scope.get_current_context().global(scope);
    let fs_key = v8::String::new(scope, "fs").unwrap();

    if let Some(fs_obj) = global.get(scope, fs_key.into()) {
        if let Ok(fs_obj) = fs_obj.try_into() {
            let fs_obj: v8::Local<v8::Object> = fs_obj;
            let mkdir_key = v8::String::new(scope, "mkdir").unwrap();

            if let Some(mkdir_fn) = fs_obj.get(scope, mkdir_key.into()) {
                if let Ok(mkdir_fn) = mkdir_fn.try_into() {
                    let mkdir_fn: v8::Local<v8::Function> = mkdir_fn;

                    let mut call_args = Vec::new();
                    for i in 0..args.length() {
                        call_args.push(args.get(i));
                    }

                    if let Some(result) = mkdir_fn.call(scope, fs_obj.into(), &call_args) {
                        rv.set(result);
                        return;
                    }
                }
            }
        }
    }

    let error = v8::String::new(scope, "FS module not available").unwrap();
    let exception = v8::Exception::error(scope, error);
    scope.throw_exception(exception);
}
