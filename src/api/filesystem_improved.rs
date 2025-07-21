use anyhow::Result;
use base64::prelude::*;
use std::fs;
use std::path::Path;
use v8;

/// Improved filesystem operations with async support
pub fn setup_filesystem_improved(
    scope: &mut v8::HandleScope,
    context: v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Create fs object
    let fs_obj = v8::Object::new(scope);

    // Sync operations
    let read_file_sync_key = v8::String::new(scope, "readFileSync").unwrap();
    let read_file_sync_template = v8::FunctionTemplate::new(scope, fs_read_file_sync);
    let read_file_sync_function = read_file_sync_template.get_function(scope).unwrap();
    fs_obj.set(
        scope,
        read_file_sync_key.into(),
        read_file_sync_function.into(),
    );

    let write_file_sync_key = v8::String::new(scope, "writeFileSync").unwrap();
    let write_file_sync_template = v8::FunctionTemplate::new(scope, fs_write_file_sync);
    let write_file_sync_function = write_file_sync_template.get_function(scope).unwrap();
    fs_obj.set(
        scope,
        write_file_sync_key.into(),
        write_file_sync_function.into(),
    );

    let exists_sync_key = v8::String::new(scope, "existsSync").unwrap();
    let exists_sync_template = v8::FunctionTemplate::new(scope, fs_exists_sync);
    let exists_sync_function = exists_sync_template.get_function(scope).unwrap();
    fs_obj.set(scope, exists_sync_key.into(), exists_sync_function.into());

    let stat_sync_key = v8::String::new(scope, "statSync").unwrap();
    let stat_sync_template = v8::FunctionTemplate::new(scope, fs_stat_sync);
    let stat_sync_function = stat_sync_template.get_function(scope).unwrap();
    fs_obj.set(scope, stat_sync_key.into(), stat_sync_function.into());

    let readdir_sync_key = v8::String::new(scope, "readdirSync").unwrap();
    let readdir_sync_template = v8::FunctionTemplate::new(scope, fs_readdir_sync);
    let readdir_sync_function = readdir_sync_template.get_function(scope).unwrap();
    fs_obj.set(scope, readdir_sync_key.into(), readdir_sync_function.into());

    let mkdir_sync_key = v8::String::new(scope, "mkdirSync").unwrap();
    let mkdir_sync_template = v8::FunctionTemplate::new(scope, fs_mkdir_sync);
    let mkdir_sync_function = mkdir_sync_template.get_function(scope).unwrap();
    fs_obj.set(scope, mkdir_sync_key.into(), mkdir_sync_function.into());

    // Async operations (simplified - call callback immediately)
    let read_file_key = v8::String::new(scope, "readFile").unwrap();
    let read_file_template = v8::FunctionTemplate::new(scope, fs_read_file);
    let read_file_function = read_file_template.get_function(scope).unwrap();
    fs_obj.set(scope, read_file_key.into(), read_file_function.into());

    let write_file_key = v8::String::new(scope, "writeFile").unwrap();
    let write_file_template = v8::FunctionTemplate::new(scope, fs_write_file);
    let write_file_function = write_file_template.get_function(scope).unwrap();
    fs_obj.set(scope, write_file_key.into(), write_file_function.into());

    // Constants
    setup_fs_constants(scope, &fs_obj)?;

    // Set fs as global
    let fs_key = v8::String::new(scope, "fs").unwrap();
    global.set(scope, fs_key.into(), fs_obj.into());

    Ok(())
}

fn setup_fs_constants(scope: &mut v8::HandleScope, fs_obj: &v8::Local<v8::Object>) -> Result<()> {
    let constants_obj = v8::Object::new(scope);

    // File access constants
    let f_ok = v8::Number::new(scope, 0.0);
    let f_ok_key = v8::String::new(scope, "F_OK").unwrap();
    constants_obj.set(scope, f_ok_key.into(), f_ok.into());

    let r_ok = v8::Number::new(scope, 4.0);
    let r_ok_key = v8::String::new(scope, "R_OK").unwrap();
    constants_obj.set(scope, r_ok_key.into(), r_ok.into());

    let w_ok = v8::Number::new(scope, 2.0);
    let w_ok_key = v8::String::new(scope, "W_OK").unwrap();
    constants_obj.set(scope, w_ok_key.into(), w_ok.into());

    let x_ok = v8::Number::new(scope, 1.0);
    let x_ok_key = v8::String::new(scope, "X_OK").unwrap();
    constants_obj.set(scope, x_ok_key.into(), x_ok.into());

    let constants_key = v8::String::new(scope, "constants").unwrap();
    fs_obj.set(scope, constants_key.into(), constants_obj.into());

    Ok(())
}

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

    // Get encoding if provided
    let encoding = if args.length() > 1 {
        let enc_arg = args.get(1);
        if enc_arg.is_string() {
            let enc_str = enc_arg.to_string(scope).unwrap();
            Some(enc_str.to_rust_string_lossy(scope))
        } else if enc_arg.is_object() {
            // Handle options object { encoding: 'utf8' }
            let enc_obj = unsafe { v8::Local::<v8::Object>::cast(enc_arg) };
            let enc_key = v8::String::new(scope, "encoding").unwrap();
            if let Some(enc_val) = enc_obj.get(scope, enc_key.into()) {
                if enc_val.is_string() {
                    let enc_str = enc_val.to_string(scope).unwrap();
                    Some(enc_str.to_rust_string_lossy(scope))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    match fs::read(&path) {
        Ok(data) => {
            if let Some(enc) = encoding {
                match enc.as_str() {
                    "utf8" | "utf-8" => match String::from_utf8(data) {
                        Ok(content) => {
                            let content_str = v8::String::new(scope, &content).unwrap();
                            rv.set(content_str.into());
                        }
                        Err(_) => {
                            let error =
                                v8::String::new(scope, &format!("Invalid UTF-8 in file: {}", path))
                                    .unwrap();
                            let exception = v8::Exception::error(scope, error);
                            scope.throw_exception(exception);
                        }
                    },
                    "base64" => {
                        let encoded = BASE64_STANDARD.encode(&data);
                        let encoded_str = v8::String::new(scope, &encoded).unwrap();
                        rv.set(encoded_str.into());
                    }
                    "hex" => {
                        let encoded = hex::encode(&data);
                        let encoded_str = v8::String::new(scope, &encoded).unwrap();
                        rv.set(encoded_str.into());
                    }
                    _ => {
                        // Default to utf8 for unknown encodings
                        match String::from_utf8(data) {
                            Ok(content) => {
                                let content_str = v8::String::new(scope, &content).unwrap();
                                rv.set(content_str.into());
                            }
                            Err(_) => {
                                let error = v8::String::new(
                                    scope,
                                    &format!("Cannot decode file as {}: {}", enc, path),
                                )
                                .unwrap();
                                let exception = v8::Exception::error(scope, error);
                                scope.throw_exception(exception);
                            }
                        }
                    }
                }
            } else {
                // Return as Buffer-like object
                let buffer_obj = create_buffer_from_bytes(scope, &data);
                rv.set(buffer_obj.into());
            }
        }
        Err(e) => {
            let error =
                v8::String::new(scope, &format!("Cannot read file '{}': {}", path, e)).unwrap();
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

    // Get encoding if provided
    let encoding = if args.length() > 2 {
        let enc_arg = args.get(2);
        if enc_arg.is_string() {
            let enc_str = enc_arg.to_string(scope).unwrap();
            Some(enc_str.to_rust_string_lossy(scope))
        } else if enc_arg.is_object() {
            let enc_obj = unsafe { v8::Local::<v8::Object>::cast(enc_arg) };
            let enc_key = v8::String::new(scope, "encoding").unwrap();
            if let Some(enc_val) = enc_obj.get(scope, enc_key.into()) {
                if enc_val.is_string() {
                    let enc_str = enc_val.to_string(scope).unwrap();
                    Some(enc_str.to_rust_string_lossy(scope))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        Some("utf8".to_string())
    };

    let bytes_to_write = match encoding.as_deref() {
        Some("utf8") | Some("utf-8") => data.into_bytes(),
        Some("base64") => match BASE64_STANDARD.decode(&data) {
            Ok(decoded) => decoded,
            Err(_) => {
                let error = v8::String::new(scope, "Invalid base64 data").unwrap();
                let exception = v8::Exception::error(scope, error);
                scope.throw_exception(exception);
                return;
            }
        },
        Some("hex") => match hex::decode(&data) {
            Ok(decoded) => decoded,
            Err(_) => {
                let error = v8::String::new(scope, "Invalid hex data").unwrap();
                let exception = v8::Exception::error(scope, error);
                scope.throw_exception(exception);
                return;
            }
        },
        _ => data.into_bytes(),
    };

    if let Err(e) = fs::write(&path, bytes_to_write) {
        let error =
            v8::String::new(scope, &format!("Cannot write file '{}': {}", path, e)).unwrap();
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

    let exists = Path::new(&path).exists();
    let exists_val = v8::Boolean::new(scope, exists);
    rv.set(exists_val.into());
}

fn fs_stat_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "statSync requires a path").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    match fs::metadata(&path) {
        Ok(metadata) => {
            let stat_obj = v8::Object::new(scope);

            // isFile()
            let is_file_key = v8::String::new(scope, "isFile").unwrap();
            let is_file_template = v8::FunctionTemplate::new(scope, stat_is_file);
            let is_file_function = is_file_template.get_function(scope).unwrap();
            stat_obj.set(scope, is_file_key.into(), is_file_function.into());

            // isDirectory()
            let is_dir_key = v8::String::new(scope, "isDirectory").unwrap();
            let is_dir_template = v8::FunctionTemplate::new(scope, stat_is_directory);
            let is_dir_function = is_dir_template.get_function(scope).unwrap();
            stat_obj.set(scope, is_dir_key.into(), is_dir_function.into());

            // size
            let size_key = v8::String::new(scope, "size").unwrap();
            let size_value = v8::Number::new(scope, metadata.len() as f64);
            stat_obj.set(scope, size_key.into(), size_value.into());

            rv.set(stat_obj.into());
        }
        Err(e) => {
            let error = v8::String::new(scope, &format!("Cannot stat '{}': {}", path, e)).unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
        }
    }
}

fn fs_readdir_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "readdirSync requires a path").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    match fs::read_dir(&path) {
        Ok(entries) => {
            let entries_array = v8::Array::new(scope, 0);
            let mut index = 0;

            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        let name_str = v8::String::new(scope, name).unwrap();
                        entries_array.set_index(scope, index, name_str.into());
                        index += 1;
                    }
                }
            }

            rv.set(entries_array.into());
        }
        Err(e) => {
            let error = v8::String::new(scope, &format!("Cannot read directory '{}': {}", path, e))
                .unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
        }
    }
}

fn fs_mkdir_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "mkdirSync requires a path").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    // Check for recursive option
    let recursive = if args.length() > 1 {
        let options = args.get(1);
        if options.is_object() {
            let options_obj = unsafe { v8::Local::<v8::Object>::cast(options) };
            let recursive_key = v8::String::new(scope, "recursive").unwrap();
            if let Some(recursive_val) = options_obj.get(scope, recursive_key.into()) {
                recursive_val.boolean_value(scope)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    let result = if recursive {
        fs::create_dir_all(&path)
    } else {
        fs::create_dir(&path)
    };

    if let Err(e) = result {
        let error =
            v8::String::new(scope, &format!("Cannot create directory '{}': {}", path, e)).unwrap();
        let exception = v8::Exception::error(scope, error);
        scope.throw_exception(exception);
    }
}

// Async operations (simplified - execute immediately)
fn fs_read_file(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(scope, "readFile requires path and callback").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let callback = args.get(args.length() - 1); // Last argument is always callback

    if !callback.is_function() {
        let error = v8::String::new(scope, "readFile requires a callback function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    let callback_fn = unsafe { v8::Local::<v8::Function>::cast(callback) };
    let undefined = v8::undefined(scope);

    match fs::read_to_string(&path) {
        Ok(content) => {
            let content_str = v8::String::new(scope, &content).unwrap();
            let args = [v8::null(scope).into(), content_str.into()];
            callback_fn.call(scope, undefined.into(), &args);
        }
        Err(e) => {
            let error_str = v8::String::new(scope, &e.to_string()).unwrap();
            let args = [error_str.into()];
            callback_fn.call(scope, undefined.into(), &args);
        }
    }
}

fn fs_write_file(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 3 {
        let error = v8::String::new(scope, "writeFile requires path, data, and callback").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_arg = args.get(0);
    let data_arg = args.get(1);
    let callback = args.get(args.length() - 1);

    if !callback.is_function() {
        let error = v8::String::new(scope, "writeFile requires a callback function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let path_str = path_arg.to_string(scope).unwrap();
    let path = path_str.to_rust_string_lossy(scope);

    let data_str = data_arg.to_string(scope).unwrap();
    let data = data_str.to_rust_string_lossy(scope);

    let callback_fn = unsafe { v8::Local::<v8::Function>::cast(callback) };
    let undefined = v8::undefined(scope);

    match fs::write(&path, &data) {
        Ok(_) => {
            let args = [v8::null(scope).into()];
            callback_fn.call(scope, undefined.into(), &args);
        }
        Err(e) => {
            let error_str = v8::String::new(scope, &e.to_string()).unwrap();
            let args = [error_str.into()];
            callback_fn.call(scope, undefined.into(), &args);
        }
    }
}

fn create_buffer_from_bytes<'a>(
    scope: &'a mut v8::HandleScope,
    data: &[u8],
) -> v8::Local<'a, v8::Object> {
    let buffer_obj = v8::Object::new(scope);

    // Set length
    let length_key = v8::String::new(scope, "length").unwrap();
    let length_value = v8::Number::new(scope, data.len() as f64);
    buffer_obj.set(scope, length_key.into(), length_value.into());

    // Store as hex for compatibility
    let hex_string = hex::encode(data);
    let hex_key = v8::String::new(scope, "_hex").unwrap();
    let hex_value = v8::String::new(scope, &hex_string).unwrap();
    buffer_obj.set(scope, hex_key.into(), hex_value.into());

    // Add toString method
    let to_string_key = v8::String::new(scope, "toString").unwrap();
    let to_string_template = v8::FunctionTemplate::new(scope, buffer_to_string);
    let to_string_function = to_string_template.get_function(scope).unwrap();
    buffer_obj.set(scope, to_string_key.into(), to_string_function.into());

    buffer_obj
}

fn buffer_to_string(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let this = args.this();

    let encoding = if args.length() > 0 {
        let enc = args.get(0);
        let enc_str = enc.to_string(scope).unwrap();
        enc_str.to_rust_string_lossy(scope)
    } else {
        "utf8".to_string()
    };

    let hex_key = v8::String::new(scope, "_hex").unwrap();
    let hex_data = this.get(scope, hex_key.into()).unwrap();
    let hex_str = hex_data.to_string(scope).unwrap();
    let hex_content = hex_str.to_rust_string_lossy(scope);

    let result = match encoding.as_str() {
        "hex" => hex_content,
        "base64" => {
            if let Ok(bytes) = hex::decode(&hex_content) {
                BASE64_STANDARD.encode(bytes)
            } else {
                hex_content
            }
        }
        "utf8" | "utf-8" => {
            if let Ok(bytes) = hex::decode(&hex_content) {
                String::from_utf8_lossy(&bytes).to_string()
            } else {
                hex_content
            }
        }
        _ => {
            if let Ok(bytes) = hex::decode(&hex_content) {
                String::from_utf8_lossy(&bytes).to_string()
            } else {
                hex_content
            }
        }
    };

    let result_str = v8::String::new(scope, &result).unwrap();
    rv.set(result_str.into());
}

// Helper functions for stat object
fn stat_is_file(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // This is a simplified implementation - in reality we'd store metadata
    rv.set(v8::Boolean::new(scope, true).into());
}

fn stat_is_directory(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // This is a simplified implementation - in reality we'd store metadata
    rv.set(v8::Boolean::new(scope, false).into());
}
