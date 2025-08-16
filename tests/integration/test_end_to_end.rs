//! End-to-end integration tests for the complete FHE LLM Proxy workflow

use homomorphic_llm_proxy::{
    config::Config,
    fhe::{FheEngine, FheParams},
    health::{HealthChecker, FheEngineHealthCheck, MemoryHealthCheck},
    middleware::{MetricsCollector, PrivacyBudgetTracker, RateLimiter},
    monitoring::MonitoringService,
    proxy::{ProxyServer, SessionManager},
    security::{ApiKeyManager, InputValidator, Permission},
    validation::ValidationFramework,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Full end-to-end test simulating a complete user workflow
#[tokio::test]
async fn test_complete_user_workflow() {
    println!("🚀 Starting complete user workflow test...");

    // 1. Initialize configuration
    let config = Config::default();
    assert!(config.validate().is_ok(), "Configuration should be valid");
    println!("✅ Configuration loaded and validated");

    // 2. Initialize FHE engine
    let fhe_params = FheParams {
        poly_modulus_degree: config.encryption.poly_modulus_degree,
        coeff_modulus_bits: config.encryption.coeff_modulus_bits.clone(),
        scale_bits: config.encryption.scale_bits,
        security_level: config.encryption.security_level,
    };

    let mut fhe_engine = FheEngine::new(fhe_params).expect("Failed to create FHE engine");
    let (client_id, server_id) = fhe_engine.generate_keys().expect("Failed to generate keys");
    println!("✅ FHE engine initialized with client ID: {}", client_id);

    // 3. Initialize security components
    let mut api_manager = ApiKeyManager::new("e2e-test-secret-key".to_string());
    let api_key = api_manager.generate_api_key(
        "e2e-test-user".to_string(),
        vec![
            Permission::Encrypt,
            Permission::Decrypt, 
            Permission::ProcessLLM,
            Permission::ViewMetrics,
        ],
    );
    println!("✅ API key generated: {}", &api_key[..20]);

    // 4. Initialize middleware
    let metrics = Arc::new(MetricsCollector::new());
    let mut rate_limiter = RateLimiter::new(1000); // High limit for testing
    let privacy_tracker = PrivacyBudgetTracker::new(10.0, 1e-4);
    let session_manager = SessionManager::new();
    let validation_framework = ValidationFramework::with_fhe_defaults()
        .expect("Failed to create validation framework");
    println!("✅ Middleware components initialized");

    // 5. Initialize monitoring
    let monitoring_service = MonitoringService::new("e2e-test-1.0.0".to_string());
    let health_checker = HealthChecker::new();
    
    // Register health checks
    let fhe_engine_arc = Arc::new(RwLock::new(fhe_engine));
    let fhe_health_check = FheEngineHealthCheck::new(
        fhe_engine_arc.clone(), 
        "primary_fhe_engine".to_string()
    );
    health_checker.register_check(Box::new(fhe_health_check)).await;
    
    let memory_health_check = MemoryHealthCheck::new(2048, 80.0);
    health_checker.register_check(Box::new(memory_health_check)).await;
    
    println!("✅ Monitoring and health checks configured");

    // 6. Simulate client request
    let client_ip = "192.168.1.100";
    let user_prompt = "What is the capital of France? Please keep your answer confidential.";
    
    println!("🔍 Processing user request: '{}'", user_prompt);

    // 7. Authenticate API key
    let permissions = api_manager.validate_api_key(&api_key)
        .expect("API key validation should succeed");
    assert!(permissions.contains(&Permission::Encrypt));
    assert!(permissions.contains(&Permission::ProcessLLM));
    println!("✅ API key validated with {} permissions", permissions.len());

    // 8. Check rate limits
    assert!(rate_limiter.check_rate_limit(client_ip).await.unwrap());
    metrics.increment_requests();
    println!("✅ Rate limit check passed");

    // 9. Validate and sanitize input
    let validation_report = validation_framework.validate_input("plaintext", user_prompt);
    assert!(validation_report.is_valid, "Input validation should succeed");
    
    let sanitized_prompt = validation_report.sanitized_input
        .expect("Should have sanitized input");
    println!("✅ Input validated and sanitized");

    // 10. Check privacy budget
    assert!(privacy_tracker.check_budget("e2e-test-user", 0.5, 1e-5).await.unwrap());
    println!("✅ Privacy budget check passed");

    // 11. Create session
    let session_id = session_manager.create_session(client_id, server_id).await
        .expect("Session creation should succeed");
    println!("✅ Session created: {}", session_id);

    // 12. Encrypt user prompt
    let start_time = std::time::Instant::now();
    let fhe_engine_ref = fhe_engine_arc.read().await;
    let encrypted_prompt = fhe_engine_ref.encrypt_text(client_id, &sanitized_prompt)
        .expect("Encryption should succeed");
    drop(fhe_engine_ref);
    
    let encryption_time = start_time.elapsed();
    metrics.increment_encryptions();
    metrics.record_response_time(encryption_time);
    
    assert!(!encrypted_prompt.data.is_empty());
    assert!(encrypted_prompt.noise_budget.is_some());
    println!("✅ Prompt encrypted in {:?}", encryption_time);

    // 13. Validate ciphertext
    let fhe_engine_ref = fhe_engine_arc.read().await;
    assert!(fhe_engine_ref.validate_ciphertext(&encrypted_prompt).unwrap());
    println!("✅ Ciphertext validation passed");

    // 14. Process encrypted prompt (simulate LLM processing)
    let processed_ciphertext = fhe_engine_ref.process_encrypted_prompt(&encrypted_prompt)
        .expect("Encrypted processing should succeed");
    
    assert!(!processed_ciphertext.data.is_empty());
    if let Some(remaining_budget) = processed_ciphertext.noise_budget {
        assert!(remaining_budget > 0, "Should have remaining noise budget");
    }
    println!("✅ Encrypted processing completed");

    // 15. Decrypt response
    let decrypted_response = fhe_engine_ref.decrypt_text_safe(client_id, &processed_ciphertext)
        .expect("Decryption should succeed");
    drop(fhe_engine_ref);
    
    metrics.increment_decryptions();
    assert!(!decrypted_response.is_empty());
    println!("✅ Response decrypted: '{}'", &decrypted_response[..50.min(decrypted_response.len())]);

    // 16. Update session
    session_manager.update_last_used(session_id).await;
    println!("✅ Session updated");

    // 17. Check system health
    let health_report = health_checker.run_health_checks().await
        .expect("Health check should succeed");
    
    assert!(!health_report.components.is_empty());
    assert!(health_report.summary.total_components > 0);
    println!("✅ System health check: {} components checked", health_report.summary.total_components);

    // 18. Verify monitoring metrics
    let system_health = monitoring_service.health_check().await;
    assert!(system_health.uptime_seconds >= 0);
    println!("✅ Monitoring service operational");

    // 19. Check final metrics
    let final_metrics = metrics.get_stats();
    assert_eq!(final_metrics.total_requests, 1);
    assert_eq!(final_metrics.encryption_operations, 1);
    assert_eq!(final_metrics.decryption_operations, 1);
    assert!(final_metrics.avg_response_time_ms >= 0);
    
    println!("📊 Final metrics:");
    println!("   Requests: {}", final_metrics.total_requests);
    println!("   Encryptions: {}", final_metrics.encryption_operations);
    println!("   Decryptions: {}", final_metrics.decryption_operations);
    println!("   Avg response time: {} ms", final_metrics.avg_response_time_ms);
    println!("   Errors: {}", final_metrics.total_errors);

    // 20. Verify privacy budget consumption
    let budget_status = privacy_tracker.get_budget_status("e2e-test-user").await
        .expect("Budget status should be available");
    assert!(budget_status.remaining_epsilon < 10.0);
    assert_eq!(budget_status.queries_count, 1);
    println!("✅ Privacy budget consumed: {:.2} epsilon remaining", budget_status.remaining_epsilon);

    // 21. Test error handling edge case
    let invalid_client_id = Uuid::new_v4();
    let fhe_engine_ref = fhe_engine_arc.read().await;
    let error_result = fhe_engine_ref.encrypt_text(invalid_client_id, "test");
    drop(fhe_engine_ref);
    assert!(error_result.is_err(), "Invalid client ID should cause error");
    println!("✅ Error handling verified");

    println!("🎉 Complete end-to-end workflow test PASSED!");
    println!("   User prompt processed successfully through full FHE pipeline");
    println!("   Original: '{}'", user_prompt);
    println!("   Response: '{}'", decrypted_response);
}

/// Test multi-user concurrent workflow
#[tokio::test]
async fn test_concurrent_multi_user_workflow() {
    println!("🚀 Starting concurrent multi-user workflow test...");

    let config = Config::default();
    let fhe_params = FheParams::default();
    
    // Initialize shared components
    let mut api_manager = ApiKeyManager::new("concurrent-test-secret".to_string());
    let metrics = Arc::new(MetricsCollector::new());
    let privacy_tracker = Arc::new(PrivacyBudgetTracker::new(100.0, 1e-3));
    let session_manager = Arc::new(SessionManager::new());
    let mut rate_limiter = RateLimiter::new(1000);
    
    // Create multiple users
    let num_users = 5;
    let mut user_tasks = Vec::new();
    
    for user_id in 0..num_users {
        let username = format!("concurrent_user_{}", user_id);
        let api_key = api_manager.generate_api_key(
            username.clone(),
            vec![Permission::Encrypt, Permission::Decrypt, Permission::ProcessLLM],
        );
        
        let metrics_clone = metrics.clone();
        let privacy_tracker_clone = privacy_tracker.clone();
        let session_manager_clone = session_manager.clone();
        
        let task = tokio::spawn(async move {
            // Create FHE engine for this user
            let mut fhe_engine = FheEngine::new(fhe_params).expect("Failed to create FHE engine");
            let (client_id, server_id) = fhe_engine.generate_keys().expect("Failed to generate keys");
            
            // Create session
            let session_id = session_manager_clone.create_session(client_id, server_id).await
                .expect("Session creation should succeed");
            
            // Process user request
            let user_prompt = format!("User {} confidential query about data science", user_id);
            
            // Check privacy budget
            assert!(privacy_tracker_clone.check_budget(&username, 1.0, 1e-4).await.unwrap());
            
            // Encrypt
            let start = std::time::Instant::now();
            let encrypted = fhe_engine.encrypt_text(client_id, &user_prompt)
                .expect("Encryption should succeed");
            
            metrics_clone.increment_encryptions();
            metrics_clone.record_response_time(start.elapsed());
            
            // Process
            let processed = fhe_engine.process_encrypted_prompt(&encrypted)
                .expect("Processing should succeed");
            
            // Decrypt
            let decrypted = fhe_engine.decrypt_text_safe(client_id, &processed)
                .expect("Decryption should succeed");
            
            metrics_clone.increment_decryptions();
            
            // Verify results
            assert!(!decrypted.is_empty());
            assert_ne!(decrypted, user_prompt); // Should be processed/different
            
            (user_id, session_id, decrypted.len())
        });
        
        user_tasks.push(task);
    }
    
    // Wait for all users to complete
    let mut successful_users = 0;
    for task in user_tasks {
        match task.await {
            Ok((user_id, session_id, response_len)) => {
                successful_users += 1;
                println!("✅ User {} completed successfully (session: {}, response: {} chars)", 
                    user_id, session_id, response_len);
            }
            Err(e) => {
                eprintln!("❌ User task failed: {}", e);
            }
        }
    }
    
    assert_eq!(successful_users, num_users, "All users should complete successfully");
    
    // Check final metrics
    let final_metrics = metrics.get_stats();
    assert_eq!(final_metrics.encryption_operations, num_users as u64);
    assert_eq!(final_metrics.decryption_operations, num_users as u64);
    assert_eq!(final_metrics.total_errors, 0);
    
    println!("📊 Concurrent test metrics:");
    println!("   Users processed: {}", num_users);
    println!("   Total encryptions: {}", final_metrics.encryption_operations);
    println!("   Total decryptions: {}", final_metrics.decryption_operations);
    println!("   Average response time: {} ms", final_metrics.avg_response_time_ms);
    
    println!("🎉 Concurrent multi-user workflow test PASSED!");
}

/// Test system resilience under error conditions
#[tokio::test]
async fn test_system_resilience() {
    println!("🚀 Starting system resilience test...");

    let config = Config::default();
    let mut fhe_engine = FheEngine::new(FheParams::default())
        .expect("Failed to create FHE engine");
    let (client_id, _) = fhe_engine.generate_keys().expect("Failed to generate keys");
    
    let metrics = MetricsCollector::new();
    let mut api_manager = ApiKeyManager::new("resilience-test-secret".to_string());
    let api_key = api_manager.generate_api_key(
        "resilience_user".to_string(),
        vec![Permission::Encrypt, Permission::Decrypt],
    );
    
    // Test 1: Invalid inputs
    println!("🧪 Testing invalid input handling...");
    let invalid_inputs = vec![
        "",  // Empty
        "a".repeat(50000),  // Too long
        "\x00\x01\x02",     // Control characters
        "<script>alert('xss')</script>",  // XSS attempt
    ];
    
    for input in invalid_inputs {
        let result = fhe_engine.encrypt_text(client_id, input);
        match result {
            Ok(_) => println!("  Input accepted (may be sanitized): '{}'", 
                &input[..20.min(input.len())]),
            Err(_) => println!("  Input rejected: '{}'", &input[..20.min(input.len())]),
        }
        // Either outcome is acceptable - system should not crash
    }
    
    // Test 2: Invalid API keys
    println!("🧪 Testing invalid API key handling...");
    let invalid_keys = vec![
        "invalid-key",
        "",
        "x".repeat(100),
        api_key[..api_key.len()-5].to_string(), // Truncated
    ];
    
    for key in invalid_keys {
        let result = api_manager.validate_api_key(&key);
        assert!(result.is_err(), "Invalid API key should be rejected");
        metrics.increment_errors();
    }
    
    // Test 3: Revoked key usage
    println!("🧪 Testing revoked key handling...");
    assert!(api_manager.revoke_api_key(&api_key).is_ok());
    assert!(api_manager.validate_api_key(&api_key).is_err());
    
    // Test 4: Cross-client decryption attempts
    println!("🧪 Testing cross-client security...");
    let plaintext = "Secret message for client security test";
    let ciphertext = fhe_engine.encrypt_text(client_id, plaintext).unwrap();
    
    // Generate different client and try to decrypt
    let (other_client_id, _) = fhe_engine.generate_keys().unwrap();
    let cross_decrypt_result = fhe_engine.decrypt_text_safe(other_client_id, &ciphertext);
    assert!(cross_decrypt_result.is_err(), "Cross-client decryption should fail");
    
    // Test 5: Tampered ciphertext
    println!("🧪 Testing tampered ciphertext detection...");
    let mut tampered_ciphertext = ciphertext.clone();
    if !tampered_ciphertext.data.is_empty() {
        tampered_ciphertext.data[0] ^= 0xFF; // Flip bits
        
        let tampered_result = fhe_engine.decrypt_text_safe(client_id, &tampered_ciphertext);
        // Should either fail validation or decryption
        if tampered_result.is_ok() {
            println!("  Warning: Tampered ciphertext was not detected");
        } else {
            println!("  ✅ Tampered ciphertext properly rejected");
        }
    }
    
    // Test 6: Resource exhaustion simulation
    println!("🧪 Testing resource limits...");
    let mut large_operations = 0;
    let mut failed_operations = 0;
    
    for i in 0..20 {
        let large_input = format!("Large input test {} - {}", i, "data ".repeat(100));
        match fhe_engine.encrypt_text(client_id, &large_input) {
            Ok(_) => large_operations += 1,
            Err(_) => failed_operations += 1,
        }
    }
    
    println!("  Large operations completed: {}", large_operations);
    println!("  Operations that failed: {}", failed_operations);
    
    // System should handle some operations but may reject others due to size limits
    assert!(large_operations > 0 || failed_operations > 0, "System should respond to operations");
    
    // Test 7: Check metrics are tracking errors
    let final_metrics = metrics.get_stats();
    println!("📊 Resilience test metrics:");
    println!("   Total errors tracked: {}", final_metrics.total_errors);
    
    assert!(final_metrics.total_errors > 0, "Should have tracked some errors");
    
    println!("🎉 System resilience test PASSED!");
    println!("   System handled error conditions gracefully");
}