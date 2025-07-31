use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, Performance, Window};

/// WASM-compatible JavaScript engine
#[derive(Clone)]
pub struct WasmEngine {
    /// Console output buffer
    console_buffer: Arc<Mutex<Vec<String>>>,
    /// Module cache
    modules: Arc<Mutex<HashMap<String, String>>>,
    /// Runtime options
    options: Arc<Mutex<Value>>,
    /// Global objects and functions
    globals: Arc<Mutex<HashMap<String, JsValue>>>,
}

impl WasmEngine {
    pub fn new() -> Result<Self> {
        Ok(WasmEngine {
            console_buffer: Arc::new(Mutex::new(Vec::new())),
            modules: Arc::new(Mutex::new(HashMap::new())),
            options: Arc::new(Mutex::new(serde_json::json!({}))),
            globals: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Execute JavaScript code synchronously in browser context
    pub fn execute(&mut self, code: &str) -> Result<String> {
        let mut output = Vec::new();
        
        // Parse and execute JavaScript using browser's built-in eval
        let result = self.execute_in_browser_context(code, &mut output)?;
        
        // Capture console output
        if let Ok(mut buffer) = self.console_buffer.lock() {
            buffer.extend(output);
        }
        
        Ok(result)
    }

    /// Execute JavaScript code asynchronously
    pub async fn execute_async(&mut self, code: &str) -> Result<String> {
        let mut output = Vec::new();
        
        // For async execution, we need to handle promises
        let result = self.execute_async_in_browser_context(code, &mut output).await?;
        
        if let Ok(mut buffer) = self.console_buffer.lock() {
            buffer.extend(output);
        }
        
        Ok(result)
    }

    /// Execute ES6 module
    pub fn execute_module(&mut self, code: &str, module_url: &str) -> Result<String> {
        // Store module in cache
        if let Ok(mut modules) = self.modules.lock() {
            modules.insert(module_url.to_string(), code.to_string());
        }
        
        // Execute as module with import/export support
        let wrapped_code = self.wrap_as_module(code, module_url)?;
        self.execute(&wrapped_code)
    }

    /// Get heap usage (approximation for WASM)
    pub fn heap_usage(&self) -> u32 {
        // In WASM, we can't directly access V8 heap info
        // Return approximation based on our data structures
        let buffer_size = self.console_buffer
            .lock()
            .map(|b| b.len())
            .unwrap_or(0);
            
        let modules_size = self.modules
            .lock()
            .map(|m| m.len())
            .unwrap_or(0);
            
        (buffer_size + modules_size) as u32 * 100 // Rough estimate
    }

    /// Get number of loaded modules
    pub fn modules_count(&self) -> u32 {
        self.modules
            .lock()
            .map(|m| m.len())
            .unwrap_or(0) as u32
    }

    /// Clear runtime state
    pub fn clear(&mut self) -> Result<()> {
        if let Ok(mut buffer) = self.console_buffer.lock() {
            buffer.clear();
        }
        
        if let Ok(mut modules) = self.modules.lock() {
            modules.clear();
        }
        
        if let Ok(mut globals) = self.globals.lock() {
            globals.clear();
        }
        
        Ok(())
    }

    /// Set runtime options
    pub fn set_options(&mut self, options: Value) -> Result<()> {
        if let Ok(mut opts) = self.options.lock() {
            *opts = options;
        }
        Ok(())
    }

    /// Execute JavaScript in browser context with Kiren APIs
    fn execute_in_browser_context(&self, code: &str, output: &mut Vec<String>) -> Result<String> {
        // Create a safe execution context with Kiren APIs
        let enhanced_code = self.wrap_with_kiren_apis(code);
        
        // Use Function constructor for safer eval alternative
        let js_code = format!(
            r#"
            (function() {{
                // Kiren console implementation
                const kirenConsole = {{
                    log: (...args) => {{
                        const message = args.map(arg => 
                            typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
                        ).join(' ');
                        console.log(message);
                        return message;
                    }},
                    error: (...args) => {{
                        const message = args.map(arg => 
                            typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
                        ).join(' ');
                        console.error(message);
                        return message;
                    }},
                    warn: (...args) => {{
                        const message = args.map(arg => 
                            typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
                        ).join(' ');
                        console.warn(message);
                        return message;
                    }}
                }};
                
                // Replace console with Kiren console
                const originalConsole = console;
                console = kirenConsole;
                
                try {{
                    const result = eval(`{}`);
                    return result !== undefined ? String(result) : 'undefined';
                }} catch (error) {{
                    console.error('Execution error:', error.message);
                    throw error;
                }} finally {{
                    console = originalConsole;
                }}
            }})()
            "#,
            enhanced_code.replace('`', r#"\`"#)
        );

        // Execute using Function constructor
        match js_sys::Function::new_no_args(&js_code).call0(&JsValue::NULL) {
            Ok(result) => {
                let result_str = result.as_string().unwrap_or_else(|| {
                    js_sys::JSON::stringify(&result)
                        .unwrap_or(JsValue::from_str("undefined"))
                        .as_string()
                        .unwrap_or("undefined".to_string())
                });
                
                Ok(result_str)
            }
            Err(error) => {
                let error_msg = error.as_string()
                    .unwrap_or_else(|| "Unknown JavaScript error".to_string());
                output.push(format!("Error: {}", error_msg));
                Err(anyhow::anyhow!("JavaScript execution failed: {}", error_msg))
            }
        }
    }

    /// Execute JavaScript asynchronously in browser context
    async fn execute_async_in_browser_context(&self, code: &str, output: &mut Vec<String>) -> Result<String> {
        let enhanced_code = self.wrap_with_kiren_apis(code);
        
        let js_code = format!(
            r#"
            (async function() {{
                const kirenConsole = {{
                    log: (...args) => {{
                        const message = args.map(arg => 
                            typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
                        ).join(' ');
                        console.log(message);
                        return message;
                    }}
                }};
                
                const originalConsole = console;
                console = kirenConsole;
                
                try {{
                    const result = await eval(`(async () => {{ {} }})()`);
                    return result !== undefined ? String(result) : 'undefined';
                }} catch (error) {{
                    console.error('Async execution error:', error.message);
                    throw error;
                }} finally {{
                    console = originalConsole;
                }}
            }})()
            "#,
            enhanced_code.replace('`', r#"\`"#)
        );

        let promise = js_sys::Function::new_no_args(&js_code).call0(&JsValue::NULL)?;
        let future = JsFuture::from(js_sys::Promise::from(promise));
        
        match future.await {
            Ok(result) => {
                let result_str = result.as_string().unwrap_or_else(|| {
                    js_sys::JSON::stringify(&result)
                        .unwrap_or(JsValue::from_str("undefined"))
                        .as_string()
                        .unwrap_or("undefined".to_string())
                });
                
                Ok(result_str)
            }
            Err(error) => {
                let error_msg = error.as_string()
                    .unwrap_or_else(|| "Unknown async error".to_string());
                output.push(format!("Async Error: {}", error_msg));
                Err(anyhow::anyhow!("Async execution failed: {}", error_msg))
            }
        }
    }

    /// Wrap code with Kiren APIs and polyfills
    fn wrap_with_kiren_apis(&self, code: &str) -> String {
        format!(
            r#"
            // Kiren Runtime APIs for WASM
            
            // Process polyfill
            const process = {{
                env: {{}},
                argv: ['kiren'],
                version: '3.0.0-wasm',
                platform: 'browser',
                exit: (code) => console.log(`Process exit with code: ${{code}}`),
            }};
            
            // Buffer polyfill (basic)
            const Buffer = {{
                from: (data, encoding = 'utf8') => {{
                    if (typeof data === 'string') {{
                        return new TextEncoder().encode(data);
                    }}
                    return data;
                }},
                alloc: (size) => new Uint8Array(size),
                isBuffer: (obj) => obj instanceof Uint8Array,
            }};
            
            // setTimeout/setInterval with Web API
            const setTimeout = (callback, delay, ...args) => {{
                return window.setTimeout(() => callback(...args), delay);
            }};
            
            const setInterval = (callback, delay, ...args) => {{
                return window.setInterval(() => callback(...args), delay);
            }};
            
            const clearTimeout = (id) => window.clearTimeout(id);
            const clearInterval = (id) => window.clearInterval(id);
            
            // Fetch API (already available in browsers)
            const fetch = window.fetch.bind(window);
            
            // Crypto API
            const crypto = window.crypto;
            
            // Performance API
            const performance = window.performance;
            
            // User code starts here
            {}
            "#,
            code
        )
    }

    /// Wrap code as ES6 module
    fn wrap_as_module(&self, code: &str, module_url: &str) -> Result<String> {
        // For browser WASM, we simulate module behavior
        let wrapped = format!(
            r#"
            // Module: {}
            (function(exports, require, module, __filename, __dirname) {{
                const module = {{ exports: {{}} }};
                const exports = module.exports;
                const require = (moduleId) => {{
                    // Basic module resolution for WASM context
                    console.log(`Requiring module: ${{moduleId}}`);
                    return {{}};
                }};
                
                {}
                
                return module.exports;
            }})()
            "#,
            module_url,
            code
        );
        
        Ok(wrapped)
    }
}