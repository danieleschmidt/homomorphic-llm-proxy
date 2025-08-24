//! Integration tests for FHE LLM Proxy

use homomorphic_llm_proxy::{
    fhe::{FheEngine, FheParams},
    validation::ValidationFramework,
};
use uuid::Uuid;

#[tokio::test]
async fn test_complete_fhe_workflow() {
    // Test the complete FHE workflow
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");

    // Generate keys
    let (client_id, _server_id) = engine.generate_keys().expect("Failed to generate keys");

    // Test encryption
    let plaintext = "Hello, secure FHE world! This is a comprehensive test.";
    let ciphertext = engine
        .encrypt_text(client_id, plaintext)
        .expect("Failed to encrypt");

    assert!(!ciphertext.data.is_empty());
    assert!(ciphertext.noise_budget.is_some());
    assert!(ciphertext.noise_budget.unwrap() > 0);

    // Test decryption
    let decrypted = engine
        .decrypt_text(client_id, &ciphertext)
        .expect("Failed to decrypt");
    assert_eq!(plaintext, decrypted);

    // Test validation
    assert!(engine
        .validate_ciphertext(&ciphertext)
        .expect("Validation failed"));

    // Test stats
    let stats = engine.get_stats();
    assert_eq!(stats.total_client_keys, 1);
    assert_eq!(stats.total_server_keys, 1);
}

#[tokio::test]
async fn test_validation_framework_integration() {
    let framework = ValidationFramework::with_fhe_defaults().expect("Failed to create validator");

    // Test valid input
    let report = framework.validate_input("plaintext", "Valid test input");
    assert!(report.is_valid);
    assert!(report.errors.is_empty());
    assert!(report.sanitized_input.is_some());

    // Test invalid input
    let report = framework.validate_input("plaintext", "");
    assert!(!report.is_valid);
    assert!(!report.errors.is_empty());

    // Test security threat detection
    let threats = framework.detect_security_threats("<script>alert('hack')</script>");
    assert!(!threats.is_empty());
    assert!(threats.iter().any(|t| t.contains("XSS")));

    // Test FHE params validation
    let params = FheParams::default();
    let report = framework.validate_fhe_params(&params);
    assert!(report.is_valid);

    // Test UUID validation
    let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let report = framework.validate_uuid(valid_uuid);
    assert!(report.is_valid);

    let invalid_uuid = "invalid-uuid-format";
    let report = framework.validate_uuid(invalid_uuid);
    assert!(!report.is_valid);
}

#[tokio::test]
async fn test_error_handling() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");

    let (client_id, _server_id) = engine.generate_keys().expect("Failed to generate keys");

    // Test invalid client ID
    let fake_client_id = Uuid::new_v4();
    let result = engine.encrypt_text(fake_client_id, "test");
    assert!(result.is_err());

    // Test empty plaintext
    let result = engine.encrypt_text(client_id, "");
    assert!(result.is_err());

    // Test too long plaintext
    let long_text = "x".repeat(15000);
    let result = engine.encrypt_text(client_id, &long_text);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");

    // Generate multiple key pairs
    let mut key_pairs = Vec::new();
    for _ in 0..5 {
        let keys = engine.generate_keys().expect("Failed to generate keys");
        key_pairs.push(keys);
    }

    // Test concurrent encryption/decryption
    for (i, (client_id, _)) in key_pairs.into_iter().enumerate() {
        let plaintext = format!("Test message {}", i);
        let ciphertext = engine
            .encrypt_text(client_id, &plaintext)
            .expect("Failed to encrypt");

        // In a real async scenario, we'd spawn tasks here
        let decrypted = engine
            .decrypt_text(client_id, &ciphertext)
            .expect("Failed to decrypt");
        assert_eq!(plaintext, decrypted);
    }

    // Verify engine stats
    let stats = engine.get_stats();
    assert_eq!(stats.total_client_keys, 5);
    assert_eq!(stats.total_server_keys, 5);
}

#[tokio::test]
async fn test_security_features() {
    let framework = ValidationFramework::with_fhe_defaults().expect("Failed to create validator");

    // Test SQL injection detection
    let sql_input = "'; DROP TABLE users; --";
    let threats = framework.detect_security_threats(sql_input);
    assert!(threats.iter().any(|t| t.contains("SQL injection")));

    // Test XSS detection
    let xss_input = "<script>alert('xss')</script>";
    let threats = framework.detect_security_threats(xss_input);
    assert!(threats.iter().any(|t| t.contains("XSS")));

    // Test command injection detection
    let cmd_input = "; rm -rf /";
    let threats = framework.detect_security_threats(cmd_input);
    assert!(threats.iter().any(|t| t.contains("command injection")));

    // Test path traversal detection
    let path_input = "../../../etc/passwd";
    let threats = framework.detect_security_threats(path_input);
    assert!(threats.iter().any(|t| t.contains("path traversal")));
}

#[tokio::test]
async fn test_base64_validation() {
    let framework = ValidationFramework::new();

    // Test valid base64
    let valid_b64 = "SGVsbG8gV29ybGQ="; // "Hello World"
    let report = framework.validate_ciphertext_data(valid_b64);
    assert!(report.is_valid);

    // Test invalid base64
    let invalid_b64 = "This is not valid base64!@#$%";
    let report = framework.validate_ciphertext_data(invalid_b64);
    assert!(!report.is_valid);
    assert!(!report.errors.is_empty());

    // Test empty base64
    let empty_b64 = "";
    let report = framework.validate_ciphertext_data(empty_b64);
    assert!(!report.is_valid);
}

#[tokio::test]
async fn test_fhe_params_edge_cases() {
    let framework = ValidationFramework::new();

    // Test valid params
    let valid_params = FheParams::default();
    let report = framework.validate_fhe_params(&valid_params);
    assert!(report.is_valid);

    // Test invalid params - zero poly modulus degree
    let mut invalid_params = FheParams::default();
    invalid_params.poly_modulus_degree = 0;
    let report = framework.validate_fhe_params(&invalid_params);
    assert!(!report.is_valid);
    assert!(report
        .errors
        .iter()
        .any(|e| e.field == "poly_modulus_degree"));

    // Test low security level
    let mut low_security_params = FheParams::default();
    low_security_params.security_level = 64; // Too low
    let report = framework.validate_fhe_params(&low_security_params);
    assert!(!report.is_valid);
    assert!(report.errors.iter().any(|e| e.field == "security_level"));

    // Test empty coefficient modulus
    let mut empty_coeff_params = FheParams::default();
    empty_coeff_params.coeff_modulus_bits = vec![];
    let report = framework.validate_fhe_params(&empty_coeff_params);
    assert!(!report.is_valid);
    assert!(report
        .errors
        .iter()
        .any(|e| e.field == "coeff_modulus_bits"));
}
