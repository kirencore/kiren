use anyhow::Result;
use v8;
use std::path::Path;
use std::fs;

pub fn setup_npm_compatibility(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
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

    // Handle built-in modules
    if let Some(builtin) = get_builtin_module(scope, &module_path) {
        rv.set(builtin);
        return;
    }

    // Try to load file-based modules
    match load_simple_module(scope, &module_path) {
        Ok(module_exports) => {
            rv.set(module_exports);
        }
        Err(e) => {
            let error_msg = format!("Cannot resolve module '{}': {}", module_path, e);
            let error_str = v8::String::new(scope, &error_msg).unwrap();
            let error = v8::Exception::error(scope, error_str);
            scope.throw_exception(error);
        }
    }
}

fn get_builtin_module<'a>(scope: &mut v8::HandleScope<'a>, module_name: &str) -> Option<v8::Local<'a, v8::Value>> {
    match module_name {
        "fs" => Some(create_simple_fs_module(scope)),
        "path" => Some(create_simple_path_module(scope)),
        "os" => Some(create_simple_os_module(scope)),
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

fn load_simple_module<'a>(scope: &mut v8::HandleScope<'a>, module_path: &str) -> Result<v8::Local<'a, v8::Value>> {
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

fn execute_module_content<'a>(scope: &mut v8::HandleScope<'a>, content: &str) -> Result<v8::Local<'a, v8::Value>> {
    // Create a simple module execution context
    let exports_obj = v8::Object::new(scope);
    
    // Wrap the module code in a simple IIFE
    let wrapped_code = format!(
        "(function() {{\n  var exports = {{}}, module = {{ exports: exports }};\n  {}\n  return module.exports;\n}})()",
        content
    );
    
    let source_string = v8::String::new(scope, &wrapped_code).unwrap();
    let script = v8::Script::compile(scope, source_string, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to compile module"))?;
    
    let result = script.run(scope)
        .ok_or_else(|| anyhow::anyhow!("Failed to run module"))?;
    
    Ok(result)
}

fn parse_json_module<'a>(scope: &mut v8::HandleScope<'a>, content: &str) -> Result<v8::Local<'a, v8::Value>> {
    let json_value: serde_json::Value = serde_json::from_str(content)?;
    Ok(json_to_v8_simple(scope, &json_value))
}

fn json_to_v8_simple<'a>(scope: &mut v8::HandleScope<'a>, value: &serde_json::Value) -> v8::Local<'a, v8::Value> {
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