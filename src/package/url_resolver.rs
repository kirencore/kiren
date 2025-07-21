#![allow(dead_code)]
use ada_url::Url;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

use super::cache::GlobalCache;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlModule {
    pub url: String,
    pub content: String,
    pub content_type: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UrlResolver {
    client: Client,
    cache: GlobalCache,
    url_cache: HashMap<String, UrlModule>,
}

impl UrlResolver {
    pub fn new(cache: GlobalCache) -> Self {
        let client = Client::builder()
            .user_agent("kiren-runtime/0.2.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            cache,
            url_cache: HashMap::new(),
        }
    }

    /// Resolve and fetch a module from URL
    pub async fn resolve_url(&mut self, url_str: &str) -> Result<UrlModule> {
        // Validate and normalize URL using ada-url
        let url = Url::parse(Box::leak(url_str.to_string().into_boxed_str()), None)?;
        let canonical_url = url.href().to_string();

        // Check memory cache first
        if let Some(module) = self.url_cache.get(&canonical_url) {
            return Ok(module.clone());
        }

        // Check disk cache
        if let Some(cached_module) = self.load_from_disk_cache(&canonical_url).await? {
            self.url_cache
                .insert(canonical_url.to_string(), cached_module.clone());
            return Ok(cached_module);
        }

        // Fetch from network
        let module = self.fetch_from_network(&canonical_url).await?;

        // Store in both caches
        self.store_in_disk_cache(&canonical_url, &module).await?;
        self.url_cache
            .insert(canonical_url.to_string(), module.clone());

        Ok(module)
    }

    /// Fetch module from network
    async fn fetch_from_network(&self, url: &str) -> Result<UrlModule> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch {}: {}",
                url,
                response.status()
            ));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/javascript")
            .to_string();

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let last_modified = response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let content = response.text().await?;
        let dependencies = self.extract_dependencies(&content);

        Ok(UrlModule {
            url: url.to_string(),
            content,
            content_type,
            etag,
            last_modified,
            dependencies,
        })
    }

    /// Extract import URLs from module content
    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Match import statements with URLs
        let import_regex = regex::Regex::new(r#"import\s+.*\s+from\s+["']([^"']+)["']"#).unwrap();
        for cap in import_regex.captures_iter(content) {
            if let Some(url_match) = cap.get(1) {
                let import_url = url_match.as_str();
                // Only include HTTP(S) URLs
                if import_url.starts_with("http://") || import_url.starts_with("https://") {
                    dependencies.push(import_url.to_string());
                }
            }
        }

        // Match dynamic imports
        let dynamic_regex = regex::Regex::new(r#"import\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();
        for cap in dynamic_regex.captures_iter(content) {
            if let Some(url_match) = cap.get(1) {
                let import_url = url_match.as_str();
                if import_url.starts_with("http://") || import_url.starts_with("https://") {
                    dependencies.push(import_url.to_string());
                }
            }
        }

        dependencies
    }

    /// Load module from disk cache
    async fn load_from_disk_cache(&self, url: &str) -> Result<Option<UrlModule>> {
        let cache_path = self.get_cache_path(url);

        if !cache_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&cache_path).await?;
        let module: UrlModule = serde_json::from_str(&content)?;

        // Check if cache is still valid (simple TTL check)
        let metadata = fs::metadata(&cache_path).await?;
        let age = metadata.modified()?.elapsed().unwrap_or_default();

        // Cache expires after 1 hour for development, 24 hours for production
        let ttl = if cfg!(debug_assertions) {
            std::time::Duration::from_secs(3600) // 1 hour
        } else {
            std::time::Duration::from_secs(86400) // 24 hours
        };

        if age > ttl {
            return Ok(None);
        }

        Ok(Some(module))
    }

    /// Store module in disk cache
    async fn store_in_disk_cache(&self, url: &str, module: &UrlModule) -> Result<()> {
        let cache_path = self.get_cache_path(url);

        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(module)?;
        fs::write(&cache_path, content).await?;

        Ok(())
    }

    /// Get cache file path for URL
    fn get_cache_path(&self, url: &str) -> PathBuf {
        // Create a safe filename from URL
        let safe_name = url
            .replace("://", "_")
            .replace("/", "_")
            .replace("?", "_")
            .replace("&", "_")
            .replace("=", "_");

        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".kiren")
            .join("url_cache");

        cache_dir.join(format!("{}.json", safe_name))
    }

    /// Resolve relative URL against base URL
    pub fn resolve_relative_url(&self, base_url: &str, relative_url: &str) -> Result<String> {
        if relative_url.starts_with("http://") || relative_url.starts_with("https://") {
            return Ok(relative_url.to_string());
        }

        let base = Url::parse(Box::leak(base_url.to_string().into_boxed_str()), None)?;

        // Manual URL joining for ada-url
        let resolved_url = if relative_url.starts_with("/") {
            // Absolute path
            format!("{}://{}{}", base.protocol(), base.host(), relative_url)
        } else {
            // Relative path
            let base_path = base.pathname();
            let base_dir = if base_path.ends_with("/") {
                base_path.to_string()
            } else {
                let mut parts: Vec<&str> = base_path.split('/').collect();
                parts.pop(); // Remove filename
                parts.join("/") + "/"
            };
            format!(
                "{}://{}{}{}",
                base.protocol(),
                base.host(),
                base_dir,
                relative_url
            )
        };

        let resolved = Url::parse(Box::leak(resolved_url.into_boxed_str()), None)?;
        Ok(resolved.href().to_string())
    }

    /// Preload dependencies recursively
    pub async fn preload_dependencies(&mut self, url: &str) -> Result<()> {
        let module = self.resolve_url(url).await?;

        for dep_url in &module.dependencies {
            // Resolve relative URLs
            let resolved_url = if dep_url.starts_with("http") {
                dep_url.clone()
            } else {
                self.resolve_relative_url(url, dep_url)?
            };

            // Recursively preload
            Box::pin(self.preload_dependencies(&resolved_url)).await?;
        }

        Ok(())
    }

    /// Clear URL cache
    pub fn clear_cache(&mut self) {
        self.url_cache.clear();
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> Result<UrlCacheStats> {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".kiren")
            .join("url_cache");

        let mut total_files = 0;
        let mut total_size = 0;

        if cache_dir.exists() {
            let mut entries = fs::read_dir(&cache_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.path().is_file() {
                    total_files += 1;
                    if let Ok(metadata) = entry.metadata().await {
                        total_size += metadata.len();
                    }
                }
            }
        }

        Ok(UrlCacheStats {
            memory_entries: self.url_cache.len(),
            disk_files: total_files,
            total_size_bytes: total_size,
        })
    }
}

#[derive(Debug)]
pub struct UrlCacheStats {
    pub memory_entries: usize,
    pub disk_files: usize,
    pub total_size_bytes: u64,
}

impl UrlCacheStats {
    pub fn print(&self) {
        println!("URL Cache Statistics:");
        println!("  Memory cache: {} modules", self.memory_entries);
        println!("  Disk cache: {} files", self.disk_files);
        println!(
            "  Total size: {:.2} MB",
            self.total_size_bytes as f64 / 1024.0 / 1024.0
        );
    }
}

/// URL Import Support for JavaScript modules
pub struct UrlImportHandler {
    resolver: UrlResolver,
}

impl UrlImportHandler {
    pub fn new(cache: GlobalCache) -> Self {
        Self {
            resolver: UrlResolver::new(cache),
        }
    }

    /// Check if import specifier is a URL
    pub fn is_url_import(&self, specifier: &str) -> bool {
        specifier.starts_with("http://") || specifier.starts_with("https://")
    }

    /// Resolve URL import and return module content
    pub async fn resolve_import(
        &mut self,
        specifier: &str,
        referrer: Option<&str>,
    ) -> Result<String> {
        let resolved_url = if let Some(ref_url) = referrer {
            self.resolver.resolve_relative_url(ref_url, specifier)?
        } else {
            specifier.to_string()
        };

        let module = self.resolver.resolve_url(&resolved_url).await?;
        Ok(module.content)
    }

    /// Preload all dependencies for faster execution
    pub async fn preload(&mut self, url: &str) -> Result<()> {
        self.resolver.preload_dependencies(url).await
    }
}
