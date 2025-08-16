//! Unit tests for FHE module

use homomorphic_llm_proxy::fhe::{FheEngine, FheParams};
use uuid::Uuid;

#[test]
fn test_fhe_engine_creation() {
    let params = FheParams::default();
    let result = FheEngine::new(params);
    
    assert!(result.is_ok(), "FHE engine creation should succeed");
}

#[test] 
fn test_fhe_default_creation() {
    let engine = FheEngine::default();
    let stats = engine.get_encryption_stats();
    
    assert_eq!(stats.security_level, 128);
    assert_eq!(stats.poly_modulus_degree, 16384);
}

#[cfg(test)]
mod fhe_operations_tests {
    use super::*;
    
    fn setup_test_engine() -> (FheEngine, Uuid) {
        let params = FheParams::default();
        let mut engine = FheEngine::new(params).expect("Failed to create engine");
        let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");
        (engine, client_id)
    }
    
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (engine, client_id) = setup_test_engine();
        let plaintext = "Hello, FHE World!";
        
        let ciphertext = engine.encrypt_text(client_id, plaintext).unwrap();
        let decrypted = engine.decrypt_text_safe(client_id, &ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
    
    #[test]
    fn test_ciphertext_validation() {
        let (engine, client_id) = setup_test_engine();
        let plaintext = "Test validation";
        
        let ciphertext = engine.encrypt_text(client_id, plaintext).unwrap();
        let is_valid = engine.validate_ciphertext(&ciphertext).unwrap();
        
        assert!(is_valid, "Valid ciphertext should pass validation");
    }
    
    #[test]
    fn test_concatenate_ciphertexts() {
        let (engine, client_id) = setup_test_engine();
        let text_a = "Hello ";
        let text_b = "World!";
        
        let ciphertext_a = engine.encrypt_text(client_id, text_a).unwrap();
        let ciphertext_b = engine.encrypt_text(client_id, text_b).unwrap();
        
        let concatenated = engine.concatenate_encrypted(&ciphertext_a, &ciphertext_b).unwrap();
        
        assert!(!concatenated.data.is_empty());
        assert!(concatenated.noise_budget.is_some());
    }
    
    #[test]
    fn test_encrypt_empty_data() {
        let (engine, client_id) = setup_test_engine();
        let result = engine.encrypt_text(client_id, "");
        
        assert!(result.is_err(), "Empty plaintext should return error");
    }
    
    #[test]
    fn test_encrypt_invalid_client() {
        let (engine, _) = setup_test_engine();
        let invalid_client_id = Uuid::new_v4();
        
        let result = engine.encrypt_text(invalid_client_id, "test");
        
        assert!(result.is_err(), "Invalid client ID should return error");
    }
    
    #[test]
    fn test_key_rotation() {
        let (mut engine, client_id) = setup_test_engine();
        
        let new_server_id = engine.rotate_keys(client_id).unwrap();
        
        assert_ne!(new_server_id, Uuid::nil());
    }
    
    #[test]
    fn test_encryption_statistics() {
        let (engine, _) = setup_test_engine();
        let stats = engine.get_encryption_stats();
        
        assert!(stats.total_client_keys > 0);
        assert!(stats.total_server_keys > 0);
        assert_eq!(stats.security_level, 128);
    }
    
    #[test]
    fn test_cost_estimation() {
        let (engine, _) = setup_test_engine();
        
        let encrypt_cost = engine.estimate_cost("encrypt", 100).unwrap();
        let decrypt_cost = engine.estimate_cost("decrypt", 100).unwrap();
        
        assert!(encrypt_cost > 0);
        assert!(decrypt_cost > 0);
        assert!(encrypt_cost > decrypt_cost); // Encryption typically more expensive
    }
    
    #[test]
    fn test_noise_budget_tracking() {
        let (engine, client_id) = setup_test_engine();
        let plaintext = "Test noise budget";
        
        let ciphertext = engine.encrypt_text(client_id, plaintext).unwrap();
        
        assert!(ciphertext.noise_budget.is_some());
        assert!(ciphertext.noise_budget.unwrap() > 0);
    }
}