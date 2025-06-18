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
            return;
        }
        "path" => {
            let path_obj = v8::Object::new(scope);
            let join_key = v8::String::new(scope, "join").unwrap();
            let join_str =
                v8::String::new(scope, "function join() { return arguments[0]; }").unwrap();
            path_obj.set(scope, join_key.into(), join_str.into());
            rv.set(path_obj.into());
            return;
        }
        "http" => {
            let http_obj = v8::Object::new(scope);
            rv.set(http_obj.into());
            return;
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

    if let Err(_) = std::fs::write(&path, &content) {
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
