use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use v8;

use super::resolver::{ModuleResolver, ModuleType, ResolvedModule};
use super::cache::{ModuleCache, ModuleState, LoadedModule};

/// Improved CommonJS module system with proper resolution and caching
pub struct CommonJSSystem {
    resolver: ModuleResolver,
    cache: ModuleCache,
    builtin_modules: HashMap<String, Box<dyn Fn(&mut v8::HandleScope) -> v8::Local<v8::Object> + Send + Sync>>,
}

impl CommonJSSystem {
    pub fn new() -> Self {
        let mut system = Self {
            resolver: ModuleResolver::new(),
            cache: ModuleCache::new(),
            builtin_modules: HashMap::new(),
        };

        // Register built-in modules
        system.register_builtins();
        system
    }

    /// Setup CommonJS globals in V8 context
    pub fn setup(&self, scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
        let global = context.global(scope);

        // Create require function with proper closure
        let require_key = v8::String::new(scope, "require").unwrap();
        let require_template = v8::FunctionTemplate::new(scope, Self::global_require);
        let require_function = require_template.get_function(scope).unwrap();

        // Store reference to this system in the require function
        // Note: In a real implementation, we'd use external data or a proper closure
        global.set(scope, require_key.into(), require_function.into());

        // Create module object
        let module_key = v8::String::new(scope, "module").unwrap();
        let module_obj = v8::Object::new(scope);
        
        // Create exports object
        let exports_obj = v8::Object::new(scope);
        let exports_key = v8::String::new(scope, "exports").unwrap();
        module_obj.set(scope, exports_key.into(), exports_obj.into());
        
        // Set module.id, module.filename, etc.
        let id_key = v8::String::new(scope, "id").unwrap();
        let id_value = v8::String::new(scope, ".").unwrap();
        module_obj.set(scope, id_key.into(), id_value.into());

        let filename_key = v8::String::new(scope, "filename").unwrap();
        let filename_value = v8::String::new(scope, "<main>").unwrap();
        module_obj.set(scope, filename_key.into(), filename_value.into());

        let loaded_key = v8::String::new(scope, "loaded").unwrap();
        let loaded_value = v8::Boolean::new(scope, false);
        module_obj.set(scope, loaded_key.into(), loaded_value.into());

        global.set(scope, module_key.into(), module_obj.into());

        // Set global exports reference
        let global_exports_key = v8::String::new(scope, "exports").unwrap();
        global.set(scope, global_exports_key.into(), exports_obj.into());

        // Set __dirname and __filename
        let dirname_key = v8::String::new(scope, "__dirname").unwrap();
        let dirname_value = v8::String::new(scope, std::env::current_dir()?.to_string_lossy().as_ref()).unwrap();
        global.set(scope, dirname_key.into(), dirname_value.into());

        let filename_global_key = v8::String::new(scope, "__filename").unwrap();
        let filename_global_value = v8::String::new(scope, "<main>").unwrap();
        global.set(scope, filename_global_key.into(), filename_global_value.into());

        Ok(())
    }

    /// Require a module with proper resolution and caching
    pub fn require_module(
        &mut self,
        scope: &mut v8::HandleScope,
        specifier: &str,
        from: Option<&std::path::Path>,
    ) -> Result<v8::Local<v8::Object>> {
        // Resolve the module
        let resolved = self.resolver.resolve(specifier, from)?;

        match resolved.module_type {
            ModuleType::BuiltIn => self.load_builtin_module(scope, specifier),
            ModuleType::CommonJS => self.load_commonjs_module(scope, resolved),
            ModuleType::JSON => self.load_json_module(scope, resolved),
            ModuleType::ESModule => {
                // For now, treat ES modules as CommonJS with warning
                println!("Warning: Loading ES module '{}' as CommonJS", specifier);
                self.load_commonjs_module(scope, resolved)
            }
        }
    }

    /// Load a built-in Node.js module
    fn load_builtin_module(
        &self,
        scope: &mut v8::HandleScope,
        specifier: &str,
    ) -> Result<v8::Local<v8::Object>> {
        if let Some(builder) = self.builtin_modules.get(specifier) {
            Ok(builder(scope))
        } else {
            Err(anyhow!("Built-in module '{}' not implemented", specifier))
        }
    }

    /// Load a CommonJS module with caching
    fn load_commonjs_module(
        &self,
        scope: &mut v8::HandleScope,
        resolved: ResolvedModule,
    ) -> Result<v8::Local<v8::Object>> {
        // Check cache first
        if let Some(state) = self.cache.get(&resolved.path) {
            match state {
                ModuleState::Loaded(module) => {
                    // Return cached exports
                    return Ok(v8::Local::new(scope, &module.exports));
                }
                ModuleState::Loading => {
                    // Circular dependency - return empty object for now
                    println!("Warning: Circular dependency detected for {}", resolved.path.display());
                    return Ok(v8::Object::new(scope));
                }
                ModuleState::Error(err) => {
                    return Err(anyhow!("Module previously failed to load: {}", err));
                }
            }
        }

        // Mark as loading
        self.cache.mark_loading(resolved.path.clone())?;

        // Create module context
        let result = self.execute_commonjs_module(scope, &resolved);

        match result {
            Ok(exports) => {
                // Store in cache
                let exports_global = v8::Global::new(scope, exports);
                let loaded_module = Arc::new(LoadedModule {
                    path: resolved.path.clone(),
                    exports: exports_global,
                    source: resolved.content,
                    dependencies: Vec::new(), // TODO: Track dependencies
                    loaded_at: std::time::SystemTime::now(),
                });

                self.cache.mark_loaded(resolved.path, loaded_module);
                Ok(exports)
            }
            Err(e) => {
                self.cache.mark_error(resolved.path, e.to_string());
                Err(e)
            }
        }
    }

    /// Execute CommonJS module in isolated context
    fn execute_commonjs_module(
        &self,
        scope: &mut v8::HandleScope,
        resolved: &ResolvedModule,
    ) -> Result<v8::Local<v8::Object>> {
        // Create new context for module
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        // Create module and exports objects
        let module_obj = v8::Object::new(scope);
        let exports_obj = v8::Object::new(scope);
        
        // Set up module object
        let exports_key = v8::String::new(scope, "exports").unwrap();
        module_obj.set(scope, exports_key.into(), exports_obj.into());

        let id_key = v8::String::new(scope, "id").unwrap();
        let id_value = v8::String::new(scope, resolved.path.to_string_lossy().as_ref()).unwrap();
        module_obj.set(scope, id_key.into(), id_value.into());

        let filename_key = v8::String::new(scope, "filename").unwrap();
        let filename_value = v8::String::new(scope, resolved.path.to_string_lossy().as_ref()).unwrap();
        module_obj.set(scope, filename_key.into(), filename_value.into());

        let loaded_key = v8::String::new(scope, "loaded").unwrap();
        let loaded_value = v8::Boolean::new(scope, false);
        module_obj.set(scope, loaded_key.into(), loaded_value.into());

        // Set up global variables
        let global = context.global(scope);
        
        let module_global_key = v8::String::new(scope, "module").unwrap();
        global.set(scope, module_global_key.into(), module_obj.into());

        let exports_global_key = v8::String::new(scope, "exports").unwrap();
        global.set(scope, exports_global_key.into(), exports_obj.into());

        // Set __dirname and __filename
        if let Some(parent) = resolved.path.parent() {
            let dirname_key = v8::String::new(scope, "__dirname").unwrap();
            let dirname_value = v8::String::new(scope, parent.to_string_lossy().as_ref()).unwrap();
            global.set(scope, dirname_key.into(), dirname_value.into());
        }

        let filename_global_key = v8::String::new(scope, "__filename").unwrap();
        global.set(scope, filename_global_key.into(), filename_value.into());

        // Set up require function for this module
        let require_key = v8::String::new(scope, "require").unwrap();
        let require_template = v8::FunctionTemplate::new(scope, Self::module_require);
        let require_function = require_template.get_function(scope).unwrap();
        global.set(scope, require_key.into(), require_function.into());

        // Wrap module code in CommonJS wrapper
        let wrapped_code = format!(
            "(function(exports, require, module, __filename, __dirname) {{\n{}\n}});",
            resolved.content
        );

        // Compile and execute
        let source_string = v8::String::new(scope, &wrapped_code).unwrap();
        let filename_script = v8::String::new(scope, resolved.path.to_string_lossy().as_ref()).unwrap();

        let mut try_catch = v8::TryCatch::new(scope);
        let script = v8::Script::compile(&mut try_catch, source_string, Some(&v8::ScriptOrigin::new(
            scope,
            filename_script.into(),
            0, 0, false, 0, v8::undefined(scope).into(), false, false, false,
        )));

        let script = match script {
            Some(script) => script,
            None => {
                let exception = try_catch.exception().unwrap();
                let exc_str = exception.to_string(&mut try_catch).unwrap();
                return Err(anyhow!("Compilation error: {}", exc_str.to_rust_string_lossy(&mut try_catch)));
            }
        };

        let result = script.run(&mut try_catch);
        let wrapper_function = match result {
            Some(result) => result,
            None => {
                let exception = try_catch.exception().unwrap();
                let exc_str = exception.to_string(&mut try_catch).unwrap();
                return Err(anyhow!("Execution error: {}", exc_str.to_rust_string_lossy(&mut try_catch)));
            }
        };

        // Call the wrapper function
        if wrapper_function.is_function() {
            let wrapper_fn = unsafe { v8::Local::<v8::Function>::cast(wrapper_function) };
            let args = [
                exports_obj.into(),
                require_function.into(),
                module_obj.into(),
                filename_value.into(),
                v8::String::new(scope, resolved.path.parent().unwrap_or_else(|| std::path::Path::new(".")).to_string_lossy().as_ref()).unwrap().into(),
            ];

            let mut try_catch = v8::TryCatch::new(scope);
            wrapper_fn.call(&mut try_catch, global.into(), &args);

            if try_catch.has_caught() {
                let exception = try_catch.exception().unwrap();
                let exc_str = exception.to_string(&mut try_catch).unwrap();
                return Err(anyhow!("Module execution error: {}", exc_str.to_rust_string_lossy(&mut try_catch)));
            }
        }

        // Mark as loaded
        let loaded_true = v8::Boolean::new(scope, true);
        module_obj.set(scope, loaded_key.into(), loaded_true.into());

        // Return module.exports
        let final_exports = module_obj.get(scope, exports_key.into()).unwrap();
        Ok(unsafe { v8::Local::<v8::Object>::cast(final_exports) })
    }

    /// Load JSON module
    fn load_json_module(
        &self,
        scope: &mut v8::HandleScope,
        resolved: ResolvedModule,
    ) -> Result<v8::Local<v8::Object>> {
        // Parse JSON content
        let json_str = v8::String::new(scope, &resolved.content).unwrap();
        let mut try_catch = v8::TryCatch::new(scope);
        
        let parsed = v8::json::parse(&mut try_catch, json_str);
        
        match parsed {
            Some(value) => {
                if value.is_object() {
                    Ok(unsafe { v8::Local::<v8::Object>::cast(value) })
                } else {
                    // Wrap primitive values in an object
                    let wrapper = v8::Object::new(scope);
                    let default_key = v8::String::new(scope, "default").unwrap();
                    wrapper.set(scope, default_key.into(), value);
                    Ok(wrapper)
                }
            }
            None => {
                let exception = try_catch.exception().unwrap();
                let exc_str = exception.to_string(&mut try_catch).unwrap();
                Err(anyhow!("JSON parsing error: {}", exc_str.to_rust_string_lossy(&mut try_catch)))
            }
        }
    }

    /// Register built-in modules
    fn register_builtins(&mut self) {
        // File system module
        self.builtin_modules.insert("fs".to_string(), Box::new(|scope| {
            Self::create_fs_module(scope)
        }));

        // Path module
        self.builtin_modules.insert("path".to_string(), Box::new(|scope| {
            Self::create_path_module(scope)
        }));

        // Events module
        self.builtin_modules.insert("events".to_string(), Box::new(|scope| {
            Self::create_events_module(scope)
        }));

        // Add more built-ins as needed
    }

    /// Create fs module
    fn create_fs_module(scope: &mut v8::HandleScope) -> v8::Local<v8::Object> {
        let fs_obj = v8::Object::new(scope);

        // readFileSync
        let read_file_key = v8::String::new(scope, "readFileSync").unwrap();
        let read_file_tmpl = v8::FunctionTemplate::new(scope, Self::fs_read_file_sync);
        let read_file_fn = read_file_tmpl.get_function(scope).unwrap();
        fs_obj.set(scope, read_file_key.into(), read_file_fn.into());

        // writeFileSync
        let write_file_key = v8::String::new(scope, "writeFileSync").unwrap();
        let write_file_tmpl = v8::FunctionTemplate::new(scope, Self::fs_write_file_sync);
        let write_file_fn = write_file_tmpl.get_function(scope).unwrap();
        fs_obj.set(scope, write_file_key.into(), write_file_fn.into());

        // existsSync
        let exists_key = v8::String::new(scope, "existsSync").unwrap();
        let exists_tmpl = v8::FunctionTemplate::new(scope, Self::fs_exists_sync);
        let exists_fn = exists_tmpl.get_function(scope).unwrap();
        fs_obj.set(scope, exists_key.into(), exists_fn.into());

        fs_obj
    }

    /// Create path module
    fn create_path_module(scope: &mut v8::HandleScope) -> v8::Local<v8::Object> {
        let path_obj = v8::Object::new(scope);

        // join
        let join_key = v8::String::new(scope, "join").unwrap();
        let join_tmpl = v8::FunctionTemplate::new(scope, Self::path_join);
        let join_fn = join_tmpl.get_function(scope).unwrap();
        path_obj.set(scope, join_key.into(), join_fn.into());

        // resolve
        let resolve_key = v8::String::new(scope, "resolve").unwrap();
        let resolve_tmpl = v8::FunctionTemplate::new(scope, Self::path_resolve);
        let resolve_fn = resolve_tmpl.get_function(scope).unwrap();
        path_obj.set(scope, resolve_key.into(), resolve_fn.into());

        // basename
        let basename_key = v8::String::new(scope, "basename").unwrap();
        let basename_tmpl = v8::FunctionTemplate::new(scope, Self::path_basename);
        let basename_fn = basename_tmpl.get_function(scope).unwrap();
        path_obj.set(scope, basename_key.into(), basename_fn.into());

        // dirname
        let dirname_key = v8::String::new(scope, "dirname").unwrap();
        let dirname_tmpl = v8::FunctionTemplate::new(scope, Self::path_dirname);
        let dirname_fn = dirname_tmpl.get_function(scope).unwrap();
        path_obj.set(scope, dirname_key.into(), dirname_fn.into());

        path_obj
    }

    /// Create events module (placeholder)
    fn create_events_module(scope: &mut v8::HandleScope) -> v8::Local<v8::Object> {
        let events_obj = v8::Object::new(scope);
        
        // Return our existing EventEmitter
        let emitter_key = v8::String::new(scope, "EventEmitter").unwrap();
        let emitter_fn = v8::Object::new(scope); // Placeholder
        events_obj.set(scope, emitter_key.into(), emitter_fn.into());

        events_obj
    }

    // V8 callback functions (static methods to work with V8's C API)

    fn global_require(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        mut rv: v8::ReturnValue,
    ) {
        // This is a placeholder - in a real implementation, we'd need to access
        // the CommonJSSystem instance through external data or a global registry
        rv.set(v8::Object::new(scope).into());
    }

    fn module_require(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        mut rv: v8::ReturnValue,
    ) {
        // This is a placeholder - in a real implementation, we'd need to access
        // the CommonJSSystem instance and current module context
        rv.set(v8::Object::new(scope).into());
    }

    fn fs_read_file_sync(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        mut rv: v8::ReturnValue,
    ) {
        if args.length() == 0 {
            let error = v8::String::new(scope, "readFileSync requires a file path").unwrap();
            let exception = v8::Exception::type_error(scope, error);
            scope.throw_exception(exception);
            return;
        }

        let path_arg = args.get(0);
        let path_str = path_arg.to_string(scope).unwrap();
        let path = path_str.to_rust_string_lossy(scope);

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let content_str = v8::String::new(scope, &content).unwrap();
                rv.set(content_str.into());
            }
            Err(e) => {
                let error = v8::String::new(scope, &format!("Cannot read file '{}': {}", path, e)).unwrap();
                let exception = v8::Exception::error(scope, error);
                scope.throw_exception(exception);
            }
        }
    }

    fn fs_write_file_sync(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        _rv: v8::ReturnValue,
    ) {
        if args.length() < 2 {
            let error = v8::String::new(scope, "writeFileSync requires path and content").unwrap();
            let exception = v8::Exception::type_error(scope, error);
            scope.throw_exception(exception);
            return;
        }

        let path_arg = args.get(0);
        let path_str = path_arg.to_string(scope).unwrap();
        let path = path_str.to_rust_string_lossy(scope);

        let content_arg = args.get(1);
        let content_str = content_arg.to_string(scope).unwrap();
        let content = content_str.to_rust_string_lossy(scope);

        if let Err(e) = std::fs::write(&path, &content) {
            let error = v8::String::new(scope, &format!("Cannot write file '{}': {}", path, e)).unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
        }
    }

    fn fs_exists_sync(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        mut rv: v8::ReturnValue,
    ) {
        if args.length() == 0 {
            let false_val = v8::Boolean::new(scope, false);
            rv.set(false_val.into());
            return;
        }

        let path_arg = args.get(0);
        let path_str = path_arg.to_string(scope).unwrap();
        let path = path_str.to_rust_string_lossy(scope);

        let exists = std::path::Path::new(&path).exists();
        let exists_val = v8::Boolean::new(scope, exists);
        rv.set(exists_val.into());
    }

    fn path_join(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        mut rv: v8::ReturnValue,
    ) {
        let mut path = PathBuf::new();
        
        for i in 0..args.length() {
            let arg = args.get(i);
            let arg_str = arg.to_string(scope).unwrap();
            let part = arg_str.to_rust_string_lossy(scope);
            path.push(part);
        }

        let result_str = v8::String::new(scope, path.to_string_lossy().as_ref()).unwrap();
        rv.set(result_str.into());
    }

    fn path_resolve(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        mut rv: v8::ReturnValue,
    ) {
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        for i in 0..args.length() {
            let arg = args.get(i);
            let arg_str = arg.to_string(scope).unwrap();
            let part = arg_str.to_rust_string_lossy(scope);
            
            let part_path = PathBuf::from(part);
            if part_path.is_absolute() {
                path = part_path;
            } else {
                path.push(part_path);
            }
        }

        let result_str = v8::String::new(scope, path.to_string_lossy().as_ref()).unwrap();
        rv.set(result_str.into());
    }

    fn path_basename(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        mut rv: v8::ReturnValue,
    ) {
        if args.length() == 0 {
            let empty_str = v8::String::new(scope, "").unwrap();
            rv.set(empty_str.into());
            return;
        }

        let path_arg = args.get(0);
        let path_str = path_arg.to_string(scope).unwrap();
        let path = path_str.to_rust_string_lossy(scope);

        let path_buf = PathBuf::from(path);
        let basename = path_buf.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        let result_str = v8::String::new(scope, basename).unwrap();
        rv.set(result_str.into());
    }

    fn path_dirname(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        mut rv: v8::ReturnValue,
    ) {
        if args.length() == 0 {
            let dot_str = v8::String::new(scope, ".").unwrap();
            rv.set(dot_str.into());
            return;
        }

        let path_arg = args.get(0);
        let path_str = path_arg.to_string(scope).unwrap();
        let path = path_str.to_rust_string_lossy(scope);

        let path_buf = PathBuf::from(path);
        let dirname = path_buf.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        let result_str = v8::String::new(scope, &dirname).unwrap();
        rv.set(result_str.into());
    }
}