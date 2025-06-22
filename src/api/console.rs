use anyhow::Result;
use v8;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

// Console level constants
const LEVEL_DEBUG: u8 = 0;
const LEVEL_INFO: u8 = 1;
const LEVEL_WARN: u8 = 2;
const LEVEL_ERROR: u8 = 3;

// Global logging configuration
static CONSOLE_CONFIG: Lazy<Arc<Mutex<ConsoleConfig>>> = Lazy::new(|| {
    Arc::new(Mutex::new(ConsoleConfig::default()))
});

#[derive(Debug, Clone)]
pub struct ConsoleConfig {
    pub level: u8,
    pub verbose: bool,
    pub silent: bool,
    pub timestamp: bool,
    pub colors: bool,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            level: LEVEL_INFO,
            verbose: false,
            silent: false,
            timestamp: true,
            colors: true,
        }
    }
}

pub fn configure_console(verbose: bool, silent: bool) {
    let mut config = CONSOLE_CONFIG.lock().unwrap();
    config.verbose = verbose;
    config.silent = silent;
    config.level = if verbose {
        LEVEL_DEBUG
    } else if silent {
        LEVEL_ERROR
    } else {
        LEVEL_INFO
    };
}

pub fn setup_console(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
    let global = context.global(scope);
    let console_key = v8::String::new(scope, "console").unwrap();
    let console_obj = v8::Object::new(scope);

    // console.log
    let log_key = v8::String::new(scope, "log").unwrap();
    let log_template = v8::FunctionTemplate::new(scope, console_log);
    let log_function = log_template.get_function(scope).unwrap();
    console_obj.set(scope, log_key.into(), log_function.into());

    // console.info
    let info_key = v8::String::new(scope, "info").unwrap();
    let info_template = v8::FunctionTemplate::new(scope, console_info);
    let info_function = info_template.get_function(scope).unwrap();
    console_obj.set(scope, info_key.into(), info_function.into());

    // console.warn
    let warn_key = v8::String::new(scope, "warn").unwrap();
    let warn_template = v8::FunctionTemplate::new(scope, console_warn);
    let warn_function = warn_template.get_function(scope).unwrap();
    console_obj.set(scope, warn_key.into(), warn_function.into());

    // console.error
    let error_key = v8::String::new(scope, "error").unwrap();
    let error_template = v8::FunctionTemplate::new(scope, console_error);
    let error_function = error_template.get_function(scope).unwrap();
    console_obj.set(scope, error_key.into(), error_function.into());

    // console.debug
    let debug_key = v8::String::new(scope, "debug").unwrap();
    let debug_template = v8::FunctionTemplate::new(scope, console_debug);
    let debug_function = debug_template.get_function(scope).unwrap();
    console_obj.set(scope, debug_key.into(), debug_function.into());

    // console.trace
    let trace_key = v8::String::new(scope, "trace").unwrap();
    let trace_template = v8::FunctionTemplate::new(scope, console_trace);
    let trace_function = trace_template.get_function(scope).unwrap();
    console_obj.set(scope, trace_key.into(), trace_function.into());

    // console.assert
    let assert_key = v8::String::new(scope, "assert").unwrap();
    let assert_template = v8::FunctionTemplate::new(scope, console_assert);
    let assert_function = assert_template.get_function(scope).unwrap();
    console_obj.set(scope, assert_key.into(), assert_function.into());

    // console.clear
    let clear_key = v8::String::new(scope, "clear").unwrap();
    let clear_template = v8::FunctionTemplate::new(scope, console_clear);
    let clear_function = clear_template.get_function(scope).unwrap();
    console_obj.set(scope, clear_key.into(), clear_function.into());

    // console.time
    let time_key = v8::String::new(scope, "time").unwrap();
    let time_template = v8::FunctionTemplate::new(scope, console_time);
    let time_function = time_template.get_function(scope).unwrap();
    console_obj.set(scope, time_key.into(), time_function.into());

    // console.timeEnd
    let time_end_key = v8::String::new(scope, "timeEnd").unwrap();
    let time_end_template = v8::FunctionTemplate::new(scope, console_time_end);
    let time_end_function = time_end_template.get_function(scope).unwrap();
    console_obj.set(scope, time_end_key.into(), time_end_function.into());

    global.set(scope, console_key.into(), console_obj.into());

    Ok(())
}

fn should_log(level: u8) -> bool {
    let config = CONSOLE_CONFIG.lock().unwrap();
    !config.silent && level >= config.level
}

fn format_message(level: &str, message: &str) -> String {
    let config = CONSOLE_CONFIG.lock().unwrap();
    
    let mut formatted = String::new();
    
    if config.timestamp {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let secs = now.as_secs();
        let millis = now.subsec_millis();
        let time = secs % 86400; // seconds in a day
        let hours = time / 3600;
        let minutes = (time % 3600) / 60;
        let seconds = time % 60;
        formatted.push_str(&format!("[{:02}:{:02}:{:02}.{:03}] ", hours, minutes, seconds, millis));
    }
    
    if config.colors {
        let colored_level = match level {
            "DEBUG" => format!("\x1b[36m{}\x1b[0m", level), // Cyan
            "INFO" => format!("\x1b[32m{}\x1b[0m", level),  // Green
            "WARN" => format!("\x1b[33m{}\x1b[0m", level),  // Yellow
            "ERROR" => format!("\x1b[31m{}\x1b[0m", level), // Red
            _ => level.to_string(),
        };
        formatted.push_str(&format!("[{}] ", colored_level));
    } else {
        formatted.push_str(&format!("[{}] ", level));
    }
    
    formatted.push_str(message);
    formatted
}

fn console_log(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if !should_log(LEVEL_INFO) {
        return;
    }
    
    let message = format_args(scope, &args);
    println!("{}", format_message("INFO", &message));
}

fn console_info(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if !should_log(LEVEL_INFO) {
        return;
    }
    
    let message = format_args(scope, &args);
    println!("{}", format_message("INFO", &message));
}

fn console_warn(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if !should_log(LEVEL_WARN) {
        return;
    }
    
    let message = format_args(scope, &args);
    eprintln!("{}", format_message("WARN", &message));
}

fn console_error(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if !should_log(LEVEL_ERROR) {
        return;
    }
    
    let message = format_args(scope, &args);
    eprintln!("{}", format_message("ERROR", &message));
}

fn console_debug(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if !should_log(LEVEL_DEBUG) {
        return;
    }
    
    let message = format_args(scope, &args);
    println!("{}", format_message("DEBUG", &message));
}

fn console_trace(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if !should_log(LEVEL_INFO) {
        return;
    }
    
    let message = format_args(scope, &args);
    eprintln!("{}", format_message("TRACE", &message));
    
    // Get stack trace - simplified approach
    eprintln!("    at trace (console.trace)");
}

fn console_assert(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        return;
    }
    
    let condition = args.get(0);
    let is_truthy = condition.is_true() || (!condition.is_false() && !condition.is_null_or_undefined());
    
    if !is_truthy {
        let message = if args.length() > 1 {
            let mut parts = Vec::new();
            for i in 1..args.length() {
                let arg = args.get(i);
                let str_val = arg.to_string(scope).unwrap();
                parts.push(str_val.to_rust_string_lossy(scope));
            }
            parts.join(" ")
        } else {
            "Assertion failed".to_string()
        };
        
        eprintln!("{}", format_message("ERROR", &message));
    }
}

fn console_clear(
    _scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    print!("\x1b[2J\x1b[H");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}

fn format_args(scope: &mut v8::HandleScope, args: &v8::FunctionCallbackArguments) -> String {
    let mut output = String::new();
    for i in 0..args.length() {
        if i > 0 {
            output.push(' ');
        }
        let arg = args.get(i);
        
        // Handle different types
        if arg.is_string() {
            let str_val = arg.to_string(scope).unwrap();
            output.push_str(&str_val.to_rust_string_lossy(scope));
        } else if arg.is_object() && !arg.is_null() {
            // Try to stringify objects
            let json_key = v8::String::new(scope, "JSON").unwrap();
            let global = scope.get_current_context().global(scope);
            
            if let Some(json_obj) = global.get(scope, json_key.into()) {
                if let Some(json_obj) = json_obj.to_object(scope) {
                    let stringify_key = v8::String::new(scope, "stringify").unwrap();
                    if let Some(stringify_fn) = json_obj.get(scope, stringify_key.into()) {
                        if let Some(stringify_fn) = stringify_fn.to_object(scope) {
                            if stringify_fn.is_function() {
                                let stringify_fn = v8::Local::<v8::Function>::try_from(stringify_fn).unwrap();
                                let args_array = [arg];
                                if let Some(result) = stringify_fn.call(scope, json_obj.into(), &args_array) {
                                    let str_val = result.to_string(scope).unwrap();
                                    output.push_str(&str_val.to_rust_string_lossy(scope));
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
            
            // Fallback to toString
            let str_val = arg.to_string(scope).unwrap();
            output.push_str(&str_val.to_rust_string_lossy(scope));
        } else {
            let str_val = arg.to_string(scope).unwrap();
            output.push_str(&str_val.to_rust_string_lossy(scope));
        }
    }
    output
}

fn console_time(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let label = if args.length() > 0 {
        let arg = args.get(0);
        let str_val = arg.to_string(scope).unwrap();
        str_val.to_rust_string_lossy(scope)
    } else {
        "default".to_string()
    };

    // Simple time tracking using system time
    println!("Timer '{}' started", label);
}

fn console_time_end(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let label = if args.length() > 0 {
        let arg = args.get(0);
        let str_val = arg.to_string(scope).unwrap();
        str_val.to_rust_string_lossy(scope)
    } else {
        "default".to_string()
    };

    println!("Timer '{}' ended", label);
}
