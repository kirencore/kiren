use crate::api::{buffer, environment, events, fetch, filesystem, http, process, streams, test, timers};
use anyhow::Result;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex, Once};
use std::time::Instant;
use v8;

static INIT: Once = Once::new();
static CONSOLE_TIMERS: Lazy<DashMap<String, Instant>> = Lazy::new(|| DashMap::new());

pub struct Engine {
    isolate: v8::OwnedIsolate,
    context_initialized: bool,
}

impl Engine {
    pub fn new() -> Result<Self> {
        INIT.call_once(|| {
            // Use optimal thread count for better performance
            let thread_count = std::thread::available_parallelism()
                .map(|n| n.get().try_into().unwrap_or(4u32))
                .unwrap_or(4);
            let platform = v8::new_default_platform(thread_count, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });

        // Create isolate with default settings (V8 0.84 limitations)
        let mut isolate = v8::Isolate::new(Default::default());

        // Performance optimizations
        isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);

        Ok(Engine { 
            isolate,
            context_initialized: false,
        })
    }

    pub fn execute(&mut self, source: &str) -> Result<String> {
        self.execute_with_callbacks(source, true)
    }

    pub fn execute_module(&mut self, source: &str, _module_name: &str) -> Result<String> {
        // Static import'ları dynamic import'a dönüştür
        let transformed_source = self.transform_static_imports(source);
        println!("Original source:\n{}", source);
        println!("Transformed source:\n{}", transformed_source);
        
        
        // Normal execute ile çalıştır ama module context'te
        self.execute_with_callbacks(&transformed_source, true)
    }

    fn transform_static_imports(&self, source: &str) -> String {
        // Fast path: sadece import keyword'ü varsa transform et
        if !source.contains("import ") && !source.contains("export ") {
            return source.to_string();
        }
        
        // Önce named imports'u transform et - custom import function kullan
        let mut transformed = NAMED_IMPORT_REGEX.replace_all(source, "const { $1 } = await __kiren_import('$2');").to_string();
        
        // Sonra default imports'u transform et - custom import function kullan
        transformed = DEFAULT_IMPORT_REGEX.replace_all(&transformed, "const $1 = (await __kiren_import('$2')).default;").to_string();
        
        // Export statement'ları comment out et (basit replace)
        if transformed.contains("export ") {
            transformed = transformed.replace("export {", "// export {");
            transformed = transformed.replace("export default", "// export default");
        }
        
        // Async wrapper sadece transformation yapıldıysa ekle
        if transformed != source {
            format!("(async () => {{\n{}\n}})();", transformed)
        } else {
            transformed
        }
    }

    pub fn execute_with_callbacks(
        &mut self,
        source: &str,
        process_callbacks: bool,
    ) -> Result<String> {
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        
        // Fast context creation
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);
        let global = context.global(scope);

        // Setup all APIs - simplified for now
        crate::api::console::setup_console(scope, context)?;
        crate::api::errors::setup_error_handling(scope, context)?;
        crate::api::npm_simple::setup_npm_compatibility(scope, context)?;
        crate::api::express::setup_express(scope, context)?;
        timers::setup_timers(scope, context)?;
        fetch::setup_fetch(scope, context)?;
        buffer::initialize_buffer_api(scope, global)?;
        events::initialize_events_api(scope, global)?;
        streams::initialize_streams_api(scope, global)?;
        filesystem::setup_filesystem(scope, context)?;
        process::setup_process(scope, context)?;
        environment::setup_environment(scope, context)?;
        http::setup_http(scope, context)?;
        test::setup_test_framework(scope, context)?;
        crate::modules::es_modules_simple::setup_es_modules(scope, context)?;
        crate::modules::commonjs_simple::setup_commonjs(scope, context)?;

        // Execute JavaScript with better error handling
        let source_string = v8::String::new(scope, source).unwrap();
        let filename = v8::String::new(scope, "<eval>").unwrap();

        // Create script origin for better stack traces
        let undefined_val = v8::undefined(scope);
        let origin = v8::ScriptOrigin::new(
            scope,
            filename.into(),
            0,                    // line offset
            0,                    // column offset
            false,                // is shared cross origin
            0,                    // script id
            undefined_val.into(), // source map url
            false,                // is opaque
            false,                // is wasm
            false,                // is module
        );

        let script = match v8::Script::compile(scope, source_string, Some(&origin)) {
            Some(script) => script,
            None => {
                return Err(anyhow::anyhow!(
                    "Syntax Error: Failed to compile JavaScript"
                ));
            }
        };

        match script.run(scope) {
            Some(result) => {
                let result_str = result.to_string(scope).unwrap();
                let result_string = result_str.to_rust_string_lossy(scope);

                // Process any queued timer callbacks if requested
                if process_callbacks {
                    timers::process_timer_callbacks(scope)?;
                }

                Ok(result_string)
            }
            None => Err(anyhow::anyhow!(
                "Runtime Error: Failed to execute JavaScript"
            )),
        }
    }

    // This function is not used anymore - all setup is done in execute_with_callbacks
    fn _unused_setup_apis(&mut self, scope: &mut v8::ContextScope<v8::HandleScope>, context: v8::Local<v8::Context>) -> Result<()> {
        Ok(())
    }


}

fn console_log(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    let mut output = String::new();
    for i in 0..args.length() {
        if i > 0 {
            output.push(' ');
        }
        let arg = args.get(i);
        let str_val = arg.to_string(scope).unwrap();
        output.push_str(&str_val.to_rust_string_lossy(scope));
    }
    println!("{}", output);
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

    CONSOLE_TIMERS.insert(label, Instant::now());
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

    if let Some((_, start_time)) = CONSOLE_TIMERS.remove(&label) {
        let elapsed = start_time.elapsed();
        println!("{}: {:.3}ms", label, elapsed.as_secs_f64() * 1000.0);
    } else {
        println!("Timer '{}' does not exist", label);
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // Cleanup is handled automatically by V8
        // No need for unsafe dispose in modern V8
    }
}

// Performance utilities
static MODULE_CACHE: Lazy<Arc<Mutex<std::collections::HashMap<String, String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(std::collections::HashMap::new())));

static IMPORT_META_RESOLVE: Lazy<Arc<Mutex<std::collections::HashMap<String, String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(std::collections::HashMap::new())));

// Precompiled regex patterns for performance
static NAMED_IMPORT_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r#"import\s+\{([^}]+)\}\s+from\s+['"]([^'"]+)['"];?"#).unwrap()
});

static DEFAULT_IMPORT_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r#"import\s+(\w+)\s+from\s+['"]([^'"]+)['"];?"#).unwrap()
});

