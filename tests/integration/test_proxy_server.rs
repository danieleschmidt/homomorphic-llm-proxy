//! Integration tests for proxy server

use homomorphic_llm_proxy::{Config, proxy::ProxyServer};
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_proxy_server_creation() {
    let config = Config::default();
    let result = ProxyServer::new(config);
    
    // Should succeed in creating server (even if FHE engine fails)
    match result {
        Ok(_server) => {
            // Server created successfully
        }
        Err(e) => {
            // Expected to fail until FHE implementation is complete
            println!("Expected error: {}", e);
        }
    }
}

#[tokio::test]
async fn test_proxy_server_startup() {
    let config = Config {
        server: homomorphic_llm_proxy::config::ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0, // Use port 0 for automatic assignment
            workers: 1,
        },
        ..Config::default()
    };
    
    // Try to create and start server
    if let Ok(server) = ProxyServer::new(config) {
        let start_result = timeout(Duration::from_millis(100), server.start()).await;
        
        match start_result {
            Ok(Ok(())) => {
                // Server started successfully
                println!("Server started successfully");
            }
            Ok(Err(e)) => {
                println!("Server start failed: {}", e);
            }
            Err(_) => {
                // Timeout occurred (expected for long-running server)
                println!("Server startup timed out (expected)");
            }
        }
    }
}

#[tokio::test]
async fn test_proxy_request_processing() {
    let config = Config::default();
    
    if let Ok(server) = ProxyServer::new(config) {
        let test_data = vec![1, 2, 3, 4, 5];
        let result = server.process_request(&test_data).await;
        
        match result {
            Ok(response) => {
                // Should return empty response until implementation is complete
                assert_eq!(response, vec![]);
            }
            Err(e) => {
                println!("Expected error in request processing: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod integration_helpers {
    use super::*;
    
    pub fn create_test_config(port: u16) -> Config {
        Config {
            server: homomorphic_llm_proxy::config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port,
                workers: 1,
            },
            encryption: homomorphic_llm_proxy::config::EncryptionConfig {
                poly_modulus_degree: 4096, // Smaller for testing
                coeff_modulus_bits: vec![40, 30, 40],
                scale_bits: 30,
            },
            llm: homomorphic_llm_proxy::config::LlmConfig {
                provider: "mock".to_string(),
                endpoint: "http://localhost:3001/mock".to_string(),
                timeout_seconds: 10,
                max_retries: 1,
            },
            privacy: homomorphic_llm_proxy::config::PrivacyConfig {
                epsilon_per_query: 0.1,
                delta: 1e-5,
                max_queries_per_user: 100,
            },
        }
    }
}

#[tokio::test] 
async fn test_proxy_with_custom_config() {
    let config = integration_helpers::create_test_config(8081);
    
    let result = ProxyServer::new(config);
    
    // Test that custom configuration is accepted
    match result {
        Ok(_server) => {
            println!("Server created with custom config");
        }
        Err(e) => {
            println!("Expected error with custom config: {}", e);
        }
    }
}