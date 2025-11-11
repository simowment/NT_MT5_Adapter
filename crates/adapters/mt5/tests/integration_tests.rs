//! Integration tests for the MT5 adapter.

use nautilus_adapters_mt5::common::Mt5Credential;
use nautilus_adapters_mt5::common::Mt5Url;
use nautilus_adapters_mt5::config::Mt5Config;

#[test]
fn test_http_client_initialization() {
    let cred = Mt5Credential::builder()
        .login("test_user")
        .password("test_pass")
        .server("test_server")
        .build()
        .expect("Failed to build credential");

    let url = Mt5Url::new("http://localhost:8080");
    let config = Mt5Config {
        base_url: "http://localhost:8080".to_string(),
        ws_url: "ws://localhost:8080".to_string(),
        http_timeout: 30,
        ws_timeout: 30,
        proxy: None,
    };

    let result = nautilus_adapters_mt5::http::Mt5HttpClient::new(config, cred, url);
    assert!(result.is_ok(), "Should successfully create HTTP client");
}

#[test]
fn test_credential_with_proxy() {
    let cred = Mt5Credential::builder()
        .login("test_user")
        .password("test_pass")
        .server("test_server")
        .proxy("http://proxy:3128".to_string()) // Fix: proxy() expects String, not &str
        .build()
        .expect("Failed to build credential");

    assert_eq!(cred.proxy, Some("http://proxy:3128".to_string()));
}
