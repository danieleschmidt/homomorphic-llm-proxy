//! Performance benchmarks for FHE operations

use homomorphic_llm_proxy::{
    fhe::{FheEngine, FheParams},
    scaling::{FheConnectionPool, CiphertextCache},
    middleware::MetricsCollector,
};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use uuid::Uuid;

#[tokio::test]
async fn bench_encryption_performance() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).unwrap();
    let (client_id, _) = engine.generate_keys().unwrap();
    
    let test_cases = vec![
        ("short", "Hello"),
        ("medium", &"A".repeat(100)),
        ("long", &"B".repeat(1000)),
        ("very_long", &"C".repeat(5000)),
    ];
    
    println!("\n=== Encryption Performance Benchmarks ===");
    
    for (name, plaintext) in test_cases {
        let iterations = if plaintext.len() > 1000 { 10 } else { 50 };
        let mut total_time = Duration::ZERO;
        let mut successful_ops = 0;
        
        for _ in 0..iterations {
            let start = Instant::now();
            match engine.encrypt_text(client_id, plaintext) {
                Ok(_) => {
                    total_time += start.elapsed();
                    successful_ops += 1;
                }
                Err(e) => {
                    println!("Encryption failed for {}: {}", name, e);
                }
            }
        }
        
        if successful_ops > 0 {
            let avg_time = total_time / successful_ops;
            let throughput = successful_ops as f64 / total_time.as_secs_f64();
            
            println!("{:12} | {:>8} chars | {:>10.2}ms avg | {:>8.1} ops/sec | {}/{} success", 
                    name, plaintext.len(), avg_time.as_millis(), throughput, successful_ops, iterations);
        }
    }
}

#[tokio::test]
async fn bench_decryption_performance() {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).unwrap();
    let (client_id, _) = engine.generate_keys().unwrap();
    
    // Pre-encrypt test data
    let test_data = vec![
        ("short", "Hello World"),
        ("medium", &"Test data ".repeat(20)),
        ("long", &"Performance test ".repeat(100)),
    ];
    
    let mut ciphertexts = Vec::new();
    for (name, plaintext) in &test_data {
        match engine.encrypt_text(client_id, plaintext) {
            Ok(ciphertext) => ciphertexts.push((name, ciphertext)),
            Err(e) => println!("Failed to pre-encrypt {}: {}", name, e),
        }
    }
    
    println!("\n=== Decryption Performance Benchmarks ===");
    
    for (name, ciphertext) in ciphertexts {
        let iterations = 50;
        let mut total_time = Duration::ZERO;
        let mut successful_ops = 0;
        
        for _ in 0..iterations {
            let start = Instant::now();
            match engine.decrypt_text(client_id, &ciphertext) {
                Ok(_) => {
                    total_time += start.elapsed();
                    successful_ops += 1;
                }
                Err(e) => {
                    println!("Decryption failed for {}: {}", name, e);
                }
            }
        }
        
        if successful_ops > 0 {
            let avg_time = total_time / successful_ops;
            let throughput = successful_ops as f64 / total_time.as_secs_f64();
            
            println!("{:12} | {:>10.2}ms avg | {:>8.1} ops/sec | {}/{} success", 
                    name, avg_time.as_millis(), throughput, successful_ops, iterations);
        }
    }
}

#[tokio::test]
async fn bench_connection_pool_performance() {
    let params = FheParams::default();
    let pool = FheConnectionPool::new(4, 20, params).unwrap();
    
    println!("\n=== Connection Pool Performance ===");
    
    let test_cases = vec![
        (10, "low_concurrency"),
        (50, "medium_concurrency"),
        (100, "high_concurrency"),
    ];
    
    for (num_ops, name) in test_cases {
        let start = Instant::now();
        let mut tasks = Vec::new();
        
        for i in 0..num_ops {
            let pool = pool.clone();
            let task = tokio::spawn(async move {
                let client_id = Uuid::new_v4();
                let plaintext = format!("Test message {}", i);
                
                let ciphertext = pool.encrypt_balanced(client_id, &plaintext).await?;
                let _decrypted = pool.decrypt_balanced(client_id, &ciphertext).await?;
                
                Ok::<(), homomorphic_llm_proxy::error::Error>(())
            });
            tasks.push(task);
        }
        
        let mut successful_ops = 0;
        for task in tasks {
            match task.await {
                Ok(Ok(())) => successful_ops += 1,
                Ok(Err(e)) => println!("Operation failed: {}", e),
                Err(e) => println!("Task failed: {}", e),
            }
        }
        
        let total_time = start.elapsed();
        let throughput = successful_ops as f64 / total_time.as_secs_f64();
        
        println!("{:18} | {:>3} ops | {:>8.2}s total | {:>8.1} ops/sec | {}/{} success",
                name, num_ops, total_time.as_secs_f64(), throughput, successful_ops, num_ops);
    }
}

#[tokio::test]
async fn bench_cache_performance() {
    let cache = CiphertextCache::new(1000, Duration::from_secs(300));
    
    // Pre-populate cache
    let num_entries = 500;
    let mut ciphertext_ids = Vec::new();
    
    for i in 0..num_entries {
        let ciphertext = create_test_ciphertext(format!("Cache test data {}", i));
        ciphertext_ids.push(ciphertext.id);
        cache.put(ciphertext.id, ciphertext).await;
    }
    
    println!("\n=== Cache Performance Benchmarks ===");
    
    // Benchmark cache hits
    let iterations = 1000;
    let start = Instant::now();
    let mut hits = 0;
    
    for _ in 0..iterations {
        let random_id = &ciphertext_ids[rand::random::<usize>() % ciphertext_ids.len()];
        if cache.get(random_id).await.is_some() {
            hits += 1;
        }
    }
    
    let hit_time = start.elapsed();
    let hit_throughput = hits as f64 / hit_time.as_secs_f64();
    
    println!("Cache hits      | {:>4} ops | {:>10.2}ms avg | {:>8.0} ops/sec",
            hits, hit_time.as_millis() as f64 / hits as f64, hit_throughput);
    
    // Benchmark cache misses
    let start = Instant::now();
    let mut misses = 0;
    
    for _ in 0..100 {
        let random_id = Uuid::new_v4(); // Random UUID not in cache
        if cache.get(&random_id).await.is_none() {
            misses += 1;
        }
    }
    
    let miss_time = start.elapsed();
    let miss_throughput = misses as f64 / miss_time.as_secs_f64();
    
    println!("Cache misses    | {:>4} ops | {:>10.2}ms avg | {:>8.0} ops/sec",
            misses, miss_time.as_millis() as f64 / misses as f64, miss_throughput);
    
    // Show cache statistics
    let stats = cache.get_stats().await;
    println!("Cache stats: {} hits, {} misses, {} entries", stats.hits, stats.misses, stats.current_size);
}

#[tokio::test]
async fn bench_metrics_collection() {
    let metrics = MetricsCollector::new();
    
    println!("\n=== Metrics Collection Performance ===");
    
    let iterations = 100_000;
    let start = Instant::now();
    
    // Benchmark metrics updates
    for i in 0..iterations {
        metrics.increment_requests();
        
        if i % 100 == 0 {
            metrics.increment_encryptions();
        }
        
        if i % 150 == 0 {
            metrics.increment_decryptions();
        }
        
        if i % 1000 == 0 {
            metrics.record_response_time(Duration::from_millis(50));
        }
    }
    
    let total_time = start.elapsed();
    let throughput = iterations as f64 / total_time.as_secs_f64();
    
    println!("Metrics updates | {:>6} ops | {:>8.2}ms total | {:>8.0} ops/sec",
            iterations, total_time.as_millis(), throughput);
    
    // Benchmark stats retrieval
    let start = Instant::now();
    let stats_iterations = 1000;
    
    for _ in 0..stats_iterations {
        let _stats = metrics.get_stats();
    }
    
    let stats_time = start.elapsed();
    let stats_throughput = stats_iterations as f64 / stats_time.as_secs_f64();
    
    println!("Stats retrieval | {:>6} ops | {:>8.2}ms total | {:>8.0} ops/sec",
            stats_iterations, stats_time.as_millis(), stats_throughput);
}

#[tokio::test]
async fn bench_memory_usage() {
    println!("\n=== Memory Usage Analysis ===");
    
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).unwrap();
    let (client_id, _) = engine.generate_keys().unwrap();
    
    // Test memory usage with different text sizes
    let text_sizes = vec![10, 100, 1000, 5000];
    
    for size in text_sizes {
        let plaintext = "A".repeat(size);
        
        match engine.encrypt_text(client_id, &plaintext) {
            Ok(ciphertext) => {
                let compression_ratio = ciphertext.data.len() as f64 / plaintext.len() as f64;
                println!("Text size: {:>5} bytes -> Ciphertext: {:>6} bytes (ratio: {:.2}x)",
                        plaintext.len(), ciphertext.data.len(), compression_ratio);
            }
            Err(e) => {
                println!("Failed to encrypt {}-byte text: {}", size, e);
            }
        }
    }
}

#[tokio::test]
async fn bench_timeout_handling() {
    let params = FheParams::default();
    let pool = FheConnectionPool::new(2, 4, params).unwrap();
    
    println!("\n=== Timeout Handling Performance ===");
    
    let timeout_durations = vec![
        Duration::from_millis(100),
        Duration::from_millis(500),
        Duration::from_secs(1),
    ];
    
    for timeout_duration in timeout_durations {
        let start = Instant::now();
        let client_id = Uuid::new_v4();
        let plaintext = "Timeout test message";
        
        match timeout(timeout_duration, pool.encrypt_balanced(client_id, plaintext)).await {
            Ok(Ok(_)) => {
                println!("Operation completed within {:?}", timeout_duration);
            }
            Ok(Err(e)) => {
                println!("Operation failed within {:?}: {}", timeout_duration, e);
            }
            Err(_) => {
                println!("Operation timed out after {:?}", timeout_duration);
            }
        }
        
        let actual_time = start.elapsed();
        println!("  Actual time: {:?}", actual_time);
    }
}

fn create_test_ciphertext(data: String) -> homomorphic_llm_proxy::fhe::Ciphertext {
    use homomorphic_llm_proxy::fhe::{Ciphertext, FheParams};
    
    Ciphertext {
        id: Uuid::new_v4(),
        data: data.into_bytes(),
        params: FheParams::default(),
        noise_budget: Some(40),
    }
}

#[tokio::test]
async fn bench_end_to_end_latency() {
    println!("\n=== End-to-End Latency Analysis ===");
    
    let params = FheParams::default();
    let pool = FheConnectionPool::new(2, 10, params).unwrap();
    
    let test_messages = vec![
        "Short message",
        "This is a medium-length message for testing purposes",
        &"Long message ".repeat(50),
    ];
    
    for (i, message) in test_messages.iter().enumerate() {
        let iterations = 10;
        let mut latencies = Vec::new();
        
        for _ in 0..iterations {
            let client_id = Uuid::new_v4();
            let start = Instant::now();
            
            // Full round trip: encrypt -> decrypt
            match pool.encrypt_balanced(client_id, message).await {
                Ok(ciphertext) => {
                    match pool.decrypt_balanced(client_id, &ciphertext).await {
                        Ok(decrypted) => {
                            let latency = start.elapsed();
                            if decrypted.trim() == message.trim() {
                                latencies.push(latency);
                            }
                        }
                        Err(e) => println!("Decryption failed: {}", e),
                    }
                }
                Err(e) => println!("Encryption failed: {}", e),
            }
        }
        
        if !latencies.is_empty() {
            latencies.sort();
            let avg = latencies.iter().sum::<Duration>() / latencies.len() as u32;
            let p50 = latencies[latencies.len() / 2];
            let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];
            let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];
            
            println!("Message {} ({} chars):", i + 1, message.len());
            println!("  Avg: {:>8.2}ms | P50: {:>8.2}ms | P95: {:>8.2}ms | P99: {:>8.2}ms",
                    avg.as_millis(), p50.as_millis(), p95.as_millis(), p99.as_millis());
        }
    }
}