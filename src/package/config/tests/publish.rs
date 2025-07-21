use crate::package::config::*;

#[test]
fn test_publish_config_defaults() {
    let publish_config = PublishConfig::default();
    
    assert_eq!(publish_config.registry, None);
    assert!(matches!(publish_config.access, AccessLevel::Public));
    assert!(publish_config.ignore.is_some());
    assert!(!publish_config.ignore.unwrap().is_empty());
}

#[test]
fn test_publish_config_creation() {
    let publish_config = PublishConfig {
        registry: Some("https://registry.kiren.dev".to_string()),
        access: AccessLevel::Private,
        files: Some(vec!["src/**".to_string(), "README.md".to_string()]),
        ignore: Some(vec!["*.test.js".to_string()]),
        keywords: vec!["javascript".to_string(), "runtime".to_string()],
        categories: vec!["development-tools".to_string()],
    };
    
    assert_eq!(publish_config.registry, Some("https://registry.kiren.dev".to_string()));
    assert!(matches!(publish_config.access, AccessLevel::Private));
    assert!(publish_config.files.is_some());
    assert_eq!(publish_config.keywords.len(), 2);
    assert_eq!(publish_config.categories.len(), 1);
}

#[test]
fn test_access_level_serialization() {
    use serde_json;
    
    // Test Public access level
    let public_access = AccessLevel::Public;
    let json = serde_json::to_string(&public_access).unwrap();
    assert_eq!(json, "\"public\"");
    
    let parsed_public: AccessLevel = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed_public, AccessLevel::Public));
    
    // Test Private access level
    let private_access = AccessLevel::Private;
    let json = serde_json::to_string(&private_access).unwrap();
    assert_eq!(json, "\"private\"");
    
    let parsed_private: AccessLevel = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed_private, AccessLevel::Private));
}

#[test]
fn test_publish_config_with_project() {
    let project_config = ProjectConfig::new("publish-test");
    
    // Test that publish config can be integrated with project config
    // (This would be extended if publish config becomes part of project config)
    let publish_config = PublishConfig::default();
    
    assert!(matches!(publish_config.access, AccessLevel::Public));
    assert!(publish_config.ignore.is_some());
    
    // Verify project config is still valid
    assert_eq!(project_config.package.name, "publish-test");
}

#[test]
fn test_default_ignore_patterns() {
    let publish_config = PublishConfig::default();
    let ignore_patterns = publish_config.ignore.unwrap();
    
    // Check for common ignore patterns
    assert!(ignore_patterns.contains(&"node_modules/".to_string()));
    assert!(ignore_patterns.contains(&".git/".to_string()));
    assert!(ignore_patterns.contains(&"*.log".to_string()));
    assert!(ignore_patterns.contains(&".DS_Store".to_string()));
}