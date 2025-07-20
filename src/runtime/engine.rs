use crate::api::{buffer, environment, events, fetch, filesystem, http, process, streams, test, timers, crypto, filesystem_improved, process_improved};
use crate::typescript;
use anyhow::Result;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex, Once, atomic::{AtomicBool, Ordering}};
use std::time::{Instant, Duration};
use tokio::time::sleep;
use v8;

static INIT: Once = Once::new();
static CONSOLE_TIMERS: Lazy<DashMap<String, Instant>> = Lazy::new(|| DashMap::new());

pub struct Engine {
    isolate: v8::OwnedIsolate,
    context_initialized: bool,
    event_loop_running: Arc<AtomicBool>,
    cached_context: Option<v8::Global<v8::Context>>,
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

        // Create isolate with optimized settings for performance
        let create_params = v8::CreateParams::default()
            .heap_limits(0, 128 * 1024 * 1024); // Set reasonable heap limit (128MB)
        let mut isolate = v8::Isolate::new(create_params);

        // Performance optimizations
        isolate.set_capture_stack_trace_for_uncaught_exceptions(false, 0); // Disable for performance
        isolate.set_allow_atomics_wait(false); // Not needed for most scripts

        Ok(Engine { 
            isolate,
            context_initialized: false,
            event_loop_running: Arc::new(AtomicBool::new(false)),
            cached_context: None,
        })
    }

    pub fn execute(&mut self, source: &str) -> Result<String> {
        self.execute_with_callbacks(source, true)
    }

    /// Execute JavaScript with continuous timer callback processing for long-running scripts
    pub async fn execute_with_event_loop(&mut self, source: &str) -> Result<String> {
        // Start the event loop
        self.event_loop_running.store(true, Ordering::SeqCst);
        
        // Clone the flag for the background task
        let event_loop_flag = self.event_loop_running.clone();
        
        // Start background timer callback processor
        let _background_task = tokio::spawn(async move {
            while event_loop_flag.load(Ordering::SeqCst) {
                // Process timer callbacks periodically
                if let Err(e) = Self::process_queued_callbacks().await {
                    eprintln!("Timer callback processing error: {}", e);
                }
                
                // Small delay to prevent busy waiting
                sleep(Duration::from_millis(10)).await;
            }
        });

        // Execute the main script
        let result = self.execute_with_callbacks(source, true)?;
        
        // Keep the event loop running for a bit to process timers
        sleep(Duration::from_millis(100)).await;
        
        // Keep processing callbacks until queue is empty or timeout
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(5); // 5 second timeout for cleanup
        
        while start_time.elapsed() < timeout {
            let had_callbacks = self.execute_with_callbacks("", true).is_ok();
            if !had_callbacks {
                break;
            }
            sleep(Duration::from_millis(50)).await;
        }
        
        // Stop the event loop
        self.event_loop_running.store(false, Ordering::SeqCst);
        
        Ok(result)
    }

    /// Process queued timer callbacks (static method for background task)
    async fn process_queued_callbacks() -> Result<()> {
        // This is a placeholder - we'll need a different approach since
        // we can't access V8 scope from background thread
        Ok(())
    }

    pub fn execute_module(&mut self, source: &str, module_name: &str) -> Result<String> {
        // Check cache first for performance
        {
            let cache = MODULE_CACHE.lock().unwrap();
            if let Some(cached_source) = cache.get(module_name) {
                return self.execute_with_callbacks(cached_source, true);
            }
        }
        
        // Check if it's TypeScript and transpile if needed
        let processed_source = if module_name.ends_with(".ts") || module_name.ends_with(".tsx") {
            match typescript::transpile_typescript_content(source) {
                Ok(js_code) => js_code,
                Err(e) => return Err(e),
            }
        } else {
            source.to_string()
        };

        // Static import'ları dynamic import'a dönüştür
        let transformed_source = self.transform_static_imports(&processed_source);
        
        // Cache the transformed source for future use
        {
            let mut cache = MODULE_CACHE.lock().unwrap();
            cache.insert(module_name.to_string(), transformed_source.clone());
        }
        
        // Normal execute ile çalıştır ama module context'te
        self.execute_with_callbacks(&transformed_source, true)
    }

    fn transform_static_imports(&self, source: &str) -> String {
        // Fast path: sadece import keyword'ü varsa transform et
        if !source.contains("import ") && !source.contains("export ") {
            return source.to_string();
        }
        
        let mut transformed = source.to_string();
        let mut has_imports = false;
        let mut exports = Vec::new();
        
        // Named imports'u transform et
        transformed = NAMED_IMPORT_REGEX.replace_all(&transformed, |caps: &regex::Captures| {
            has_imports = true;
            format!("const {{ {} }} = await __kiren_import('{}');", &caps[1], &caps[2])
        }).to_string();
        
        // Default imports'u transform et  
        transformed = DEFAULT_IMPORT_REGEX.replace_all(&transformed, |caps: &regex::Captures| {
            has_imports = true;
            format!("const {} = (await __kiren_import('{}')).default;", &caps[1], &caps[2])
        }).to_string();
        
        // Export function declarations - basit regex ile
        if transformed.contains("export function") {
            let export_fn_regex = regex::Regex::new(r"export\s+function\s+(\w+)").unwrap();
            transformed = export_fn_regex.replace_all(&transformed, "function $1").to_string();
            for caps in export_fn_regex.captures_iter(source) {
                exports.push(format!("{}: {}", &caps[1], &caps[1]));
            }
        }
        
        // Export const/let declarations
        if transformed.contains("export const") || transformed.contains("export let") {
            let export_const_regex = regex::Regex::new(r"export\s+(?:const|let)\s+(\w+)").unwrap();
            for caps in export_const_regex.captures_iter(source) {
                exports.push(format!("{}: {}", &caps[1], &caps[1]));
            }
            transformed = export_const_regex.replace_all(&transformed, "const $1").to_string();
        }
        
        // Export default - basit replace
        if transformed.contains("export default") {
            transformed = transformed.replace("export default", "const __default_export =");
            exports.push("default: __default_export".to_string());
        }
        
        // Eğer export'lar varsa module.exports ekle (CommonJS uyumluluğu için)
        if !exports.is_empty() {
            transformed += &format!("\n\n// Module exports for compatibility\nif (typeof module !== 'undefined' && module.exports) {{\n  Object.assign(module.exports, {{ {} }});\n}}", exports.join(", "));
        }
        
        // Async wrapper sadece transformation yapıldıysa ekle
        if has_imports || !exports.is_empty() {
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
        
        // Use cached context or create new one with API setup
        let context = if let Some(ref cached) = self.cached_context {
            v8::Local::new(scope, cached)
        } else {
            let new_context = v8::Context::new(scope);
            
            // Setup APIs only once when creating new context
            {
                let scope = &mut v8::ContextScope::new(scope, new_context);
                let global = new_context.global(scope);

                // Core APIs - setup once for performance
                let _ = crate::api::console::setup_console(scope, new_context);
                let _ = crate::api::errors::setup_error_handling(scope, new_context);
                let _ = timers::setup_timers(scope, new_context);
                let _ = fetch::setup_fetch(scope, new_context);
                let _ = buffer::initialize_buffer_api(scope, global);
                let _ = events::initialize_events_api(scope, global);
                let _ = streams::initialize_streams_api(scope, global);
                
                // Enhanced APIs
                let _ = filesystem_improved::setup_filesystem_improved(scope, new_context);
                let _ = process_improved::setup_process_object(scope, new_context);
                let _ = crypto::setup_crypto(scope, new_context);
                
                // Legacy APIs for compatibility
                let _ = filesystem::setup_filesystem(scope, new_context);
                let _ = process::setup_process(scope, new_context);
                let _ = environment::setup_environment(scope, new_context);
                let _ = http::setup_http(scope, new_context);
                let _ = test::setup_test_framework(scope, new_context);
                let _ = crate::modules::es_modules_simple::setup_es_modules(scope, new_context);
                let _ = crate::modules::commonjs_simple::setup_commonjs(scope, new_context);
                
                // Optional APIs (can be disabled for performance)
                let _ = crate::api::npm_simple::setup_npm_compatibility(scope, new_context);
                let _ = crate::api::express::setup_express(scope, new_context);
            }
            
            self.cached_context = Some(v8::Global::new(scope, new_context));
            self.context_initialized = true;
            new_context
        };
        
        let scope = &mut v8::ContextScope::new(scope, context);
        let _global = context.global(scope);

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

        // Use TryCatch for better error reporting
        let mut try_catch = v8::TryCatch::new(scope);
        
        let script = match v8::Script::compile(&mut try_catch, source_string, Some(&origin)) {
            Some(script) => script,
            None => {
                if let Some(exception) = try_catch.exception() {
                    let exception_str = exception.to_string(&mut try_catch).unwrap();
                    let error_msg = exception_str.to_rust_string_lossy(&mut try_catch);
                    
                    // Try to get stack trace if available
                    if let Some(stack_trace) = try_catch.stack_trace() {
                        let stack_str = stack_trace.to_string(&mut try_catch).unwrap();
                        let stack_msg = stack_str.to_rust_string_lossy(&mut try_catch);
                        return Err(anyhow::anyhow!("Syntax Error: {}\n{}", error_msg, stack_msg));
                    }
                    
                    return Err(anyhow::anyhow!("Syntax Error: {}", error_msg));
                }
                return Err(anyhow::anyhow!("Syntax Error: Failed to compile JavaScript"));
            }
        };

        match script.run(&mut try_catch) {
            Some(result) => {
                let result_str = result.to_string(&mut try_catch).unwrap();
                let result_string = result_str.to_rust_string_lossy(&mut try_catch);

                // Process any queued timer callbacks if requested
                if process_callbacks {
                    timers::process_timer_callbacks(&mut try_catch)?;
                }

                Ok(result_string)
            }
            None => {
                if let Some(exception) = try_catch.exception() {
                    let exception_str = exception.to_string(&mut try_catch).unwrap();
                    let error_msg = exception_str.to_rust_string_lossy(&mut try_catch);
                    
                    // Try to get stack trace if available
                    if let Some(stack_trace) = try_catch.stack_trace() {
                        let stack_str = stack_trace.to_string(&mut try_catch).unwrap();
                        let stack_msg = stack_str.to_rust_string_lossy(&mut try_catch);
                        return Err(anyhow::anyhow!("Runtime Error: {}\n{}", error_msg, stack_msg));
                    }
                    
                    return Err(anyhow::anyhow!("Runtime Error: {}", error_msg));
                }
                Err(anyhow::anyhow!("Runtime Error: Failed to execute JavaScript"))
            }
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

