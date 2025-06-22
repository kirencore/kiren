use anyhow::Result;
use v8;
use std::path::{Path, PathBuf};
use std::fs;
use serde_json::{Value, Map};

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub main: Option<String>,
    pub module: Option<String>,
    pub types: Option<String>,
    pub dependencies: Map<String, Value>,
}

pub fn setup_npm_compatibility(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
    let global = context.global(scope);

    // require() function for CommonJS compatibility
    let require_key = v8::String::new(scope, "require").unwrap();
    let require_template = v8::FunctionTemplate::new(scope, require_function);
    let require_function_obj = require_template.get_function(scope).unwrap();
    global.set(scope, require_key.into(), require_function_obj.into());

    // __dirname and __filename globals
    let dirname_key = v8::String::new(scope, "__dirname").unwrap();
    let dirname_value = v8::String::new(scope, &std::env::current_dir().unwrap().to_string_lossy()).unwrap();
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

fn require_function(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::Exception::error(scope, v8::String::new(scope, "require() missing path argument").unwrap());
        scope.throw_exception(error);
        return;
    }

    let module_path_arg = args.get(0);
    let module_path_str = module_path_arg.to_string(scope).unwrap();
    let module_path = module_path_str.to_rust_string_lossy(scope);

    // Handle built-in modules
    if let Some(builtin) = resolve_builtin_module(scope, &module_path) {
        rv.set(builtin);
        return;
    }

    // Try to resolve and load the module
    match resolve_module(&module_path) {
        Ok(resolved_path) => {
            match load_module(scope, &resolved_path) {
                Ok(module_exports) => {
                    rv.set(module_exports);
                }
                Err(e) => {
                    let error_msg = format!("Failed to load module '{}': {}", module_path, e);
                    let error = v8::Exception::error(scope, v8::String::new(scope, &error_msg).unwrap());
                    scope.throw_exception(error);
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Cannot resolve module '{}': {}", module_path, e);
            let error = v8::Exception::error(scope, v8::String::new(scope, &error_msg).unwrap());
            scope.throw_exception(error);
        }
    }
}

fn resolve_builtin_module<'a>(scope: &mut v8::HandleScope<'a>, module_name: &str) -> Option<v8::Local<'a, v8::Value>> {
    match module_name {
        "fs" => Some(create_fs_module(scope)),
        "path" => Some(create_path_module(scope)),
        "url" => Some(create_url_module(scope)),
        "util" => Some(create_util_module(scope)),
        "os" => Some(create_os_module(scope)),
        "crypto" => Some(create_crypto_module(scope)),
        _ => None,
    }
}

fn create_fs_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let fs_obj = v8::Object::new(scope);
    
    // readFileSync
    let read_file_sync_key = v8::String::new(scope, "readFileSync").unwrap();
    let read_file_sync_template = v8::FunctionTemplate::new(scope, fs_read_file_sync);
    let read_file_sync_fn = read_file_sync_template.get_function(scope).unwrap();
    fs_obj.set(scope, read_file_sync_key.into(), read_file_sync_fn.into());
    
    // writeFileSync
    let write_file_sync_key = v8::String::new(scope, "writeFileSync").unwrap();
    let write_file_sync_template = v8::FunctionTemplate::new(scope, fs_write_file_sync);
    let write_file_sync_fn = write_file_sync_template.get_function(scope).unwrap();
    fs_obj.set(scope, write_file_sync_key.into(), write_file_sync_fn.into());
    
    // existsSync
    let exists_sync_key = v8::String::new(scope, "existsSync").unwrap();
    let exists_sync_template = v8::FunctionTemplate::new(scope, fs_exists_sync);
    let exists_sync_fn = exists_sync_template.get_function(scope).unwrap();
    fs_obj.set(scope, exists_sync_key.into(), exists_sync_fn.into());

    fs_obj.into()
}

fn create_path_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let path_obj = v8::Object::new(scope);
    
    // join
    let join_key = v8::String::new(scope, "join").unwrap();
    let join_template = v8::FunctionTemplate::new(scope, path_join);
    let join_fn = join_template.get_function(scope).unwrap();
    path_obj.set(scope, join_key.into(), join_fn.into());
    
    // resolve
    let resolve_key = v8::String::new(scope, "resolve").unwrap();
    let resolve_template = v8::FunctionTemplate::new(scope, path_resolve);
    let resolve_fn = resolve_template.get_function(scope).unwrap();
    path_obj.set(scope, resolve_key.into(), resolve_fn.into());
    
    // dirname
    let dirname_key = v8::String::new(scope, "dirname").unwrap();
    let dirname_template = v8::FunctionTemplate::new(scope, path_dirname);
    let dirname_fn = dirname_template.get_function(scope).unwrap();
    path_obj.set(scope, dirname_key.into(), dirname_fn.into());
    
    // basename
    let basename_key = v8::String::new(scope, "basename").unwrap();
    let basename_template = v8::FunctionTemplate::new(scope, path_basename);
    let basename_fn = basename_template.get_function(scope).unwrap();
    path_obj.set(scope, basename_key.into(), basename_fn.into());

    path_obj.into()
}

fn create_url_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let url_obj = v8::Object::new(scope);
    
    // Simple URL constructor
    let url_key = v8::String::new(scope, "URL").unwrap();
    let url_template = v8::FunctionTemplate::new(scope, url_constructor);
    let url_fn = url_template.get_function(scope).unwrap();
    url_obj.set(scope, url_key.into(), url_fn.into());

    url_obj.into()
}

fn create_util_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let util_obj = v8::Object::new(scope);
    
    // inspect
    let inspect_key = v8::String::new(scope, "inspect").unwrap();
    let inspect_template = v8::FunctionTemplate::new(scope, util_inspect);
    let inspect_fn = inspect_template.get_function(scope).unwrap();
    util_obj.set(scope, inspect_key.into(), inspect_fn.into());

    util_obj.into()
}

fn create_os_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let os_obj = v8::Object::new(scope);
    
    // platform
    let platform_key = v8::String::new(scope, "platform").unwrap();
    let platform_template = v8::FunctionTemplate::new(scope, os_platform);
    let platform_fn = platform_template.get_function(scope).unwrap();
    os_obj.set(scope, platform_key.into(), platform_fn.into());
    
    // homedir
    let homedir_key = v8::String::new(scope, "homedir").unwrap();
    let homedir_template = v8::FunctionTemplate::new(scope, os_homedir);
    let homedir_fn = homedir_template.get_function(scope).unwrap();
    os_obj.set(scope, homedir_key.into(), homedir_fn.into());

    os_obj.into()
}

fn create_crypto_module<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    let crypto_obj = v8::Object::new(scope);
    
    // randomBytes (simplified)
    let random_bytes_key = v8::String::new(scope, "randomBytes").unwrap();
    let random_bytes_template = v8::FunctionTemplate::new(scope, crypto_random_bytes);
    let random_bytes_fn = random_bytes_template.get_function(scope).unwrap();
    crypto_obj.set(scope, random_bytes_key.into(), random_bytes_fn.into());

    crypto_obj.into()
}

fn resolve_module(module_path: &str) -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    
    // Handle relative paths
    if module_path.starts_with("./") || module_path.starts_with("../") {
        let resolved = current_dir.join(module_path);
        return try_resolve_file(&resolved);
    }
    
    // Handle absolute paths
    if module_path.starts_with("/") {
        let path = PathBuf::from(module_path);
        return try_resolve_file(&path);
    }
    
    // Handle node_modules resolution
    let node_modules_path = current_dir.join("node_modules").join(module_path);
    if let Ok(resolved) = try_resolve_package(&node_modules_path) {
        return Ok(resolved);
    }
    
    // Fallback: try as direct file
    try_resolve_file(&PathBuf::from(module_path))
}

fn try_resolve_file(base_path: &Path) -> Result<PathBuf> {
    // Try exact path
    if base_path.exists() && base_path.is_file() {
        return Ok(base_path.to_path_buf());
    }
    
    // Try with .js extension
    let js_path = base_path.with_extension("js");
    if js_path.exists() {
        return Ok(js_path);
    }
    
    // Try with .json extension
    let json_path = base_path.with_extension("json");
    if json_path.exists() {
        return Ok(json_path);
    }
    
    // Try index.js in directory
    if base_path.is_dir() {
        let index_path = base_path.join("index.js");
        if index_path.exists() {
            return Ok(index_path);
        }
    }
    
    Err(anyhow::anyhow!("Module not found: {}", base_path.display()))
}

fn try_resolve_package(package_path: &Path) -> Result<PathBuf> {
    let package_json_path = package_path.join("package.json");
    
    if package_json_path.exists() {
        let package_json_content = fs::read_to_string(&package_json_path)?;
        let package_json: Value = serde_json::from_str(&package_json_content)?;
        
        // Try main field
        if let Some(main) = package_json.get("main").and_then(|v| v.as_str()) {
            let main_path = package_path.join(main);
            if let Ok(resolved) = try_resolve_file(&main_path) {
                return Ok(resolved);
            }
        }
        
        // Try module field (ES modules)
        if let Some(module) = package_json.get("module").and_then(|v| v.as_str()) {
            let module_path = package_path.join(module);
            if let Ok(resolved) = try_resolve_file(&module_path) {
                return Ok(resolved);
            }
        }
    }
    
    // Fallback to index.js
    try_resolve_file(&package_path.join("index.js"))
}

fn load_module<'a>(scope: &mut v8::HandleScope<'a>, module_path: &Path) -> Result<v8::Local<'a, v8::Value>> {
    let content = fs::read_to_string(module_path)?;
    
    // Handle JSON files
    if module_path.extension().and_then(|ext| ext.to_str()) == Some("json") {
        let json_value: Value = serde_json::from_str(&content)?;
        return Ok(json_to_v8_value(scope, &json_value));
    }
    
    // Create module context
    let module_obj = v8::Object::new(scope);
    let exports_obj = v8::Object::new(scope);
    
    let exports_key = v8::String::new(scope, "exports").unwrap();
    module_obj.set(scope, exports_key.into(), exports_obj.into());
    
    // Wrap the module code
    let wrapped_code = format!(
        "(function(module, exports, require, __filename, __dirname) {{\n{}\n}})",
        content
    );
    
    // Compile and run the module
    let source_string = v8::String::new(scope, &wrapped_code).unwrap();
    let script = v8::Script::compile(scope, source_string, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to compile module"))?;
    
    let function = script.run(scope)
        .ok_or_else(|| anyhow::anyhow!("Failed to run module wrapper"))?;
    
    let function = v8::Local::<v8::Function>::try_from(function)
        .map_err(|_| anyhow::anyhow!("Module wrapper is not a function"))?;
    
    // Prepare arguments
    let filename = v8::String::new(scope, &module_path.to_string_lossy()).unwrap();
    let dirname = v8::String::new(scope, &module_path.parent().unwrap_or(Path::new(".")).to_string_lossy()).unwrap();
    
    // Create require function for this module
    let require_template = v8::FunctionTemplate::new(scope, require_function);
    let require_fn = require_template.get_function(scope).unwrap();
    
    let args = [
        module_obj.into(),
        exports_obj.into(),
        require_fn.into(),
        filename.into(),
        dirname.into(),
    ];
    
    // Execute the module function
    let undefined = v8::undefined(scope);
    function.call(scope, undefined.into(), &args);
    
    // Return module.exports
    let exports_result = module_obj.get(scope, exports_key.into())
        .ok_or_else(|| anyhow::anyhow!("Failed to get module exports"))?;
    
    Ok(exports_result)
}

fn json_to_v8_value<'a>(scope: &mut v8::HandleScope<'a>, value: &Value) -> v8::Local<'a, v8::Value> {
    match value {
        Value::Null => v8::null(scope).into(),
        Value::Bool(b) => v8::Boolean::new(scope, *b).into(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                v8::Integer::new(scope, i as i32).into()
            } else if let Some(f) = n.as_f64() {
                v8::Number::new(scope, f).into()
            } else {
                v8::null(scope).into()
            }
        }
        Value::String(s) => v8::String::new(scope, s).unwrap().into(),
        Value::Array(arr) => {
            let v8_array = v8::Array::new(scope, arr.len() as i32);
            for (i, item) in arr.iter().enumerate() {
                let v8_item = json_to_v8_value(scope, item);
                v8_array.set_index(scope, i as u32, v8_item);
            }
            v8_array.into()
        }
        Value::Object(obj) => {
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

// Builtin module implementations
fn fs_read_file_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::Exception::error(scope, v8::String::new(scope, "readFileSync requires a path").unwrap());
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
            let error = v8::Exception::error(scope, v8::String::new(scope, &error_msg).unwrap());
            scope.throw_exception(error);
        }
    }
}

fn fs_write_file_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::Exception::error(scope, v8::String::new(scope, "writeFileSync requires path and data").unwrap());
        scope.throw_exception(error);
        return;
    }
    
    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);
    
    let data_arg = args.get(1);
    let data_str = data_arg.to_string(scope).unwrap();
    let data = data_str.to_rust_string_lossy(scope);
    
    if let Err(e) = fs::write(&path, data) {
        let error_msg = format!("Failed to write file '{}': {}", path, e);
        let error = v8::Exception::error(scope, v8::String::new(scope, &error_msg).unwrap());
        scope.throw_exception(error);
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
    
    let exists = Path::new(&path).exists();
    rv.set(v8::Boolean::new(scope, exists).into());
}

fn path_join(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut path_buf = PathBuf::new();
    
    for i in 0..args.length() {
        let arg = args.get(i);
        let str_val = arg.to_string(scope).unwrap();
        let part = str_val.to_rust_string_lossy(scope);
        path_buf.push(part);
    }
    
    let result = v8::String::new(scope, &path_buf.to_string_lossy()).unwrap();
    rv.set(result.into());
}

fn path_resolve(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut path_buf = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    for i in 0..args.length() {
        let arg = args.get(i);
        let str_val = arg.to_string(scope).unwrap();
        let part = str_val.to_rust_string_lossy(scope);
        
        if Path::new(&part).is_absolute() {
            path_buf = PathBuf::from(part);
        } else {
            path_buf.push(part);
        }
    }
    
    let result = v8::String::new(scope, &path_buf.to_string_lossy()).unwrap();
    rv.set(result.into());
}

fn path_dirname(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        rv.set(v8::String::new(scope, ".").unwrap().into());
        return;
    }
    
    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);
    
    let dirname = Path::new(&path).parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    
    let result = v8::String::new(scope, &dirname).unwrap();
    rv.set(result.into());
}

fn path_basename(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        rv.set(v8::String::new(scope, "").unwrap().into());
        return;
    }
    
    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);
    
    let basename = Path::new(&path).file_name()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "".to_string());
    
    let result = v8::String::new(scope, &basename).unwrap();
    rv.set(result.into());
}

fn url_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::Exception::error(scope, v8::String::new(scope, "URL constructor requires a URL string").unwrap());
        scope.throw_exception(error);
        return;
    }
    
    let url_arg = args.get(0);
    let url_str = url_arg.to_string(scope).unwrap();
    let url = url_str.to_rust_string_lossy(scope);
    
    // Simple URL object
    let url_obj = v8::Object::new(scope);
    
    let href_key = v8::String::new(scope, "href").unwrap();
    let href_val = v8::String::new(scope, &url).unwrap();
    url_obj.set(scope, href_key.into(), href_val.into());
    
    rv.set(url_obj.into());
}

fn util_inspect(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        rv.set(v8::String::new(scope, "undefined").unwrap().into());
        return;
    }
    
    let obj = args.get(0);
    let str_val = obj.to_string(scope).unwrap();
    rv.set(str_val.into());
}

fn os_platform(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let platform = std::env::consts::OS;
    let result = v8::String::new(scope, platform).unwrap();
    rv.set(result.into());
}

fn os_homedir(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let homedir = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());
    
    let result = v8::String::new(scope, &homedir).unwrap();
    rv.set(result.into());
}

fn crypto_random_bytes(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let size = if args.length() > 0 {
        args.get(0).uint32_value(scope).unwrap_or(16)
    } else {
        16
    };
    
    // Simple random bytes (not cryptographically secure)
    let bytes: Vec<u8> = (0..size).map(|_| rand::random::<u8>()).collect();
    let bytes_str = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    
    let result = v8::String::new(scope, &bytes_str).unwrap();
    rv.set(result.into());
}