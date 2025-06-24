use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub package: PackageInfo,
    
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, String>,
    
    #[serde(default)]
    pub scripts: HashMap<String, String>,
    
    #[serde(default)]
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    
    #[serde(default = "default_runtime")]
    pub runtime: String,
}

fn default_runtime() -> String {
    "kiren".to_string()
}

impl Default for ProjectConfig {
    fn default() -> Self {
        let mut scripts = HashMap::new();
        scripts.insert("dev".to_string(), "kiren --watch src/main.js".to_string());
        scripts.insert("build".to_string(), "kiren bundle src/main.js".to_string());
        scripts.insert("test".to_string(), "kiren test".to_string());
        scripts.insert("start".to_string(), "kiren src/main.js".to_string());

        Self {
            package: PackageInfo {
                name: "kiren-app".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                license: Some("MIT".to_string()),
                runtime: "kiren".to_string(),
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            scripts,
            config: HashMap::new(),
        }
    }
}

impl ProjectConfig {
    /// Create a new project config with given name
    pub fn new(name: &str) -> Self {
        let mut config = Self::default();
        config.package.name = name.to_string();
        config
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, name: &str, version: &str) {
        self.dependencies.insert(name.to_string(), version.to_string());
    }

    /// Add a dev dependency
    pub fn add_dev_dependency(&mut self, name: &str, version: &str) {
        self.dev_dependencies.insert(name.to_string(), version.to_string());
    }

    /// Add a script
    pub fn add_script(&mut self, name: &str, command: &str) {
        self.scripts.insert(name.to_string(), command.to_string());
    }

    /// Add a config value
    pub fn add_config(&mut self, key: &str, value: &str) {
        self.config.insert(key.to_string(), value.to_string());
    }

    /// Get all dependencies (including dev dependencies in dev mode)
    pub fn get_all_dependencies(&self, include_dev: bool) -> HashMap<String, String> {
        let mut all_deps = self.dependencies.clone();
        
        if include_dev {
            all_deps.extend(self.dev_dependencies.clone());
        }
        
        all_deps
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate package name
        if self.package.name.is_empty() {
            errors.push("Package name cannot be empty".to_string());
        }

        if !self.package.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            errors.push("Package name can only contain alphanumeric characters, hyphens, and underscores".to_string());
        }

        // Validate version format (basic semver check)
        if !is_valid_semver(&self.package.version) {
            errors.push("Package version must be valid semver format (e.g., 1.0.0)".to_string());
        }

        // Validate runtime
        if self.package.runtime != "kiren" {
            errors.push("Runtime must be 'kiren'".to_string());
        }

        // Validate dependencies
        for (name, version) in &self.dependencies {
            if name.is_empty() {
                errors.push("Dependency name cannot be empty".to_string());
            }
            if version.is_empty() {
                errors.push(format!("Version for dependency '{}' cannot be empty", name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Convert to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Load from TOML string
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }
}

/// Basic semver validation
fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| {
        part.parse::<u32>().is_ok()
    })
}

/// Package configuration for publishing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishConfig {
    pub registry: Option<String>,
    pub access: AccessLevel,
    pub files: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    
    #[serde(default)]
    pub keywords: Vec<String>,
    
    #[serde(default)]
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    Public,
    Private,
}

impl Default for PublishConfig {
    fn default() -> Self {
        Self {
            registry: None,
            access: AccessLevel::Public,
            files: None,
            ignore: Some(vec![
                "node_modules/".to_string(),
                ".git/".to_string(),
                "*.log".to_string(),
                ".DS_Store".to_string(),
            ]),
            keywords: Vec::new(),
            categories: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_config_creation() {
        let config = ProjectConfig::new("test-app");
        assert_eq!(config.package.name, "test-app");
        assert_eq!(config.package.version, "1.0.0");
        assert_eq!(config.package.runtime, "kiren");
    }

    #[test]
    fn test_config_validation() {
        let mut config = ProjectConfig::new("valid-app");
        assert!(config.validate().is_ok());

        config.package.name = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_semver_validation() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("0.1.2"));
        assert!(is_valid_semver("10.20.30"));
        
        assert!(!is_valid_semver("1.0"));
        assert!(!is_valid_semver("1.0.0.1"));
        assert!(!is_valid_semver("v1.0.0"));
        assert!(!is_valid_semver("1.0.x"));
    }

    #[test]
    fn test_toml_serialization() {
        let config = ProjectConfig::new("test-app");
        let toml_string = config.to_toml().unwrap();
        let parsed_config = ProjectConfig::from_toml(&toml_string).unwrap();
        
        assert_eq!(config.package.name, parsed_config.package.name);
        assert_eq!(config.package.version, parsed_config.package.version);
    }
}