#![allow(dead_code)]
use serde_json::Value;
use std::collections::HashMap;
use v8;

pub fn create_request_object<'a>(
    scope: &mut v8::HandleScope<'a>,
    method: &str,
    path: &str,
    query: HashMap<String, String>,
    params: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: &str,
) -> v8::Local<'a, v8::Object> {
    let req_obj = v8::Object::new(scope);

    // req.method
    let method_key = v8::String::new(scope, "method").unwrap();
    let method_val = v8::String::new(scope, method).unwrap();
    req_obj.set(scope, method_key.into(), method_val.into());

    // req.path
    let path_key = v8::String::new(scope, "path").unwrap();
    let path_val = v8::String::new(scope, path).unwrap();
    req_obj.set(scope, path_key.into(), path_val.into());

    // req.url
    let url_key = v8::String::new(scope, "url").unwrap();
    let url_val = v8::String::new(scope, path).unwrap();
    req_obj.set(scope, url_key.into(), url_val.into());

    // req.params
    let params_key = v8::String::new(scope, "params").unwrap();
    let params_obj = v8::Object::new(scope);
    for (key, value) in params {
        let param_key = v8::String::new(scope, &key).unwrap();
        let param_val = v8::String::new(scope, &value).unwrap();
        params_obj.set(scope, param_key.into(), param_val.into());
    }
    req_obj.set(scope, params_key.into(), params_obj.into());

    // req.query
    let query_key = v8::String::new(scope, "query").unwrap();
    let query_obj = v8::Object::new(scope);
    for (key, value) in query {
        let q_key = v8::String::new(scope, &key).unwrap();
        let q_val = v8::String::new(scope, &value).unwrap();
        query_obj.set(scope, q_key.into(), q_val.into());
    }
    req_obj.set(scope, query_key.into(), query_obj.into());

    // req.headers
    let headers_key = v8::String::new(scope, "headers").unwrap();
    let headers_obj = v8::Object::new(scope);
    for (key, value) in headers {
        let h_key = v8::String::new(scope, &key.to_lowercase()).unwrap();
        let h_val = v8::String::new(scope, &value).unwrap();
        headers_obj.set(scope, h_key.into(), h_val.into());
    }
    req_obj.set(scope, headers_key.into(), headers_obj.into());

    // req.body (parse JSON if possible)
    let body_key = v8::String::new(scope, "body").unwrap();
    if !body.is_empty() {
        if let Ok(json_value) = serde_json::from_str::<Value>(body) {
            let body_obj = json_to_v8(scope, &json_value);
            req_obj.set(scope, body_key.into(), body_obj);
        } else {
            let body_val = v8::String::new(scope, body).unwrap();
            req_obj.set(scope, body_key.into(), body_val.into());
        }
    } else {
        let empty_obj = v8::Object::new(scope);
        req_obj.set(scope, body_key.into(), empty_obj.into());
    }

    // req.get() method
    let get_key = v8::String::new(scope, "get").unwrap();
    let get_template = v8::FunctionTemplate::new(scope, req_get_header);
    let get_fn = get_template.get_function(scope).unwrap();
    req_obj.set(scope, get_key.into(), get_fn.into());

    req_obj
}

pub fn create_response_object<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Object> {
    let res_obj = v8::Object::new(scope);

    // res.status()
    let status_key = v8::String::new(scope, "status").unwrap();
    let status_template = v8::FunctionTemplate::new(scope, res_status);
    let status_fn = status_template.get_function(scope).unwrap();
    res_obj.set(scope, status_key.into(), status_fn.into());

    // res.json()
    let json_key = v8::String::new(scope, "json").unwrap();
    let json_template = v8::FunctionTemplate::new(scope, res_json);
    let json_fn = json_template.get_function(scope).unwrap();
    res_obj.set(scope, json_key.into(), json_fn.into());

    // res.send()
    let send_key = v8::String::new(scope, "send").unwrap();
    let send_template = v8::FunctionTemplate::new(scope, res_send);
    let send_fn = send_template.get_function(scope).unwrap();
    res_obj.set(scope, send_key.into(), send_fn.into());

    // res.set() / res.header()
    let set_key = v8::String::new(scope, "set").unwrap();
    let set_template = v8::FunctionTemplate::new(scope, res_set_header);
    let set_fn = set_template.get_function(scope).unwrap();
    res_obj.set(scope, set_key.into(), set_fn.into());

    let header_key = v8::String::new(scope, "header").unwrap();
    res_obj.set(scope, header_key.into(), set_fn.into());

    // res.end()
    let end_key = v8::String::new(scope, "end").unwrap();
    let end_template = v8::FunctionTemplate::new(scope, res_end);
    let end_fn = end_template.get_function(scope).unwrap();
    res_obj.set(scope, end_key.into(), end_fn.into());

    // Internal properties for tracking response state
    let status_code_key = v8::String::new(scope, "_statusCode").unwrap();
    let status_code_val = v8::Integer::new(scope, 200);
    res_obj.set(scope, status_code_key.into(), status_code_val.into());

    let headers_key = v8::String::new(scope, "_headers").unwrap();
    let headers_obj = v8::Object::new(scope);
    res_obj.set(scope, headers_key.into(), headers_obj.into());

    let finished_key = v8::String::new(scope, "_finished").unwrap();
    let finished_val = v8::Boolean::new(scope, false);
    res_obj.set(scope, finished_key.into(), finished_val.into());

    res_obj
}

// Request methods
fn req_get_header(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        rv.set(v8::undefined(scope).into());
        return;
    }

    let header_name_arg = args.get(0);
    let header_name_str = header_name_arg.to_string(scope).unwrap();
    let header_name = header_name_str.to_rust_string_lossy(scope).to_lowercase();

    // Get the request object (this)
    let this = args.this();
    let headers_key = v8::String::new(scope, "headers").unwrap();

    if let Some(headers_obj) = this.get(scope, headers_key.into()) {
        if let Ok(headers_obj) = headers_obj.try_into() {
            let headers_obj: v8::Local<v8::Object> = headers_obj;
            let header_key = v8::String::new(scope, &header_name).unwrap();
            if let Some(header_value) = headers_obj.get(scope, header_key.into()) {
                rv.set(header_value);
                return;
            }
        }
    }

    rv.set(v8::undefined(scope).into());
}

// Response methods
fn res_status(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let this = args.this();

    if args.length() > 0 {
        let status_arg = args.get(0);
        let status_code = status_arg.uint32_value(scope).unwrap_or(200);

        let status_key = v8::String::new(scope, "_statusCode").unwrap();
        let status_val = v8::Integer::new(scope, status_code as i32);
        this.set(scope, status_key.into(), status_val.into());
    }

    // Return this for chaining
    rv.set(this.into());
}

fn res_json(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let this = args.this();

    if args.length() > 0 {
        let data = args.get(0);

        // Set Content-Type to application/json
        let headers_key = v8::String::new(scope, "_headers").unwrap();
        if let Some(headers_obj) = this.get(scope, headers_key.into()) {
            if let Ok(headers_obj) = headers_obj.try_into() {
                let headers_obj: v8::Local<v8::Object> = headers_obj;
                let content_type_key = v8::String::new(scope, "content-type").unwrap();
                let content_type_val = v8::String::new(scope, "application/json").unwrap();
                headers_obj.set(scope, content_type_key.into(), content_type_val.into());
            }
        }

        // Convert data to JSON string and store
        let json_str = if data.is_object() && !data.is_string() {
            // Try to stringify object
            let global = scope.get_current_context().global(scope);
            let json_key = v8::String::new(scope, "JSON").unwrap();

            if let Some(json_obj) = global.get(scope, json_key.into()) {
                if let Ok(json_obj) = json_obj.try_into() {
                    let json_obj: v8::Local<v8::Object> = json_obj;
                    let stringify_key = v8::String::new(scope, "stringify").unwrap();

                    if let Some(stringify_fn) = json_obj.get(scope, stringify_key.into()) {
                        if let Ok(stringify_fn) = stringify_fn.try_into() {
                            let stringify_fn: v8::Local<v8::Function> = stringify_fn;
                            let args_array = [data];

                            if let Some(result) =
                                stringify_fn.call(scope, json_obj.into(), &args_array)
                            {
                                result.to_string(scope).unwrap().to_rust_string_lossy(scope)
                            } else {
                                data.to_string(scope).unwrap().to_rust_string_lossy(scope)
                            }
                        } else {
                            data.to_string(scope).unwrap().to_rust_string_lossy(scope)
                        }
                    } else {
                        data.to_string(scope).unwrap().to_rust_string_lossy(scope)
                    }
                } else {
                    data.to_string(scope).unwrap().to_rust_string_lossy(scope)
                }
            } else {
                data.to_string(scope).unwrap().to_rust_string_lossy(scope)
            }
        } else {
            data.to_string(scope).unwrap().to_rust_string_lossy(scope)
        };

        // Store the response body
        let body_key = v8::String::new(scope, "_body").unwrap();
        let body_val = v8::String::new(scope, &json_str).unwrap();
        this.set(scope, body_key.into(), body_val.into());

        // Mark as finished
        let finished_key = v8::String::new(scope, "_finished").unwrap();
        let finished_val = v8::Boolean::new(scope, true);
        this.set(scope, finished_key.into(), finished_val.into());

        println!("Response JSON: {}", json_str);
    }
}

fn res_send(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let this = args.this();

    if args.length() > 0 {
        let data = args.get(0);
        let data_str = data.to_string(scope).unwrap().to_rust_string_lossy(scope);

        // Store the response body
        let body_key = v8::String::new(scope, "_body").unwrap();
        let body_val = v8::String::new(scope, &data_str).unwrap();
        this.set(scope, body_key.into(), body_val.into());

        // Set default Content-Type if not set
        let headers_key = v8::String::new(scope, "_headers").unwrap();
        if let Some(headers_obj) = this.get(scope, headers_key.into()) {
            if let Ok(headers_obj) = headers_obj.try_into() {
                let headers_obj: v8::Local<v8::Object> = headers_obj;
                let content_type_key = v8::String::new(scope, "content-type").unwrap();

                if headers_obj.get(scope, content_type_key.into()).is_none() {
                    let content_type_val = v8::String::new(scope, "text/html").unwrap();
                    headers_obj.set(scope, content_type_key.into(), content_type_val.into());
                }
            }
        }

        // Mark as finished
        let finished_key = v8::String::new(scope, "_finished").unwrap();
        let finished_val = v8::Boolean::new(scope, true);
        this.set(scope, finished_key.into(), finished_val.into());

        println!("Response sent: {}", data_str);
    }
}

fn res_set_header(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let this = args.this();

    if args.length() >= 2 {
        let header_name_arg = args.get(0);
        let header_value_arg = args.get(1);

        let header_name = header_name_arg
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope)
            .to_lowercase();
        let header_value = header_value_arg
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope);

        let headers_key = v8::String::new(scope, "_headers").unwrap();
        if let Some(headers_obj) = this.get(scope, headers_key.into()) {
            if let Ok(headers_obj) = headers_obj.try_into() {
                let headers_obj: v8::Local<v8::Object> = headers_obj;
                let h_key = v8::String::new(scope, &header_name).unwrap();
                let h_val = v8::String::new(scope, &header_value).unwrap();
                headers_obj.set(scope, h_key.into(), h_val.into());
            }
        }
    }

    // Return this for chaining
    rv.set(this.into());
}

fn res_end(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue) {
    let this = args.this();

    if args.length() > 0 {
        let data = args.get(0);
        let data_str = data.to_string(scope).unwrap().to_rust_string_lossy(scope);

        let body_key = v8::String::new(scope, "_body").unwrap();
        let body_val = v8::String::new(scope, &data_str).unwrap();
        this.set(scope, body_key.into(), body_val.into());
    }

    // Mark as finished
    let finished_key = v8::String::new(scope, "_finished").unwrap();
    let finished_val = v8::Boolean::new(scope, true);
    this.set(scope, finished_key.into(), finished_val.into());
}

// Helper function to convert JSON to V8 value
fn json_to_v8<'a>(scope: &mut v8::HandleScope<'a>, value: &Value) -> v8::Local<'a, v8::Value> {
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
                let v8_item = json_to_v8(scope, item);
                v8_array.set_index(scope, i as u32, v8_item);
            }
            v8_array.into()
        }
        Value::Object(obj) => {
            let v8_obj = v8::Object::new(scope);
            for (key, val) in obj {
                let v8_key = v8::String::new(scope, key).unwrap();
                let v8_val = json_to_v8(scope, val);
                v8_obj.set(scope, v8_key.into(), v8_val);
            }
            v8_obj.into()
        }
    }
}

// Helper to extract response data
pub fn extract_response_data(
    scope: &mut v8::HandleScope,
    res_obj: v8::Local<v8::Object>,
) -> (u16, HashMap<String, String>, String) {
    let mut status_code = 200u16;
    let mut headers = HashMap::new();
    let mut body = String::new();

    // Get status code
    let status_key = v8::String::new(scope, "_statusCode").unwrap();
    if let Some(status_val) = res_obj.get(scope, status_key.into()) {
        status_code = status_val.uint32_value(scope).unwrap_or(200) as u16;
    }

    // Get headers
    let headers_key = v8::String::new(scope, "_headers").unwrap();
    if let Some(headers_obj) = res_obj.get(scope, headers_key.into()) {
        if let Ok(headers_obj) = headers_obj.try_into() {
            let _headers_obj: v8::Local<v8::Object> = headers_obj;
            // Note: In a real implementation, we'd iterate over header keys
            // For now, we'll add common headers manually
            headers.insert("content-type".to_string(), "application/json".to_string());
        }
    }

    // Get body
    let body_key = v8::String::new(scope, "_body").unwrap();
    if let Some(body_val) = res_obj.get(scope, body_key.into()) {
        body = body_val
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope);
    }

    (status_code, headers, body)
}
