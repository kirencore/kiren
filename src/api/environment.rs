use anyhow::Result;
use v8;

pub fn setup_environment(
    scope: &mut v8::HandleScope,
    context: v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Create process object
    let process_key = v8::String::new(scope, "process").unwrap();
    let process_obj = v8::Object::new(scope);

    // Add process.env - simple object with basic environment variables
    let env_key = v8::String::new(scope, "env").unwrap();
    let env_obj = v8::Object::new(scope);

    // Add NODE_ENV
    let node_env_key = v8::String::new(scope, "NODE_ENV").unwrap();
    let node_env_value = v8::String::new(
        scope,
        &std::env::var("NODE_ENV").unwrap_or_else(|_| "development".to_string()),
    )
    .unwrap();
    env_obj.set(scope, node_env_key.into(), node_env_value.into());

    // Add PORT
    let port_key = v8::String::new(scope, "PORT").unwrap();
    let port_value = v8::String::new(
        scope,
        &std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()),
    )
    .unwrap();
    env_obj.set(scope, port_key.into(), port_value.into());

    process_obj.set(scope, env_key.into(), env_obj.into());

    // Add process.exit
    let exit_key = v8::String::new(scope, "exit").unwrap();
    let exit_template = v8::FunctionTemplate::new(scope, process_exit);
    let exit_function = exit_template.get_function(scope).unwrap();
    process_obj.set(scope, exit_key.into(), exit_function.into());

    // Add process.platform
    let platform_key = v8::String::new(scope, "platform").unwrap();
    let platform_value = v8::String::new(scope, std::env::consts::OS).unwrap();
    process_obj.set(scope, platform_key.into(), platform_value.into());

    // Add process.version
    let version_key = v8::String::new(scope, "version").unwrap();
    let version_value = v8::String::new(scope, "v0.2.0").unwrap();
    process_obj.set(scope, version_key.into(), version_value.into());

    global.set(scope, process_key.into(), process_obj.into());

    Ok(())
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

    eprintln!("Process exiting with code: {}", exit_code);
    std::process::exit(exit_code);
}
