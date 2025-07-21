use crate::package::config::*;

#[test]
fn test_toml_serialization() {
    let config = ProjectConfig::new("test-app");
    let toml_string = config.to_toml().unwrap();
    let parsed_config = ProjectConfig::from_toml(&toml_string).unwrap();

    assert_eq!(config.package.name, parsed_config.package.name);
    assert_eq!(config.package.version, parsed_config.package.version);
}

#[test]
fn test_toml_roundtrip() {
    let mut config = ProjectConfig::new("roundtrip-test");
    
    // Add some data
    config.add_dependency("react", "^18.0.0");
    config.add_script("build", "kiren build");
    config.add_config("env", "production");
    
    // Serialize to TOML
    let toml_string = config.to_toml().unwrap();
    
    // Deserialize back
    let parsed_config = ProjectConfig::from_toml(&toml_string).unwrap();
    
    // Verify all data is preserved
    assert_eq!(config.package.name, parsed_config.package.name);
    assert_eq!(config.dependencies, parsed_config.dependencies);
    assert_eq!(config.scripts, parsed_config.scripts);
    assert_eq!(config.config, parsed_config.config);
}

#[test]
fn test_toml_with_optional_fields() {
    let mut config = ProjectConfig::new("optional-test");
    
    // Set optional fields
    config.package.description = Some("Test description".to_string());
    config.package.author = Some("Test Author".to_string());
    config.package.license = Some("MIT".to_string());
    
    let toml_string = config.to_toml().unwrap();
    let parsed_config = ProjectConfig::from_toml(&toml_string).unwrap();
    
    assert_eq!(config.package.description, parsed_config.package.description);
    assert_eq!(config.package.author, parsed_config.package.author);
    assert_eq!(config.package.license, parsed_config.package.license);
}

#[test]
fn test_invalid_toml_parsing() {
    let invalid_toml = "invalid toml content [[[";
    let result = ProjectConfig::from_toml(invalid_toml);
    
    assert!(result.is_err());
}

#[test]
fn test_empty_toml_parsing() {
    // Empty TOML should fail since required fields are missing
    let empty_toml = "";
    let result = ProjectConfig::from_toml(empty_toml);
    
    assert!(result.is_err());
}

#[test]
fn test_minimal_valid_toml() {
    let minimal_toml = r#"
[package]
name = "minimal-app"
version = "1.0.0"
runtime = "kiren"
"#;
    
    let result = ProjectConfig::from_toml(minimal_toml);
    assert!(result.is_ok());
    
    let config = result.unwrap();
    assert_eq!(config.package.name, "minimal-app");
    assert_eq!(config.package.version, "1.0.0");
    assert_eq!(config.package.runtime, "kiren");
}