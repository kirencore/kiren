#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

use super::manager::Package;
use super::resolver::PackageSpec;

#[derive(Debug, Serialize, Deserialize)]
struct CacheMetadata {
    version: String,
    packages: HashMap<String, CachedPackageInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedPackageInfo {
    name: String,
    version: String,
    cached_at: u64,
    integrity: String,
    size: u64,
}

#[derive(Debug, Clone)]
pub struct GlobalCache {
    cache_dir: PathBuf,
}

impl GlobalCache {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Check if package exists in cache
    pub async fn get(&self, spec: &PackageSpec) -> Result<Option<Package>> {
        let package_dir = self.get_package_dir(&spec.name, "latest"); // Simplified for now
        let package_file = package_dir.join("package.toml");

        if package_file.exists() {
            let content = fs::read_to_string(&package_file).await?;
            let package: Package = toml::from_str(&content)?;
            return Ok(Some(package));
        }

        Ok(None)
    }

    /// Store package in cache
    pub async fn store(&self, package: &Package) -> Result<()> {
        let package_dir = self.get_package_dir(&package.name, &package.version);

        // Create package directory
        fs::create_dir_all(&package_dir).await?;

        // Write package metadata
        let package_file = package_dir.join("package.toml");
        let package_content = toml::to_string_pretty(package)?;
        fs::write(&package_file, package_content).await?;

        // Create lib directory structure (simplified)
        let lib_dir = package_dir.join("lib");
        fs::create_dir_all(&lib_dir).await?;

        // For now, create a simple main file
        // In production, this would extract from downloaded package
        let main_file = lib_dir.join(&package.main);
        if !main_file.exists() {
            let default_content = format!(
                "// Package: {}@{}\nmodule.exports = {{\n  name: '{}',\n  version: '{}'\n}};",
                package.name, package.version, package.name, package.version
            );
            fs::write(&main_file, default_content).await?;
        }

        println!("📦 Cached package: {}@{}", package.name, package.version);
        Ok(())
    }

    /// Get path to package in cache
    pub fn get_package_path(&self, package: &Package) -> PathBuf {
        self.get_package_dir(&package.name, &package.version)
    }

    /// Remove package from cache
    pub async fn remove_package(&self, name: &str, version: &str) -> Result<()> {
        let package_dir = self.get_package_dir(name, version);
        if package_dir.exists() {
            fs::remove_dir_all(&package_dir).await?;
            println!("🗑️  Removed from cache: {}@{}", name, version);
        }
        Ok(())
    }

    /// Clean old packages from cache
    pub async fn clean(&self, _max_age_days: u64) -> Result<()> {
        // Simplified - just show message for now
        println!("🧹 Cache clean functionality coming soon!");
        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> Result<CacheStats> {
        // Simplified stats
        Ok(CacheStats {
            package_count: 0,
            total_size: 0,
            cache_dir: self.cache_dir.clone(),
        })
    }

    fn get_package_dir(&self, name: &str, version: &str) -> PathBuf {
        self.cache_dir.join(format!("{}@{}", name, version))
    }

    // Removed calculate_package_size and save_metadata for simplicity
}

#[derive(Debug)]
pub struct CacheStats {
    pub package_count: usize,
    pub total_size: u64,
    pub cache_dir: PathBuf,
}

impl CacheStats {
    pub fn format_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = self.total_size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }

    pub fn print(&self) {
        println!("📊 Cache Statistics:");
        println!("   Packages: {}", self.package_count);
        println!("   Total Size: {}", self.format_size());
        println!("   Location: {}", self.cache_dir.display());
    }
}
