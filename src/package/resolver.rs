use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageSpec {
    pub name: String,
    pub version: VersionSpec,
    pub source: PackageSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionSpec {
    Exact(String),          // "1.2.3"
    Range(String),          // "^1.2.0", "~1.2.0"
    Latest,                 // "latest"
    Tag(String),           // "beta", "alpha"
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageSource {
    Registry,                           // kiren:package or package
    Url(String),                       // https://kiren.dev/package@1.0.0
    Git { url: String, commit: Option<String> }, // git://github.com/user/repo
    Local(String),                     // ./local-package
}

pub struct PackageResolver {
    version_regex: Regex,
    url_regex: Regex,
}

impl PackageResolver {
    pub fn new() -> Self {
        Self {
            version_regex: Regex::new(r"^(.+?)@(.+)$").unwrap(),
            url_regex: Regex::new(r"^https?://").unwrap(),
        }
    }

    /// Parse a package specification string
    pub fn parse_spec(&self, spec: &str) -> Result<PackageSpec> {
        PackageSpec::parse(spec)
    }

    /// Resolve version range to concrete version
    pub fn resolve_version(&self, spec: &VersionSpec, available_versions: &[String]) -> Result<String> {
        match spec {
            VersionSpec::Exact(version) => {
                if available_versions.contains(version) {
                    Ok(version.clone())
                } else {
                    Err(anyhow::anyhow!("Version {} not found", version))
                }
            },
            VersionSpec::Latest => {
                self.get_latest_version(available_versions)
            },
            VersionSpec::Range(range) => {
                self.resolve_version_range(range, available_versions)
            },
            VersionSpec::Tag(tag) => {
                // For now, treat tags as exact versions
                if available_versions.contains(tag) {
                    Ok(tag.clone())
                } else {
                    Err(anyhow::anyhow!("Tag {} not found", tag))
                }
            }
        }
    }

    fn get_latest_version(&self, versions: &[String]) -> Result<String> {
        if versions.is_empty() {
            return Err(anyhow::anyhow!("No versions available"));
        }

        // Simple version sorting - in production use semver crate
        let mut sorted_versions = versions.to_vec();
        sorted_versions.sort_by(|a, b| {
            self.compare_versions(a, b)
        });

        Ok(sorted_versions.last().unwrap().clone())
    }

    fn resolve_version_range(&self, range: &str, versions: &[String]) -> Result<String> {
        // Simplified semver range resolution
        if range.starts_with('^') {
            let base_version = &range[1..];
            self.find_compatible_version(base_version, versions, false)
        } else if range.starts_with('~') {
            let base_version = &range[1..];
            self.find_compatible_version(base_version, versions, true)
        } else {
            // Treat as exact version
            if versions.contains(&range.to_string()) {
                Ok(range.to_string())
            } else {
                Err(anyhow::anyhow!("Version {} not found", range))
            }
        }
    }

    fn find_compatible_version(&self, base: &str, versions: &[String], patch_only: bool) -> Result<String> {
        let base_parts: Vec<&str> = base.split('.').collect();
        if base_parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid version format: {}", base));
        }

        let base_major: u32 = base_parts[0].parse()?;
        let base_minor: u32 = base_parts[1].parse()?;
        let base_patch: u32 = base_parts[2].parse()?;

        let mut compatible_versions = Vec::new();

        for version in versions {
            let parts: Vec<&str> = version.split('.').collect();
            if parts.len() != 3 { continue; }

            if let (Ok(major), Ok(minor), Ok(patch)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>()
            ) {
                let is_compatible = if patch_only {
                    // ~1.2.3 := >=1.2.3 <1.(2+1).0
                    major == base_major && minor == base_minor && patch >= base_patch
                } else {
                    // ^1.2.3 := >=1.2.3 <(1+1).0.0
                    major == base_major && 
                    (minor > base_minor || (minor == base_minor && patch >= base_patch))
                };

                if is_compatible {
                    compatible_versions.push(version.clone());
                }
            }
        }

        if compatible_versions.is_empty() {
            return Err(anyhow::anyhow!("No compatible version found for {}", base));
        }

        // Return highest compatible version
        self.get_latest_version(&compatible_versions)
    }

    fn compare_versions(&self, a: &str, b: &str) -> std::cmp::Ordering {
        let a_parts: Vec<u32> = a.split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let b_parts: Vec<u32> = b.split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        for i in 0..3 {
            let a_part = a_parts.get(i).unwrap_or(&0);
            let b_part = b_parts.get(i).unwrap_or(&0);
            
            match a_part.cmp(b_part) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }

        std::cmp::Ordering::Equal
    }
}

impl PackageSpec {
    pub fn parse(spec: &str) -> Result<Self> {
        // URL-based import
        if spec.starts_with("http://") || spec.starts_with("https://") {
            return Self::parse_url_spec(spec);
        }

        // kiren: prefix
        if spec.starts_with("kiren:") {
            let spec = &spec[6..]; // Remove "kiren:" prefix
            return Self::parse_registry_spec(spec);
        }

        // Local path
        if spec.starts_with("./") || spec.starts_with("../") || spec.starts_with("/") {
            return Ok(PackageSpec {
                name: spec.to_string(),
                version: VersionSpec::Latest,
                source: PackageSource::Local(spec.to_string()),
            });
        }

        // Git URL
        if spec.starts_with("git://") || spec.contains("git@") {
            return Self::parse_git_spec(spec);
        }

        // Default to registry
        Self::parse_registry_spec(spec)
    }

    fn parse_url_spec(spec: &str) -> Result<Self> {
        // Parse https://kiren.dev/express@4.18.2
        let url_regex = Regex::new(r"^(https?://.+?)([/@])([^@/]+)@(.+)$").unwrap();
        
        if let Some(captures) = url_regex.captures(spec) {
            let _base_url = captures.get(1).unwrap().as_str();
            let name = captures.get(3).unwrap().as_str();
            let version = captures.get(4).unwrap().as_str();
            
            Ok(PackageSpec {
                name: name.to_string(),
                version: Self::parse_version_spec(version)?,
                source: PackageSource::Url(spec.to_string()),
            })
        } else {
            Err(anyhow::anyhow!("Invalid URL spec: {}", spec))
        }
    }

    fn parse_registry_spec(spec: &str) -> Result<Self> {
        // Parse package@version or just package
        if let Some(at_pos) = spec.rfind('@') {
            let name = spec[..at_pos].to_string();
            let version_str = &spec[at_pos + 1..];
            
            Ok(PackageSpec {
                name,
                version: Self::parse_version_spec(version_str)?,
                source: PackageSource::Registry,
            })
        } else {
            Ok(PackageSpec {
                name: spec.to_string(),
                version: VersionSpec::Latest,
                source: PackageSource::Registry,
            })
        }
    }

    fn parse_git_spec(spec: &str) -> Result<Self> {
        // Simple git parsing for now
        let name = spec.split('/').last()
            .unwrap_or("unknown")
            .replace(".git", "");
            
        Ok(PackageSpec {
            name,
            version: VersionSpec::Latest,
            source: PackageSource::Git {
                url: spec.to_string(),
                commit: None,
            },
        })
    }

    fn parse_version_spec(version_str: &str) -> Result<VersionSpec> {
        match version_str {
            "latest" => Ok(VersionSpec::Latest),
            v if v.starts_with('^') || v.starts_with('~') => Ok(VersionSpec::Range(v.to_string())),
            v if v.chars().next().unwrap_or('0').is_ascii_digit() => Ok(VersionSpec::Exact(v.to_string())),
            tag => Ok(VersionSpec::Tag(tag.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_registry_spec() {
        let spec = PackageSpec::parse("express@4.18.2").unwrap();
        assert_eq!(spec.name, "express");
        assert_eq!(spec.version, VersionSpec::Exact("4.18.2".to_string()));
        assert_eq!(spec.source, PackageSource::Registry);
    }

    #[test]
    fn test_parse_url_spec() {
        let spec = PackageSpec::parse("https://kiren.dev/express@4.18.2").unwrap();
        assert_eq!(spec.name, "express");
        assert_eq!(spec.version, VersionSpec::Exact("4.18.2".to_string()));
    }

    #[test]
    fn test_parse_kiren_spec() {
        let spec = PackageSpec::parse("kiren:express@latest").unwrap();
        assert_eq!(spec.name, "express");
        assert_eq!(spec.version, VersionSpec::Latest);
        assert_eq!(spec.source, PackageSource::Registry);
    }
}