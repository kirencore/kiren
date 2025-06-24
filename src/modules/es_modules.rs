use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use v8;
use crate::package::{GlobalCache, UrlImportHandler};

// ES Module resolver and loader
pub struct ModuleLoader {
    module_map: Arc<Mutex<HashMap<String, v8::Global<v8::Module>>>>,
    base_path: PathBuf,
    url_handler: Option<UrlImportHandler>,
}

impl ModuleLoader {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            module_map: Arc::new(Mutex::new(HashMap::new())),
            base_path: base_path.as_ref().to_path_buf(),
            url_handler: None,
        }
    }

    pub fn with_url_support(base_path: impl AsRef<Path>, cache: GlobalCache) -> Self {
        Self {
            module_map: Arc::new(Mutex::new(HashMap::new())),
            base_path: base_path.as_ref().to_path_buf(),
            url_handler: Some(UrlImportHandler::new(cache)),
        }
    }

    pub async fn load_module(
        &mut self,
        scope: &mut v8::HandleScope,
        specifier: &str,
        referrer: Option<&str>,
    ) -> Result<v8::Local<v8::Module>> {
        // Check if this is a URL import
        if self.is_url_import(specifier) {
            return self.load_url_module(scope, specifier, referrer).await;
        }

        let resolved_path = self.resolve_module_path(specifier)?;
        let module_key = resolved_path.to_string_lossy().to_string();

        // Check if module is already cached
        {
            let module_map = self.module_map.lock().unwrap();
            if let Some(global_module) = module_map.get(&module_key) {
                return Ok(v8::Local::new(scope, global_module));
            }
        }

        // Read and compile module
        let source = std::fs::read_to_string(&resolved_path)
            .map_err(|e| anyhow::anyhow!("Failed to read module {}: {}", specifier, e))?;

        let module = self.compile_module(scope, &source, &resolved_path.to_string_lossy())?;

        // Cache the compiled module
        {
            let mut module_map = self.module_map.lock().unwrap();
            let global_module = v8::Global::new(scope, module);
            module_map.insert(module_key, global_module);
        }

        Ok(module)
    }

    async fn load_url_module(
        &mut self,
        scope: &mut v8::HandleScope,
        specifier: &str,
        referrer: Option<&str>,
    ) -> Result<v8::Local<v8::Module>> {
        if let Some(ref mut url_handler) = self.url_handler {
            // Check if module is already cached
            {
                let module_map = self.module_map.lock().unwrap();
                if let Some(global_module) = module_map.get(specifier) {
                    return Ok(v8::Local::new(scope, global_module));
                }
            }

            // Fetch URL module content
            let source = url_handler.resolve_import(specifier, referrer).await?;
            let module = self.compile_module(scope, &source, specifier)?;

            // Cache the compiled module
            {
                let mut module_map = self.module_map.lock().unwrap();
                let global_module = v8::Global::new(scope, module);
                module_map.insert(specifier.to_string(), global_module);
            }

            Ok(module)
        } else {
            Err(anyhow::anyhow!("URL imports not supported - missing URL handler"))
        }
    }

    fn is_url_import(&self, specifier: &str) -> bool {
        specifier.starts_with("http://") || specifier.starts_with("https://")
    }

    fn resolve_module_path(&self, specifier: &str) -> Result<PathBuf> {
        // Handle relative imports
        if specifier.starts_with("./") || specifier.starts_with("../") {
            let path = self.base_path.join(specifier);
            return Ok(self.resolve_file_extension(path)?);
        }

        // Handle absolute paths
        if specifier.starts_with('/') {
            let path = PathBuf::from(specifier);
            return Ok(self.resolve_file_extension(path)?);
        }

        // Handle bare imports (node_modules style)
        // For now, just look in the current directory
        let path = self.base_path.join(specifier);
        Ok(self.resolve_file_extension(path)?)
    }

    fn resolve_file_extension(&self, mut path: PathBuf) -> Result<PathBuf> {
        // If path already has extension, use it
        if path.extension().is_some() && path.exists() {
            return Ok(path);
        }

        // Try common JavaScript extensions
        let extensions = [".js", ".mjs", ".ts"];
        for ext in &extensions {
            path.set_extension(&ext[1..]);
            if path.exists() {
                return Ok(path);
            }
        }

        // Try index files
        if path.is_dir() {
            for ext in &extensions {
                let index_path = path.join(format!("index{}", ext));
                if index_path.exists() {
                    return Ok(index_path);
                }
            }
        }

        Err(anyhow::anyhow!("Cannot resolve module: {}", path.display()))
    }

    fn compile_module(
        &self,
        scope: &mut v8::HandleScope,
        source: &str,
        path: &str,
    ) -> Result<v8::Local<v8::Module>> {
        let source_text = v8::String::new(scope, source).unwrap();
        let resource_name = v8::String::new(scope, path).unwrap();

        let origin = v8::ScriptOrigin::new(
            scope,
            resource_name.into(),
            0,  // line_offset
            0,  // column_offset
            false, // is_shared_cross_origin
            0,  // script_id
            v8::undefined(scope).into(), // source_map_url
            false, // is_opaque
            false, // is_wasm
            true,  // is_module
        );

        let source = v8::script_compiler::Source::new(source_text, Some(&origin));

        match v8::script_compiler::compile_module(scope, source) {
            Some(module) => Ok(module),
            None => {
                let exception = scope.get_last_exception();
                let error_msg = if let Some(exc) = exception {
                    let exc_str = exc.to_string(scope).unwrap();
                    exc_str.to_rust_string_lossy(scope)
                } else {
                    "Unknown module compilation error".to_string()
                };
                Err(anyhow::anyhow!("Module compilation failed: {}", error_msg))
            }
        }
    }

    pub fn instantiate_module(
        &self,
        scope: &mut v8::HandleScope,
        module: v8::Local<v8::Module>,
        context: v8::Local<v8::Context>,
    ) -> Result<()> {
        let result = module.instantiate_module(context, Self::module_resolve_callback);
        
        if result.is_none() {
            let exception = scope.get_last_exception();
            let error_msg = if let Some(exc) = exception {
                let exc_str = exc.to_string(scope).unwrap();
                exc_str.to_rust_string_lossy(scope)
            } else {
                "Unknown module instantiation error".to_string()
            };
            return Err(anyhow::anyhow!("Module instantiation failed: {}", error_msg));
        }

        Ok(())
    }

    pub fn evaluate_module(
        &self,
        scope: &mut v8::HandleScope,
        module: v8::Local<v8::Module>,
    ) -> Result<v8::Local<v8::Value>> {
        match module.evaluate(scope) {
            Some(result) => Ok(result),
            None => {
                let exception = scope.get_last_exception();
                let error_msg = if let Some(exc) = exception {
                    let exc_str = exc.to_string(scope).unwrap();
                    exc_str.to_rust_string_lossy(scope)
                } else {
                    "Unknown module evaluation error".to_string()
                };
                Err(anyhow::anyhow!("Module evaluation failed: {}", error_msg))
            }
        }
    }

    // Module resolver callback for V8
    fn module_resolve_callback<'s>(
        context: v8::Local<'s, v8::Context>,
        specifier: v8::Local<'s, v8::String>,
        _import_assertions: v8::Local<'s, v8::FixedArray>,
        _referrer: v8::Local<'s, v8::Module>,
    ) -> Option<v8::Local<'s, v8::Module>> {
        let scope = &mut unsafe { v8::CallbackScope::new(context) };
        let specifier_str = specifier.to_rust_string_lossy(scope);
        
        // This is a simplified version - in practice, you'd want to pass
        // the ModuleLoader instance here to resolve properly
        println!("Module resolve callback: {}", specifier_str);
        
        // Return None for now - this needs proper implementation
        None
    }
}

// Setup ES Modules support in the global context
pub fn setup_es_modules(
    scope: &mut v8::HandleScope, 
    context: v8::Local<v8::Context>
) -> Result<()> {
    // Add import() function for dynamic imports
    let global = context.global(scope);
    let import_key = v8::String::new(scope, "import").unwrap();
    let import_template = v8::FunctionTemplate::new(scope, dynamic_import);
    let import_function = import_template.get_function(scope).unwrap();
    
    global.set(scope, import_key.into(), import_function.into());
    
    Ok(())
}

// Dynamic import implementation
fn dynamic_import(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "import() requires a module specifier").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let specifier_arg = args.get(0);
    let specifier_str = specifier_arg.to_string(scope).unwrap();
    let specifier = specifier_str.to_rust_string_lossy(scope);

    // Create a promise for async module loading
    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);

    // For now, reject with "not implemented"
    let error_msg = v8::String::new(scope, 
        &format!("Dynamic import not yet implemented: {}", specifier)).unwrap();
    let error = v8::Exception::error(scope, error_msg);
    resolver.reject(scope, error);

    rv.set(promise.into());
}