use anyhow::Result;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::{interval, sleep};
use uuid::Uuid;
use v8;

static TIMERS: Lazy<DashMap<String, JoinHandle<()>>> = Lazy::new(|| DashMap::new());

// Store callback information safely without V8 handles
#[derive(Clone)]
struct CallbackInfo {
    callback_source: String,
    is_function: bool,
    function_source: Option<String>, // Store serialized function for execution
}

static CALLBACKS: Lazy<Arc<Mutex<DashMap<String, CallbackInfo>>>> =
    Lazy::new(|| Arc::new(Mutex::new(DashMap::new())));

// Message queue for timer callbacks
#[derive(Debug, Clone)]
struct TimerCallback {
    id: String,
    callback_source: String,
    is_function: bool,
    function_source: Option<String>,
}

static CALLBACK_QUEUE: Lazy<Arc<Mutex<Vec<TimerCallback>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

pub fn setup_timers(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
    let global = context.global(scope);

    let set_timeout_key = v8::String::new(scope, "setTimeout").unwrap();
    let set_timeout_tmpl = v8::FunctionTemplate::new(scope, set_timeout);
    let set_timeout_fn = set_timeout_tmpl.get_function(scope).unwrap();
    global.set(scope, set_timeout_key.into(), set_timeout_fn.into());

    let clear_timeout_key = v8::String::new(scope, "clearTimeout").unwrap();
    let clear_timeout_tmpl = v8::FunctionTemplate::new(scope, clear_timeout);
    let clear_timeout_fn = clear_timeout_tmpl.get_function(scope).unwrap();
    global.set(scope, clear_timeout_key.into(), clear_timeout_fn.into());

    let set_interval_key = v8::String::new(scope, "setInterval").unwrap();
    let set_interval_tmpl = v8::FunctionTemplate::new(scope, set_interval);
    let set_interval_fn = set_interval_tmpl.get_function(scope).unwrap();
    global.set(scope, set_interval_key.into(), set_interval_fn.into());

    let clear_interval_key = v8::String::new(scope, "clearInterval").unwrap();
    let clear_interval_tmpl = v8::FunctionTemplate::new(scope, clear_interval);
    let clear_interval_fn = clear_interval_tmpl.get_function(scope).unwrap();
    global.set(scope, clear_interval_key.into(), clear_interval_fn.into());

    Ok(())
}

// Function to process queued timer callbacks
pub fn process_timer_callbacks(scope: &mut v8::HandleScope) -> Result<()> {
    let mut callbacks_to_process = Vec::new();

    // Extract all queued callbacks
    {
        let mut queue = CALLBACK_QUEUE.lock().unwrap();
        callbacks_to_process.extend(queue.drain(..));
    }

    // Process each callback
    for callback in callbacks_to_process {
        if callback.is_function {
            // For function callbacks, execute the stored function source
            if let Some(ref function_source) = callback.function_source {
                let _ = execute_function_callback(scope, &callback, function_source);
            }
        } else {
            // For string callbacks, execute them in proper scope
            let _ = execute_string_callback(scope, &callback);
        }
    }

    Ok(())
}

// Execute string callback in proper scope
fn execute_string_callback(scope: &mut v8::HandleScope, callback: &TimerCallback) -> Result<()> {
    // Get the current context
    let context = scope.get_current_context();
    let scope = &mut v8::ContextScope::new(scope, context);

    // Create the source string
    let source_string = v8::String::new(scope, &callback.callback_source).unwrap();
    let filename = v8::String::new(scope, &format!("timer-{}", callback.id)).unwrap();

    // Create script origin for better error reporting
    let undefined_val = v8::undefined(scope);
    let origin = v8::ScriptOrigin::new(
        scope,
        filename.into(),
        0,                    // line_offset
        0,                    // column_offset
        false,                // is_shared_cross_origin
        0,                    // script_id
        undefined_val.into(), // source_map_url
        false,                // is_opaque
        false,                // is_wasm
        false,                // is_module
    );

    // Use TryCatch for better error handling
    let mut try_catch = v8::TryCatch::new(scope);

    match v8::Script::compile(&mut try_catch, source_string, Some(&origin)) {
        Some(script) => match script.run(&mut try_catch) {
            Some(_) => {
                // Callback executed successfully
            }
            None => {
                // Timer callback execution failed - silent failure for production
            }
        },
        None => {
            // Timer callback compilation failed - silent failure for production
        }
    }

    Ok(())
}

// Execute function callback in proper scope
fn execute_function_callback(
    scope: &mut v8::HandleScope,
    callback: &TimerCallback,
    function_source: &str,
) -> Result<()> {
    // Get the current context
    let context = scope.get_current_context();
    let scope = &mut v8::ContextScope::new(scope, context);

    // Create a wrapper that calls the function
    let wrapper_source = format!("({})()", function_source);
    let source_string = v8::String::new(scope, &wrapper_source).unwrap();
    let filename = v8::String::new(scope, &format!("timer-function-{}", callback.id)).unwrap();

    // Create script origin for better error reporting
    let undefined_val = v8::undefined(scope);
    let origin = v8::ScriptOrigin::new(
        scope,
        filename.into(),
        0,                    // line_offset
        0,                    // column_offset
        false,                // is_shared_cross_origin
        0,                    // script_id
        undefined_val.into(), // source_map_url
        false,                // is_opaque
        false,                // is_wasm
        false,                // is_module
    );

    // Use TryCatch for better error handling
    let mut try_catch = v8::TryCatch::new(scope);

    match v8::Script::compile(&mut try_catch, source_string, Some(&origin)) {
        Some(script) => match script.run(&mut try_catch) {
            Some(_) => {
                // Function callback executed successfully
            }
            None => {
                // Function callback execution failed - silent failure for production
            }
        },
        None => {
            // Function callback compilation failed - silent failure for production
        }
    }

    Ok(())
}

fn set_timeout(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }

    let callback = args.get(0);
    let delay = args.get(1);

    let delay_ms = delay.number_value(scope).unwrap_or(0.0) as u64;
    let timer_id = Uuid::new_v4().to_string();

    let timer_id_v8 = v8::String::new(scope, &timer_id).unwrap();
    rv.set(timer_id_v8.into());

    // Store callback info safely
    let callback_info = if callback.is_function() {
        // Serialize function to string for later execution
        let callback_str = callback.to_string(scope).unwrap();
        let source = callback_str.to_rust_string_lossy(scope);
        CallbackInfo {
            callback_source: "function() { /* callback */ }".to_string(),
            is_function: true,
            function_source: Some(source),
        }
    } else if callback.is_string() {
        // If it's a string, we could potentially eval it later
        let callback_str = callback.to_string(scope).unwrap();
        let source = callback_str.to_rust_string_lossy(scope);
        CallbackInfo {
            callback_source: source,
            is_function: false,
            function_source: None,
        }
    } else {
        return; // Invalid callback type
    };

    // Store callback info
    {
        let callbacks = CALLBACKS.lock().unwrap();
        callbacks.insert(timer_id.clone(), callback_info);
    }

    let id_clone = timer_id.clone();
    let task = tokio::spawn(async move {
        sleep(Duration::from_millis(delay_ms)).await;

        // Execute callback immediately in this thread
        {
            let callbacks = CALLBACKS.lock().unwrap();
            if let Some(cb_info) = callbacks.get(&id_clone) {
                if cb_info.is_function {
                    // Queue function callback for execution
                    let timer_callback = TimerCallback {
                        id: id_clone.clone(),
                        callback_source: cb_info.callback_source.clone(),
                        is_function: cb_info.is_function,
                        function_source: cb_info.function_source.clone(),
                    };

                    // Add to callback queue
                    {
                        let mut queue = CALLBACK_QUEUE.lock().unwrap();
                        queue.push(timer_callback);
                    }
                } else {
                    // For string callbacks, queue them for execution
                    let timer_callback = TimerCallback {
                        id: id_clone.clone(),
                        callback_source: cb_info.callback_source.clone(),
                        is_function: cb_info.is_function,
                        function_source: cb_info.function_source.clone(),
                    };

                    // Add to callback queue
                    {
                        let mut queue = CALLBACK_QUEUE.lock().unwrap();
                        queue.push(timer_callback);
                    }
                }
            }
            callbacks.remove(&id_clone);
        }
    });

    TIMERS.insert(timer_id, task);
}

fn clear_timeout(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }

    let timer_id = args.get(0);
    if let Some(id_str) = timer_id.to_string(scope) {
        let id = id_str.to_rust_string_lossy(scope);
        if let Some((_, handle)) = TIMERS.remove(&id) {
            handle.abort();
            // Also remove callback info
            {
                let callbacks = CALLBACKS.lock().unwrap();
                callbacks.remove(&id);
            }
            println!("🚫 Timer {} cancelled", id);
        }
    }
}

fn set_interval(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }

    let callback = args.get(0);
    let delay = args.get(1);

    let delay_ms = delay.number_value(scope).unwrap_or(0.0) as u64;
    let timer_id = Uuid::new_v4().to_string();

    let timer_id_v8 = v8::String::new(scope, &timer_id).unwrap();
    rv.set(timer_id_v8.into());

    // Store callback info safely
    let callback_info = if callback.is_function() {
        // Serialize function to string for later execution
        let callback_str = callback.to_string(scope).unwrap();
        let source = callback_str.to_rust_string_lossy(scope);
        CallbackInfo {
            callback_source: "function() { /* interval callback */ }".to_string(),
            is_function: true,
            function_source: Some(source),
        }
    } else if callback.is_string() {
        let callback_str = callback.to_string(scope).unwrap();
        let source = callback_str.to_rust_string_lossy(scope);
        CallbackInfo {
            callback_source: source,
            is_function: false,
            function_source: None,
        }
    } else {
        return;
    };

    // Store callback info
    {
        let callbacks = CALLBACKS.lock().unwrap();
        callbacks.insert(timer_id.clone(), callback_info);
    }

    let interval_id = timer_id.clone();
    let task = tokio::spawn(async move {
        let mut interval_timer = interval(Duration::from_millis(delay_ms));
        let mut count = 0;

        loop {
            interval_timer.tick().await;
            count += 1;

            // Execute callback immediately in this thread
            let callback_data = {
                let callbacks = CALLBACKS.lock().unwrap();
                callbacks.get(&interval_id).map(|cb_info| {
                    (
                        cb_info.is_function,
                        cb_info.callback_source.clone(),
                        cb_info.function_source.clone(),
                    )
                })
            };

            let should_continue = match callback_data {
                Some((is_function, callback_source, function_source)) => {
                    if is_function {
                        // Queue function callback for execution
                        let timer_callback = TimerCallback {
                            id: format!("{}-{}", interval_id, count),
                            callback_source: callback_source.clone(),
                            is_function,
                            function_source: function_source.clone(),
                        };

                        // Add to callback queue
                        {
                            let mut queue = CALLBACK_QUEUE.lock().unwrap();
                            queue.push(timer_callback);
                        }
                    } else {
                        // For string callbacks, queue them for execution
                        let timer_callback = TimerCallback {
                            id: format!("{}-{}", interval_id, count),
                            callback_source: callback_source.clone(),
                            is_function,
                            function_source: function_source.clone(),
                        };

                        // Add to callback queue
                        {
                            let mut queue = CALLBACK_QUEUE.lock().unwrap();
                            queue.push(timer_callback);
                        }
                    }
                    true
                }
                None => false,
            };

            if !should_continue {
                // Interval was cleared
                break;
            }
        }
    });

    TIMERS.insert(timer_id, task);
}

fn clear_interval(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }

    let timer_id = args.get(0);
    if let Some(id_str) = timer_id.to_string(scope) {
        let id = id_str.to_rust_string_lossy(scope);
        if let Some((_, handle)) = TIMERS.remove(&id) {
            handle.abort();
            // Remove callback info to signal the interval to stop
            {
                let callbacks = CALLBACKS.lock().unwrap();
                callbacks.remove(&id);
            }
            println!("🛑 Interval {} cleared", id);
        }
    }
}
