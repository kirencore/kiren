use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use v8;

// CommonJS module system implementation
pub struct CommonJSLoader {
    module_cache: Arc<Mutex<HashMap<String, v8::Global<v8::Object>>>>,
    base_path: PathBuf,
}

impl CommonJSLoader {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            module_cache: Arc::new(Mutex::new(HashMap::new())),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub fn require_module(
        &self,
        scope: &mut v8::HandleScope,
        specifier: &str,
    ) -> Result<v8::Local<v8::Object>> {
        let resolved_path = self.resolve_module_path(specifier)?;
        let module_key = resolved_path.to_string_lossy().to_string();

        // Check module cache first
        {
            let cache = self.module_cache.lock().unwrap();
            if let Some(cached_module) = cache.get(&module_key) {
                return Ok(v8::Local::new(scope, cached_module));
            }
        }

        // Load and execute module
        let source = std::fs::read_to_string(&resolved_path)
            .map_err(|e| anyhow::anyhow!("Cannot find module '{}': {}", specifier, e))?;

        let module_exports = self.execute_module(scope, &source, &resolved_path)?;

        // Cache the module exports
        {
            let mut cache = self.module_cache.lock().unwrap();
            let global_exports = v8::Global::new(scope, module_exports);
            cache.insert(module_key, global_exports);
        }

        Ok(module_exports)
    }

    fn resolve_module_path(&self, specifier: &str) -> Result<PathBuf> {
        // Handle relative paths
        if specifier.starts_with("./") || specifier.starts_with("../") {
            let path = self.base_path.join(specifier);
            return self.resolve_file_extension(path);
        }

        // Handle absolute paths
        if specifier.starts_with('/') {
            let path = PathBuf::from(specifier);
            return self.resolve_file_extension(path);
        }

        // Handle node_modules style resolution
        let mut current_dir = self.base_path.clone();
        loop {
            let node_modules = current_dir.join("node_modules").join(specifier);
            if let Ok(resolved) = self.resolve_file_extension(node_modules.clone()) {
                return Ok(resolved);
            }

            // Try package.json main field
            let package_json = node_modules.join("package.json");
            if package_json.exists() {
                if let Ok(package_content) = std::fs::read_to_string(&package_json) {
                    if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&package_content) {
                        if let Some(main) = package_json.get("main").and_then(|v| v.as_str()) {
                            let main_path = node_modules.join(main);
                            if let Ok(resolved) = self.resolve_file_extension(main_path) {
                                return Ok(resolved);
                            }
                        }
                    }
                }
            }

            // Move up one directory
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                break;
            }
        }

        // Finally, try in the base directory
        let path = self.base_path.join(specifier);
        self.resolve_file_extension(path)
    }

    fn resolve_file_extension(&self, mut path: PathBuf) -> Result<PathBuf> {
        // If file exists as-is, return it
        if path.exists() && path.is_file() {
            return Ok(path);
        }

        // Try with .js extension
        if path.extension().is_none() {
            path.set_extension("js");
            if path.exists() {
                return Ok(path);
            }
        }

        // Try as directory with index.js
        path.pop();
        if path.is_dir() {
            let index_path = path.join("index.js");
            if index_path.exists() {
                return Ok(index_path);
            }
        }

        Err(anyhow::anyhow!("Cannot resolve module: {}", path.display()))
    }

    fn execute_module(
        &self,
        scope: &mut v8::HandleScope,
        source: &str,
        path: &Path,
    ) -> Result<v8::Local<v8::Object>> {
        // Create module context
        let exports = v8::Object::new(scope);
        let module_obj = v8::Object::new(scope);

        // Set module.exports = exports
        let exports_key = v8::String::new(scope, "exports").unwrap();
        module_obj.set(scope, exports_key.into(), exports.into());

        // Create require function for this module
        let require_key = v8::String::new(scope, "require").unwrap();
        let require_template = v8::FunctionTemplate::new(scope, |scope, args, mut rv| {
            if args.length() == 0 {
                let error = v8::String::new(scope, "require() missing path").unwrap();
                let exception = v8::Exception::type_error(scope, error);
                scope.throw_exception(exception);
                return;
            }

            let specifier_arg = args.get(0);
            let specifier_str = specifier_arg.to_string(scope).unwrap();
            let _specifier = specifier_str.to_rust_string_lossy(scope);

            // For now, return empty object
            // TODO: Implement proper require resolution
            let empty_obj = v8::Object::new(scope);
            rv.set(empty_obj.into());
        });
        let require_fn = require_template.get_function(scope).unwrap();

        // Wrap module source in CommonJS wrapper
        let wrapper = format!(
            "(function(exports, require, module, __filename, __dirname) {{\n{}\n}})",
            source
        );

        let wrapped_source = v8::String::new(scope, &wrapper).unwrap();
        let filename_str = v8::String::new(scope, &path.to_string_lossy()).unwrap();

        let origin = v8::ScriptOrigin::new(
            scope,
            filename_str.into(),
            0, // line_offset
            0, // column_offset
            false, // is_shared_cross_origin
            0, // script_id
            v8::undefined(scope).into(), // source_map_url
            false, // is_opaque
            false, // is_wasm
            false, // is_module
        );

        let script = match v8::Script::compile(scope, wrapped_source, Some(&origin)) {
            Some(script) => script,
            None => {
                return Err(anyhow::anyhow!("Failed to compile CommonJS module"));
            }
        };

        let wrapper_fn = match script.run(scope) {
            Some(result) => result,
            None => {
                return Err(anyhow::anyhow!("Failed to execute CommonJS wrapper"));
            }
        };

        // Call the wrapper function with CommonJS arguments
        if let Ok(function) = wrapper_fn.try_into::<v8::Function>() {
            let dirname = path.parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());
            
            let filename_val = v8::String::new(scope, &path.to_string_lossy()).unwrap();
            let dirname_val = v8::String::new(scope, &dirname).unwrap();
            
            let args = [
                exports.into(),
                require_fn.into(),
                module_obj.into(),
                filename_val.into(),
                dirname_val.into(),
            ];

            let _result = function.call(scope, v8::undefined(scope).into(), &args);
        }

        // Return the exports object
        Ok(exports)
    }
}

// Setup CommonJS support in the global context
pub fn setup_commonjs(
    scope: &mut v8::HandleScope,
    context: v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Add global require function
    let require_key = v8::String::new(scope, "require").unwrap();
    let require_template = v8::FunctionTemplate::new(scope, global_require);
    let require_function = require_template.get_function(scope).unwrap();

    global.set(scope, require_key.into(), require_function.into());

    // Add module object
    let module_key = v8::String::new(scope, "module").unwrap();
    let module_obj = v8::Object::new(scope);
    let exports_obj = v8::Object::new(scope);
    
    let exports_key = v8::String::new(scope, "exports").unwrap();
    module_obj.set(scope, exports_key.into(), exports_obj.into());
    
    global.set(scope, module_key.into(), module_obj.into());

    // Add exports object
    let exports_key = v8::String::new(scope, "exports").unwrap();
    global.set(scope, exports_key.into(), exports_obj.into());

    Ok(())
}

// Global require function implementation
fn global_require(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "require() missing path").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let specifier_arg = args.get(0);
    let specifier_str = specifier_arg.to_string(scope).unwrap();
    let specifier = specifier_str.to_rust_string_lossy(scope);

    // Handle built-in modules
    match specifier.as_str() {
        "fs" => {
            let fs_obj = v8::Object::new(scope);
            // Add fs methods here
            rv.set(fs_obj.into());
            return;
        }
        "path" => {
            let path_obj = v8::Object::new(scope);
            // Add path methods here
            rv.set(path_obj.into());
            return;
        }
        "http" => {
            let http_obj = v8::Object::new(scope);
            // Add http methods here
            rv.set(http_obj.into());
            return;
        }
        _ => {
            // For now, return empty object for other modules
            let empty_obj = v8::Object::new(scope);
            rv.set(empty_obj.into());
        }
    }
}