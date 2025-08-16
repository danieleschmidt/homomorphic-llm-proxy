//! Load testing for FHE LLM Proxy

use homomorphic_llm_proxy::fhe::{FheEngine, FheParams};
use homomorphic_llm_proxy::middleware::{MetricsCollector, RateLimiter};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use uuid::Uuid;

#[tokio::test]
async fn test_concurrent_encryption_load() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");
    let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");
    
    let metrics = Arc::new(MetricsCollector::new());
    let semaphore = Arc::new(Semaphore::new(10)); // Limit concurrent operations
    
    let num_operations = 50;
    let mut tasks = JoinSet::new();
    
    let start_time = Instant::now();
    
    for i in 0..num_operations {
        let engine_ref = &engine;
        let metrics_ref = metrics.clone();
        let semaphore_ref = semaphore.clone();
        
        tasks.spawn(async move {
            let _permit = semaphore_ref.acquire().await.unwrap();
            let start = Instant::now();
            
            let plaintext = format!("Load test message {}", i);
            let result = engine_ref.encrypt_text(client_id, &plaintext);
            
            let duration = start.elapsed();
            metrics_ref.record_response_time(duration);
            
            if result.is_ok() {
                metrics_ref.increment_encryptions();
            } else {
                metrics_ref.increment_errors();
            }
            
            result
        });
    }
    
    // Wait for all tasks to complete
    let mut successful_operations = 0;
    let mut failed_operations = 0;
    
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(_)) => successful_operations += 1,
            Ok(Err(_)) => failed_operations += 1,
            Err(_) => failed_operations += 1,
        }
    }
    
    let total_time = start_time.elapsed();
    let ops_per_second = num_operations as f64 / total_time.as_secs_f64();
    
    println!("Load Test Results:");
    println!("  Total operations: {}", num_operations);
    println!("  Successful: {}", successful_operations);
    println!("  Failed: {}", failed_operations);
    println!("  Total time: {:?}", total_time);
    println!("  Ops/second: {:.2}", ops_per_second);
    
    let stats = metrics.get_stats();
    println!("  Average response time: {} ms", stats.avg_response_time_ms);
    
    // Assertions
    assert!(successful_operations > 0, "Should have successful operations");
    assert!(ops_per_second > 0.1, "Should achieve reasonable throughput");
    assert!(stats.avg_response_time_ms < 10000, "Average response time should be reasonable");
}

#[tokio::test]
async fn test_rate_limiter_under_load() {
    let mut rate_limiter = RateLimiter::new(10); // 10 requests per minute
    let client_ip = "192.168.1.100";
    
    let mut successful_requests = 0;
    let mut blocked_requests = 0;
    
    // Attempt 20 requests rapidly
    for _ in 0..20 {
        if rate_limiter.check_rate_limit(client_ip).await.unwrap() {
            successful_requests += 1;
        } else {
            blocked_requests += 1;
        }
    }
    
    println!("Rate Limiter Load Test:");
    println!("  Successful requests: {}", successful_requests);
    println!("  Blocked requests: {}", blocked_requests);
    
    // Should allow first 10 requests, then block the rest
    assert_eq!(successful_requests, 10);
    assert_eq!(blocked_requests, 10);
    
    // Test stats
    let (requests, window_remaining) = rate_limiter.get_client_stats(client_ip).await.unwrap();
    assert_eq!(requests, 20); // Including blocked requests
    assert!(window_remaining <= Duration::from_secs(60));
}

#[tokio::test]
async fn test_memory_usage_under_load() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");
    
    // Generate multiple client keys
    let mut client_ids = Vec::new();
    for _ in 0..10 {
        let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");
        client_ids.push(client_id);
    }
    
    // Create many ciphertexts
    let mut ciphertexts = Vec::new();
    for (i, &client_id) in client_ids.iter().enumerate() {
        for j in 0..10 {
            let plaintext = format!("Memory test message {} - {}", i, j);
            if let Ok(ciphertext) = engine.encrypt_text(client_id, &plaintext) {
                ciphertexts.push(ciphertext);
            }
        }
    }
    
    println!("Memory Usage Test:");
    println!("  Created {} ciphertexts", ciphertexts.len());
    println!("  Total data size: {} bytes", 
        ciphertexts.iter().map(|c| c.data.len()).sum::<usize>());
    
    // Verify all ciphertexts are valid
    let mut valid_count = 0;
    for ciphertext in &ciphertexts {
        if engine.validate_ciphertext(ciphertext).unwrap_or(false) {
            valid_count += 1;
        }
    }
    
    assert_eq!(valid_count, ciphertexts.len(), "All ciphertexts should be valid");
    
    // Test decryption of random samples
    let mut successful_decryptions = 0;
    for (i, ciphertext) in ciphertexts.iter().enumerate().take(10) {
        let client_id = client_ids[i / 10];
        if engine.decrypt_text_safe(client_id, ciphertext).is_ok() {
            successful_decryptions += 1;
        }
    }
    
    assert!(successful_decryptions > 0, "Should successfully decrypt samples");
}

#[tokio::test]
async fn test_stress_key_operations() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");
    
    let start_time = Instant::now();
    
    // Generate many key pairs rapidly
    let key_count = 50;
    let mut key_pairs = Vec::new();
    
    for _ in 0..key_count {
        match engine.generate_keys() {
            Ok((client_id, server_id)) => {
                key_pairs.push((client_id, server_id));
            }
            Err(e) => {
                eprintln!("Key generation failed: {}", e);
            }
        }
    }
    
    let key_gen_time = start_time.elapsed();
    
    println!("Key Generation Stress Test:");
    println!("  Generated {} key pairs", key_pairs.len());
    println!("  Time taken: {:?}", key_gen_time);
    println!("  Keys/second: {:.2}", key_pairs.len() as f64 / key_gen_time.as_secs_f64());
    
    // Test key rotation for all generated keys
    let mut rotated_keys = 0;
    for &(client_id, _) in &key_pairs {
        if engine.rotate_keys(client_id).is_ok() {
            rotated_keys += 1;
        }
    }
    
    println!("  Successfully rotated {} keys", rotated_keys);
    
    // Get final statistics
    let stats = engine.get_encryption_stats();
    println!("  Final stats: {} client keys, {} server keys", 
        stats.total_client_keys, stats.total_server_keys);
    
    assert_eq!(key_pairs.len(), key_count);
    assert!(rotated_keys > 0, "Should successfully rotate some keys");
    assert!(stats.total_client_keys >= key_pairs.len());
}

#[tokio::test]
async fn test_sustained_encryption_operations() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create FHE engine");
    let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");
    
    let metrics = MetricsCollector::new();
    let test_duration = Duration::from_secs(5); // 5 second sustained test
    let start_time = Instant::now();
    
    let mut operation_count = 0;
    let mut total_latency = Duration::ZERO;
    let mut min_latency = Duration::from_secs(1);
    let mut max_latency = Duration::ZERO;
    
    while start_time.elapsed() < test_duration {
        let op_start = Instant::now();
        
        let plaintext = format!("Sustained test message {}", operation_count);
        let result = engine.encrypt_text(client_id, &plaintext);
        
        let op_latency = op_start.elapsed();
        total_latency += op_latency;
        min_latency = min_latency.min(op_latency);
        max_latency = max_latency.max(op_latency);
        
        if result.is_ok() {
            metrics.increment_encryptions();
        } else {
            metrics.increment_errors();
        }
        
        operation_count += 1;
        
        // Small delay to prevent overwhelming the system
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    let total_time = start_time.elapsed();
    let avg_latency = total_latency / operation_count;
    let ops_per_second = operation_count as f64 / total_time.as_secs_f64();
    
    println!("Sustained Load Test Results:");
    println!("  Duration: {:?}", total_time);
    println!("  Total operations: {}", operation_count);
    println!("  Ops/second: {:.2}", ops_per_second);
    println!("  Average latency: {:?}", avg_latency);
    println!("  Min latency: {:?}", min_latency);
    println!("  Max latency: {:?}", max_latency);
    
    let stats = metrics.get_stats();
    println!("  Successful encryptions: {}", stats.encryption_operations);
    println!("  Errors: {}", stats.total_errors);
    
    // Performance assertions
    assert!(operation_count > 0, "Should complete some operations");
    assert!(ops_per_second > 0.1, "Should maintain reasonable throughput");
    assert!(avg_latency < Duration::from_secs(1), "Average latency should be reasonable");
    assert!(stats.total_errors == 0, "Should have no errors under sustained load");
}