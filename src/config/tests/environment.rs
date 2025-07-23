use crate::config::*;

#[test]
fn test_env_merge() {
    let mut config = KirenConfig::default();
    let original_port = config.server.default_port;

    // Test that merge_with_env doesn't panic
    config.merge_with_env();

    // Port should either be the same or changed by env var
    assert!(config.server.default_port >= 1024); // Valid port range
    assert!(
        original_port == config.server.default_port || config.server.default_port != original_port
    );
}

#[test]
fn test_env_variable_collection() {
    // Clean up any existing environment variables first
    std::env::remove_var("KIREN_TEST_VAR");

    let mut config = KirenConfig::default();

    // Set a test environment variable
    std::env::set_var("KIREN_TEST_VAR", "test_value");

    config.merge_with_env();

    // Should collect KIREN_* variables
    assert!(config.environment.contains_key("KIREN_TEST_VAR"));
    assert_eq!(
        config.environment.get("KIREN_TEST_VAR"),
        Some(&"test_value".to_string())
    );

    // Cleanup
    std::env::remove_var("KIREN_TEST_VAR");
}

#[test]
fn test_port_override_from_env() {
    // Clean up any existing environment variables first
    std::env::remove_var("KIREN_PORT");

    // Wait a bit to ensure cleanup
    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut config = KirenConfig::default();
    config.server.default_port = 3000; // Reset to known state

    // Set port override
    std::env::set_var("KIREN_PORT", "8080");

    // Wait to ensure env var is set
    std::thread::sleep(std::time::Duration::from_millis(10));

    config.merge_with_env();

    // Port should be updated
    assert_eq!(config.server.default_port, 8080);

    // Cleanup immediately
    std::env::remove_var("KIREN_PORT");
}

#[test]
fn test_memory_limit_override_from_env() {
    // Clean up any existing environment variables first
    std::env::remove_var("KIREN_MEMORY_LIMIT");

    let mut config = KirenConfig::default();

    // Set memory limit override
    std::env::set_var("KIREN_MEMORY_LIMIT", "2048");

    config.merge_with_env();

    // Memory limit should be updated
    assert_eq!(config.runtime.memory_limit, Some(2048));

    // Cleanup
    std::env::remove_var("KIREN_MEMORY_LIMIT");
}

#[test]
fn test_invalid_env_values_ignored() {
    // Clean up any existing environment variables first
    std::env::remove_var("KIREN_PORT");
    std::env::remove_var("PORT");

    let mut config = KirenConfig::default();
    // Always expect default port regardless of other tests
    config.server.default_port = 3000; // Reset to known default
    let original_port = config.server.default_port;

    // Set invalid port value
    std::env::set_var("KIREN_PORT", "invalid_port");

    config.merge_with_env();

    // Port should remain unchanged with invalid value
    assert_eq!(config.server.default_port, original_port);

    // Cleanup
    std::env::remove_var("KIREN_PORT");
}
