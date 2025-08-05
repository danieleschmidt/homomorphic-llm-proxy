//! Integration tests for scaling functionality

use homomorphic_llm_proxy::scaling::*;
use homomorphic_llm_proxy::fhe::{FheEngine, FheParams};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
async fn test_fhe_connection_pool_load_balancing() {
    let params = FheParams::default();
    let pool = FheConnectionPool::new(3, 10, params).unwrap();
    
    // Generate keys on different engines
    let mut tasks = Vec::new();
    for i in 0..9 {
        let pool = pool.clone();
        let task = tokio::spawn(async move {
            let client_id = Uuid::new_v4();
            let plaintext = format!("Test message {}", i);
            
            // Encrypt
            let ciphertext = pool.encrypt_balanced(client_id, &plaintext).await.unwrap();
            
            // Decrypt
            let decrypted = pool.decrypt_balanced(client_id, &ciphertext).await.unwrap();
            assert_eq!(decrypted, plaintext);
            
            i
        });
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        let result = task.await.unwrap();
        println!("Completed task {}", result);
    }
    
    // Check that all engines were utilized
    let stats = pool.get_stats().await;
    assert_eq!(stats.total_operations, 18); // 9 encrypt + 9 decrypt
    assert!(stats.engine_utilization.values().all(|&count| count > 0));
}

#[tokio::test]
async fn test_ciphertext_cache_eviction() {
    let cache = CiphertextCache::new(2, Duration::from_secs(10));
    
    let ciphertext1 = create_test_ciphertext();
    let ciphertext2 = create_test_ciphertext();
    let ciphertext3 = create_test_ciphertext();
    
    // Fill cache to capacity
    cache.put(ciphertext1.id, ciphertext1.clone()).await;
    cache.put(ciphertext2.id, ciphertext2.clone()).await;
    
    // Access first ciphertext
    let _ = cache.get(&ciphertext1.id).await;
    
    // Add third ciphertext (should evict ciphertext2 as it's LRU)
    cache.put(ciphertext3.id, ciphertext3.clone()).await;
    
    // ciphertext1 should still be there, ciphertext2 should be evicted
    assert!(cache.get(&ciphertext1.id).await.is_some());
    assert!(cache.get(&ciphertext2.id).await.is_none());
    assert!(cache.get(&ciphertext3.id).await.is_some());
    
    let stats = cache.get_stats().await;
    assert_eq!(stats.evictions, 1);
}

#[tokio::test]
async fn test_cache_ttl_expiration() {
    let cache = CiphertextCache::new(10, Duration::from_millis(100));
    let ciphertext = create_test_ciphertext();
    
    // Put ciphertext in cache
    cache.put(ciphertext.id, ciphertext.clone()).await;
    
    // Should be retrievable immediately
    assert!(cache.get(&ciphertext.id).await.is_some());
    
    // Wait for TTL to expire
    sleep(Duration::from_millis(150)).await;
    
    // Should be expired now
    assert!(cache.get(&ciphertext.id).await.is_none());
}

#[tokio::test]
async fn test_circuit_breaker_state_transitions() {
    let breaker = CircuitBreaker::new(2, 2, Duration::from_millis(100));
    
    // Initially closed
    assert_eq!(breaker.get_state().await, CircuitState::Closed);
    
    // Successful operations should keep it closed
    for _ in 0..3 {
        let result = breaker.call(async { Ok::<i32, &str>(42) }).await;
        assert!(result.is_ok());
    }
    assert_eq!(breaker.get_state().await, CircuitState::Closed);
    
    // Failures should open it
    for _ in 0..2 {
        let _ = breaker.call(async { Err::<i32, &str>("failure") }).await;
    }
    assert_eq!(breaker.get_state().await, CircuitState::Open);
    
    // Requests should be rejected when open
    let result = breaker.call(async { Ok::<i32, &str>(42) }).await;
    assert!(result.is_err());
    
    // Wait for timeout
    sleep(Duration::from_millis(120)).await;
    
    // Should transition to half-open and allow requests
    let result = breaker.call(async { Ok::<i32, &str>(42) }).await;
    assert!(result.is_ok());
    assert_eq!(breaker.get_state().await, CircuitState::HalfOpen);
    
    // Successful operations should close it
    let result = breaker.call(async { Ok::<i32, &str>(42) }).await;
    assert!(result.is_ok());
    assert_eq!(breaker.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_auto_scaler_scale_up() {
    let scaler = AutoScaler::new(70.0, 10, 1, 5, Duration::from_millis(50));
    
    let high_load_metrics = ScalingMetrics {
        cpu_utilization: 90.0,
        memory_utilization: 80.0,
        queue_length: 15,
        active_connections: 25,
        response_time_p95: Duration::from_millis(300),
    };
    
    let decision = scaler.evaluate_scaling(&high_load_metrics).await;
    
    match decision {
        ScalingDecision::ScaleUp { from, to, .. } => {
            assert_eq!(from, 1);
            assert_eq!(to, 2);
            
            // Apply scaling
            scaler.apply_scaling(decision).await.unwrap();
            assert_eq!(scaler.get_current_replicas(), 2);
        }
        _ => panic!("Expected scale up decision"),
    }
}

#[tokio::test]
async fn test_auto_scaler_scale_down() {
    let scaler = AutoScaler::new(70.0, 10, 2, 5, Duration::from_millis(50));
    
    let low_load_metrics = ScalingMetrics {
        cpu_utilization: 30.0,
        memory_utilization: 25.0,
        queue_length: 2,
        active_connections: 5,
        response_time_p95: Duration::from_millis(50),
    };
    
    let decision = scaler.evaluate_scaling(&low_load_metrics).await;
    
    match decision {
        ScalingDecision::ScaleDown { from, to, .. } => {
            assert_eq!(from, 2);
            assert_eq!(to, 1);
            
            // Apply scaling
            scaler.apply_scaling(decision).await.unwrap();
            assert_eq!(scaler.get_current_replicas(), 1);
        }
        _ => panic!("Expected scale down decision"),
    }
}

#[tokio::test]
async fn test_auto_scaler_cooldown() {
    let scaler = AutoScaler::new(70.0, 10, 1, 5, Duration::from_millis(200));
    
    let high_load_metrics = ScalingMetrics {
        cpu_utilization: 90.0,
        memory_utilization: 80.0,
        queue_length: 15,
        active_connections: 25,
        response_time_p95: Duration::from_millis(300),
    };
    
    // First scaling decision should work
    let decision1 = scaler.evaluate_scaling(&high_load_metrics).await;
    assert!(matches!(decision1, ScalingDecision::ScaleUp { .. }));
    
    scaler.apply_scaling(decision1).await.unwrap();
    
    // Immediate second decision should be blocked by cooldown
    let decision2 = scaler.evaluate_scaling(&high_load_metrics).await;
    assert!(matches!(decision2, ScalingDecision::NoAction));
    
    // After cooldown period, should allow scaling again
    sleep(Duration::from_millis(250)).await;
    let decision3 = scaler.evaluate_scaling(&high_load_metrics).await;
    assert!(matches!(decision3, ScalingDecision::ScaleUp { .. }));
}

#[tokio::test]
async fn test_pool_health_check() {
    let params = FheParams::default();
    let pool = FheConnectionPool::new(3, 5, params).unwrap();
    
    let health_status = pool.health_check().await;
    
    // All engines should be healthy
    assert_eq!(health_status.len(), 3);
    assert!(health_status.iter().all(|&healthy| healthy));
}

fn create_test_ciphertext() -> homomorphic_llm_proxy::fhe::Ciphertext {
    use homomorphic_llm_proxy::fhe::{Ciphertext, FheParams};
    
    Ciphertext {
        id: Uuid::new_v4(),
        data: vec![1, 2, 3, 4, 5],
        params: FheParams::default(),
        noise_budget: Some(45),
    }
}

// Stress test for concurrent operations
#[tokio::test]
async fn test_concurrent_pool_operations() {
    let params = FheParams::default();
    let pool = FheConnectionPool::new(4, 20, params).unwrap();
    
    let num_concurrent_ops = 50;
    let mut tasks = Vec::new();
    
    for i in 0..num_concurrent_ops {
        let pool = pool.clone();
        let task = tokio::spawn(async move {
            let client_id = Uuid::new_v4();
            let plaintext = format!("Concurrent test message {}", i);
            
            // Simulate some processing time
            sleep(Duration::from_millis(10)).await;
            
            let ciphertext = pool.encrypt_balanced(client_id, &plaintext).await.unwrap();
            let decrypted = pool.decrypt_balanced(client_id, &ciphertext).await.unwrap();
            
            assert_eq!(decrypted, plaintext);
            i
        });
        tasks.push(task);
    }
    
    let start = std::time::Instant::now();
    
    // Wait for all operations to complete
    for task in tasks {
        task.await.unwrap();
    }
    
    let elapsed = start.elapsed();
    println!("Completed {} concurrent operations in {:?}", num_concurrent_ops, elapsed);
    
    // Check final statistics
    let stats = pool.get_stats().await;
    assert_eq!(stats.total_operations, num_concurrent_ops * 2); // encrypt + decrypt
    
    // All engines should have been utilized
    assert!(stats.engine_utilization.values().all(|&count| count > 0));
}