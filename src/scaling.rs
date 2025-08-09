//! Scaling and performance optimization features

use crate::error::{Error, Result};
use crate::fhe::{FheEngine, Ciphertext, FheParams};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

/// Connection pool for managing FHE operations
#[derive(Debug)]
pub struct FheConnectionPool {
    engines: Vec<Arc<RwLock<FheEngine>>>,
    current_index: AtomicUsize,
    max_concurrent_ops: Arc<Semaphore>,
    pool_stats: Arc<RwLock<PoolStats>>,
}

#[derive(Debug, Clone)]
struct PoolStats {
    total_operations: u64,
    active_operations: u64,
    avg_operation_time: Duration,
    engine_utilization: HashMap<usize, u64>,
}

impl FheConnectionPool {
    pub fn new(pool_size: usize, max_concurrent: usize, fhe_params: FheParams) -> Result<Self> {
        let mut engines = Vec::with_capacity(pool_size);
        
        for i in 0..pool_size {
            let engine = FheEngine::new(fhe_params.clone())?;
            engines.push(Arc::new(RwLock::new(engine)));
            log::info!("Initialized FHE engine {} in pool", i);
        }

        let mut engine_utilization = HashMap::new();
        for i in 0..pool_size {
            engine_utilization.insert(i, 0);
        }

        Ok(Self {
            engines,
            current_index: AtomicUsize::new(0),
            max_concurrent_ops: Arc::new(Semaphore::new(max_concurrent)),
            pool_stats: Arc::new(RwLock::new(PoolStats {
                total_operations: 0,
                active_operations: 0,
                avg_operation_time: Duration::from_millis(0),
                engine_utilization,
            })),
        })
    }

    /// Dynamically scale the pool size based on load
    pub async fn scale_pool(&mut self, target_size: usize, fhe_params: FheParams) -> Result<()> {
        let current_size = self.engines.len();
        
        if target_size > current_size {
            // Scale up - add new engines
            for i in current_size..target_size {
                let engine = FheEngine::new(fhe_params.clone())?;
                self.engines.push(Arc::new(RwLock::new(engine)));
                
                // Update utilization tracking
                let mut stats = self.pool_stats.write().await;
                stats.engine_utilization.insert(i, 0);
                
                log::info!("Scaled up: Added FHE engine {} to pool", i);
            }
            log::info!("Pool scaled up from {} to {} engines", current_size, target_size);
        } else if target_size < current_size {
            // Scale down - remove engines (but keep minimum of 1)
            let actual_target = target_size.max(1);
            self.engines.truncate(actual_target);
            
            // Clean up utilization tracking
            let mut stats = self.pool_stats.write().await;
            stats.engine_utilization.retain(|&k, _| k < actual_target);
            
            log::info!("Pool scaled down from {} to {} engines", current_size, actual_target);
        }
        
        Ok(())
    }

    /// Get optimal engine based on current load
    async fn get_optimal_engine(&self) -> (usize, Arc<RwLock<FheEngine>>) {
        let stats = self.pool_stats.read().await;
        
        // Find engine with lowest utilization
        let optimal_idx = stats.engine_utilization
            .iter()
            .min_by_key(|(_, &utilization)| utilization)
            .map(|(&idx, _)| idx)
            .unwrap_or(0);
        
        drop(stats);
        (optimal_idx, self.engines[optimal_idx].clone())
    }

    /// Get next available engine using round-robin
    async fn get_engine(&self) -> (usize, Arc<RwLock<FheEngine>>) {
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) % self.engines.len();
        (index, self.engines[index].clone())
    }

    /// Perform encryption with load balancing
    pub async fn encrypt_balanced(&self, client_id: Uuid, plaintext: &str) -> Result<Ciphertext> {
        let _permit = self.max_concurrent_ops.acquire().await
            .map_err(|_| Error::Internal("Failed to acquire semaphore permit".to_string()))?;
        
        let start = Instant::now();
        let (engine_idx, engine) = self.get_optimal_engine().await;
        
        // Update stats
        {
            let mut stats = self.pool_stats.write().await;
            stats.active_operations += 1;
            stats.total_operations += 1;
        }

        let result = {
            let engine = engine.read().await;
            engine.encrypt_text(client_id, plaintext)
        };

        let elapsed = start.elapsed();

        // Update engine utilization and timing stats
        {
            let mut stats = self.pool_stats.write().await;
            stats.active_operations -= 1;
            *stats.engine_utilization.get_mut(&engine_idx).unwrap() += 1;
            
            // Update average operation time (simple moving average)
            let current_avg = stats.avg_operation_time.as_millis() as f64;
            let new_avg = (current_avg + elapsed.as_millis() as f64) / 2.0;
            stats.avg_operation_time = Duration::from_millis(new_avg as u64);
        }

        log::debug!("Encrypted using engine {} in {:?}", engine_idx, elapsed);
        result
    }

    /// Perform decryption with load balancing
    pub async fn decrypt_balanced(&self, client_id: Uuid, ciphertext: &Ciphertext) -> Result<String> {
        let _permit = self.max_concurrent_ops.acquire().await
            .map_err(|_| Error::Internal("Failed to acquire semaphore permit".to_string()))?;
        
        let start = Instant::now();
        let (engine_idx, engine) = self.get_optimal_engine().await;
        
        {
            let mut stats = self.pool_stats.write().await;
            stats.active_operations += 1;
            stats.total_operations += 1;
        }

        let result = {
            let engine = engine.read().await;
            engine.decrypt_text_safe(client_id, ciphertext)
        };

        let elapsed = start.elapsed();

        {
            let mut stats = self.pool_stats.write().await;
            stats.active_operations -= 1;
            *stats.engine_utilization.get_mut(&engine_idx).unwrap() += 1;
        }

        log::debug!("Decrypted using engine {} in {:?}", engine_idx, elapsed);
        result
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        self.pool_stats.read().await.clone()
    }

    /// Health check for all engines
    pub async fn health_check(&self) -> Vec<bool> {
        let mut health_status = Vec::new();
        
        for (i, engine) in self.engines.iter().enumerate() {
            let is_healthy = match tokio::time::timeout(Duration::from_secs(5), async {
                let engine = engine.read().await;
                // Simple health check - try to get parameters
                let _params = engine.get_params();
                true
            }).await {
                Ok(result) => result,
                Err(_) => false,
            };
            
            health_status.push(is_healthy);
            if !is_healthy {
                log::warn!("FHE engine {} failed health check", i);
            }
        }
        
        health_status
    }
}

/// Batch processing for multiple operations
#[derive(Debug)]
pub struct BatchProcessor {
    batch_size: usize,
    flush_interval: Duration,
    pending_operations: Arc<RwLock<Vec<BatchOperation>>>,
    processor_handle: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug)]
struct BatchOperation {
    id: Uuid,
    operation_type: BatchOperationType,
    timestamp: Instant,
}

#[derive(Debug, Clone)]
enum BatchOperationType {
    Encrypt { client_id: Uuid, plaintext: String },
    Decrypt { client_id: Uuid, ciphertext: Ciphertext },
}

#[derive(Debug, Clone)]
enum BatchResult {
    Encrypted(Ciphertext),
    Decrypted(String),
}

impl BatchProcessor {
    pub fn new(batch_size: usize, flush_interval: Duration) -> Self {
        Self {
            batch_size,
            flush_interval,
            pending_operations: Arc::new(RwLock::new(Vec::new())),
            processor_handle: None,
        }
    }

    pub fn start(&mut self, pool: Arc<FheConnectionPool>) -> Result<()> {
        let pending_operations = self.pending_operations.clone();
        let batch_size = self.batch_size;
        let flush_interval = self.flush_interval;

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(flush_interval);
            
            loop {
                interval.tick().await;
                
                let operations = {
                    let mut pending = pending_operations.write().await;
                    if pending.is_empty() {
                        continue;
                    }
                    
                    let to_process = if pending.len() >= batch_size {
                        pending.drain(0..batch_size).collect::<Vec<_>>()
                    } else {
                        // Process all if batch timeout reached
                        let oldest_age = pending.first()
                            .map(|op| op.timestamp.elapsed())
                            .unwrap_or(Duration::ZERO);
                        
                        if oldest_age >= flush_interval {
                            pending.drain(..).collect::<Vec<_>>()
                        } else {
                            continue;
                        }
                    };
                    to_process
                };

                // Process batch
                Self::process_batch(operations, pool.clone()).await;
            }
        });

        self.processor_handle = Some(handle);
        log::info!("Started batch processor with batch_size={}, flush_interval={:?}", 
                   batch_size, flush_interval);
        Ok(())
    }

    async fn process_batch(operations: Vec<BatchOperation>, pool: Arc<FheConnectionPool>) {
        log::debug!("Processing batch of {} operations", operations.len());
        
        let mut tasks = Vec::new();
        
        for operation in operations {
            let pool = pool.clone();
            let task = tokio::spawn(async move {
                let result = match operation.operation_type {
                    BatchOperationType::Encrypt { client_id, plaintext } => {
                        pool.encrypt_balanced(client_id, &plaintext).await
                            .map(BatchResult::Encrypted)
                    }
                    BatchOperationType::Decrypt { client_id, ciphertext } => {
                        pool.decrypt_balanced(client_id, &ciphertext).await
                            .map(BatchResult::Decrypted)
                    }
                };
                
                (operation.id, result)
            });
            tasks.push(task);
        }

        // Wait for all operations to complete
        for task in tasks {
            if let Ok((op_id, result)) = task.await {
                log::debug!("Completed batch operation {}: {:?}", op_id, result.is_ok());
            }
        }
    }

    pub async fn add_operation(&self, operation_type: BatchOperationType) {
        let operation = BatchOperation {
            id: Uuid::new_v4(),
            operation_type,
            timestamp: Instant::now(),
        };

        let mut pending = self.pending_operations.write().await;
        pending.push(operation);
    }
}

/// Caching layer for frequently accessed ciphertexts
#[derive(Debug)]
pub struct CiphertextCache {
    cache: Arc<RwLock<HashMap<Uuid, CacheEntry>>>,
    max_size: usize,
    ttl: Duration,
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    ciphertext: Ciphertext,
    last_accessed: Instant,
    access_count: u64,
}

#[derive(Debug, Clone)]
struct CacheStats {
    hits: u64,
    misses: u64,
    evictions: u64,
    current_size: usize,
}

impl CiphertextCache {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            ttl,
            stats: Arc::new(RwLock::new(CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                current_size: 0,
            })),
        }
    }

    pub async fn get(&self, id: &Uuid) -> Option<Ciphertext> {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = cache.get_mut(id) {
            // Check if entry is still valid
            if entry.last_accessed.elapsed() <= self.ttl {
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                stats.hits += 1;
                return Some(entry.ciphertext.clone());
            } else {
                // Entry expired, remove it
                cache.remove(id);
                stats.current_size = cache.len();
            }
        }

        stats.misses += 1;
        None
    }

    pub async fn put(&self, id: Uuid, ciphertext: Ciphertext) {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        // Check if we need to evict entries
        if cache.len() >= self.max_size {
            self.evict_lru(&mut cache, &mut stats).await;
        }

        let entry = CacheEntry {
            ciphertext,
            last_accessed: Instant::now(),
            access_count: 1,
        };

        cache.insert(id, entry);
        stats.current_size = cache.len();
    }

    async fn evict_lru(&self, cache: &mut HashMap<Uuid, CacheEntry>, stats: &mut CacheStats) {
        // Find entry with oldest last_accessed time
        let oldest_id = cache.iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(id, _)| *id);

        if let Some(id) = oldest_id {
            cache.remove(&id);
            stats.evictions += 1;
            log::debug!("Evicted ciphertext {} from cache", id);
        }
    }

    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;
        
        let now = Instant::now();
        let expired_keys: Vec<Uuid> = cache.iter()
            .filter(|(_, entry)| now.duration_since(entry.last_accessed) > self.ttl)
            .map(|(id, _)| *id)
            .collect();

        for key in expired_keys {
            cache.remove(&key);
            stats.evictions += 1;
        }
        
        stats.current_size = cache.len();
        
        if !cache.is_empty() {
            log::debug!("Cleaned up expired cache entries, {} entries remaining", cache.len());
        }
    }

    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Intelligent prefetching based on access patterns
    pub async fn prefetch_likely_accessed(&self, prediction_engine: &PredictionEngine) {
        let access_predictions = prediction_engine.predict_next_accesses().await;
        
        for prediction in access_predictions {
            if let Some(ciphertext) = self.get(&prediction.ciphertext_id).await {
                log::debug!("Prefetched ciphertext {} (confidence: {:.2})", 
                          prediction.ciphertext_id, prediction.confidence);
            }
        }
    }

    /// Warm up cache with commonly accessed ciphertexts
    pub async fn warm_cache(&self, warm_up_data: Vec<(Uuid, Ciphertext)>) {
        log::info!("Warming cache with {} entries", warm_up_data.len());
        
        for (id, ciphertext) in warm_up_data {
            self.put(id, ciphertext).await;
        }
        
        let stats = self.get_stats().await;
        log::info!("Cache warmed: {} entries loaded", stats.current_size);
    }
}

/// Prediction engine for intelligent caching
#[derive(Debug)]
pub struct PredictionEngine {
    access_patterns: Arc<RwLock<HashMap<Uuid, AccessPattern>>>,
    ml_model: Arc<RwLock<SimplePredictionModel>>,
}

#[derive(Debug, Clone)]
struct AccessPattern {
    access_count: u64,
    last_access: Instant,
    access_frequency: f64, // accesses per hour
    correlation_score: f64,
}

#[derive(Debug)]
struct SimplePredictionModel {
    weights: Vec<f64>,
    threshold: f64,
}

#[derive(Debug, Clone)]
pub struct AccessPrediction {
    pub ciphertext_id: Uuid,
    pub confidence: f64,
    pub predicted_access_time: Duration,
}

impl PredictionEngine {
    pub fn new() -> Self {
        Self {
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            ml_model: Arc::new(RwLock::new(SimplePredictionModel {
                weights: vec![0.5, 0.3, 0.2], // frequency, recency, correlation
                threshold: 0.6,
            })),
        }
    }

    pub async fn record_access(&self, ciphertext_id: Uuid) {
        let mut patterns = self.access_patterns.write().await;
        let pattern = patterns.entry(ciphertext_id).or_insert(AccessPattern {
            access_count: 0,
            last_access: Instant::now(),
            access_frequency: 0.0,
            correlation_score: 0.5,
        });

        pattern.access_count += 1;
        pattern.last_access = Instant::now();
        
        // Update frequency (simplified calculation)
        let hours_since_first = pattern.access_count as f64 / 24.0; // Approximate
        pattern.access_frequency = pattern.access_count as f64 / hours_since_first.max(1.0);
    }

    pub async fn predict_next_accesses(&self) -> Vec<AccessPrediction> {
        let patterns = self.access_patterns.read().await;
        let model = self.ml_model.read().await;
        let mut predictions = Vec::new();

        for (&ciphertext_id, pattern) in patterns.iter() {
            let recency_score = 1.0 / (1.0 + pattern.last_access.elapsed().as_secs() as f64 / 3600.0);
            let frequency_score = pattern.access_frequency / 10.0; // Normalize
            let correlation_score = pattern.correlation_score;

            let confidence = model.weights[0] * frequency_score +
                           model.weights[1] * recency_score +
                           model.weights[2] * correlation_score;

            if confidence > model.threshold {
                predictions.push(AccessPrediction {
                    ciphertext_id,
                    confidence,
                    predicted_access_time: Duration::from_secs((3600.0 / pattern.access_frequency) as u64),
                });
            }
        }

        // Sort by confidence descending
        predictions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        predictions.truncate(10); // Top 10 predictions

        predictions
    }
}

/// Auto-scaling coordinator
#[derive(Debug)]
pub struct AutoScaler {
    target_cpu_percent: f64,
    target_queue_length: usize,
    scale_up_threshold: f64,
    scale_down_threshold: f64,
    current_replicas: AtomicUsize,
    max_replicas: usize,
    min_replicas: usize,
    last_scale_event: Arc<RwLock<Instant>>,
    cooldown_period: Duration,
}

impl AutoScaler {
    pub fn new(
        target_cpu_percent: f64,
        target_queue_length: usize,
        min_replicas: usize,
        max_replicas: usize,
        cooldown_period: Duration,
    ) -> Self {
        Self {
            target_cpu_percent,
            target_queue_length,
            scale_up_threshold: target_cpu_percent * 1.2,
            scale_down_threshold: target_cpu_percent * 0.6,
            current_replicas: AtomicUsize::new(min_replicas),
            max_replicas,
            min_replicas,
            last_scale_event: Arc::new(RwLock::new(Instant::now())),
            cooldown_period,
        }
    }

    pub async fn evaluate_scaling(&self, metrics: &ScalingMetrics) -> ScalingDecision {
        let last_scale = *self.last_scale_event.read().await;
        if last_scale.elapsed() < self.cooldown_period {
            return ScalingDecision::NoAction;
        }

        let current = self.current_replicas.load(Ordering::Relaxed);
        
        // Evaluate scale up conditions
        if (metrics.cpu_utilization > self.scale_up_threshold || 
            metrics.queue_length > self.target_queue_length) &&
            current < self.max_replicas {
            return ScalingDecision::ScaleUp {
                from: current,
                to: (current + 1).min(self.max_replicas),
                reason: format!("CPU: {:.1}%, Queue: {}", metrics.cpu_utilization, metrics.queue_length),
            };
        }

        // Evaluate scale down conditions
        if metrics.cpu_utilization < self.scale_down_threshold &&
           metrics.queue_length < self.target_queue_length / 2 &&
           current > self.min_replicas {
            return ScalingDecision::ScaleDown {
                from: current,
                to: (current - 1).max(self.min_replicas),
                reason: format!("CPU: {:.1}%, Queue: {}", metrics.cpu_utilization, metrics.queue_length),
            };
        }

        ScalingDecision::NoAction
    }

    pub async fn apply_scaling(&self, decision: ScalingDecision) -> Result<()> {
        match decision {
            ScalingDecision::ScaleUp { from, to, reason } => {
                log::info!("Scaling up from {} to {} replicas: {}", from, to, reason);
                self.current_replicas.store(to, Ordering::Relaxed);
                *self.last_scale_event.write().await = Instant::now();
            }
            ScalingDecision::ScaleDown { from, to, reason } => {
                log::info!("Scaling down from {} to {} replicas: {}", from, to, reason);
                self.current_replicas.store(to, Ordering::Relaxed);
                *self.last_scale_event.write().await = Instant::now();
            }
            ScalingDecision::NoAction => {}
        }
        Ok(())
    }

    pub fn get_current_replicas(&self) -> usize {
        self.current_replicas.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
pub struct ScalingMetrics {
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub queue_length: usize,
    pub active_connections: usize,
    pub response_time_p95: Duration,
}

#[derive(Debug, Clone)]
pub enum ScalingDecision {
    ScaleUp { from: usize, to: usize, reason: String },
    ScaleDown { from: usize, to: usize, reason: String },
    NoAction,
}

/// Circuit breaker for external dependencies
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    state: Arc<RwLock<CircuitState>>,
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing, reject requests
    HalfOpen,  // Testing if service recovered
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            timeout,
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(CircuitState::Closed)),
        }
    }

    pub async fn call<F, T, E>(&self, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Open => {
                // Check if timeout has passed
                let last_failure = *self.last_failure_time.read().await;
                if let Some(failure_time) = last_failure {
                    if failure_time.elapsed() >= self.timeout {
                        // Transition to half-open
                        *self.state.write().await = CircuitState::HalfOpen;
                        log::info!("Circuit breaker transitioning to half-open");
                    } else {
                        return Err(Error::Internal("Circuit breaker is open".to_string()));
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests to test service
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        // Execute operation
        match operation.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(Error::Internal(format!("Circuit breaker operation failed: {}", e)))
            }
        }
    }

    async fn on_success(&self) {
        let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        let state = *self.state.read().await;
        if state == CircuitState::HalfOpen && success_count >= self.success_threshold as u64 {
            *self.state.write().await = CircuitState::Closed;
            self.failure_count.store(0, Ordering::Relaxed);
            self.success_count.store(0, Ordering::Relaxed);
            log::info!("Circuit breaker closed after successful recovery");
        }
    }

    async fn on_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure_time.write().await = Some(Instant::now());
        
        if failure_count >= self.failure_threshold as u64 {
            *self.state.write().await = CircuitState::Open;
            log::warn!("Circuit breaker opened after {} failures", failure_count);
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_fhe_connection_pool() {
        let params = FheParams::default();
        let pool = FheConnectionPool::new(2, 4, params).unwrap();
        
        // Generate keys in the first engine and replicate to others
        let client_id = {
            let mut first_engine = pool.engines[0].write().await;
            let (client_id, _) = first_engine.generate_keys().unwrap();
            client_id
        };
        
        // Replicate the client key to all other engines to handle load balancing
        let client_key = {
            let first_engine = pool.engines[0].read().await;
            first_engine.client_keys.get(&client_id).unwrap().clone()
        };
        
        for (i, pool_engine) in pool.engines.iter().enumerate() {
            if i > 0 { // Skip first engine as it already has the key
                let mut engine_guard = pool_engine.write().await;
                engine_guard.client_keys.insert(client_id, client_key.clone());
            }

        }
        
        let plaintext = "Hello, world!";
        
        // Test encryption
        let ciphertext = pool.encrypt_balanced(client_id, plaintext).await.unwrap();
        assert!(!ciphertext.data.is_empty());
        
        // Test decryption
        let decrypted = pool.decrypt_balanced(client_id, &ciphertext).await.unwrap();
        assert_eq!(decrypted, plaintext);
        
        // Check stats
        let stats = pool.get_stats().await;
        assert_eq!(stats.total_operations, 2);
    }

    #[tokio::test]
    async fn test_ciphertext_cache() {
        let cache = CiphertextCache::new(2, Duration::from_secs(1));
        let id = Uuid::new_v4();
        let ciphertext = Ciphertext {
            id,
            data: vec![1, 2, 3, 4],
            params: FheParams::default(),
            noise_budget: Some(50),
        };
        
        // Test put and get
        cache.put(id, ciphertext.clone()).await;
        let retrieved = cache.get(&id).await.unwrap();
        assert_eq!(retrieved.data, ciphertext.data);
        
        // Test cache stats
        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.current_size, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(2, 1, Duration::from_millis(100));
        
        // Test successful operation
        let result = breaker.call(async { Ok::<i32, &str>(42) }).await;
        assert!(result.is_ok());
        
        // Test failures
        for _ in 0..2 {
            let _ = breaker.call(async { Err::<i32, &str>("failure") }).await;
        }
        
        // Circuit should be open now
        let state = breaker.get_state().await;
        assert_eq!(state, CircuitState::Open);
        
        // Test that requests are rejected
        let result = breaker.call(async { Ok::<i32, &str>(42) }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_scaler() {
        let scaler = AutoScaler::new(70.0, 10, 1, 5, Duration::from_millis(50));
        
        // Wait for cooldown period to pass
        sleep(Duration::from_millis(60)).await;
        
        let metrics = ScalingMetrics {
            cpu_utilization: 85.0,
            memory_utilization: 60.0,
            queue_length: 15,
            active_connections: 20,
            response_time_p95: Duration::from_millis(200),
        };
        
        let decision = scaler.evaluate_scaling(&metrics).await;
        
        match decision {
            ScalingDecision::ScaleUp { from, to, .. } => {
                assert_eq!(from, 1);
                assert_eq!(to, 2);
            }
            _ => panic!("Expected scale up decision"),
        }
    }
}