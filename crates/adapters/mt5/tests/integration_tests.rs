//! Integration tests for the MT5 adapter.

use nautilus_mt5::common::Mt5Credential;
use nautilus_mt5::config::Mt5Config;

#[test]
fn test_http_client_initialization() {
    let config = Mt5Config {
        base_url: "http://localhost:5000".to_string(),
        http_timeout: 30,
        proxy: None,
    };

    let result = nautilus_mt5::http::Mt5HttpClient::new(
        config,
        "http://localhost:5000".to_string(),
    );
    assert!(result.is_ok(), "Should successfully create HTTP client");
}

#[test]
fn test_credential_builder() {
    let cred = Mt5Credential::builder()
        .login("test_user")
        .password("test_pass")
        .server("test_server")
        .build()
        .expect("Failed to build credential");

    assert_eq!(cred.login, "test_user");
    assert_eq!(cred.password, "test_pass");
    assert_eq!(cred.server, "test_server");
    assert_eq!(cred.proxy, None);
}

#[test]
fn test_credential_with_proxy() {
    let cred = Mt5Credential::builder()
        .login("test_user")
        .password("test_pass")
        .server("test_server")
        .proxy("http://proxy:3128".to_string())
        .build()
        .expect("Failed to build credential");

    assert_eq!(cred.proxy, Some("http://proxy:3128".to_string()));
}

#[test]
fn test_config_default() {
    let config = Mt5Config::default();
    assert_eq!(config.base_url, "http://localhost:5000");
    assert_eq!(config.http_timeout, 30);
    assert_eq!(config.proxy, None);
}

#[test]
fn test_config_with_base_url() {
    let config = Mt5Config::with_base_url("http://custom:8080".to_string());
    assert_eq!(config.base_url, "http://custom:8080");
    assert_eq!(config.http_timeout, 30);
}
