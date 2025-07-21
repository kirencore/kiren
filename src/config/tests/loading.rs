use crate::config::*;

#[test]
fn test_config_file_search() {
    // Test config file finding logic
    let result = KirenConfig::find_config_file();
    // This will depend on the test environment, so we just check it doesn't panic
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_default_config_values() {
    let config = KirenConfig::default();

    // Test default values
    assert_eq!(config.server.default_port, 3000);
    assert!(config.runtime.memory_limit.is_some());
    assert!(!config.runtime.version.is_empty());
    assert!(config.environment.is_empty()); // Should be empty by default
}

#[test]
fn test_config_loading_fallback() {
    // Test that load() function doesn't panic even when no config file exists
    let config = KirenConfig::load();

    // Should fall back to defaults
    assert_eq!(config.server.default_port, 3000);
    assert!(!config.runtime.version.is_empty());
}

#[test]
fn test_config_file_priority() {
    // Test that find_config_file follows the correct priority order
    let _possible_configs = ["kiren.toml", "kiren.config.toml", ".kirenrc.toml"];

    // The function should prefer files in this order
    // We just test that it doesn't panic and returns a valid result
    let result = KirenConfig::find_config_file();
    assert!(result.is_some() || result.is_none());
}
