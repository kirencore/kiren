use anyhow::Result;
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;

use super::manager::Package;
use super::resolver::{PackageSource, PackageSpec, VersionSpec};

#[derive(Debug, Clone)]
pub struct KirenRegistry {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct RegistryPackageInfo {
    name: String,
    versions: HashMap<String, RegistryVersionInfo>,
    latest: String,
    tags: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RegistryVersionInfo {
    version: String,
    main: String,
    dependencies: HashMap<String, String>,
    kiren_version: Option<String>,
    integrity: String,
    tarball: String,
    deprecated: Option<bool>,
}

impl KirenRegistry {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("kiren-package-manager/0.1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { base_url, client }
    }

    /// Fetch package from registry
    pub async fn fetch_package(&self, spec: &PackageSpec) -> Result<Package> {
        match &spec.source {
            PackageSource::Registry => self.fetch_from_registry(spec).await,
            PackageSource::Url(url) => self.fetch_from_url(url, spec).await,
            PackageSource::Git { url, commit } => {
                self.fetch_from_git(url, commit.as_deref(), spec).await
            }
            PackageSource::Local(path) => self.fetch_from_local(path, spec).await,
        }
    }

    async fn fetch_from_registry(&self, spec: &PackageSpec) -> Result<Package> {
        // For now, simulate registry responses since we don't have a real registry yet

        // Simulate some popular packages
        match spec.name.as_str() {
            "express" => self.create_express_package(&spec.version),
            "lodash" => self.create_lodash_package(&spec.version),
            "axios" => self.create_axios_package(&spec.version),
            "moment" => self.create_moment_package(&spec.version),
            _ => self.create_generic_package(&spec.name, &spec.version),
        }
    }

    async fn fetch_from_url(&self, _url: &str, spec: &PackageSpec) -> Result<Package> {
        // For now, simulate URL-based packages
        // In production, this would download and extract the package
        self.create_generic_package(&spec.name, &spec.version)
    }

    async fn fetch_from_git(
        &self,
        _url: &str,
        _commit: Option<&str>,
        spec: &PackageSpec,
    ) -> Result<Package> {
        // For now, simulate git packages
        self.create_generic_package(&spec.name, &spec.version)
    }

    async fn fetch_from_local(&self, _path: &str, spec: &PackageSpec) -> Result<Package> {
        // For now, simulate local packages
        self.create_generic_package(&spec.name, &spec.version)
    }

    /// Get available versions for a package
    pub async fn get_versions(&self, name: &str) -> Result<Vec<String>> {
        // Simulate version lists
        match name {
            "express" => Ok(vec![
                "4.17.1".to_string(),
                "4.18.0".to_string(),
                "4.18.1".to_string(),
                "4.18.2".to_string(),
            ]),
            "lodash" => Ok(vec![
                "4.17.19".to_string(),
                "4.17.20".to_string(),
                "4.17.21".to_string(),
            ]),
            _ => Ok(vec![
                "1.0.0".to_string(),
                "1.0.1".to_string(),
                "1.0.2".to_string(),
            ]),
        }
    }

    // Simulated package creators (in production these would fetch real packages)

    fn create_express_package(&self, version_spec: &VersionSpec) -> Result<Package> {
        let version =
            self.resolve_version(version_spec, &["4.17.1", "4.18.0", "4.18.1", "4.18.2"])?;

        Ok(Package {
            name: "express".to_string(),
            version: version.clone(),
            main: "lib/express.js".to_string(),
            dependencies: {
                let mut deps = HashMap::new();
                deps.insert("body-parser".to_string(), "1.20.1".to_string());
                deps.insert("cookie-parser".to_string(), "1.4.6".to_string());
                deps
            },
            kiren_version: Some(">=0.1.0".to_string()),
            integrity: format!("sha256-express-{}", version),
            source_url: format!(
                "https://registry.kiren.dev/express/{}/express-{}.tar.gz",
                version, version
            ),
        })
    }

    fn create_lodash_package(&self, version_spec: &VersionSpec) -> Result<Package> {
        let version = self.resolve_version(version_spec, &["4.17.19", "4.17.20", "4.17.21"])?;

        Ok(Package {
            name: "lodash".to_string(),
            version: version.clone(),
            main: "lodash.js".to_string(),
            dependencies: HashMap::new(),
            kiren_version: Some(">=0.1.0".to_string()),
            integrity: format!("sha256-lodash-{}", version),
            source_url: format!(
                "https://registry.kiren.dev/lodash/{}/lodash-{}.tar.gz",
                version, version
            ),
        })
    }

    fn create_axios_package(&self, version_spec: &VersionSpec) -> Result<Package> {
        let version = self.resolve_version(version_spec, &["0.27.2", "1.0.0", "1.1.0", "1.2.0"])?;

        Ok(Package {
            name: "axios".to_string(),
            version: version.clone(),
            main: "lib/axios.js".to_string(),
            dependencies: HashMap::new(),
            kiren_version: Some(">=0.1.0".to_string()),
            integrity: format!("sha256-axios-{}", version),
            source_url: format!(
                "https://registry.kiren.dev/axios/{}/axios-{}.tar.gz",
                version, version
            ),
        })
    }

    fn create_moment_package(&self, version_spec: &VersionSpec) -> Result<Package> {
        let version =
            self.resolve_version(version_spec, &["2.29.1", "2.29.2", "2.29.3", "2.29.4"])?;

        Ok(Package {
            name: "moment".to_string(),
            version: version.clone(),
            main: "moment.js".to_string(),
            dependencies: HashMap::new(),
            kiren_version: Some(">=0.1.0".to_string()),
            integrity: format!("sha256-moment-{}", version),
            source_url: format!(
                "https://registry.kiren.dev/moment/{}/moment-{}.tar.gz",
                version, version
            ),
        })
    }

    fn create_generic_package(&self, name: &str, version_spec: &VersionSpec) -> Result<Package> {
        let version = self.resolve_version(version_spec, &["1.0.0", "1.0.1", "1.0.2"])?;

        Ok(Package {
            name: name.to_string(),
            version: version.clone(),
            main: "index.js".to_string(),
            dependencies: HashMap::new(),
            kiren_version: Some(">=0.1.0".to_string()),
            integrity: format!("sha256-{}-{}", name, version),
            source_url: format!(
                "https://registry.kiren.dev/{}/{}/{}-{}.tar.gz",
                name, version, name, version
            ),
        })
    }

    fn resolve_version(&self, version_spec: &VersionSpec, available: &[&str]) -> Result<String> {
        let _available_strings: Vec<String> = available.iter().map(|s| s.to_string()).collect();

        match version_spec {
            VersionSpec::Exact(v) => {
                if available.contains(&v.as_str()) {
                    Ok(v.clone())
                } else {
                    Err(anyhow::anyhow!("Version {} not available", v))
                }
            }
            VersionSpec::Latest => Ok(available.last().unwrap().to_string()),
            VersionSpec::Range(range) => {
                // Simple range resolution
                if range.starts_with('^') {
                    let base = &range[1..];
                    for version in available.iter().rev() {
                        if version.starts_with(&base[..base.rfind('.').unwrap_or(0)]) {
                            return Ok(version.to_string());
                        }
                    }
                }
                Ok(available.last().unwrap().to_string())
            }
            VersionSpec::Tag(tag) => match tag.as_str() {
                "latest" => Ok(available.last().unwrap().to_string()),
                "beta" | "alpha" => Ok(available.last().unwrap().to_string()),
                _ => Err(anyhow::anyhow!("Unknown tag: {}", tag)),
            },
        }
    }

    /// Check if registry is available
    pub async fn health_check(&self) -> Result<bool> {
        // For now, always return true since we're simulating
        Ok(true)
    }

    /// Search packages in registry
    pub async fn search(&self, query: &str) -> Result<Vec<String>> {
        // Simulate search results
        let all_packages = vec![
            "express",
            "lodash",
            "axios",
            "moment",
            "react",
            "vue",
            "angular",
            "typescript",
            "webpack",
            "babel",
            "eslint",
            "prettier",
            "jest",
            "mocha",
            "chai",
            "sinon",
            "nodemon",
            "cors",
            "helmet",
            "morgan",
        ];

        let results: Vec<String> = all_packages
            .into_iter()
            .filter(|pkg| pkg.contains(query))
            .map(|s| s.to_string())
            .collect();

        Ok(results)
    }
}
