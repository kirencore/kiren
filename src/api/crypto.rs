use anyhow::Result;
use base64::prelude::*;
use md5;
use rand::Rng;
use sha2::{Digest, Sha256, Sha512};
use v8;

/// Crypto module implementation for Node.js compatibility
pub fn setup_crypto(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
    let global = context.global(scope);

    // Create crypto object
    let crypto_obj = v8::Object::new(scope);

    // Add createHash function
    let create_hash_key = v8::String::new(scope, "createHash").unwrap();
    let create_hash_template = v8::FunctionTemplate::new(scope, crypto_create_hash);
    let create_hash_function = create_hash_template.get_function(scope).unwrap();
    crypto_obj.set(scope, create_hash_key.into(), create_hash_function.into());

    // Add randomBytes function
    let random_bytes_key = v8::String::new(scope, "randomBytes").unwrap();
    let random_bytes_template = v8::FunctionTemplate::new(scope, crypto_random_bytes);
    let random_bytes_function = random_bytes_template.get_function(scope).unwrap();
    crypto_obj.set(scope, random_bytes_key.into(), random_bytes_function.into());

    // Add randomUUID function
    let random_uuid_key = v8::String::new(scope, "randomUUID").unwrap();
    let random_uuid_template = v8::FunctionTemplate::new(scope, crypto_random_uuid);
    let random_uuid_function = random_uuid_template.get_function(scope).unwrap();
    crypto_obj.set(scope, random_uuid_key.into(), random_uuid_function.into());

    // Add createHmac function
    let create_hmac_key = v8::String::new(scope, "createHmac").unwrap();
    let create_hmac_template = v8::FunctionTemplate::new(scope, crypto_create_hmac);
    let create_hmac_function = create_hmac_template.get_function(scope).unwrap();
    crypto_obj.set(scope, create_hmac_key.into(), create_hmac_function.into());

    // Add pbkdf2 function
    let pbkdf2_key = v8::String::new(scope, "pbkdf2").unwrap();
    let pbkdf2_template = v8::FunctionTemplate::new(scope, crypto_pbkdf2);
    let pbkdf2_function = pbkdf2_template.get_function(scope).unwrap();
    crypto_obj.set(scope, pbkdf2_key.into(), pbkdf2_function.into());

    // Add pbkdf2Sync function
    let pbkdf2_sync_key = v8::String::new(scope, "pbkdf2Sync").unwrap();
    let pbkdf2_sync_template = v8::FunctionTemplate::new(scope, crypto_pbkdf2_sync);
    let pbkdf2_sync_function = pbkdf2_sync_template.get_function(scope).unwrap();
    crypto_obj.set(scope, pbkdf2_sync_key.into(), pbkdf2_sync_function.into());

    // Set crypto as global require target
    let crypto_key = v8::String::new(scope, "crypto").unwrap();
    global.set(scope, crypto_key.into(), crypto_obj.into());

    Ok(())
}

fn crypto_create_hash(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "createHash requires algorithm parameter").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let algorithm = args.get(0);
    let algorithm_str = algorithm.to_string(scope).unwrap();
    let algorithm_name = algorithm_str.to_rust_string_lossy(scope);

    // Create hash object
    let hash_obj = v8::Object::new(scope);

    // Store algorithm type
    let algo_key = v8::String::new(scope, "_algorithm").unwrap();
    let algo_value = v8::String::new(scope, &algorithm_name).unwrap();
    hash_obj.set(scope, algo_key.into(), algo_value.into());

    // Initialize data buffer
    let data_key = v8::String::new(scope, "_data").unwrap();
    let data_value = v8::String::new(scope, "").unwrap();
    hash_obj.set(scope, data_key.into(), data_value.into());

    // Add update method
    let update_key = v8::String::new(scope, "update").unwrap();
    let update_template = v8::FunctionTemplate::new(scope, hash_update);
    let update_function = update_template.get_function(scope).unwrap();
    hash_obj.set(scope, update_key.into(), update_function.into());

    // Add digest method
    let digest_key = v8::String::new(scope, "digest").unwrap();
    let digest_template = v8::FunctionTemplate::new(scope, hash_digest);
    let digest_function = digest_template.get_function(scope).unwrap();
    hash_obj.set(scope, digest_key.into(), digest_function.into());

    rv.set(hash_obj.into());
}

fn hash_update(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let this = args.this();

    if args.length() == 0 {
        rv.set(this.into());
        return;
    }

    let data = args.get(0);
    let data_str = data.to_string(scope).unwrap();
    let new_data = data_str.to_rust_string_lossy(scope);

    // Get existing data
    let data_key = v8::String::new(scope, "_data").unwrap();
    let existing_data = this.get(scope, data_key.into()).unwrap();
    let existing_str = existing_data.to_string(scope).unwrap();
    let existing = existing_str.to_rust_string_lossy(scope);

    // Append new data
    let combined = format!("{}{}", existing, new_data);
    let combined_value = v8::String::new(scope, &combined).unwrap();
    this.set(scope, data_key.into(), combined_value.into());

    // Return this for chaining
    rv.set(this.into());
}

fn hash_digest(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let this = args.this();

    // Get encoding (default to hex)
    let encoding = if args.length() > 0 {
        let enc = args.get(0);
        let enc_str = enc.to_string(scope).unwrap();
        enc_str.to_rust_string_lossy(scope)
    } else {
        "hex".to_string()
    };

    // Get algorithm
    let algo_key = v8::String::new(scope, "_algorithm").unwrap();
    let algorithm = this.get(scope, algo_key.into()).unwrap();
    let algo_str = algorithm.to_string(scope).unwrap();
    let algo_name = algo_str.to_rust_string_lossy(scope);

    // Get data
    let data_key = v8::String::new(scope, "_data").unwrap();
    let data = this.get(scope, data_key.into()).unwrap();
    let data_str = data.to_string(scope).unwrap();
    let data_content = data_str.to_rust_string_lossy(scope);

    // Compute hash
    let hash_result = match algo_name.as_str() {
        "md5" => {
            let digest = md5::compute(data_content.as_bytes());
            format!("{:x}", digest)
        }
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(data_content.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        "sha512" => {
            let mut hasher = Sha512::new();
            hasher.update(data_content.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        _ => {
            let error =
                v8::String::new(scope, &format!("Unsupported algorithm: {}", algo_name)).unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
            return;
        }
    };

    // Return based on encoding
    let result = match encoding.as_str() {
        "hex" => hash_result,
        "base64" => {
            // Convert hex to base64
            let bytes = hex::decode(&hash_result).unwrap_or_default();
            base64::prelude::BASE64_STANDARD.encode(bytes)
        }
        _ => hash_result, // Default to hex
    };

    let result_str = v8::String::new(scope, &result).unwrap();
    rv.set(result_str.into());
}

fn crypto_random_bytes(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "randomBytes requires size parameter").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let size = args.get(0);
    let size_num = size.int32_value(scope).unwrap_or(0) as usize;

    if size_num == 0 || size_num > 65536 {
        let error = v8::String::new(scope, "Invalid size for randomBytes").unwrap();
        let exception = v8::Exception::range_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    // Generate random bytes
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..size_num).map(|_| rng.gen()).collect();

    // Create Buffer-like object
    let buffer_obj = v8::Object::new(scope);

    // Set length
    let length_key = v8::String::new(scope, "length").unwrap();
    let length_value = v8::Number::new(scope, size_num as f64);
    buffer_obj.set(scope, length_key.into(), length_value.into());

    // Set data as hex string for compatibility
    let hex_string = hex::encode(&random_bytes);
    let hex_key = v8::String::new(scope, "_hex").unwrap();
    let hex_value = v8::String::new(scope, &hex_string).unwrap();
    buffer_obj.set(scope, hex_key.into(), hex_value.into());

    // Add toString method
    let to_string_key = v8::String::new(scope, "toString").unwrap();
    let to_string_template = v8::FunctionTemplate::new(scope, random_bytes_to_string);
    let to_string_function = to_string_template.get_function(scope).unwrap();
    buffer_obj.set(scope, to_string_key.into(), to_string_function.into());

    rv.set(buffer_obj.into());
}

fn random_bytes_to_string(
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
        "hex".to_string()
    };

    let hex_key = v8::String::new(scope, "_hex").unwrap();
    let hex_data = this.get(scope, hex_key.into()).unwrap();
    let hex_str = hex_data.to_string(scope).unwrap();
    let hex_content = hex_str.to_rust_string_lossy(scope);

    let result = match encoding.as_str() {
        "hex" => hex_content,
        "base64" => {
            if let Ok(bytes) = hex::decode(&hex_content) {
                base64::prelude::BASE64_STANDARD.encode(bytes)
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
        _ => hex_content,
    };

    let result_str = v8::String::new(scope, &result).unwrap();
    rv.set(result_str.into());
}

fn crypto_random_uuid(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let uuid = uuid::Uuid::new_v4().to_string();
    let uuid_str = v8::String::new(scope, &uuid).unwrap();
    rv.set(uuid_str.into());
}

fn crypto_create_hmac(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error =
            v8::String::new(scope, "createHmac requires algorithm and key parameters").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let algorithm = args.get(0);
    let key = args.get(1);

    let algorithm_str = algorithm.to_string(scope).unwrap();
    let algorithm_name = algorithm_str.to_rust_string_lossy(scope);

    let key_str = key.to_string(scope).unwrap();
    let key_content = key_str.to_rust_string_lossy(scope);

    // Create HMAC object (simplified)
    let hmac_obj = v8::Object::new(scope);

    // Store algorithm and key
    let algo_key = v8::String::new(scope, "_algorithm").unwrap();
    let algo_value = v8::String::new(scope, &algorithm_name).unwrap();
    hmac_obj.set(scope, algo_key.into(), algo_value.into());

    let key_key = v8::String::new(scope, "_key").unwrap();
    let key_value = v8::String::new(scope, &key_content).unwrap();
    hmac_obj.set(scope, key_key.into(), key_value.into());

    let data_key = v8::String::new(scope, "_data").unwrap();
    let data_value = v8::String::new(scope, "").unwrap();
    hmac_obj.set(scope, data_key.into(), data_value.into());

    // Add update and digest methods (similar to hash)
    let update_key = v8::String::new(scope, "update").unwrap();
    let update_template = v8::FunctionTemplate::new(scope, hash_update);
    let update_function = update_template.get_function(scope).unwrap();
    hmac_obj.set(scope, update_key.into(), update_function.into());

    let digest_key = v8::String::new(scope, "digest").unwrap();
    let digest_template = v8::FunctionTemplate::new(scope, hmac_digest);
    let digest_function = digest_template.get_function(scope).unwrap();
    hmac_obj.set(scope, digest_key.into(), digest_function.into());

    rv.set(hmac_obj.into());
}

fn hmac_digest(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Simplified HMAC digest - in a real implementation, this would use proper HMAC
    // For now, just concatenate key and data and hash
    let this = args.this();

    let encoding = if args.length() > 0 {
        let enc = args.get(0);
        let enc_str = enc.to_string(scope).unwrap();
        enc_str.to_rust_string_lossy(scope)
    } else {
        "hex".to_string()
    };

    // Get key and data
    let key_key = v8::String::new(scope, "_key").unwrap();
    let key = this.get(scope, key_key.into()).unwrap();
    let key_str = key.to_string(scope).unwrap();
    let key_content = key_str.to_rust_string_lossy(scope);

    let data_key = v8::String::new(scope, "_data").unwrap();
    let data = this.get(scope, data_key.into()).unwrap();
    let data_str = data.to_string(scope).unwrap();
    let data_content = data_str.to_rust_string_lossy(scope);

    // Simple HMAC simulation (key + data)
    let combined = format!("{}{}", key_content, data_content);
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let hash_result = format!("{:x}", hasher.finalize());

    let result = match encoding.as_str() {
        "hex" => hash_result,
        "base64" => {
            let bytes = hex::decode(&hash_result).unwrap_or_default();
            base64::prelude::BASE64_STANDARD.encode(bytes)
        }
        _ => hash_result,
    };

    let result_str = v8::String::new(scope, &result).unwrap();
    rv.set(result_str.into());
}

fn crypto_pbkdf2(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    // Async PBKDF2 - simplified implementation
    if args.length() < 5 {
        let error = v8::String::new(
            scope,
            "pbkdf2 requires password, salt, iterations, keylen, and callback",
        )
        .unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    // For now, just call the callback with a simple result
    let callback = args.get(4);
    if callback.is_function() {
        let callback_fn = unsafe { v8::Local::<v8::Function>::cast(callback) };
        let undefined = v8::undefined(scope);

        // Simple derived key (in real implementation, use proper PBKDF2)
        let derived_key = v8::String::new(scope, "derived_key_placeholder").unwrap();
        let result_args = [v8::null(scope).into(), derived_key.into()];

        callback_fn.call(scope, undefined.into(), &result_args);
    }
}

fn crypto_pbkdf2_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Sync PBKDF2 - simplified implementation
    if args.length() < 4 {
        let error = v8::String::new(
            scope,
            "pbkdf2Sync requires password, salt, iterations, and keylen",
        )
        .unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    // Simple derived key (in real implementation, use proper PBKDF2)
    let derived_key = v8::String::new(scope, "derived_key_placeholder").unwrap();
    rv.set(derived_key.into());
}
