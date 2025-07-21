use base64::Engine;
use v8;

pub fn initialize_buffer_api(
    scope: &mut v8::HandleScope,
    global: v8::Local<v8::Object>,
) -> Result<(), anyhow::Error> {
    // Create Buffer constructor
    let buffer_name = v8::String::new(scope, "Buffer").unwrap();
    let buffer_template = v8::FunctionTemplate::new(scope, buffer_constructor);

    // Create constructor function first
    let buffer_fn = buffer_template.get_function(scope).unwrap();

    // Static methods
    let alloc_name = v8::String::new(scope, "alloc").unwrap();
    let alloc_fn = v8::Function::new(scope, buffer_alloc).unwrap();
    buffer_fn.set(scope, alloc_name.into(), alloc_fn.into());

    let allocunsafe_name = v8::String::new(scope, "allocUnsafe").unwrap();
    let allocunsafe_fn = v8::Function::new(scope, buffer_alloc_unsafe).unwrap();
    buffer_fn.set(scope, allocunsafe_name.into(), allocunsafe_fn.into());

    let from_name = v8::String::new(scope, "from").unwrap();
    let from_fn = v8::Function::new(scope, buffer_from).unwrap();
    buffer_fn.set(scope, from_name.into(), from_fn.into());

    let concat_name = v8::String::new(scope, "concat").unwrap();
    let concat_fn = v8::Function::new(scope, buffer_concat).unwrap();
    buffer_fn.set(scope, concat_name.into(), concat_fn.into());

    let isbuffer_name = v8::String::new(scope, "isBuffer").unwrap();
    let isbuffer_fn = v8::Function::new(scope, buffer_is_buffer).unwrap();
    buffer_fn.set(scope, isbuffer_name.into(), isbuffer_fn.into());

    // Prototype methods
    let proto = buffer_template.prototype_template(scope);

    let tostring_name = v8::String::new(scope, "toString").unwrap();
    let tostring_fn = v8::FunctionTemplate::new(scope, buffer_to_string);
    proto.set(tostring_name.into(), tostring_fn.into());

    let tojson_name = v8::String::new(scope, "toJSON").unwrap();
    let tojson_fn = v8::FunctionTemplate::new(scope, buffer_to_json);
    proto.set(tojson_name.into(), tojson_fn.into());

    let slice_name = v8::String::new(scope, "slice").unwrap();
    let slice_fn = v8::FunctionTemplate::new(scope, buffer_slice);
    proto.set(slice_name.into(), slice_fn.into());

    let write_name = v8::String::new(scope, "write").unwrap();
    let write_fn = v8::FunctionTemplate::new(scope, buffer_write);
    proto.set(write_name.into(), write_fn.into());

    // Set Buffer constructor in global scope
    global.set(scope, buffer_name.into(), buffer_fn.into());

    Ok(())
}

fn buffer_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() == 0 {
        let msg =
            v8::String::new(scope, "Buffer constructor requires at least one argument").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let arg0 = args.get(0);

    if arg0.is_number() {
        // Buffer(size) - deprecated but supported
        let size = arg0.number_value(scope).unwrap_or(0.0) as usize;
        if size > 0x3fffffff {
            let msg = v8::String::new(scope, "Array buffer allocation failed").unwrap();
            let exception = v8::Exception::range_error(scope, msg);
            scope.throw_exception(exception);
            return;
        }

        let array_buffer = v8::ArrayBuffer::new(scope, size);
        let uint8_array = v8::Uint8Array::new(scope, array_buffer, 0, size).unwrap();
        retval.set(uint8_array.into());
    } else if arg0.is_string() {
        // Buffer(string, encoding)
        let string_val = arg0.to_string(scope).unwrap();
        let encoding = if args.length() > 1 {
            args.get(1)
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope)
        } else {
            "utf8".to_string()
        };

        let data = match encoding.as_str() {
            "utf8" | "utf-8" => string_val.to_rust_string_lossy(scope).into_bytes(),
            "ascii" => {
                let s = string_val.to_rust_string_lossy(scope);
                s.chars()
                    .take_while(|&c| c.is_ascii())
                    .map(|c| c as u8)
                    .collect()
            }
            "base64" => {
                match base64::engine::general_purpose::STANDARD
                    .decode(string_val.to_rust_string_lossy(scope))
                {
                    Ok(decoded) => decoded,
                    Err(_) => {
                        let msg = v8::String::new(scope, "Invalid base64 string").unwrap();
                        let exception = v8::Exception::type_error(scope, msg);
                        scope.throw_exception(exception);
                        return;
                    }
                }
            }
            "hex" => {
                let s = string_val.to_rust_string_lossy(scope);
                match hex::decode(s) {
                    Ok(decoded) => decoded,
                    Err(_) => {
                        let msg = v8::String::new(scope, "Invalid hex string").unwrap();
                        let exception = v8::Exception::type_error(scope, msg);
                        scope.throw_exception(exception);
                        return;
                    }
                }
            }
            _ => {
                let msg =
                    v8::String::new(scope, &format!("Unknown encoding: {}", encoding)).unwrap();
                let exception = v8::Exception::type_error(scope, msg);
                scope.throw_exception(exception);
                return;
            }
        };

        let array_buffer = v8::ArrayBuffer::new(scope, data.len());
        let backing_store = array_buffer.get_backing_store();
        let slice = unsafe {
            std::slice::from_raw_parts_mut(
                backing_store.data().unwrap().as_ptr() as *mut u8,
                data.len(),
            )
        };
        slice.copy_from_slice(&data);

        let uint8_array = v8::Uint8Array::new(scope, array_buffer, 0, data.len()).unwrap();
        retval.set(uint8_array.into());
    } else if arg0.is_array() || arg0.is_uint8_array() {
        // Buffer(array) or Buffer(buffer)
        if arg0.is_uint8_array() {
            let typed_array = unsafe { v8::Local::<v8::Uint8Array>::cast(arg0) };
            let length = typed_array.length();
            let array_buffer = v8::ArrayBuffer::new(scope, length);

            let dest_backing_store = array_buffer.get_backing_store();
            let dest_slice = unsafe {
                std::slice::from_raw_parts_mut(
                    dest_backing_store.data().unwrap().as_ptr() as *mut u8,
                    length,
                )
            };

            let src_backing_store = typed_array.buffer(scope).unwrap().get_backing_store();
            let src_slice = unsafe {
                std::slice::from_raw_parts(
                    (src_backing_store.data().unwrap().as_ptr() as *const u8)
                        .add(typed_array.byte_offset()),
                    length,
                )
            };
            dest_slice.copy_from_slice(src_slice);

            let uint8_array = v8::Uint8Array::new(scope, array_buffer, 0, length).unwrap();
            retval.set(uint8_array.into());
        } else if arg0.is_array() {
            let array = unsafe { v8::Local::<v8::Array>::cast(arg0) };
            let length = array.length() as usize;
            let array_buffer = v8::ArrayBuffer::new(scope, length);

            let backing_store = array_buffer.get_backing_store();
            let slice = unsafe {
                std::slice::from_raw_parts_mut(
                    backing_store.data().unwrap().as_ptr() as *mut u8,
                    length,
                )
            };

            for i in 0..length {
                let val = array.get_index(scope, i as u32).unwrap();
                slice[i] = val.number_value(scope).unwrap_or(0.0) as u8;
            }

            let uint8_array = v8::Uint8Array::new(scope, array_buffer, 0, length).unwrap();
            retval.set(uint8_array.into());
        }
    }
}

fn buffer_alloc(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() == 0 {
        let msg = v8::String::new(scope, "Buffer.alloc() requires a size argument").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let size = args.get(0).number_value(scope).unwrap_or(0.0) as usize;
    if size > 0x3fffffff {
        let msg = v8::String::new(scope, "Array buffer allocation failed").unwrap();
        let exception = v8::Exception::range_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let fill_value = if args.length() > 1 {
        args.get(1).number_value(scope).unwrap_or(0.0) as u8
    } else {
        0
    };

    let array_buffer = v8::ArrayBuffer::new(scope, size);
    let backing_store = array_buffer.get_backing_store();
    let slice = unsafe {
        std::slice::from_raw_parts_mut(backing_store.data().unwrap().as_ptr() as *mut u8, size)
    };
    slice.fill(fill_value);

    let uint8_array = v8::Uint8Array::new(scope, array_buffer, 0, size).unwrap();
    retval.set(uint8_array.into());
}

fn buffer_alloc_unsafe(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() == 0 {
        let msg = v8::String::new(scope, "Buffer.allocUnsafe() requires a size argument").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let size = args.get(0).number_value(scope).unwrap_or(0.0) as usize;
    if size > 0x3fffffff {
        let msg = v8::String::new(scope, "Array buffer allocation failed").unwrap();
        let exception = v8::Exception::range_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let array_buffer = v8::ArrayBuffer::new(scope, size);
    let uint8_array = v8::Uint8Array::new(scope, array_buffer, 0, size).unwrap();
    retval.set(uint8_array.into());
}

fn buffer_from(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    retval: v8::ReturnValue,
) {
    if args.length() == 0 {
        let msg = v8::String::new(scope, "Buffer.from() requires an argument").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    // Delegate to constructor logic
    buffer_constructor(scope, args, retval);
}

fn buffer_concat(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() == 0 || !args.get(0).is_array() {
        let msg = v8::String::new(scope, "Buffer.concat() requires an array argument").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let array = unsafe { v8::Local::<v8::Array>::cast(args.get(0)) };
    let length = array.length();

    let mut total_size = 0;
    let mut buffers = Vec::new();

    // Calculate total size and collect buffers
    for i in 0..length {
        let val = array.get_index(scope, i).unwrap();
        if val.is_uint8_array() {
            let buffer = unsafe { v8::Local::<v8::Uint8Array>::cast(val) };
            total_size += buffer.length();
            buffers.push(buffer);
        }
    }

    let target_size = if args.length() > 1 {
        std::cmp::min(
            args.get(1).number_value(scope).unwrap_or(total_size as f64) as usize,
            total_size,
        )
    } else {
        total_size
    };

    let array_buffer = v8::ArrayBuffer::new(scope, target_size);
    let dest_backing_store = array_buffer.get_backing_store();
    let dest_slice = unsafe {
        std::slice::from_raw_parts_mut(
            dest_backing_store.data().unwrap().as_ptr() as *mut u8,
            target_size,
        )
    };

    let mut offset = 0;
    for buffer in buffers {
        if offset >= target_size {
            break;
        }

        let copy_size = std::cmp::min(buffer.length(), target_size - offset);
        let src_backing_store = buffer.buffer(scope).unwrap().get_backing_store();
        let src_slice = unsafe {
            std::slice::from_raw_parts(
                (src_backing_store.data().unwrap().as_ptr() as *const u8).add(buffer.byte_offset()),
                copy_size,
            )
        };
        dest_slice[offset..offset + copy_size].copy_from_slice(src_slice);
        offset += copy_size;
    }

    let uint8_array = v8::Uint8Array::new(scope, array_buffer, 0, target_size).unwrap();
    retval.set(uint8_array.into());
}

fn buffer_is_buffer(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let is_buffer = if args.length() > 0 {
        args.get(0).is_uint8_array()
    } else {
        false
    };

    retval.set(v8::Boolean::new(scope, is_buffer).into());
}

fn buffer_to_string(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    if this.is_uint8_array() {
        let buffer = unsafe { v8::Local::<v8::Uint8Array>::cast(this) };
        let encoding = if args.length() > 0 {
            args.get(0)
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope)
        } else {
            "utf8".to_string()
        };

        let start = if args.length() > 1 {
            args.get(1).number_value(scope).unwrap_or(0.0) as usize
        } else {
            0
        };

        let end = if args.length() > 2 {
            args.get(2)
                .number_value(scope)
                .unwrap_or(buffer.length() as f64) as usize
        } else {
            buffer.length()
        };

        let backing_store = buffer.buffer(scope).unwrap().get_backing_store();
        let slice = unsafe {
            std::slice::from_raw_parts(
                (backing_store.data().unwrap().as_ptr() as *const u8)
                    .add(buffer.byte_offset() + start),
                std::cmp::min(end - start, buffer.length() - start),
            )
        };

        let result_str = match encoding.as_str() {
            "utf8" | "utf-8" => String::from_utf8_lossy(slice).to_string(),
            "ascii" => slice.iter().map(|&b| b as char).collect(),
            "base64" => base64::engine::general_purpose::STANDARD.encode(slice),
            "hex" => hex::encode(slice),
            _ => String::from_utf8_lossy(slice).to_string(),
        };

        let result = v8::String::new(scope, &result_str).unwrap();
        retval.set(result.into());
    }
}

fn buffer_to_json(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    if this.is_uint8_array() {
        let buffer = unsafe { v8::Local::<v8::Uint8Array>::cast(this) };
        let obj = v8::Object::new(scope);

        let type_key = v8::String::new(scope, "type").unwrap();
        let type_val = v8::String::new(scope, "Buffer").unwrap();
        obj.set(scope, type_key.into(), type_val.into());

        let data_key = v8::String::new(scope, "data").unwrap();
        let data_array = v8::Array::new(scope, buffer.length() as i32);

        let backing_store = buffer.buffer(scope).unwrap().get_backing_store();
        let slice = unsafe {
            std::slice::from_raw_parts(
                (backing_store.data().unwrap().as_ptr() as *const u8).add(buffer.byte_offset()),
                buffer.length(),
            )
        };

        for (i, &byte) in slice.iter().enumerate() {
            let val = v8::Number::new(scope, byte as f64);
            data_array.set_index(scope, i as u32, val.into());
        }

        obj.set(scope, data_key.into(), data_array.into());
        retval.set(obj.into());
    }
}

fn buffer_slice(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    if this.is_uint8_array() {
        let buffer = unsafe { v8::Local::<v8::Uint8Array>::cast(this) };
        let start = if args.length() > 0 {
            args.get(0).number_value(scope).unwrap_or(0.0) as usize
        } else {
            0
        };

        let end = if args.length() > 1 {
            args.get(1)
                .number_value(scope)
                .unwrap_or(buffer.length() as f64) as usize
        } else {
            buffer.length()
        };

        let slice_length = if end > start { end - start } else { 0 };
        let array_buffer = v8::ArrayBuffer::new(scope, slice_length);

        let dest_backing_store = array_buffer.get_backing_store();
        let src_backing_store = buffer.buffer(scope).unwrap().get_backing_store();
        let dest_slice = unsafe {
            std::slice::from_raw_parts_mut(
                dest_backing_store.data().unwrap().as_ptr() as *mut u8,
                slice_length,
            )
        };
        let src_slice = unsafe {
            std::slice::from_raw_parts(
                (src_backing_store.data().unwrap().as_ptr() as *const u8)
                    .add(buffer.byte_offset() + start),
                slice_length,
            )
        };
        dest_slice.copy_from_slice(src_slice);

        let uint8_array = v8::Uint8Array::new(scope, array_buffer, 0, slice_length).unwrap();
        retval.set(uint8_array.into());
    }
}

fn buffer_write(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    if this.is_uint8_array() {
        let buffer = unsafe { v8::Local::<v8::Uint8Array>::cast(this) };
        if args.length() == 0 {
            retval.set(v8::Number::new(scope, 0.0).into());
            return;
        }

        let string_val = args.get(0).to_string(scope).unwrap();
        let offset = if args.length() > 1 {
            args.get(1).number_value(scope).unwrap_or(0.0) as usize
        } else {
            0
        };

        let length = if args.length() > 2 {
            args.get(2)
                .number_value(scope)
                .unwrap_or((buffer.length() - offset) as f64) as usize
        } else {
            buffer.length() - offset
        };

        let encoding = if args.length() > 3 {
            args.get(3)
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope)
        } else {
            "utf8".to_string()
        };

        let data = match encoding.as_str() {
            "utf8" | "utf-8" => string_val.to_rust_string_lossy(scope).into_bytes(),
            "ascii" => {
                let s = string_val.to_rust_string_lossy(scope);
                s.chars()
                    .take_while(|&c| c.is_ascii())
                    .map(|c| c as u8)
                    .collect()
            }
            "base64" => {
                match base64::engine::general_purpose::STANDARD
                    .decode(string_val.to_rust_string_lossy(scope))
                {
                    Ok(decoded) => decoded,
                    Err(_) => {
                        retval.set(v8::Number::new(scope, 0.0).into());
                        return;
                    }
                }
            }
            "hex" => {
                let s = string_val.to_rust_string_lossy(scope);
                match hex::decode(s) {
                    Ok(decoded) => decoded,
                    Err(_) => {
                        retval.set(v8::Number::new(scope, 0.0).into());
                        return;
                    }
                }
            }
            _ => string_val.to_rust_string_lossy(scope).into_bytes(),
        };

        let write_length = std::cmp::min(data.len(), length);

        let backing_store = buffer.buffer(scope).unwrap().get_backing_store();
        let slice = unsafe {
            std::slice::from_raw_parts_mut(
                (backing_store.data().unwrap().as_ptr() as *mut u8)
                    .add(buffer.byte_offset() + offset),
                write_length,
            )
        };
        slice.copy_from_slice(&data[..write_length]);

        retval.set(v8::Number::new(scope, write_length as f64).into());
    }
}
