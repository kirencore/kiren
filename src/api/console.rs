use anyhow::Result;
use v8;

pub fn setup_console(scope: &mut v8::ContextScope<v8::HandleScope>) -> Result<()> {
    let global = scope.get_current_context().global(scope);

    let console_obj = v8::Object::new(scope);

    let log_key = v8::String::new(scope, "log").unwrap();
    let log_fn = v8::Function::new(scope, console_log).unwrap();
    console_obj.set(scope, log_key.into(), log_fn.into());

    let error_key = v8::String::new(scope, "error").unwrap();
    let error_fn = v8::Function::new(scope, console_error).unwrap();
    console_obj.set(scope, error_key.into(), error_fn.into());

    let warn_key = v8::String::new(scope, "warn").unwrap();
    let warn_fn = v8::Function::new(scope, console_warn).unwrap();
    console_obj.set(scope, warn_key.into(), warn_fn.into());

    let info_key = v8::String::new(scope, "info").unwrap();
    let info_fn = v8::Function::new(scope, console_info).unwrap();
    console_obj.set(scope, info_key.into(), info_fn.into());

    let console_key = v8::String::new(scope, "console").unwrap();
    global.set(scope, console_key.into(), console_obj.into());

    Ok(())
}

fn console_log(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let output = args_to_string(scope, &args);
    println!("{}", output);
}

fn console_error(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let output = args_to_string(scope, &args);
    eprintln!("ERROR: {}", output);
}

fn console_warn(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let output = args_to_string(scope, &args);
    eprintln!("WARN: {}", output);
}

fn console_info(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let output = args_to_string(scope, &args);
    println!("INFO: {}", output);
}

fn args_to_string(scope: &mut v8::HandleScope, args: &v8::FunctionCallbackArguments) -> String {
    let mut parts = Vec::new();

    for i in 0..args.length() {
        let arg = args.get(i);
        let str_value = arg.to_string(scope).unwrap();
        parts.push(str_value.to_rust_string_lossy(scope));
    }

    parts.join(" ")
}
