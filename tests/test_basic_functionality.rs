//! Basic integration tests for FHE LLM Proxy

use homomorphic_llm_proxy::config::Config;
use homomorphic_llm_proxy::fhe::{FheEngine, FheParams};
use homomorphic_llm_proxy::proxy::ProxyServer;
use uuid::Uuid;

#[tokio::test]
async fn test_fhe_engine_basic_operations() {
    // Test FHE engine basic functionality
    let params = FheParams::default();
    let mut fhe_engine = FheEngine::new(params).expect("Failed to create FHE engine");

    // Generate keys
    let (client_id, _server_id) = fhe_engine.generate_keys().expect("Failed to generate keys");

    // Test encryption and decryption
    let plaintext = "Hello, World!";
    let ciphertext = fhe_engine
        .encrypt_text(client_id, plaintext)
        .expect("Failed to encrypt");

    assert!(
        !ciphertext.data.is_empty(),
        "Ciphertext should not be empty"
    );
    assert_eq!(
        ciphertext.params.security_level, 128,
        "Security level should be 128"
    );

    let decrypted = fhe_engine
        .decrypt_text_safe(client_id, &ciphertext)
        .expect("Failed to decrypt");
    assert_eq!(decrypted, plaintext, "Decrypted text should match original");

    println!("✅ FHE Engine basic operations test passed");
}

#[tokio::test]
async fn test_config_loading() {
    // Test configuration loading
    let config = Config::load().expect("Failed to load config");

    // Validate configuration
    config.validate().expect("Config validation failed");

    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.encryption.security_level, 128);

    println!("✅ Configuration loading test passed");
}

#[tokio::test]
async fn test_proxy_server_creation() {
    // Test proxy server creation
    let config = Config::load().expect("Failed to load config");
    let server = ProxyServer::new(config).expect("Failed to create proxy server");

    // Server creation should succeed
    println!("✅ Proxy server creation test passed");
}

#[test]
fn test_fhe_params_serialization() {
    // Test FHE parameters serialization
    let params = FheParams::default();

    let serialized = serde_json::to_string(&params).expect("Failed to serialize params");
    let deserialized: FheParams =
        serde_json::from_str(&serialized).expect("Failed to deserialize params");

    assert_eq!(params.poly_modulus_degree, deserialized.poly_modulus_degree);
    assert_eq!(params.security_level, deserialized.security_level);

    println!("✅ FHE parameters serialization test passed");
}

#[tokio::test]
async fn test_error_handling() {
    // Test error handling
    let params = FheParams::default();
    let fhe_engine = FheEngine::new(params).expect("Failed to create FHE engine");

    // Try to encrypt with non-existent client ID
    let fake_client_id = Uuid::new_v4();
    let result = fhe_engine.encrypt_text(fake_client_id, "test");

    assert!(result.is_err(), "Should fail with non-existent client ID");

    println!("✅ Error handling test passed");
}
