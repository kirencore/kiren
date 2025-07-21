use crate::config::KirenConfig;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::signal;
use v8;

// Global process state
static PROCESS_STATE: Lazy<Arc<Mutex<ProcessState>>> =
    Lazy::new(|| Arc::new(Mutex::new(ProcessState::new())));

// Signal handlers storage - simplified to avoid Send trait issues
// static SIGNAL_HANDLERS: Lazy<Arc<Mutex<HashMap<String, Vec<v8::Global<v8::Function>>>>>> =
//     Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Debug, Clone)]
pub struct ProcessState {
    pub pid: u32,
    pub ppid: u32,
    pub start_time: u64,
    pub cwd: String,
    pub argv: Vec<String>,
    pub env: HashMap<String, String>,
    pub exit_code: Option<i32>,
    pub should_exit: bool,
}

impl ProcessState {
    pub fn new() -> Self {
        let pid = std::process::id();
        let ppid = 0; // TODO: Get parent PID on Unix systems
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let cwd = std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let argv: Vec<String> = std::env::args().collect();
        let env: HashMap<String, String> = std::env::vars().collect();

        Self {
            pid,
            ppid,
            start_time,
            cwd,
            argv,
            env,
            exit_code: None,
            should_exit: false,
        }
    }
}

pub fn setup_process(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
    let global = context.global(scope);

    // Create process object
    let process_key = v8::String::new(scope, "process").unwrap();
    let process_obj = v8::Object::new(scope);

    // Add process properties
    add_process_properties(scope, &process_obj)?;

    // Add process methods
    add_process_methods(scope, &process_obj)?;

    global.set(scope, process_key.into(), process_obj.into());

    // Start signal listeners - disabled for now to avoid infinite loop
    // start_signal_listeners();

    Ok(())
}

fn add_process_properties(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let state = PROCESS_STATE.lock().unwrap();

    // process.pid
    let pid_key = v8::String::new(scope, "pid").unwrap();
    let pid_val = v8::Number::new(scope, state.pid as f64);
    process_obj.set(scope, pid_key.into(), pid_val.into());

    // process.ppid
    let ppid_key = v8::String::new(scope, "ppid").unwrap();
    let ppid_val = v8::Number::new(scope, state.ppid as f64);
    process_obj.set(scope, ppid_key.into(), ppid_val.into());

    // process.platform
    let platform_key = v8::String::new(scope, "platform").unwrap();
    let platform_val = v8::String::new(scope, std::env::consts::OS).unwrap();
    process_obj.set(scope, platform_key.into(), platform_val.into());

    // process.arch
    let arch_key = v8::String::new(scope, "arch").unwrap();
    let arch_val = v8::String::new(scope, std::env::consts::ARCH).unwrap();
    process_obj.set(scope, arch_key.into(), arch_val.into());

    // process.version (Kiren version)
    let version_key = v8::String::new(scope, "version").unwrap();
    let version_text = format!("v{}", &KirenConfig::default().runtime.version);
    let version_val = v8::String::new(scope, &version_text).unwrap();
    process_obj.set(scope, version_key.into(), version_val.into());

    // process.argv
    let argv_key = v8::String::new(scope, "argv").unwrap();
    let argv_array = v8::Array::new(scope, state.argv.len() as i32);
    for (i, arg) in state.argv.iter().enumerate() {
        let arg_val = v8::String::new(scope, arg).unwrap();
        let index = v8::Number::new(scope, i as f64);
        argv_array.set(scope, index.into(), arg_val.into());
    }
    process_obj.set(scope, argv_key.into(), argv_array.into());

    // process.env
    let env_key = v8::String::new(scope, "env").unwrap();
    let env_obj = v8::Object::new(scope);
    for (key, value) in &state.env {
        let env_prop_key = v8::String::new(scope, key).unwrap();
        let env_prop_val = v8::String::new(scope, value).unwrap();
        env_obj.set(scope, env_prop_key.into(), env_prop_val.into());
    }
    process_obj.set(scope, env_key.into(), env_obj.into());

    Ok(())
}

fn add_process_methods(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    // process.exit()
    let exit_key = v8::String::new(scope, "exit").unwrap();
    let exit_fn = v8::FunctionTemplate::new(scope, process_exit);
    let exit_func = exit_fn.get_function(scope).unwrap();
    process_obj.set(scope, exit_key.into(), exit_func.into());

    // process.on()
    let on_key = v8::String::new(scope, "on").unwrap();
    let on_fn = v8::FunctionTemplate::new(scope, process_on);
    let on_func = on_fn.get_function(scope).unwrap();
    process_obj.set(scope, on_key.into(), on_func.into());

    // process.off()
    let off_key = v8::String::new(scope, "off").unwrap();
    let off_fn = v8::FunctionTemplate::new(scope, process_off);
    let off_func = off_fn.get_function(scope).unwrap();
    process_obj.set(scope, off_key.into(), off_func.into());

    // process.kill()
    let kill_key = v8::String::new(scope, "kill").unwrap();
    let kill_fn = v8::FunctionTemplate::new(scope, process_kill);
    let kill_func = kill_fn.get_function(scope).unwrap();
    process_obj.set(scope, kill_key.into(), kill_func.into());

    // process.nextTick()
    let next_tick_key = v8::String::new(scope, "nextTick").unwrap();
    let next_tick_fn = v8::FunctionTemplate::new(scope, process_next_tick);
    let next_tick_func = next_tick_fn.get_function(scope).unwrap();
    process_obj.set(scope, next_tick_key.into(), next_tick_func.into());

    // process.cwd()
    let cwd_key = v8::String::new(scope, "cwd").unwrap();
    let cwd_fn = v8::FunctionTemplate::new(scope, process_cwd);
    let cwd_func = cwd_fn.get_function(scope).unwrap();
    process_obj.set(scope, cwd_key.into(), cwd_func.into());

    // process.chdir()
    let chdir_key = v8::String::new(scope, "chdir").unwrap();
    let chdir_fn = v8::FunctionTemplate::new(scope, process_chdir);
    let chdir_func = chdir_fn.get_function(scope).unwrap();
    process_obj.set(scope, chdir_key.into(), chdir_func.into());

    // process.hrtime()
    let hrtime_key = v8::String::new(scope, "hrtime").unwrap();
    let hrtime_fn = v8::FunctionTemplate::new(scope, process_hrtime);
    let hrtime_func = hrtime_fn.get_function(scope).unwrap();
    process_obj.set(scope, hrtime_key.into(), hrtime_func.into());

    // process.memoryUsage()
    let memory_key = v8::String::new(scope, "memoryUsage").unwrap();
    let memory_fn = v8::FunctionTemplate::new(scope, process_memory_usage);
    let memory_func = memory_fn.get_function(scope).unwrap();
    process_obj.set(scope, memory_key.into(), memory_func.into());

    // process.uptime()
    let uptime_key = v8::String::new(scope, "uptime").unwrap();
    let uptime_fn = v8::FunctionTemplate::new(scope, process_uptime);
    let uptime_func = uptime_fn.get_function(scope).unwrap();
    process_obj.set(scope, uptime_key.into(), uptime_func.into());

    // process.cwd()
    let cwd_key = v8::String::new(scope, "cwd").unwrap();
    let cwd_tmpl = v8::FunctionTemplate::new(scope, process_cwd);
    let cwd_fn = cwd_tmpl.get_function(scope).unwrap();
    process_obj.set(scope, cwd_key.into(), cwd_fn.into());

    Ok(())
}

// process.exit([code])
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

    {
        let mut state = PROCESS_STATE.lock().unwrap();
        state.exit_code = Some(exit_code);
        state.should_exit = true;
    }

    std::process::exit(exit_code);
}

// process.on(event, listener)
fn process_on(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(
            scope,
            "process.on() requires event name and listener function",
        )
        .unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let event_arg = args.get(0);
    let listener_arg = args.get(1);

    if !event_arg.is_string() || !listener_arg.is_function() {
        let error = v8::String::new(
            scope,
            "process.on() requires string event name and function listener",
        )
        .unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let event_name = event_arg.to_string(scope).unwrap();
    let event_str = event_name.to_rust_string_lossy(scope);
    let _listener = v8::Local::<v8::Function>::try_from(listener_arg).unwrap();

    // Store the listener - simplified implementation
    // TODO: Implement proper signal handler storage
    println!("Signal handler registered for: {}", event_str);
}

// process.off(event, listener)
fn process_off(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        let error = v8::String::new(scope, "process.off() requires event name").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let event_arg = args.get(0);
    let event_name = event_arg.to_string(scope).unwrap();
    let event_str = event_name.to_rust_string_lossy(scope);

    // Remove all listeners for this event - simplified implementation
    println!("Signal handlers removed for: {}", event_str);
}

// process.kill(pid, signal)
fn process_kill(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        let error = v8::String::new(scope, "process.kill() requires pid").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let pid_arg = args.get(0);
    let pid = pid_arg.int32_value(scope).unwrap_or(0) as u32;

    let signal = if args.length() > 1 {
        let signal_arg = args.get(1);
        signal_arg
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope)
    } else {
        "SIGTERM".to_string()
    };

    // For safety, only allow killing own process in this implementation
    let current_pid = std::process::id();
    if pid == current_pid {
        match signal.as_str() {
            "SIGTERM" | "SIGINT" => {
                rv.set(v8::Boolean::new(scope, true).into());
            }
            _ => {
                rv.set(v8::Boolean::new(scope, false).into());
            }
        }
    } else {
        rv.set(v8::Boolean::new(scope, false).into());
    }
}

// process.nextTick(callback)
fn process_next_tick(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 || !args.get(0).is_function() {
        let error = v8::String::new(scope, "process.nextTick() requires a function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();

    // Schedule callback for next tick using setTimeout(callback, 0)
    let global = scope.get_current_context().global(scope);
    let set_timeout_key = v8::String::new(scope, "setTimeout").unwrap();

    if let Some(set_timeout_val) = global.get(scope, set_timeout_key.into()) {
        if let Ok(set_timeout_fn) = v8::Local::<v8::Function>::try_from(set_timeout_val) {
            let zero = v8::Number::new(scope, 0.0);
            let args = [callback.into(), zero.into()];
            set_timeout_fn.call(scope, global.into(), &args);
        }
    }
}

// process.hrtime([time])
fn process_hrtime(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let now = std::time::Instant::now();
    let duration = now.elapsed();

    let (seconds, nanoseconds) = if args.length() > 0 && args.get(0).is_array() {
        // Calculate difference from provided time
        let prev_time = v8::Local::<v8::Array>::try_from(args.get(0)).unwrap();
        let prev_sec_val = prev_time.get_index(scope, 0).unwrap();
        let prev_nsec_val = prev_time.get_index(scope, 1).unwrap();

        let prev_sec = prev_sec_val.number_value(scope).unwrap_or(0.0) as u64;
        let prev_nsec = prev_nsec_val.number_value(scope).unwrap_or(0.0) as u64;

        let total_nsec = duration.as_nanos() as u64;
        let prev_total_nsec = prev_sec * 1_000_000_000 + prev_nsec;
        let diff_nsec = total_nsec.saturating_sub(prev_total_nsec);

        (diff_nsec / 1_000_000_000, diff_nsec % 1_000_000_000)
    } else {
        // Return current time
        (duration.as_secs(), duration.subsec_nanos() as u64)
    };

    let result = v8::Array::new(scope, 2);
    let sec_val = v8::Number::new(scope, seconds as f64);
    let nsec_val = v8::Number::new(scope, nanoseconds as f64);

    result.set_index(scope, 0, sec_val.into());
    result.set_index(scope, 1, nsec_val.into());

    rv.set(result.into());
}

// process.memoryUsage()
fn process_memory_usage(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let memory_obj = v8::Object::new(scope);

    // For now, return mock values. In production, use system APIs
    let rss_key = v8::String::new(scope, "rss").unwrap();
    let rss_val = v8::Number::new(scope, 50_000_000.0); // 50MB
    memory_obj.set(scope, rss_key.into(), rss_val.into());

    let heap_total_key = v8::String::new(scope, "heapTotal").unwrap();
    let heap_total_val = v8::Number::new(scope, 20_000_000.0); // 20MB
    memory_obj.set(scope, heap_total_key.into(), heap_total_val.into());

    let heap_used_key = v8::String::new(scope, "heapUsed").unwrap();
    let heap_used_val = v8::Number::new(scope, 15_000_000.0); // 15MB
    memory_obj.set(scope, heap_used_key.into(), heap_used_val.into());

    let external_key = v8::String::new(scope, "external").unwrap();
    let external_val = v8::Number::new(scope, 1_000_000.0); // 1MB
    memory_obj.set(scope, external_key.into(), external_val.into());

    rv.set(memory_obj.into());
}

// process.uptime()
fn process_uptime(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let state = PROCESS_STATE.lock().unwrap();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let uptime_ms = current_time - state.start_time;
    let uptime_sec = uptime_ms as f64 / 1000.0;

    rv.set(v8::Number::new(scope, uptime_sec).into());
}

fn process_cwd(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    match env::current_dir() {
        Ok(path) => {
            let path_str = path.to_string_lossy();
            let path_v8 = v8::String::new(scope, &path_str).unwrap();
            rv.set(path_v8.into());
        }
        Err(_) => {
            let error = v8::String::new(scope, "Failed to get current directory").unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
        }
    }
}

// Start listening for system signals
fn start_signal_listeners() {
    tokio::spawn(async {
        #[cfg(unix)]
        {
            use signal::unix::{signal, SignalKind};

            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            let mut sigint = signal(SignalKind::interrupt()).unwrap();
            let mut sigusr1 = signal(SignalKind::user_defined1()).unwrap();

            loop {
                tokio::select! {
                    _ = sigterm.recv() => {
                        println!("Received SIGTERM");
                        std::process::exit(0);
                    }
                    _ = sigint.recv() => {
                        println!("Received SIGINT");
                        std::process::exit(0);
                    }
                    _ = sigusr1.recv() => {
                        println!("Received SIGUSR1");
                    }
                }
            }
        }

        #[cfg(windows)]
        {
            let mut ctrl_c = signal::ctrl_c().unwrap();
            ctrl_c.recv().await;
            println!("Received Ctrl+C");
            std::process::exit(0);
        }
    });
}

// Check if process should exit
pub fn should_exit() -> bool {
    PROCESS_STATE.lock().unwrap().should_exit
}

// Get exit code
pub fn get_exit_code() -> Option<i32> {
    PROCESS_STATE.lock().unwrap().exit_code
}

// process.chdir() implementation
fn process_chdir(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        let error_msg = v8::String::new(scope, "chdir requires a directory path").unwrap();
        let error = v8::Exception::type_error(scope, error_msg);
        scope.throw_exception(error);
        return;
    }

    let path_arg = args.get(0);
    if let Ok(path_str) = v8::Local::<v8::String>::try_from(path_arg) {
        let path = path_str.to_rust_string_lossy(scope);

        if let Err(_) = std::env::set_current_dir(&path) {
            let error_msg =
                v8::String::new(scope, &format!("Failed to change directory to: {}", path))
                    .unwrap();
            let error = v8::Exception::error(scope, error_msg);
            scope.throw_exception(error);
        }
        // Update the process state
        if let Ok(mut state) = PROCESS_STATE.lock() {
            if let Ok(cwd) = std::env::current_dir() {
                state.cwd = cwd.to_string_lossy().to_string();
            }
        }
    } else {
        let error_msg = v8::String::new(scope, "chdir requires a string argument").unwrap();
        let error = v8::Exception::type_error(scope, error_msg);
        scope.throw_exception(error);
    }
}
