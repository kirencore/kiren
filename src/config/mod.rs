use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KirenConfig {
    pub runtime: RuntimeConfig,
    pub server: ServerConfig,
    pub watch: WatchConfig,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub v8_flags: Vec<String>,
    pub memory_limit: Option<usize>,       // MB
    pub max_old_space_size: Option<usize>, // MB
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub default_port: u16,
    pub cors_enabled: bool,
    pub cors_origins: Vec<String>,
    pub request_timeout: u64,    // seconds
    pub max_request_size: usize, // bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    pub enabled: bool,
    pub ignore_patterns: Vec<String>,
    pub debounce_ms: u64,
}

impl Default for KirenConfig {
    fn default() -> Self {
        Self {
            runtime: RuntimeConfig {
                v8_flags: vec![],
                memory_limit: Some(512),
                max_old_space_size: Some(256),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            server: ServerConfig {
                default_port: 3000,
                cors_enabled: true,
                cors_origins: vec!["*".to_string()],
                request_timeout: 30,
                max_request_size: 1024 * 1024, // 1MB
            },
            watch: WatchConfig {
                enabled: false,
                ignore_patterns: vec![
                    "node_modules/**".to_string(),
                    ".git/**".to_string(),
                    "*.log".to_string(),
                ],
                debounce_ms: 50,
            },
            environment: HashMap::new(),
        }
    }
}

impl KirenConfig {
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: KirenConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn find_config_file() -> Option<PathBuf> {
        // Look for kiren.toml in current directory
        let current_dir = std::env::current_dir().ok()?;
        let config_files = ["kiren.toml", "kiren.config.toml", ".kirenrc.toml"];

        for filename in &config_files {
            let path = current_dir.join(filename);
            if path.exists() {
                return Some(path);
            }
        }

        // Look in home directory
        if let Some(home_dir) = dirs::home_dir() {
            let path = home_dir.join(".kiren").join("config.toml");
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    pub fn load() -> Self {
        if let Some(config_path) = Self::find_config_file() {
            match Self::load_from_file(&config_path) {
                Ok(config) => {
                    eprintln!("Loaded config from: {}", config_path.display());
                    config
                }
                Err(e) => {
                    eprintln!(
                        "Failed to load config from {}: {}",
                        config_path.display(),
                        e
                    );
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    pub fn merge_with_env(&mut self) {
        // Override with environment variables
        if let Ok(port) = std::env::var("KIREN_PORT") {
            if let Ok(port) = port.parse::<u16>() {
                self.server.default_port = port;
            }
        }

        if let Ok(memory_limit) = std::env::var("KIREN_MEMORY_LIMIT") {
            if let Ok(limit) = memory_limit.parse::<usize>() {
                self.runtime.memory_limit = Some(limit);
            }
        }

        // Load all KIREN_* environment variables
        for (key, value) in std::env::vars() {
            if key.starts_with("KIREN_") {
                self.environment.insert(key, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_version_matches_cargo_pkg() {
        let config = KirenConfig::default();

        println!("Runtime version: {}", config.runtime.version);
        // Test that version dynamically matches package version
        assert_eq!(config.runtime.version, env!("CARGO_PKG_VERSION"));

        // Verify it's not hardcoded
        assert!(!config.runtime.version.is_empty());
        assert_ne!(config.runtime.version, "0.0.0");
    }

    #[test]
    fn test_runtime_version_env_override() {
        let mut config = KirenConfig::default();
        let original_version = config.runtime.version.clone();

        // Verify environment merge doesn't affect version (version should stay dynamic)
        config.merge_with_env();

        // Version should still match cargo package version
        assert_eq!(config.runtime.version, original_version);
        assert_eq!(config.runtime.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_multiple_config_instances_same_version() {
        let config1 = KirenConfig::default();
        let config2 = KirenConfig::default();

        // All instances should have same dynamic version
        assert_eq!(config1.runtime.version, config2.runtime.version);
        assert_eq!(config1.runtime.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_version_comparison_helper() {
        let config = KirenConfig::default();

        // Helper function to compare versions
        fn is_valid_semver(version: &str) -> bool {
            let parts: Vec<&str> = version.split('.').collect();
            parts.len() >= 2 && parts.iter().all(|p| p.parse::<u32>().is_ok())
        }

        assert!(is_valid_semver(&config.runtime.version));

        // Test that it's a real version, not placeholder
        assert_ne!(config.runtime.version, "unknown");
        assert_ne!(config.runtime.version, "dev");
        assert_ne!(config.runtime.version, "");
    }

    #[test]
    fn test_config_clone_preserves_version() {
        let original = KirenConfig::default();
        let cloned = original.clone();

        // Clone should preserve the dynamic version
        assert_eq!(original.runtime.version, cloned.runtime.version);
        assert_eq!(cloned.runtime.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_runtime_config_individual_version() {
        let runtime_config = RuntimeConfig {
            v8_flags: vec![],
            memory_limit: Some(512),
            max_old_space_size: Some(256),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        // Direct RuntimeConfig should also have correct version
        assert_eq!(runtime_config.version, env!("CARGO_PKG_VERSION"));
        assert!(!runtime_config.version.is_empty());
    }
}
