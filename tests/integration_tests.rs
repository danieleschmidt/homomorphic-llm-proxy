//! Integration tests for the FHE LLM Proxy

use homomorphic_llm_proxy::fhe::{FheEngine, FheParams};
use homomorphic_llm_proxy::middleware::{MetricsCollector, PrivacyBudgetTracker, RateLimiter};
use homomorphic_llm_proxy::monitoring::{MonitoringService, PerformanceProfiler};
use homomorphic_llm_proxy::security::{
    AdaptiveRateLimiter, ApiKeyManager, InputValidator, Permission,
};
use homomorphic_llm_proxy::{Config, Error};
// Temporarily remove secrecy dependency
// use secrecy::SecretString;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Test configuration loading and validation
#[tokio::test]
async fn test_config_loading() {
    let config = Config::default();
    assert!(config.validate().is_ok());

    // Test with invalid configuration
    let mut invalid_config = config.clone();
    invalid_config.server.port = 0;
    assert!(invalid_config.validate().is_err());

    invalid_config.server.port = 8080;
    invalid_config.encryption.poly_modulus_degree = 15000; // Not power of 2
    assert!(invalid_config.validate().is_err());
}

/// Test FHE engine encryption/decryption workflow
#[tokio::test]
async fn test_fhe_encryption_workflow() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");

    // Generate keys
    let (client_id, _server_id) = engine.generate_keys().expect("Failed to generate keys");

    // Test encryption
    let plaintext = "Hello, secure world!";
    let ciphertext = engine
        .encrypt_text(client_id, plaintext)
        .expect("Failed to encrypt text");

    assert!(!ciphertext.data.is_empty());
    assert!(ciphertext.noise_budget.is_some());

    // Test decryption
    let decrypted = engine
        .decrypt_text_safe(client_id, &ciphertext)
        .expect("Failed to decrypt text");

    assert_eq!(decrypted, plaintext);

    // Test ciphertext validation
    assert!(engine.validate_ciphertext(&ciphertext).is_ok());

    // Test encryption statistics
    let stats = engine.get_encryption_stats();
    assert!(stats.total_client_keys > 0);
}

/// Test input validation and sanitization
#[test]
fn test_input_validation() {
    // Valid inputs
    assert!(InputValidator::sanitize_text("Hello world!").is_ok());
    assert!(InputValidator::sanitize_text("Testing with émojis 🔐").is_ok());

    // Invalid inputs
    assert!(InputValidator::sanitize_text("").is_err());
    assert!(InputValidator::sanitize_text(&"a".repeat(20000)).is_err());

    // Suspicious patterns
    assert!(InputValidator::sanitize_text("<script>alert('xss')</script>").is_err());
    assert!(InputValidator::sanitize_text("'; DROP TABLE users; --").is_err());
    assert!(InputValidator::sanitize_text("javascript:alert('xss')").is_err());

    // UUID validation
    assert!(InputValidator::validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
    assert!(InputValidator::validate_uuid("invalid-uuid").is_err());

    // FHE parameter validation
    assert!(InputValidator::validate_fhe_params(16384, 128).is_ok());
    assert!(InputValidator::validate_fhe_params(15000, 128).is_err());
    assert!(InputValidator::validate_fhe_params(16384, 99).is_err());

    // Base64 validation
    assert!(InputValidator::validate_base64("SGVsbG8gd29ybGQ=").is_ok());
    assert!(InputValidator::validate_base64("invalid-base64!@#").is_err());
}

/// Test API key management system
#[tokio::test]
async fn test_api_key_management() {
    let master_secret = "test-master-secret-key-12345".to_string();
    let mut manager = ApiKeyManager::new(master_secret);

    // Generate API key
    let permissions = vec![Permission::Encrypt, Permission::Decrypt];
    let api_key = manager.generate_api_key("test-user".to_string(), permissions.clone());

    assert!(!api_key.is_empty());
    assert!(api_key.len() > 20); // Reasonable length

    // Validate API key
    let validated_permissions = manager
        .validate_api_key(&api_key)
        .expect("Failed to validate API key");
    assert_eq!(validated_permissions.len(), 2);

    // Test permissions
    assert!(manager.has_permission(&api_key, &Permission::Encrypt));
    assert!(manager.has_permission(&api_key, &Permission::Decrypt));
    assert!(!manager.has_permission(&api_key, &Permission::Admin));

    // Test usage stats
    let (usage_count, last_used) = manager.get_key_stats(&api_key).unwrap();
    assert_eq!(usage_count, 1); // Used once during validation
    assert!(last_used.is_some());

    // Test key revocation
    assert!(manager.revoke_api_key(&api_key).is_ok());
    assert!(manager.validate_api_key(&api_key).is_err());

    // Test invalid key
    assert!(manager.validate_api_key("invalid-key").is_err());
}

/// Test rate limiting functionality
#[tokio::test]
async fn test_rate_limiting() {
    let mut limiter = RateLimiter::new(5); // 5 requests per minute
    let client_ip = "192.168.1.100";

    // Should allow first few requests
    for _ in 0..5 {
        assert!(limiter.check_rate_limit(client_ip).await.unwrap());
    }

    // Should block the 6th request
    assert!(!limiter.check_rate_limit(client_ip).await.unwrap());

    // Test stats
    let (requests, window_remaining) = limiter.get_client_stats(client_ip).await.unwrap();
    assert_eq!(requests, 6); // Including the blocked request
    assert!(window_remaining < Duration::from_secs(60));
}

/// Test adaptive rate limiting
#[tokio::test]
async fn test_adaptive_rate_limiting() {
    let mut limiter = AdaptiveRateLimiter::new(1000.0, 10.0); // Global limits
    let client_ip = "192.168.1.200";

    // Initially should allow requests
    assert!(limiter.check_request(client_ip, 1.0).unwrap());

    // Exhaust rate limit
    for _ in 0..100 {
        limiter.check_request(client_ip, 1.0).unwrap();
    }

    // Should eventually be rate limited
    assert!(!limiter.check_request(client_ip, 1.0).unwrap());

    // Test cleanup
    limiter.cleanup_old_entries();
}

/// Test metrics collection
#[tokio::test]
async fn test_metrics_collection() {
    let metrics = MetricsCollector::new();

    // Test basic increments
    metrics.increment_requests();
    metrics.increment_errors();
    metrics.increment_encryptions();
    metrics.increment_decryptions();

    let response_time = Duration::from_millis(150);
    metrics.record_response_time(response_time);

    // Get stats
    let stats = metrics.get_stats();
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.total_errors, 1);
    assert_eq!(stats.encryption_operations, 1);
    assert_eq!(stats.decryption_operations, 1);
    assert!(stats.avg_response_time_ms > 0);
}

/// Test privacy budget tracking
#[tokio::test]
async fn test_privacy_budget_tracking() {
    let tracker = PrivacyBudgetTracker::new(1.0, 1e-5); // Total epsilon=1.0, delta=1e-5
    let user_id = "test-user-123";

    // Should allow queries within budget
    assert!(tracker.check_budget(user_id, 0.1, 1e-6).await.unwrap());
    assert!(tracker.check_budget(user_id, 0.2, 2e-6).await.unwrap());

    // Check budget status
    let status = tracker.get_budget_status(user_id).await.unwrap();
    assert!(status.remaining_epsilon < 1.0);
    assert!(status.queries_count > 0);

    // Should reject queries that exceed budget
    assert!(!tracker.check_budget(user_id, 1.0, 1e-5).await.unwrap());

    // Test budget reset
    assert!(tracker.reset_budget(user_id).await.is_ok());
    let status_after_reset = tracker.get_budget_status(user_id).await.unwrap();
    assert_eq!(status_after_reset.remaining_epsilon, 1.0);
    assert_eq!(status_after_reset.queries_count, 0);
}

/// Test monitoring and health checks
#[tokio::test]
async fn test_monitoring_service() {
    let service = MonitoringService::new("test-1.0.0".to_string());

    // Test health check
    let health = service.health_check().await;
    assert!(!health.components.is_empty());
    assert_eq!(health.version, "test-1.0.0");
    assert!(health.uptime_seconds >= 0);

    // Test liveness and readiness
    assert!(service.liveness_check().await);
    assert!(service.readiness_check().await);

    // Test error recording
    service
        .record_error("TestError".to_string(), "Test error message".to_string())
        .await;

    // Wait a bit for async operations
    sleep(Duration::from_millis(10)).await;
}

/// Test performance profiling
#[tokio::test]
async fn test_performance_profiling() {
    let profiler = PerformanceProfiler::new();

    // Simulate some operations
    for i in 0..5 {
        let _timer = profiler.start_timer("test_operation");
        sleep(Duration::from_millis(10)).await;
        // Timer is dropped here, recording the measurement
    }

    // Wait for async measurement recording
    sleep(Duration::from_millis(50)).await;

    // Get statistics
    let stats = profiler.get_stats("test_operation").await;
    if let Some(stats) = stats {
        assert_eq!(stats.total_calls, 5);
        assert!(stats.avg_duration >= Duration::from_millis(5)); // Should be at least 5ms average
        assert!(stats.min_duration > Duration::ZERO);
        assert!(stats.max_duration >= stats.min_duration);
    }

    // Test all stats
    let all_stats = profiler.get_all_stats().await;
    assert!(!all_stats.is_empty());
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_error_handling() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");

    // Test with invalid client ID
    let invalid_client_id = Uuid::new_v4();
    let result = engine.encrypt_text(invalid_client_id, "test");
    assert!(result.is_err());

    // Test empty plaintext
    let (client_id, _) = engine.generate_keys().unwrap();
    let result = engine.encrypt_text(client_id, "");
    assert!(result.is_err());

    // Test very long plaintext
    let long_text = "a".repeat(20000);
    let result = engine.encrypt_text(client_id, &long_text);
    assert!(result.is_err());
}

/// Test concurrent operations
#[tokio::test]
async fn test_concurrent_operations() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");
    let (client_id, _) = engine.generate_keys().unwrap();

    // Test concurrent encryptions
    let mut handles = Vec::new();

    for i in 0..10 {
        let plaintext = format!("Test message {}", i);
        let ciphertext = engine.encrypt_text(client_id, &plaintext).unwrap();

        let handle = tokio::spawn(async move {
            // Simulate some processing
            sleep(Duration::from_millis(10)).await;
            ciphertext
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let results: Result<Vec<_>, _> = futures::future::join_all(handles)
        .await
        .into_iter()
        .collect();
    let ciphertexts = results.expect("All encryption operations should succeed");

    assert_eq!(ciphertexts.len(), 10);

    // Verify all ciphertexts are valid and different
    for ciphertext in ciphertexts {
        assert!(!ciphertext.data.is_empty());
        assert!(ciphertext.noise_budget.is_some());
    }
}

/// Test memory safety and cleanup
#[tokio::test]
async fn test_memory_safety() {
    // Test that we can create and drop multiple engines without issues
    for _ in 0..10 {
        let params = FheParams::default();
        let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");
        let (client_id, _) = engine.generate_keys().unwrap();

        let ciphertext = engine.encrypt_text(client_id, "test").unwrap();
        let _decrypted = engine.decrypt_text_safe(client_id, &ciphertext).unwrap();
    }

    // Test metrics collector memory usage
    let metrics = MetricsCollector::new();
    for _ in 0..1000 {
        metrics.increment_requests();
        metrics.record_response_time(Duration::from_millis(100));
    }

    let stats = metrics.get_stats();
    assert_eq!(stats.total_requests, 1000);
}

// Add futures dependency for concurrent testing
use futures;

#[tokio::test]
async fn test_full_workflow_integration() {
    // Create configuration
    let config = Config::default();
    assert!(config.validate().is_ok());

    // Initialize FHE engine
    let params = FheParams {
        poly_modulus_degree: config.encryption.poly_modulus_degree,
        coeff_modulus_bits: config.encryption.coeff_modulus_bits.clone(),
        scale_bits: config.encryption.scale_bits,
        security_level: config.encryption.security_level,
    };

    let mut fhe_engine = FheEngine::new(params).expect("Failed to create FHE engine");
    let (client_id, _server_id) = fhe_engine.generate_keys().expect("Failed to generate keys");

    // Initialize security components
    let master_secret = "integration-test-secret-key".to_string();
    let mut api_manager = ApiKeyManager::new(master_secret);
    let api_key = api_manager.generate_api_key(
        "integration-test-user".to_string(),
        vec![
            Permission::Encrypt,
            Permission::Decrypt,
            Permission::ProcessLLM,
        ],
    );

    // Initialize monitoring
    let monitoring = MonitoringService::new("integration-test-1.0.0".to_string());
    let metrics = MetricsCollector::new();
    let mut rate_limiter = RateLimiter::new(100);
    let privacy_tracker = PrivacyBudgetTracker::new(10.0, 1e-4);

    // Simulate full request workflow
    let client_ip = "192.168.1.100";
    let test_plaintext = "This is a secret message for FHE processing";

    // 1. Authenticate API key
    let permissions = api_manager
        .validate_api_key(&api_key)
        .expect("API key validation failed");
    assert!(permissions.contains(&Permission::Encrypt));

    // 2. Check rate limits
    assert!(rate_limiter.check_rate_limit(client_ip).await.unwrap());
    metrics.increment_requests();

    // 3. Validate input
    let sanitized_input =
        InputValidator::sanitize_text(test_plaintext).expect("Input sanitization failed");

    // 4. Check privacy budget
    assert!(privacy_tracker
        .check_budget("integration-test-user", 0.1, 1e-5)
        .await
        .unwrap());

    // 5. Encrypt data
    let start_time = std::time::Instant::now();
    let ciphertext = fhe_engine
        .encrypt_text(client_id, &sanitized_input)
        .expect("Encryption failed");
    let encrypt_duration = start_time.elapsed();

    metrics.increment_encryptions();
    metrics.record_response_time(encrypt_duration);

    // 6. Validate ciphertext
    assert!(fhe_engine.validate_ciphertext(&ciphertext).unwrap());

    // 7. Process (simulate LLM processing)
    let processed_ciphertext = fhe_engine
        .process_encrypted_prompt(&ciphertext)
        .expect("Encrypted processing failed");

    // 8. Decrypt result
    let decrypted_result = fhe_engine
        .decrypt_text_safe(client_id, &processed_ciphertext)
        .expect("Decryption failed");

    metrics.increment_decryptions();

    // 9. Verify health status
    let health = monitoring.health_check().await;
    assert_eq!(health.status, "healthy");

    // 10. Check final metrics
    let final_stats = metrics.get_stats();
    assert_eq!(final_stats.total_requests, 1);
    assert_eq!(final_stats.encryption_operations, 1);
    assert_eq!(final_stats.decryption_operations, 1);
    assert!(final_stats.avg_response_time_ms >= 0);

    // Verify the workflow completed successfully
    assert!(!decrypted_result.is_empty());
    println!("Integration test completed successfully!");
    println!("Original: {}", test_plaintext);
    println!("Processed result: {}", decrypted_result);
}
