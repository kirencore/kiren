use crate::package::config::*;

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
fn test_dependency_management() {
    let mut config = ProjectConfig::new("test-app");
    
    // Test adding dependencies
    config.add_dependency("lodash", "^4.17.21");
    config.add_dev_dependency("typescript", "^4.9.0");
    
    let all_deps = config.get_all_dependencies(true);
    assert!(all_deps.contains_key("lodash"));
    assert!(all_deps.contains_key("typescript"));
    
    let prod_deps = config.get_all_dependencies(false);
    assert!(prod_deps.contains_key("lodash"));
    assert!(!prod_deps.contains_key("typescript"));
}

#[test]
fn test_script_management() {
    let mut config = ProjectConfig::new("test-app");
    
    config.add_script("test", "kiren test");
    config.add_script("build", "kiren build");
    
    assert_eq!(config.scripts.get("test"), Some(&"kiren test".to_string()));
    assert_eq!(config.scripts.get("build"), Some(&"kiren build".to_string()));
}

#[test]
fn test_config_value_management() {
    let mut config = ProjectConfig::new("test-app");
    
    config.add_config("debug", "true");
    config.add_config("port", "3000");
    
    assert_eq!(config.config.get("debug"), Some(&"true".to_string()));
    assert_eq!(config.config.get("port"), Some(&"3000".to_string()));
}