use anyhow::Result;
use std::fs;
use std::path::Path;
use v8;

pub fn setup_filesystem(
    scope: &mut v8::HandleScope,
    context: v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Create fs object
    let fs_key = v8::String::new(scope, "fs").unwrap();
    let fs_obj = v8::Object::new(scope);

    // readFile
    let read_file_key = v8::String::new(scope, "readFile").unwrap();
    let read_file_tmpl = v8::FunctionTemplate::new(scope, read_file);
    let read_file_fn = read_file_tmpl.get_function(scope).unwrap();
    fs_obj.set(scope, read_file_key.into(), read_file_fn.into());

    // writeFile
    let write_file_key = v8::String::new(scope, "writeFile").unwrap();
    let write_file_tmpl = v8::FunctionTemplate::new(scope, write_file);
    let write_file_fn = write_file_tmpl.get_function(scope).unwrap();
    fs_obj.set(scope, write_file_key.into(), write_file_fn.into());

    // exists
    let exists_key = v8::String::new(scope, "exists").unwrap();
    let exists_tmpl = v8::FunctionTemplate::new(scope, exists);
    let exists_fn = exists_tmpl.get_function(scope).unwrap();
    fs_obj.set(scope, exists_key.into(), exists_fn.into());

    // mkdir
    let mkdir_key = v8::String::new(scope, "mkdir").unwrap();
    let mkdir_tmpl = v8::FunctionTemplate::new(scope, mkdir);
    let mkdir_fn = mkdir_tmpl.get_function(scope).unwrap();
    fs_obj.set(scope, mkdir_key.into(), mkdir_fn.into());

    global.set(scope, fs_key.into(), fs_obj.into());

    Ok(())
}

fn read_file(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        let error = v8::String::new(scope, "readFile requires a file path").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    match fs::read_to_string(&path) {
        Ok(content) => {
            let content_v8 = v8::String::new(scope, &content).unwrap();
            rv.set(content_v8.into());
        }
        Err(e) => {
            let error_msg = format!("Failed to read file {}: {}", path, e);
            let error = v8::String::new(scope, &error_msg).unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
        }
    }
}

fn write_file(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(scope, "writeFile requires path and content").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let content_arg = args.get(1);

    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    let content_str = content_arg.to_string(scope).unwrap();
    let content = content_str.to_rust_string_lossy(scope);

    match fs::write(&path, content) {
        Ok(_) => {
            let success = v8::Boolean::new(scope, true);
            rv.set(success.into());
        }
        Err(e) => {
            let error_msg = format!("Failed to write file {}: {}", path, e);
            let error = v8::String::new(scope, &error_msg).unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
        }
    }
}

fn exists(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        let error = v8::String::new(scope, "exists requires a file path").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    let exists = Path::new(&path).exists();
    let result = v8::Boolean::new(scope, exists);
    rv.set(result.into());
}

fn mkdir(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        let error = v8::String::new(scope, "mkdir requires a directory path").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    match fs::create_dir_all(&path) {
        Ok(_) => {
            let success = v8::Boolean::new(scope, true);
            rv.set(success.into());
        }
        Err(e) => {
            let error_msg = format!("Failed to create directory {}: {}", path, e);
            let error = v8::String::new(scope, &error_msg).unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
        }
    }
}
