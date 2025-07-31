//! Unit tests for configuration module

use homomorphic_llm_proxy::config::{Config, ServerConfig, EncryptionConfig};

#[test]  
fn test_default_config() {
    let config = Config::default();
    
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.workers, 4);
    
    assert_eq!(config.encryption.poly_modulus_degree, 16384);
    assert_eq!(config.encryption.scale_bits, 40);
    
    assert_eq!(config.llm.provider, "openai");
    assert_eq!(config.llm.timeout_seconds, 300);
    
    assert_eq!(config.privacy.epsilon_per_query, 0.1);
    assert_eq!(config.privacy.max_queries_per_user, 1000);
}

#[test]
fn test_server_config_validation() {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 3000,
        workers: 8,
    };
    
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3000);
    assert_eq!(config.workers, 8);
}

#[test]
fn test_encryption_config_validation() {
    let config = EncryptionConfig {
        poly_modulus_degree: 8192,  
        coeff_modulus_bits: vec![50, 30, 30, 50],
        scale_bits: 30,
    };
    
    assert_eq!(config.poly_modulus_degree, 8192);
    assert_eq!(config.coeff_modulus_bits.len(), 4);
    assert_eq!(config.scale_bits, 30);
}

#[cfg(test)]
mod config_loading_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_load_missing_config_returns_default() {
        let result = Config::load();
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.server.port, 8080); // Default value
    }
    
    #[test]
    fn test_load_valid_config_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        
        let config_content = r#"
[server]
host = "localhost"
port = 9000
workers = 2

[encryption]
poly_modulus_degree = 32768
coeff_modulus_bits = [60, 40, 60]  
scale_bits = 50
"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        // Change to temp dir to test loading
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        
        let result = Config::load();
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
        
        assert!(result.is_ok());
        let config = result.unwrap();
        
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.server.workers, 2);
        assert_eq!(config.encryption.poly_modulus_degree, 32768);
        assert_eq!(config.encryption.scale_bits, 50);
    }
}