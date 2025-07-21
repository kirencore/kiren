use crate::package::config::*;

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
fn test_package_info_validation() {
    let mut package_info = PackageInfo {
        name: "valid-name".to_string(),
        version: "1.0.0".to_string(),
        runtime: "kiren".to_string(),
        description: Some("Test package".to_string()),
        license: Some("MIT".to_string()),
        author: None,
    };

    // Valid package info should pass
    assert!(is_valid_semver(&package_info.version));
    assert!(!package_info.name.is_empty());

    // Invalid version should fail
    package_info.version = "invalid".to_string();
    assert!(!is_valid_semver(&package_info.version));
}

#[test]
fn test_project_validation_empty_name() {
    let mut config = ProjectConfig::new("valid-name");

    // Valid config should pass
    assert!(config.validate().is_ok());

    // Empty name should fail
    config.package.name = "".to_string();
    let result = config.validate();
    assert!(result.is_err());

    if let Err(errors) = result {
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("name")));
    }
}

#[test]
fn test_project_validation_invalid_version() {
    let mut config = ProjectConfig::new("test-app");

    // Valid config should pass
    assert!(config.validate().is_ok());

    // Invalid version should fail
    config.package.version = "not-a-version".to_string();
    let result = config.validate();
    assert!(result.is_err());

    if let Err(errors) = result {
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("version")));
    }
}

#[test]
fn test_project_validation_multiple_errors() {
    let mut config = ProjectConfig::new("test-app");

    // Create multiple validation errors
    config.package.name = "".to_string();
    config.package.version = "invalid".to_string();

    let result = config.validate();
    assert!(result.is_err());

    if let Err(errors) = result {
        assert!(errors.len() >= 2);
    }
}
