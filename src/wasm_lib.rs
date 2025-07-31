// Minimal Kiren WASM Library for Playground
// This is a simplified version that works in WebAssembly

use wasm_bindgen::prelude::*;
use web_sys::{console, Performance};
use js_sys::{Date, Error, JSON, Object, Reflect};

// Enable console.log from Rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

// Runtime statistics
#[wasm_bindgen]
pub struct RuntimeStats {
    version: String,
    initialized: bool,
    heap_used: u32,
    modules_loaded: u32,
}

#[wasm_bindgen]
impl RuntimeStats {
    #[wasm_bindgen(getter)]
    pub fn version(&self) -> String {
        self.version.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn initialized(&self) -> bool {
        self.initialized
    }

    #[wasm_bindgen(getter)]
    pub fn heap_used(&self) -> u32 {
        self.heap_used
    }

    #[wasm_bindgen(getter)]
    pub fn modules_loaded(&self) -> u32 {
        self.modules_loaded
    }
}

// Main Kiren Runtime for WASM
#[wasm_bindgen]
pub struct KirenRuntime {
    initialized: bool,
    modules_loaded: u32,
}

#[wasm_bindgen]
impl KirenRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> KirenRuntime {
        console_log!("🦀 Initializing Kiren WASM Runtime...");
        KirenRuntime {
            initialized: true,
            modules_loaded: 0,
        }
    }

    /// Execute JavaScript code synchronously
    #[wasm_bindgen]
    pub fn execute(&mut self, code: &str) -> Result<String, JsValue> {
        if !self.initialized {
            return Err(JsValue::from_str("Runtime not initialized"));
        }

        // Get the global window object
        let window = web_sys::window().ok_or("No global window object")?;
        let global = window.as_ref();

        // Create a safe execution context
        let result = match self.execute_safe(code, global) {
            Ok(val) => val,
            Err(e) => return Err(e),
        };

        Ok(result)
    }

    /// Execute JavaScript code asynchronously
    #[wasm_bindgen]
    pub fn execute_async(&mut self, code: &str) -> js_sys::Promise {
        let code = code.to_string();
        let initialized = self.initialized;

        wasm_bindgen_futures::future_to_promise(async move {
            if !initialized {
                return Err(JsValue::from_str("Runtime not initialized"));
            }

            // For async execution, wrap code in an async function
            let async_code = format!("(async () => {{ {} }})()", code);
            
            // Get global context
            let window = web_sys::window().ok_or("No global window object")?;
            
            // Evaluate the async code
            let result = js_sys::eval(&async_code)
                .map_err(|e| JsValue::from_str(&format!("Execution error: {:?}", e)))?;

            // If it's a promise, await it
            if result.has_type::<js_sys::Promise>() {
                let promise: js_sys::Promise = result.into();
                let awaited = wasm_bindgen_futures::JsFuture::from(promise).await?;
                
                // Convert result to string
                if awaited.is_undefined() {
                    Ok(JsValue::from_str("undefined"))
                } else {
                    let stringified = JSON::stringify(&awaited)
                        .unwrap_or(JsValue::from_str("undefined"));
                    Ok(stringified)
                }
            } else {
                // Convert synchronous result to string
                let stringified = JSON::stringify(&result)
                    .unwrap_or(JsValue::from_str("undefined"));
                Ok(stringified)
            }
        })
    }

    /// Execute ES6 module
    #[wasm_bindgen]
    pub fn execute_module(&mut self, code: &str, _module_url: &str) -> Result<String, JsValue> {
        console_log!("Executing module: {}", _module_url);
        self.modules_loaded += 1;
        self.execute(code)
    }

    /// Get runtime version
    #[wasm_bindgen]
    pub fn version(&self) -> String {
        "3.0.0-wasm".to_string()
    }

    /// Get runtime statistics
    #[wasm_bindgen]
    pub fn stats(&self) -> RuntimeStats {
        // Try to get memory usage from performance API
        let heap_used = web_sys::window()
            .and_then(|w| w.performance())
            .and_then(|p| {
                let memory = Reflect::get(&p, &JsValue::from_str("memory")).ok()?;
                let used_heap = Reflect::get(&memory, &JsValue::from_str("usedJSHeapSize")).ok()?;
                used_heap.as_f64().map(|v| (v / 1024.0 / 1024.0) as u32)
            })
            .unwrap_or(1024); // Default 1MB

        RuntimeStats {
            version: self.version(),
            initialized: self.initialized,
            heap_used,
            modules_loaded: self.modules_loaded,
        }
    }

    /// Clear runtime state
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        console_log!("Clearing runtime state...");
        self.modules_loaded = 0;
    }

    /// Set runtime options
    #[wasm_bindgen]
    pub fn set_options(&mut self, _options: &JsValue) {
        // For now, just log that options were set
        console_log!("Runtime options updated");
    }

    // Private helper method for safe execution
    fn execute_safe(&self, code: &str, _global: &JsValue) -> Result<String, JsValue> {
        // Capture console output
        let original_console_log = web_sys::window()
            .and_then(|w| Reflect::get(&w, &JsValue::from_str("console")).ok())
            .and_then(|c| Reflect::get(&c, &JsValue::from_str("log")).ok());

        let mut captured_output = Vec::new();
        
        // Try to execute the code
        let result = js_sys::eval(code)
            .map_err(|e| {
                let error_msg = format!("Execution error: {:?}", e);
                JsValue::from_str(&error_msg)
            })?;

        // Convert result to string
        let output = if result.is_undefined() {
            "undefined".to_string()
        } else if result.is_null() {
            "null".to_string()
        } else {
            // Try to stringify the result
            match JSON::stringify(&result) {
                Ok(stringified) => stringified.as_string().unwrap_or("undefined".to_string()),
                Err(_) => {
                    // Fallback to toString
                    match result.as_string() {
                        Some(s) => s,
                        None => "undefined".to_string(),
                    }
                }
            }
        };

        Ok(output)
    }
}

// Global initialization function
#[wasm_bindgen]
pub fn init_kiren() -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async move {
        console_log!("🦀 Kiren WASM Runtime initialized successfully!");
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
    // Check if fetch is available in the global scope
    web_sys::window()
        .and_then(|w| Reflect::get(&w, &JsValue::from_str("fetch")).ok())
        .map(|f| !f.is_undefined())
        .unwrap_or(false)
}

// Benchmark execution function
#[wasm_bindgen]
pub fn benchmark_execution(code: &str, iterations: u32) -> js_sys::Promise {
    let code = code.to_string();
    
    wasm_bindgen_futures::future_to_promise(async move {
        let window = web_sys::window().ok_or("No global window object")?;
        let performance = window.performance().ok_or("No performance API")?;
        
        let start = performance.now();
        
        for _ in 0..iterations {
            js_sys::eval(&code)
                .map_err(|e| JsValue::from_str(&format!("Benchmark error: {:?}", e)))?;
        }
        
        let end = performance.now();
        let total_time = end - start;
        
        let result = Object::new();
        Reflect::set(&result, &JsValue::from_str("iterations"), &JsValue::from_f64(iterations as f64))?;
        Reflect::set(&result, &JsValue::from_str("total_time_ms"), &JsValue::from_f64(total_time))?;
        Reflect::set(&result, &JsValue::from_str("avg_time_ms"), &JsValue::from_f64(total_time / iterations as f64))?;
        Reflect::set(&result, &JsValue::from_str("ops_per_second"), &JsValue::from_f64((iterations as f64 * 1000.0) / total_time))?;
        
        Ok(result.into())
    })
}