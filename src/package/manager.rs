use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::resolver::{PackageSpec, PackageResolver};
use super::cache::GlobalCache;
use super::registry::KirenRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub main: String,
    pub dependencies: HashMap<String, String>,
    pub kiren_version: Option<String>,
    pub integrity: String,
    pub source_url: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub package: Package,
    pub local_path: PathBuf,
    pub dependencies: Vec<ResolvedPackage>,
}

pub struct KirenPackageManager {
    cache: GlobalCache,
    registry: KirenRegistry,
    resolver: PackageResolver,
}

impl KirenPackageManager {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".kiren")
            .join("cache");

        let cache = GlobalCache::new(cache_dir)?;
        let registry = KirenRegistry::new("https://registry.kiren.dev".to_string());
        let resolver = PackageResolver::new();

        Ok(Self {
            cache,
            registry,
            resolver,
        })
    }

    /// Resolve a package spec to a concrete package
    pub async fn resolve(&self, spec: &str) -> Result<ResolvedPackage> {
        
        let package_spec = PackageSpec::parse(spec)?;
        
        // Check local cache first
        if let Some(cached) = self.cache.get(&package_spec).await? {
            return Ok(self.build_resolved_package(cached).await?);
        }

        // Fetch from registry
        let package = self.registry.fetch_package(&package_spec).await?;
        
        // Verify integrity
        self.verify_package_integrity(&package)?;
        
        // Store in cache
        self.cache.store(&package).await?;
        
        Ok(self.build_resolved_package(package).await?)
    }

    /// Install packages from kiren.toml
    pub async fn install(&self, project_dir: &PathBuf) -> Result<Vec<ResolvedPackage>> {
        let config_path = project_dir.join("kiren.toml");
        
        if !config_path.exists() {
            return Err(anyhow::anyhow!("kiren.toml not found in project directory"));
        }

        let config_content = fs::read_to_string(&config_path).await?;
        let config: super::config::ProjectConfig = toml::from_str(&config_content)?;

        let mut resolved_packages = Vec::new();

        
        // Install main dependencies
        for (name, version) in &config.dependencies {
            let spec = format!("{}@{}", name, version);
            let resolved = self.resolve(&spec).await?;
            resolved_packages.push(resolved);
        }

        // Install dev dependencies if in dev mode
        if std::env::var("NODE_ENV").unwrap_or_default() != "production" {
            for (name, version) in &config.dev_dependencies {
                let spec = format!("{}@{}", name, version);
                let resolved = self.resolve(&spec).await?;
                resolved_packages.push(resolved);
            }
        }

        Ok(resolved_packages)
    }

    /// Get installed package for import resolution
    pub async fn get_package(&self, name: &str) -> Result<Option<ResolvedPackage>> {
        // For now, try to resolve latest version
        // In production, this would check lock file first
        let spec = format!("{}@latest", name);
        match self.resolve(&spec).await {
            Ok(package) => Ok(Some(package)),
            Err(_) => Ok(None),
        }
    }

    async fn build_resolved_package(&self, package: Package) -> Result<ResolvedPackage> {
        let local_path = self.cache.get_package_path(&package);
        
        // Recursively resolve dependencies with Box::pin
        let mut dependencies = Vec::new();
        for (dep_name, dep_version) in &package.dependencies {
            let dep_spec = format!("{}@{}", dep_name, dep_version);
            let resolved_dep = Box::pin(self.resolve(&dep_spec)).await?;
            dependencies.push(resolved_dep);
        }

        Ok(ResolvedPackage {
            package,
            local_path,
            dependencies,
        })
    }

    fn verify_package_integrity(&self, _package: &Package) -> Result<()> {
        // TODO: Implement SHA-256 integrity check
        Ok(())
    }

    /// Create a new project with kiren.toml
    pub async fn init(project_dir: &PathBuf, name: &str) -> Result<()> {
        let config = super::config::ProjectConfig {
            package: super::config::PackageInfo {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                license: None,
                runtime: "kiren".to_string(),
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            scripts: {
                let mut scripts = HashMap::new();
                scripts.insert("dev".to_string(), "kiren --watch src/main.js".to_string());
                scripts.insert("build".to_string(), "kiren bundle src/main.js".to_string());
                scripts.insert("test".to_string(), "kiren test".to_string());
                scripts
            },
            config: HashMap::new(),
        };

        let config_path = project_dir.join("kiren.toml");
        let config_content = toml::to_string_pretty(&config)?;
        
        fs::create_dir_all(project_dir).await?;
        fs::write(&config_path, config_content).await?;

        // Create basic project structure
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).await?;
        
        let main_js = src_dir.join("main.js");
        let main_content = r#"// Welcome to Kiren!
console.log("Hello from Kiren!");

// Import packages with URL-based imports
// import express from "kiren:express@4.18.2";
// import lodash from "https://kiren.dev/lodash@latest";

// Your code here...
"#;
        fs::write(&main_js, main_content).await?;

        
        Ok(())
    }
}