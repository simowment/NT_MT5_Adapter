//! Integration tests for the MT5 adapter.

use nautilus_adapters_mt5::credential::Mt5Credential;
use nautilus_adapters_mt5::client::http::HttpClient;
use nautilus_adapters_mt5::urls::Mt5Url;

#[test]
fn test_http_client_initialization() {
    let cred = Mt5Credential::builder()
        .login("test_user")
        .password("test_pass")
        .server("test_server")
        .build()
        .expect("Failed to build credential");

    let url = Mt5Url::new("http://localhost:8080");
    let client = HttpClient::new(cred, url);

    assert_eq!(client.credential().login, "test_user");
    assert_eq!(client.url().base_url(), "http://localhost:8080");
}

#[test]
fn test_credential_with_proxy() {
    let cred = Mt5Credential::builder()
        .login("test_user")
        .password("test_pass")
        .server("test_server")
        .proxy("http://proxy:3128")
        .build()
        .expect("Failed to build credential");

    assert_eq!(cred.proxy, Some("http://proxy:3128".to_string()));
}
