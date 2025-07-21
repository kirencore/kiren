use crate::config::*;

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