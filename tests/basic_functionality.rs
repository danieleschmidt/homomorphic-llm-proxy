//! Basic functionality integration tests

use homomorphic_llm_proxy::fhe::{FheEngine, FheParams};
use uuid::Uuid;

#[tokio::test]
async fn test_fhe_engine_creation() {
    let params = FheParams::default();
    let engine = FheEngine::new(params);
    assert!(engine.is_ok());
    
    let engine = engine.unwrap();
    assert_eq!(engine.get_params().security_level, 128);
}

#[tokio::test] 
async fn test_key_generation() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");
    
    let result = engine.generate_keys();
    assert!(result.is_ok());
    
    let (client_id, server_id) = result.unwrap();
    assert_ne!(client_id, Uuid::nil());
    assert_ne!(server_id, Uuid::nil());
    
    let stats = engine.get_stats();
    assert_eq!(stats.total_client_keys, 1);
    assert_eq!(stats.total_server_keys, 1);
}

#[tokio::test]
async fn test_encryption_decryption() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");
    
    let (client_id, _server_id) = engine.generate_keys().expect("Failed to generate keys");
    
    let plaintext = "Hello, FHE World!";
    let ciphertext = engine.encrypt_text(client_id, plaintext).expect("Failed to encrypt");
    
    assert!(!ciphertext.data.is_empty());
    assert!(ciphertext.noise_budget.is_some());
    
    let decrypted = engine.decrypt_text(client_id, &ciphertext).expect("Failed to decrypt");
    assert_eq!(plaintext, decrypted);
}

#[tokio::test]
async fn test_input_validation() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");
    
    let (client_id, _server_id) = engine.generate_keys().expect("Failed to generate keys");
    
    // Test empty input
    let result = engine.encrypt_text(client_id, "");
    assert!(result.is_err());
    
    // Test too long input
    let long_text = "a".repeat(20000);
    let result = engine.encrypt_text(client_id, &long_text);
    assert!(result.is_err());
    
    // Test malicious input
    let malicious = "Hello <script>alert('xss')</script>";
    let result = engine.encrypt_text(client_id, malicious);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ciphertext_validation() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");
    
    let (client_id, _server_id) = engine.generate_keys().expect("Failed to generate keys");
    
    let ciphertext = engine.encrypt_text(client_id, "Valid text").expect("Failed to encrypt");
    let is_valid = engine.validate_ciphertext(&ciphertext).expect("Validation failed");
    
    assert!(is_valid);
}