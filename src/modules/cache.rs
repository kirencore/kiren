use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};
use v8;

/// Module execution state
#[derive(Debug, Clone)]
pub enum ModuleState {
    /// Module is currently being loaded (prevents circular dependencies)
    Loading,
    /// Module has been successfully loaded
    Loaded(Arc<LoadedModule>),
    /// Module failed to load
    Error(String),
}

/// A loaded module with its exports and metadata
#[derive(Debug)]
pub struct LoadedModule {
    /// Module file path
    pub path: PathBuf,
    /// Module exports object (V8 handle stored as persistent)
    pub exports: v8::Global<v8::Object>,
    /// Module source code (for debugging)
    pub source: String,
    /// Dependencies of this module
    pub dependencies: Vec<PathBuf>,
    /// Timestamp when module was loaded
    pub loaded_at: std::time::SystemTime,
}

/// Thread-safe module cache with circular dependency detection
pub struct ModuleCache {
    /// Map of module path to its state
    cache: Arc<Mutex<HashMap<PathBuf, ModuleState>>>,
    /// Stack of currently loading modules (for circular dependency detection)
    loading_stack: Arc<Mutex<Vec<PathBuf>>>,
}

impl ModuleCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            loading_stack: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get a module from cache, returning None if not cached
    pub fn get(&self, path: &PathBuf) -> Option<ModuleState> {
        let cache = self.cache.lock().unwrap();
        cache.get(path).cloned()
    }

    /// Set a module in cache
    pub fn set(&self, path: PathBuf, state: ModuleState) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(path, state);
    }

    /// Mark a module as currently loading (for circular dependency detection)
    pub fn mark_loading(&self, path: PathBuf) -> Result<()> {
        let mut loading_stack = self.loading_stack.lock().unwrap();

        // Check for circular dependency
        if loading_stack.contains(&path) {
            let cycle = loading_stack
                .iter()
                .skip_while(|&p| p != &path)
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");

            return Err(anyhow!(
                "Circular dependency detected: {} -> {}",
                cycle,
                path.display()
            ));
        }

        loading_stack.push(path.clone());

        // Mark as loading in cache
        let mut cache = self.cache.lock().unwrap();
        cache.insert(path, ModuleState::Loading);

        Ok(())
    }

    /// Mark a module as finished loading
    pub fn mark_loaded(&self, path: PathBuf, module: Arc<LoadedModule>) {
        // Remove from loading stack
        {
            let mut loading_stack = self.loading_stack.lock().unwrap();
            if let Some(pos) = loading_stack.iter().position(|p| p == &path) {
                loading_stack.remove(pos);
            }
        }

        // Update cache
        let mut cache = self.cache.lock().unwrap();
        cache.insert(path, ModuleState::Loaded(module));
    }

    /// Mark a module as failed to load
    pub fn mark_error(&self, path: PathBuf, error: String) {
        // Remove from loading stack
        {
            let mut loading_stack = self.loading_stack.lock().unwrap();
            if let Some(pos) = loading_stack.iter().position(|p| p == &path) {
                loading_stack.remove(pos);
            }
        }

        // Update cache
        let mut cache = self.cache.lock().unwrap();
        cache.insert(path, ModuleState::Error(error));
    }

    /// Check if a module is currently being loaded
    pub fn is_loading(&self, path: &PathBuf) -> bool {
        let loading_stack = self.loading_stack.lock().unwrap();
        loading_stack.contains(path)
    }

    /// Get current loading stack (for debugging circular dependencies)
    pub fn get_loading_stack(&self) -> Vec<PathBuf> {
        let loading_stack = self.loading_stack.lock().unwrap();
        loading_stack.clone()
    }

    /// Clear the entire cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();

        let mut loading_stack = self.loading_stack.lock().unwrap();
        loading_stack.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> ModuleCacheStats {
        let cache = self.cache.lock().unwrap();
        let loading_stack = self.loading_stack.lock().unwrap();

        let mut loaded_count = 0;
        let mut loading_count = 0;
        let mut error_count = 0;

        for state in cache.values() {
            match state {
                ModuleState::Loading => loading_count += 1,
                ModuleState::Loaded(_) => loaded_count += 1,
                ModuleState::Error(_) => error_count += 1,
            }
        }

        ModuleCacheStats {
            total_modules: cache.len(),
            loaded_modules: loaded_count,
            loading_modules: loading_count,
            error_modules: error_count,
            loading_stack_depth: loading_stack.len(),
        }
    }

    /// Remove a specific module from cache (useful for hot reloading)
    pub fn invalidate(&self, path: &PathBuf) -> bool {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(path).is_some()
    }

    /// Get all loaded module paths
    pub fn get_loaded_modules(&self) -> Vec<PathBuf> {
        let cache = self.cache.lock().unwrap();
        cache
            .iter()
            .filter_map(|(path, state)| {
                if matches!(state, ModuleState::Loaded(_)) {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct ModuleCacheStats {
    pub total_modules: usize,
    pub loaded_modules: usize,
    pub loading_modules: usize,
    pub error_modules: usize,
    pub loading_stack_depth: usize,
}

impl ModuleCacheStats {
    pub fn print(&self) {
        println!("Module Cache Statistics:");
        println!("  Total modules: {}", self.total_modules);
        println!("  Loaded: {}", self.loaded_modules);
        println!("  Loading: {}", self.loading_modules);
        println!("  Errors: {}", self.error_modules);
        println!("  Loading stack depth: {}", self.loading_stack_depth);
    }
}

/// Circular dependency safe module executor
pub struct ModuleExecutor {
    cache: ModuleCache,
}

impl ModuleExecutor {
    pub fn new() -> Self {
        Self {
            cache: ModuleCache::new(),
        }
    }

    /// Execute a module with circular dependency protection
    pub fn execute_module<F>(
        &self,
        path: PathBuf,
        source: &str,
        executor: F,
    ) -> Result<Arc<LoadedModule>>
    where
        F: FnOnce(&str) -> Result<v8::Global<v8::Object>>,
    {
        // Check cache first
        if let Some(state) = self.cache.get(&path) {
            match state {
                ModuleState::Loaded(module) => return Ok(module),
                ModuleState::Loading => {
                    // Return a placeholder for circular dependencies
                    return Err(anyhow!("Circular dependency: {}", path.display()));
                }
                ModuleState::Error(err) => {
                    return Err(anyhow!("Module previously failed: {}", err));
                }
            }
        }

        // Mark as loading
        self.cache.mark_loading(path.clone())?;

        // Execute the module
        match executor(source) {
            Ok(exports) => {
                let module = Arc::new(LoadedModule {
                    path: path.clone(),
                    exports,
                    source: source.to_string(),
                    dependencies: Vec::new(), // TODO: Track dependencies
                    loaded_at: std::time::SystemTime::now(),
                });

                self.cache.mark_loaded(path, module.clone());
                Ok(module)
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.cache.mark_error(path, error_msg.clone());
                Err(e)
            }
        }
    }

    /// Get the module cache
    pub fn cache(&self) -> &ModuleCache {
        &self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = ModuleCache::new();
        let path = PathBuf::from("/test/module.js");

        // Initially empty
        assert!(cache.get(&path).is_none());

        // Mark as loading
        assert!(cache.mark_loading(path.clone()).is_ok());
        assert!(cache.is_loading(&path));

        // Check loading state
        if let Some(ModuleState::Loading) = cache.get(&path) {
            // Expected
        } else {
            panic!("Expected loading state");
        }
    }

    #[test]
    fn test_circular_dependency_detection() {
        let cache = ModuleCache::new();
        let path1 = PathBuf::from("/test/module1.js");
        let path2 = PathBuf::from("/test/module2.js");

        // Mark first module as loading
        assert!(cache.mark_loading(path1.clone()).is_ok());

        // Mark second module as loading
        assert!(cache.mark_loading(path2.clone()).is_ok());

        // Try to load first module again - should detect circular dependency
        assert!(cache.mark_loading(path1.clone()).is_err());
    }

    #[test]
    fn test_cache_stats() {
        let cache = ModuleCache::new();
        let path1 = PathBuf::from("/test/module1.js");
        let path2 = PathBuf::from("/test/module2.js");

        cache.mark_loading(path1.clone()).unwrap();
        cache.mark_error(path2.clone(), "Test error".to_string());

        let stats = cache.stats();
        assert_eq!(stats.total_modules, 2);
        assert_eq!(stats.loading_modules, 1);
        assert_eq!(stats.error_modules, 1);
        assert_eq!(stats.loaded_modules, 0);
    }
}
