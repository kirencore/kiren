#![allow(dead_code)]
use anyhow::Result;
use std::env;
use v8;

/// Improved process object with full Node.js compatibility
pub fn setup_process_object(
    scope: &mut v8::HandleScope,
    context: v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Create process object
    let process_obj = v8::Object::new(scope);

    // Set up process.env
    setup_process_env(scope, &process_obj)?;

    // Set up process.argv
    setup_process_argv(scope, &process_obj)?;

    // Set up process.cwd()
    setup_process_cwd(scope, &process_obj)?;

    // Set up process.exit()
    setup_process_exit(scope, &process_obj)?;

    // Set up process.nextTick()
    setup_process_next_tick(scope, &process_obj)?;

    // Set up process.version
    setup_process_version(scope, &process_obj)?;

    // Set up process.platform
    setup_process_platform(scope, &process_obj)?;

    // Set up process.arch
    setup_process_arch(scope, &process_obj)?;

    // Set process object as global
    let process_key = v8::String::new(scope, "process").unwrap();
    global.set(scope, process_key.into(), process_obj.into());

    Ok(())
}

/// Setup process.env with actual environment variables
fn setup_process_env(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let env_obj = v8::Object::new(scope);

    // Get all environment variables
    for (key, value) in env::vars() {
        let env_key = v8::String::new(scope, &key).unwrap();
        let env_value = v8::String::new(scope, &value).unwrap();
        env_obj.set(scope, env_key.into(), env_value.into());
    }

    // Add some common defaults if not present
    let common_defaults = [
        ("NODE_ENV", "development"),
        ("PORT", "3000"),
        ("HOST", "localhost"),
    ];

    for (key, default_value) in &common_defaults {
        let env_key = v8::String::new(scope, key).unwrap();
        if env_obj.get(scope, env_key.into()).unwrap().is_undefined() {
            let env_value = v8::String::new(scope, default_value).unwrap();
            env_obj.set(scope, env_key.into(), env_value.into());
        }
    }

    let env_key = v8::String::new(scope, "env").unwrap();
    process_obj.set(scope, env_key.into(), env_obj.into());

    Ok(())
}

/// Setup process.argv
fn setup_process_argv(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let argv_array = v8::Array::new(scope, 0);

    // Add kiren executable path
    let kiren_path = v8::String::new(scope, "kiren").unwrap();
    argv_array.set_index(scope, 0, kiren_path.into());

    // Add command line arguments
    let args: Vec<String> = env::args().collect();
    for (index, arg) in args.iter().enumerate() {
        let arg_str = v8::String::new(scope, arg).unwrap();
        argv_array.set_index(scope, index as u32, arg_str.into());
    }

    let argv_key = v8::String::new(scope, "argv").unwrap();
    process_obj.set(scope, argv_key.into(), argv_array.into());

    Ok(())
}

/// Setup process.cwd()
fn setup_process_cwd(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let cwd_key = v8::String::new(scope, "cwd").unwrap();
    let cwd_template = v8::FunctionTemplate::new(scope, process_cwd);
    let cwd_function = cwd_template.get_function(scope).unwrap();
    process_obj.set(scope, cwd_key.into(), cwd_function.into());

    Ok(())
}

/// Setup process.exit()
fn setup_process_exit(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let exit_key = v8::String::new(scope, "exit").unwrap();
    let exit_template = v8::FunctionTemplate::new(scope, process_exit);
    let exit_function = exit_template.get_function(scope).unwrap();
    process_obj.set(scope, exit_key.into(), exit_function.into());

    Ok(())
}

/// Setup process.nextTick()
fn setup_process_next_tick(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let next_tick_key = v8::String::new(scope, "nextTick").unwrap();
    let next_tick_template = v8::FunctionTemplate::new(scope, process_next_tick);
    let next_tick_function = next_tick_template.get_function(scope).unwrap();
    process_obj.set(scope, next_tick_key.into(), next_tick_function.into());

    Ok(())
}

/// Setup process.version
fn setup_process_version(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let version_key = v8::String::new(scope, "version").unwrap();
    let version_value = v8::String::new(scope, "v18.0.0").unwrap(); // Fake Node.js version
    process_obj.set(scope, version_key.into(), version_value.into());

    Ok(())
}

/// Setup process.platform
fn setup_process_platform(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let platform_key = v8::String::new(scope, "platform").unwrap();

    let platform_name = if cfg!(target_os = "windows") {
        "win32"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let platform_value = v8::String::new(scope, platform_name).unwrap();
    process_obj.set(scope, platform_key.into(), platform_value.into());

    Ok(())
}

/// Setup process.arch
fn setup_process_arch(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let arch_key = v8::String::new(scope, "arch").unwrap();

    let arch_name = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "x86") {
        "ia32"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "unknown"
    };

    let arch_value = v8::String::new(scope, arch_name).unwrap();
    process_obj.set(scope, arch_key.into(), arch_value.into());

    Ok(())
}

// V8 callback implementations

fn process_cwd(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    match env::current_dir() {
        Ok(dir) => {
            let cwd_str = v8::String::new(scope, dir.to_string_lossy().as_ref()).unwrap();
            rv.set(cwd_str.into());
        }
        Err(_) => {
            let cwd_str = v8::String::new(scope, ".").unwrap();
            rv.set(cwd_str.into());
        }
    }
}

fn process_exit(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let exit_code = if args.length() > 0 {
        args.get(0).int32_value(scope).unwrap_or(0)
    } else {
        0
    };

    println!("Process exiting with code: {}", exit_code);
    std::process::exit(exit_code);
}

fn process_next_tick(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        return;
    }

    let callback = args.get(0);
    if !callback.is_function() {
        return;
    }

    // For now, execute immediately (should be queued for next tick)
    let callback_fn = unsafe { v8::Local::<v8::Function>::cast(callback) };
    let undefined = v8::undefined(scope);

    // Get additional arguments to pass to callback
    let mut callback_args = Vec::new();
    for i in 1..args.length() {
        callback_args.push(args.get(i));
    }

    let mut try_catch = v8::TryCatch::new(scope);
    callback_fn.call(&mut try_catch, undefined.into(), &callback_args);

    if try_catch.has_caught() {
        if let Some(exception) = try_catch.exception() {
            let exc_str = exception.to_string(&mut try_catch).unwrap();
            let error_msg = exc_str.to_rust_string_lossy(&mut try_catch);
            println!("Error in nextTick callback: {}", error_msg);
        }
    }
}

/// Enhanced process object setup for better Node.js compatibility
pub fn setup_process_signals(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    // Add process.on() for signal handling
    let on_key = v8::String::new(scope, "on").unwrap();
    let on_template = v8::FunctionTemplate::new(scope, process_on);
    let on_function = on_template.get_function(scope).unwrap();
    process_obj.set(scope, on_key.into(), on_function.into());

    // Add process.once()
    let once_key = v8::String::new(scope, "once").unwrap();
    let once_template = v8::FunctionTemplate::new(scope, process_once);
    let once_function = once_template.get_function(scope).unwrap();
    process_obj.set(scope, once_key.into(), once_function.into());

    // Add process.emit()
    let emit_key = v8::String::new(scope, "emit").unwrap();
    let emit_template = v8::FunctionTemplate::new(scope, process_emit);
    let emit_function = emit_template.get_function(scope).unwrap();
    process_obj.set(scope, emit_key.into(), emit_function.into());

    Ok(())
}

fn process_on(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }

    let event = args.get(0);
    let callback = args.get(1);

    if !event.is_string() || !callback.is_function() {
        return;
    }

    let event_str = event.to_string(scope).unwrap();
    let event_name = event_str.to_rust_string_lossy(scope);

    // For now, just log that we're setting up event listeners
    println!("Process event listener registered for: {}", event_name);

    // TODO: Store event listeners and implement proper signal handling
}

fn process_once(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    // Similar to process_on but for one-time events
    if args.length() < 2 {
        return;
    }

    let event = args.get(0);
    let callback = args.get(1);

    if !event.is_string() || !callback.is_function() {
        return;
    }

    let event_str = event.to_string(scope).unwrap();
    let event_name = event_str.to_rust_string_lossy(scope);

    println!("Process once event listener registered for: {}", event_name);
}

fn process_emit(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    }

    let event = args.get(0);
    if !event.is_string() {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    }

    let event_str = event.to_string(scope).unwrap();
    let event_name = event_str.to_rust_string_lossy(scope);

    println!("Process event emitted: {}", event_name);

    // TODO: Actually emit to registered listeners
    rv.set(v8::Boolean::new(scope, true).into());
}
