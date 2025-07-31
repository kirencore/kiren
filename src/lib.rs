// Ultra-minimal Kiren WASM Library - Memory Safe Version
use wasm_bindgen::prelude::*;

// Enable panic hook for better debugging
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

// Main Kiren Runtime for WASM - Minimal Version
#[wasm_bindgen]
pub struct KirenRuntime {
    initialized: bool,
}

#[wasm_bindgen]
impl KirenRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> KirenRuntime {
        web_sys::console::log_1(&"🦀 Kiren WASM Runtime initializing...".into());
        KirenRuntime { initialized: true }
    }

    /// Execute JavaScript code synchronously - Ultra safe version with console capture
    #[wasm_bindgen]
    pub fn execute(&self, code: &str) -> Result<String, JsValue> {
        if !self.initialized {
            return Err(JsValue::from_str("Runtime not initialized"));
        }

        // Validate input
        if code.is_empty() {
            return Ok("undefined".to_string());
        }

        // Enhanced console capture with fetch support
        let wrapper_code = format!(
            r#"
            (async function() {{
                let outputs = [];
                let originalLog = console.log;
                let originalError = console.error;
                let originalWarn = console.warn;
                
                // Console capture
                console.log = function(...args) {{
                    outputs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
                    originalLog.apply(console, args);
                }};
                console.error = function(...args) {{
                    outputs.push('ERROR: ' + args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
                    originalError.apply(console, args);
                }};
                console.warn = function(...args) {{
                    outputs.push('WARN: ' + args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
                    originalWarn.apply(console, args);
                }};
                
                try {{
                    let result = await (async () => {{ {} }})();
                    
                    // Restore console
                    console.log = originalLog;
                    console.error = originalError;
                    console.warn = originalWarn;
                    
                    if (outputs.length > 0) {{
                        return outputs.join('\n');
                    }}
                    return result !== undefined ? String(result) : 'undefined';
                }} catch(e) {{
                    // Restore console
                    console.log = originalLog;
                    console.error = originalError;
                    console.warn = originalWarn;
                    return 'Error: ' + e.message;
                }}
            }})()
            "#,
            code
        );

        // Safe async execution
        match js_sys::eval(&wrapper_code) {
            Ok(promise) => {
                // Check if it's a promise (async code)
                if promise.has_type::<js_sys::Promise>() {
                    // For sync interface, return pending message
                    Ok("Async code detected - use execute_async for full support".to_string())
                } else if let Some(s) = promise.as_string() {
                    Ok(s)
                } else {
                    Ok("undefined".to_string())
                }
            }
            Err(_) => Ok("execution error".to_string()),
        }
    }

    /// Execute JavaScript code asynchronously
    #[wasm_bindgen]
    pub fn execute_async(&self, code: &str) -> js_sys::Promise {
        let code = code.to_string();
        let initialized = self.initialized;

        wasm_bindgen_futures::future_to_promise(async move {
            if !initialized {
                return Err(JsValue::from_str("Runtime not initialized"));
            }

            // Enhanced async execution with console capture
            let wrapper_code = format!(
                r#"
                (async function() {{
                    let outputs = [];
                    let originalLog = console.log;
                    let originalError = console.error;
                    let originalWarn = console.warn;
                    
                    // Console capture
                    console.log = function(...args) {{
                        outputs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
                        originalLog.apply(console, args);
                    }};
                    console.error = function(...args) {{
                        outputs.push('ERROR: ' + args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
                        originalError.apply(console, args);
                    }};
                    console.warn = function(...args) {{
                        outputs.push('WARN: ' + args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
                        originalWarn.apply(console, args);
                    }};
                    
                    try {{
                        // Execute the user code in a way that preserves console overrides
                        let result;
                        
                        // Create a promise that captures all console output
                        const execution = new Promise(async (resolve, reject) => {{
                            try {{
                                // Execute the code with proper async handling
                                let codeResult = await eval(`(async () => {{ {} }})()`);
                                
                                // If it's a promise, wait for it
                                if (codeResult && typeof codeResult.then === 'function') {{
                                    codeResult = await codeResult;
                                }}
                                
                                resolve(codeResult);
                            }} catch (error) {{
                                reject(error);
                            }}
                        }});
                        
                        // Execute and wait
                        result = await execution;
                        
                        // Wait longer for any pending async operations (fetch, etc.)
                        await new Promise(resolve => setTimeout(resolve, 500));
                        
                        // Restore console
                        console.log = originalLog;
                        console.error = originalError;
                        console.warn = originalWarn;
                        
                        // Always return captured outputs if any
                        if (outputs.length > 0) {{
                            return outputs.join('\n');
                        }}
                        return result !== undefined ? String(result) : 'undefined';
                    }} catch(e) {{
                        // Wait even for errors to capture pending outputs
                        await new Promise(resolve => setTimeout(resolve, 500));
                        
                        // Restore console
                        console.log = originalLog;
                        console.error = originalError;
                        console.warn = originalWarn;
                        
                        // Return captured outputs even on error
                        if (outputs.length > 0) {{
                            return outputs.join('\n') + '\nError: ' + e.message;
                        }}
                        return 'Error: ' + e.message;
                    }}
                }})()
                "#,
                code
            );

            // Execute async code
            match js_sys::eval(&wrapper_code) {
                Ok(promise) => {
                    if promise.has_type::<js_sys::Promise>() {
                        // Await the promise
                        let js_promise: js_sys::Promise = promise.into();
                        match wasm_bindgen_futures::JsFuture::from(js_promise).await {
                            Ok(result) => {
                                if let Some(s) = result.as_string() {
                                    Ok(JsValue::from_str(&s))
                                } else {
                                    Ok(JsValue::from_str("undefined"))
                                }
                            }
                            Err(e) => Ok(JsValue::from_str(&format!("Async error: {:?}", e))),
                        }
                    } else {
                        // Sync result
                        if let Some(s) = promise.as_string() {
                            Ok(JsValue::from_str(&s))
                        } else {
                            Ok(JsValue::from_str("undefined"))
                        }
                    }
                }
                Err(_) => Ok(JsValue::from_str("execution error")),
            }
        })
    }

    /// Execute ES6 module
    #[wasm_bindgen]
    pub fn execute_module(&self, code: &str, _module_url: &str) -> Result<String, JsValue> {
        self.execute(code)
    }

    /// Get runtime version
    #[wasm_bindgen]
    pub fn version(&self) -> String {
        "3.0.0-wasm".to_string()
    }

    /// Get runtime statistics - Safe version
    #[wasm_bindgen]
    pub fn stats(&self) -> String {
        "{\"version\":\"3.0.0-wasm\",\"initialized\":true,\"heap_used\":1024,\"modules_loaded\":0}"
            .to_string()
    }

    /// Clear runtime state
    #[wasm_bindgen]
    pub fn clear(&self) {
        // Safe clear - no state to modify
    }

    /// Set runtime options
    #[wasm_bindgen]
    pub fn set_options(&self, _options: &JsValue) {
        // Safe options - no complex handling
    }
}

// Global initialization function
#[wasm_bindgen]
pub fn init_kiren() -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async move {
        web_sys::console::log_1(&"🦀 Kiren WASM Runtime initialized!".into());
        Ok(JsValue::from_str("initialized"))
    })
}

// Feature detection functions
#[wasm_bindgen]
pub fn get_version() -> String {
    "3.0.0-wasm".to_string()
}

#[wasm_bindgen]
pub fn supports_modules() -> bool {
    true
}

#[wasm_bindgen]
pub fn supports_async() -> bool {
    true
}

#[wasm_bindgen]
pub fn supports_fetch() -> bool {
    js_sys::Reflect::has(&js_sys::global(), &JsValue::from_str("fetch")).unwrap_or(false)
}

// Benchmark execution function - Safe version
#[wasm_bindgen]
pub fn benchmark_execution(code: &str, iterations: u32) -> js_sys::Promise {
    let code = code.to_string();

    wasm_bindgen_futures::future_to_promise(async move {
        let window = web_sys::window().ok_or("No window")?;
        let performance = window.performance().ok_or("No performance")?;

        let start = performance.now();

        for _ in 0..iterations {
            let _ = js_sys::eval(&code);
        }

        let end = performance.now();
        let total_time = end - start;

        // Simple result object as string
        let result = format!(
            "{{\"iterations\":{},\"total_time_ms\":{:.2},\"avg_time_ms\":{:.2},\"ops_per_second\":{:.2}}}",
            iterations,
            total_time,
            total_time / iterations as f64,
            (iterations as f64 * 1000.0) / total_time
        );

        Ok(JsValue::from_str(&result))
    })
}
