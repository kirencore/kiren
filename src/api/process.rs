use anyhow::Result;
use std::env;
use v8;

pub fn setup_process(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
    let global = context.global(scope);

    // Create process object
    let process_key = v8::String::new(scope, "process").unwrap();
    let process_obj = v8::Object::new(scope);

    // process.env
    setup_process_env(scope, &process_obj)?;

    // process.argv
    setup_process_argv(scope, &process_obj)?;

    // process.exit
    let exit_key = v8::String::new(scope, "exit").unwrap();
    let exit_tmpl = v8::FunctionTemplate::new(scope, process_exit);
    let exit_fn = exit_tmpl.get_function(scope).unwrap();
    process_obj.set(scope, exit_key.into(), exit_fn.into());

    // process.cwd
    let cwd_key = v8::String::new(scope, "cwd").unwrap();
    let cwd_tmpl = v8::FunctionTemplate::new(scope, process_cwd);
    let cwd_fn = cwd_tmpl.get_function(scope).unwrap();
    process_obj.set(scope, cwd_key.into(), cwd_fn.into());

    global.set(scope, process_key.into(), process_obj.into());

    Ok(())
}

fn setup_process_env(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let env_key = v8::String::new(scope, "env").unwrap();
    let env_obj = v8::Object::new(scope);

    // Add all environment variables
    for (key, value) in env::vars() {
        let env_var_key = v8::String::new(scope, &key).unwrap();
        let env_var_value = v8::String::new(scope, &value).unwrap();
        env_obj.set(scope, env_var_key.into(), env_var_value.into());
    }

    process_obj.set(scope, env_key.into(), env_obj.into());

    Ok(())
}

fn setup_process_argv(
    scope: &mut v8::HandleScope,
    process_obj: &v8::Local<v8::Object>,
) -> Result<()> {
    let argv_key = v8::String::new(scope, "argv").unwrap();
    let argv_array = v8::Array::new(scope, 0);

    // Add command line arguments
    let args: Vec<String> = env::args().collect();

    for (index, arg) in args.iter().enumerate() {
        let arg_value = v8::String::new(scope, arg).unwrap();
        let index_v8 = index as u32;
        argv_array.set_index(scope, index_v8, arg_value.into());
    }

    process_obj.set(scope, argv_key.into(), argv_array.into());

    Ok(())
}

fn process_exit(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let exit_code = if args.length() > 0 {
        args.get(0).number_value(scope).unwrap_or(0.0) as i32
    } else {
        0
    };

    std::process::exit(exit_code);
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
