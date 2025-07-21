use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Module resolution result
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub path: PathBuf,
    pub module_type: ModuleType,
    pub content: String,
}

/// Module type detection
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleType {
    CommonJS,
    ESModule,
    JSON,
    BuiltIn,
}

/// Module resolver with Node.js-compatible resolution algorithm
pub struct ModuleResolver {
    /// Base paths to search for modules (like NODE_PATH)
    base_paths: Vec<PathBuf>,
    /// File extensions to try
    extensions: Vec<String>,
    /// Package.json cache
    package_cache: HashMap<PathBuf, PackageJson>,
    /// Current working directory
    cwd: PathBuf,
}

/// Simplified package.json representation
#[derive(Debug, Clone)]
struct PackageJson {
    main: Option<String>,
    module: Option<String>,
    exports: Option<Value>,
    module_type: Option<String>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self {
            base_paths: vec![],
            extensions: vec![
                ".js".to_string(),
                ".mjs".to_string(),
                ".cjs".to_string(),
                ".json".to_string(),
            ],
            package_cache: HashMap::new(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Resolve a module specifier to a file path and content
    pub fn resolve(&mut self, specifier: &str, from: Option<&Path>) -> Result<ResolvedModule> {
        // 1. Handle built-in modules
        if let Some(builtin) = self.resolve_builtin(specifier) {
            return Ok(builtin);
        }

        // 2. Determine the starting directory for resolution
        let from_dir = if let Some(from_path) = from {
            if from_path.is_file() {
                from_path.parent().unwrap_or(&self.cwd).to_path_buf()
            } else {
                from_path.to_path_buf()
            }
        } else {
            self.cwd.clone()
        };

        // 3. Handle different types of specifiers
        if specifier.starts_with("./") || specifier.starts_with("../") || specifier.starts_with("/")
        {
            // Relative or absolute path
            self.resolve_file_or_directory(&from_dir, specifier)
        } else {
            // Module name - search in node_modules
            self.resolve_node_module(&from_dir, specifier)
        }
    }

    /// Resolve built-in Node.js modules
    fn resolve_builtin(&self, specifier: &str) -> Option<ResolvedModule> {
        let builtin_modules = [
            "fs",
            "path",
            "http",
            "https",
            "url",
            "querystring",
            "crypto",
            "os",
            "util",
            "events",
            "stream",
            "buffer",
            "assert",
            "child_process",
            "cluster",
            "console",
            "constants",
            "dgram",
            "dns",
            "domain",
            "module",
            "net",
            "process",
            "readline",
            "repl",
            "string_decoder",
            "sys",
            "timers",
            "tls",
            "tty",
            "vm",
            "zlib",
        ];

        if builtin_modules.contains(&specifier) {
            Some(ResolvedModule {
                path: PathBuf::from(format!("node:{}", specifier)),
                module_type: ModuleType::BuiltIn,
                content: String::new(), // Built-ins don't have file content
            })
        } else {
            None
        }
    }

    /// Resolve file or directory (relative/absolute paths)
    fn resolve_file_or_directory(
        &mut self,
        from_dir: &Path,
        specifier: &str,
    ) -> Result<ResolvedModule> {
        let target_path = if specifier.starts_with('/') {
            PathBuf::from(specifier)
        } else {
            from_dir.join(specifier)
        };

        // Try as file with extensions
        if let Ok(resolved) = self.try_file(&target_path) {
            return Ok(resolved);
        }

        // Try as directory with index files
        if target_path.is_dir() {
            return self.try_directory(&target_path);
        }

        Err(anyhow!("Module not found: {}", specifier))
    }

    /// Resolve node_modules packages
    fn resolve_node_module(&mut self, from_dir: &Path, specifier: &str) -> Result<ResolvedModule> {
        let mut current_dir = from_dir;

        // Walk up the directory tree looking for node_modules
        loop {
            let node_modules = current_dir.join("node_modules");
            if node_modules.exists() {
                let module_path = node_modules.join(specifier);

                // Try as file first
                if let Ok(resolved) = self.try_file(&module_path) {
                    return Ok(resolved);
                }

                // Try as directory with package.json
                if module_path.is_dir() {
                    if let Ok(resolved) = self.try_package(&module_path) {
                        return Ok(resolved);
                    }
                    // Fallback to index files
                    if let Ok(resolved) = self.try_directory(&module_path) {
                        return Ok(resolved);
                    }
                }
            }

            // Move up one directory
            if let Some(parent) = current_dir.parent() {
                current_dir = parent;
            } else {
                break;
            }
        }

        // Try global paths if nothing found
        for base_path in &self.base_paths {
            let module_path = base_path.join(specifier);
            if let Ok(resolved) = self.try_file(&module_path) {
                return Ok(resolved);
            }
            if module_path.is_dir() {
                if let Ok(resolved) = self.try_directory(&module_path) {
                    return Ok(resolved);
                }
            }
        }

        Err(anyhow!("Module not found: {}", specifier))
    }

    /// Try to resolve a file with various extensions
    fn try_file(&self, path: &Path) -> Result<ResolvedModule> {
        // Try exact path first
        if path.is_file() {
            return self.load_file(path);
        }

        // Try with extensions
        for ext in &self.extensions {
            let path_with_ext = path.with_extension(&ext[1..]); // Remove leading dot
            if path_with_ext.is_file() {
                return self.load_file(&path_with_ext);
            }

            // Also try adding extension to existing path
            let mut path_str = path.to_string_lossy().to_string();
            path_str.push_str(ext);
            let extended_path = PathBuf::from(path_str);
            if extended_path.is_file() {
                return self.load_file(&extended_path);
            }
        }

        Err(anyhow!("File not found: {}", path.display()))
    }

    /// Try to resolve a directory with index files
    fn try_directory(&self, dir_path: &Path) -> Result<ResolvedModule> {
        for ext in &self.extensions {
            let index_file = dir_path.join(format!("index{}", ext));
            if index_file.is_file() {
                return self.load_file(&index_file);
            }
        }

        Err(anyhow!(
            "No index file found in directory: {}",
            dir_path.display()
        ))
    }

    /// Try to resolve a package directory using package.json
    fn try_package(&mut self, package_dir: &Path) -> Result<ResolvedModule> {
        let package_json_path = package_dir.join("package.json");

        if !package_json_path.is_file() {
            return Err(anyhow!("No package.json found"));
        }

        // Load and cache package.json
        let package_json = if let Some(cached) = self.package_cache.get(package_dir) {
            cached.clone()
        } else {
            let content = std::fs::read_to_string(&package_json_path)?;
            let parsed: Value = serde_json::from_str(&content)?;

            let package_info = PackageJson {
                main: parsed
                    .get("main")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                module: parsed
                    .get("module")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                exports: parsed.get("exports").cloned(),
                module_type: parsed
                    .get("type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            };

            self.package_cache
                .insert(package_dir.to_path_buf(), package_info.clone());
            package_info
        };

        // Try module field first (ES modules)
        if let Some(module_path) = &package_json.module {
            let full_path = package_dir.join(module_path);
            if let Ok(resolved) = self.try_file(&full_path) {
                return Ok(resolved);
            }
        }

        // Try main field
        if let Some(main_path) = &package_json.main {
            let full_path = package_dir.join(main_path);
            if let Ok(resolved) = self.try_file(&full_path) {
                return Ok(resolved);
            }
        }

        // Default to index files
        self.try_directory(package_dir)
    }

    /// Load file content and determine module type
    fn load_file(&self, path: &Path) -> Result<ResolvedModule> {
        let content = std::fs::read_to_string(path)?;
        let module_type = self.detect_module_type(path, &content);

        Ok(ResolvedModule {
            path: path.to_path_buf(),
            module_type,
            content,
        })
    }

    /// Detect module type based on file extension and content
    fn detect_module_type(&self, path: &Path, content: &str) -> ModuleType {
        if let Some(ext) = path.extension() {
            match ext.to_string_lossy().as_ref() {
                "mjs" => return ModuleType::ESModule,
                "cjs" => return ModuleType::CommonJS,
                "json" => return ModuleType::JSON,
                _ => {}
            }
        }

        // For .js files, check content for ES module syntax
        if content.contains("import ") || content.contains("export ") {
            ModuleType::ESModule
        } else {
            ModuleType::CommonJS
        }
    }

    /// Add a base path for module resolution
    pub fn add_base_path(&mut self, path: PathBuf) {
        if !self.base_paths.contains(&path) {
            self.base_paths.push(path);
        }
    }

    /// Set current working directory
    pub fn set_cwd(&mut self, cwd: PathBuf) {
        self.cwd = cwd;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_builtin_resolution() {
        let mut resolver = ModuleResolver::new();

        let result = resolver.resolve("fs", None).unwrap();
        assert_eq!(result.module_type, ModuleType::BuiltIn);
        assert_eq!(result.path, PathBuf::from("node:fs"));
    }

    #[test]
    fn test_relative_path_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.js");
        fs::write(&test_file, "module.exports = {};").unwrap();

        let mut resolver = ModuleResolver::new();
        resolver.set_cwd(temp_dir.path().to_path_buf());

        let result = resolver.resolve("./test.js", None).unwrap();
        assert_eq!(result.module_type, ModuleType::CommonJS);
        assert!(result.path.ends_with("test.js"));
    }

    #[test]
    fn test_module_type_detection() {
        let resolver = ModuleResolver::new();

        // ES Module detection
        let es_content = "import { foo } from 'bar';";
        assert_eq!(
            resolver.detect_module_type(Path::new("test.js"), es_content),
            ModuleType::ESModule
        );

        // CommonJS detection
        let cjs_content = "const foo = require('bar');";
        assert_eq!(
            resolver.detect_module_type(Path::new("test.js"), cjs_content),
            ModuleType::CommonJS
        );

        // Extension-based detection
        assert_eq!(
            resolver.detect_module_type(Path::new("test.mjs"), ""),
            ModuleType::ESModule
        );
        assert_eq!(
            resolver.detect_module_type(Path::new("test.cjs"), ""),
            ModuleType::CommonJS
        );
        assert_eq!(
            resolver.detect_module_type(Path::new("test.json"), ""),
            ModuleType::JSON
        );
    }
}
