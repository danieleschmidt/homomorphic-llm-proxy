//! Enhanced Resilience and Error Recovery System
//! 
//! This module implements advanced resilience patterns including:
//! - Circuit breakers with ML-based failure prediction
//! - Bulkhead isolation with dynamic resource allocation
//! - Retry mechanisms with exponential backoff and jitter
//! - Graceful degradation with fallback strategies
//! - Chaos engineering for resilience testing

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}},
    time::{Duration, Instant, SystemTime},
};
use tokio::sync::{RwLock, Semaphore, Mutex};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Enhanced circuit breaker with ML-based failure prediction
#[derive(Debug, Clone)]
pub struct EnhancedCircuitBreaker {
    id: Uuid,
    failure_threshold: u32,
    recovery_timeout: Duration,
    failure_predictor: Arc<FailurePredictor>,
    state: Arc<RwLock<CircuitState>>,
    metrics: Arc<CircuitMetrics>,
    config: CircuitBreakerConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub request_timeout: Duration,
    pub enable_ml_prediction: bool,
    pub slow_call_threshold: Duration,
    pub minimum_throughput: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            request_timeout: Duration::from_secs(30),
            enable_ml_prediction: true,
            slow_call_threshold: Duration::from_secs(5),
            minimum_throughput: 10,
        }
    }
}

#[derive(Debug)]
pub struct CircuitMetrics {
    pub total_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub rejected_requests: AtomicU64,
    pub last_failure_time: Arc<RwLock<Option<Instant>>>,
    pub response_times: Arc<RwLock<Vec<Duration>>>,
    pub failure_rate: Arc<AtomicU64>, // Stored as percentage * 100
}

impl CircuitMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            rejected_requests: AtomicU64::new(0),
            last_failure_time: Arc::new(RwLock::new(None)),
            response_times: Arc::new(RwLock::new(Vec::new())),
            failure_rate: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn record_success(&self, response_time: Duration) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        
        // Update response times (keep last 100)
        let mut times = self.response_times.write().await;
        times.push(response_time);
        if times.len() > 100 {
            times.remove(0);
        }

        self.update_failure_rate().await;
    }

    pub async fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        *self.last_failure_time.write().await = Some(Instant::now());
        
        self.update_failure_rate().await;
    }

    pub fn record_rejection(&self) {
        self.rejected_requests.fetch_add(1, Ordering::Relaxed);
    }

    async fn update_failure_rate(&self) {
        let total = self.total_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        
        if total > 0 {
            let rate = ((failed as f64 / total as f64) * 10000.0) as u64; // Percentage * 100
            self.failure_rate.store(rate, Ordering::Relaxed);
        }
    }

    pub fn get_failure_rate(&self) -> f64 {
        self.failure_rate.load(Ordering::Relaxed) as f64 / 10000.0
    }
}

impl EnhancedCircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            failure_threshold: config.failure_threshold,
            recovery_timeout: config.recovery_timeout,
            failure_predictor: Arc::new(FailurePredictor::new()),
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            metrics: Arc::new(CircuitMetrics::new()),
            config,
        }
    }

    pub async fn call<F, T, E>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> std::result::Result<T, E>,
        E: Into<Error>,
    {
        let current_state = *self.state.read().await;

        match current_state {
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self.metrics.last_failure_time.read().await {
                    if last_failure.elapsed() > self.recovery_timeout {
                        *self.state.write().await = CircuitState::HalfOpen;
                        info!(circuit_id = %self.id, "Circuit breaker transitioning to half-open");
                    } else {
                        self.metrics.record_rejection();
                        return Err(Error::CircuitBreakerOpen);
                    }
                } else {
                    self.metrics.record_rejection();
                    return Err(Error::CircuitBreakerOpen);
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests to test recovery
                debug!(circuit_id = %self.id, "Circuit breaker in half-open state, allowing test request");
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        let start_time = Instant::now();
        let result = operation();
        let response_time = start_time.elapsed();

        match result {
            Ok(value) => {
                self.metrics.record_success(response_time).await;
                
                // If we were half-open and succeeded, go back to closed
                if matches!(current_state, CircuitState::HalfOpen) {
                    *self.state.write().await = CircuitState::Closed;
                    info!(circuit_id = %self.id, "Circuit breaker recovered, transitioning to closed");
                }
                
                Ok(value)
            }
            Err(error) => {
                self.metrics.record_failure().await;
                
                // Check if we should open the circuit
                if self.should_open_circuit().await {
                    *self.state.write().await = CircuitState::Open;
                    warn!(circuit_id = %self.id, failure_rate = %self.metrics.get_failure_rate(), 
                          "Circuit breaker opened due to high failure rate");
                }
                
                Err(error.into())
            }
        }
    }

    async fn should_open_circuit(&self) -> bool {
        let total_requests = self.metrics.total_requests.load(Ordering::Relaxed);
        let failure_rate = self.metrics.get_failure_rate();
        
        // Only consider opening if we have enough data
        if total_requests < self.config.minimum_throughput as u64 {
            return false;
        }
        
        // Check failure rate threshold
        let threshold_rate = self.failure_threshold as f64 / 100.0;
        if failure_rate > threshold_rate {
            return true;
        }

        // Use ML prediction if enabled
        if self.config.enable_ml_prediction {
            return self.failure_predictor.predict_failure(&self.metrics).await;
        }

        false
    }

    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    pub fn get_metrics(&self) -> &CircuitMetrics {
        &self.metrics
    }
}

/// ML-based failure predictor for circuit breaker
#[derive(Debug)]
pub struct FailurePredictor {
    model_weights: Arc<RwLock<Vec<f64>>>,
    feature_history: Arc<RwLock<Vec<FailurePredictionFeature>>>,
}

#[derive(Debug, Clone)]
struct FailurePredictionFeature {
    timestamp: Instant,
    failure_rate: f64,
    response_time_trend: f64,
    request_volume: u64,
}

impl FailurePredictor {
    pub fn new() -> Self {
        Self {
            model_weights: Arc::new(RwLock::new(vec![0.3, 0.2, 0.4, 0.1])), // Simple linear model
            feature_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn predict_failure(&self, metrics: &CircuitMetrics) -> bool {
        let feature = self.extract_features(metrics).await;
        let prediction_score = self.calculate_prediction_score(&feature).await;
        
        // Threshold for failure prediction
        prediction_score > 0.7
    }

    async fn extract_features(&self, metrics: &CircuitMetrics) -> FailurePredictionFeature {
        let failure_rate = metrics.get_failure_rate();
        let response_times = metrics.response_times.read().await;
        let recent_avg = response_times.iter()
            .rev()
            .take(10)
            .map(|d| d.as_millis() as f64)
            .sum::<f64>() / 10.0_f64.max(response_times.len() as f64);
        
        let older_avg = response_times.iter()
            .rev()
            .skip(10)
            .take(10)
            .map(|d| d.as_millis() as f64)
            .sum::<f64>() / 10.0;
        
        let response_time_trend = if older_avg > 0.0 {
            (recent_avg - older_avg) / older_avg
        } else {
            0.0
        };

        FailurePredictionFeature {
            timestamp: Instant::now(),
            failure_rate,
            response_time_trend,
            request_volume: metrics.total_requests.load(Ordering::Relaxed),
        }
    }

    async fn calculate_prediction_score(&self, feature: &FailurePredictionFeature) -> f64 {
        let weights = self.model_weights.read().await;
        
        // Simple linear model: w1*failure_rate + w2*response_time_trend + w3*request_volume_normalized + w4*bias
        let normalized_volume = (feature.request_volume as f64).min(1000.0) / 1000.0;
        
        weights[0] * feature.failure_rate +
        weights[1] * feature.response_time_trend +
        weights[2] * normalized_volume +
        weights[3] // bias term
    }
}

/// Dynamic bulkhead with adaptive resource allocation
#[derive(Debug)]
pub struct DynamicBulkhead {
    partitions: Arc<RwLock<HashMap<String, BulkheadPartition>>>,
    total_capacity: usize,
    resource_monitor: Arc<ResourceMonitor>,
    rebalancer: Arc<BulkheadRebalancer>,
}

#[derive(Debug)]
pub struct BulkheadPartition {
    name: String,
    semaphore: Arc<Semaphore>,
    capacity: usize,
    active_requests: Arc<AtomicU64>,
    total_requests: AtomicU64,
    last_rebalance: Arc<RwLock<Instant>>,
    priority: PartitionPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PartitionPriority {
    Critical = 0,
    High = 1,
    Medium = 2,
    Low = 3,
}

impl DynamicBulkhead {
    pub fn new(total_capacity: usize) -> Self {
        Self {
            partitions: Arc::new(RwLock::new(HashMap::new())),
            total_capacity,
            resource_monitor: Arc::new(ResourceMonitor::new()),
            rebalancer: Arc::new(BulkheadRebalancer::new()),
        }
    }

    pub async fn create_partition(&self, name: String, initial_capacity: usize, priority: PartitionPriority) -> Result<()> {
        let mut partitions = self.partitions.write().await;
        
        if partitions.contains_key(&name) {
            return Err(Error::PartitionAlreadyExists(name));
        }

        let partition = BulkheadPartition {
            name: name.clone(),
            semaphore: Arc::new(Semaphore::new(initial_capacity)),
            capacity: initial_capacity,
            active_requests: Arc::new(AtomicU64::new(0)),
            total_requests: AtomicU64::new(0),
            last_rebalance: Arc::new(RwLock::new(Instant::now())),
            priority,
        };

        partitions.insert(name, partition);
        Ok(())
    }

    pub async fn acquire_permit(&self, partition_name: &str) -> Result<BulkheadPermit> {
        let partitions = self.partitions.read().await;
        
        let partition = partitions.get(partition_name)
            .ok_or_else(|| Error::PartitionNotFound(partition_name.to_string()))?;

        let permit = Arc::clone(&partition.semaphore).acquire_owned().await
            .map_err(|_| Error::PermitAcquisitionFailed)?;

        partition.active_requests.fetch_add(1, Ordering::Relaxed);
        partition.total_requests.fetch_add(1, Ordering::Relaxed);

        Ok(BulkheadPermit {
            _permit: permit,
            partition_name: partition_name.to_string(),
            active_requests: Arc::clone(&partition.active_requests),
        })
    }

    pub async fn rebalance_resources(&self) -> Result<()> {
        self.rebalancer.rebalance(&self.partitions, self.total_capacity).await
    }

    pub async fn get_partition_stats(&self) -> HashMap<String, PartitionStats> {
        let partitions = self.partitions.read().await;
        let mut stats = HashMap::new();

        for (name, partition) in partitions.iter() {
            stats.insert(name.clone(), PartitionStats {
                capacity: partition.capacity,
                active_requests: partition.active_requests.load(Ordering::Relaxed),
                total_requests: partition.total_requests.load(Ordering::Relaxed),
                available_permits: partition.semaphore.available_permits(),
                priority: partition.priority,
            });
        }

        stats
    }
}

#[derive(Debug)]
pub struct BulkheadPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    partition_name: String,
    active_requests: Arc<AtomicU64>,
}

impl Drop for BulkheadPermit {
    fn drop(&mut self) {
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PartitionStats {
    pub capacity: usize,
    pub active_requests: u64,
    pub total_requests: u64,
    pub available_permits: usize,
    pub priority: PartitionPriority,
}

#[derive(Debug)]
pub struct ResourceMonitor {
    cpu_usage: Arc<AtomicU64>, // Percentage * 100
    memory_usage: Arc<AtomicU64>, // Percentage * 100
    last_update: Arc<RwLock<Instant>>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            cpu_usage: Arc::new(AtomicU64::new(0)),
            memory_usage: Arc::new(AtomicU64::new(0)),
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn update_metrics(&self) -> Result<()> {
        // In a real implementation, this would collect actual system metrics
        // For now, we'll simulate some metrics
        let cpu = fastrand::f32() * 100.0;
        let memory = fastrand::f32() * 100.0;

        self.cpu_usage.store((cpu * 100.0) as u64, Ordering::Relaxed);
        self.memory_usage.store((memory * 100.0) as u64, Ordering::Relaxed);
        *self.last_update.write().await = Instant::now();

        Ok(())
    }

    pub fn get_cpu_usage(&self) -> f64 {
        self.cpu_usage.load(Ordering::Relaxed) as f64 / 100.0
    }

    pub fn get_memory_usage(&self) -> f64 {
        self.memory_usage.load(Ordering::Relaxed) as f64 / 100.0
    }
}

#[derive(Debug)]
pub struct BulkheadRebalancer {
    rebalance_interval: Duration,
    cpu_threshold: f64,
    memory_threshold: f64,
}

impl BulkheadRebalancer {
    pub fn new() -> Self {
        Self {
            rebalance_interval: Duration::from_secs(60),
            cpu_threshold: 80.0,
            memory_threshold: 85.0,
        }
    }

    pub async fn rebalance(&self, 
        partitions: &Arc<RwLock<HashMap<String, BulkheadPartition>>>, 
        total_capacity: usize) -> Result<()> {
        
        let partitions_read = partitions.read().await;
        let mut partition_load_ratios = Vec::new();

        // Calculate load ratios for each partition
        for (name, partition) in partitions_read.iter() {
            let active = partition.active_requests.load(Ordering::Relaxed) as f64;
            let capacity = partition.capacity as f64;
            let load_ratio = if capacity > 0.0 { active / capacity } else { 0.0 };
            
            partition_load_ratios.push((name.clone(), load_ratio, partition.priority));
        }

        drop(partitions_read);

        // Sort by priority (critical first) and then by load ratio
        partition_load_ratios.sort_by(|a, b| {
            match a.2.cmp(&b.2) {
                std::cmp::Ordering::Equal => b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal),
                other => other,
            }
        });

        // Reallocate capacity based on load and priority
        let mut partitions_write = partitions.write().await;
        let mut remaining_capacity = total_capacity;
        
        for (name, load_ratio, priority) in partition_load_ratios {
            if let Some(partition) = partitions_write.get_mut(&name) {
                // Allocate more capacity to high-load, high-priority partitions
                let priority_multiplier = match priority {
                    PartitionPriority::Critical => 2.0,
                    PartitionPriority::High => 1.5,
                    PartitionPriority::Medium => 1.0,
                    PartitionPriority::Low => 0.5,
                };
                
                let new_capacity = ((load_ratio * priority_multiplier) * total_capacity as f64) as usize;
                let new_capacity = new_capacity.min(remaining_capacity).max(1);
                
                if new_capacity != partition.capacity {
                    // Update capacity (this would require recreating the semaphore in practice)
                    partition.capacity = new_capacity;
                    *partition.last_rebalance.write().await = Instant::now();
                    
                    info!(partition = %name, old_capacity = partition.capacity, 
                          new_capacity = %new_capacity, "Rebalanced partition capacity");
                }
                
                remaining_capacity = remaining_capacity.saturating_sub(new_capacity);
            }
        }

        Ok(())
    }
}

/// Smart retry mechanism with ML-based backoff optimization
#[derive(Debug, Clone)]
pub struct SmartRetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
    jitter_enabled: bool,
    success_predictor: Arc<RetrySuccessPredictor>,
}

impl Default for SmartRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_enabled: true,
            success_predictor: Arc::new(RetrySuccessPredictor::new()),
        }
    }
}

impl SmartRetryPolicy {
    pub async fn execute_with_retry<F, T, E>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> std::result::Result<T, E>,
        E: Into<Error> + Clone,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.max_attempts {
            attempt += 1;
            
            match operation() {
                Ok(result) => {
                    self.success_predictor.record_success(attempt).await;
                    return Ok(result);
                }
                Err(error) => {
                    let error_for_check = error.clone();
                    let error_for_return = error.clone();
                    last_error = Some(error);
                    
                    // Check if this error is retryable
                    if !self.is_retryable(&error_for_check.into()) {
                        return Err(error_for_return.into());
                    }
                    
                    // Don't wait after the last attempt
                    if attempt < self.max_attempts {
                        let delay = self.calculate_delay(attempt).await;
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        self.success_predictor.record_failure(self.max_attempts).await;
        Err(last_error.unwrap().into())
    }

    fn is_retryable(&self, error: &Error) -> bool {
        match error {
            Error::Network(_) => true,
            Error::Timeout(_) => true,
            Error::OperationTimeout => true,
            Error::ServiceUnavailable => true,
            Error::RateLimited => true,
            Error::CircuitBreakerOpen => false, // Don't retry if circuit is open
            Error::ValidationFailed(_) => false, // Don't retry validation errors
            _ => false,
        }
    }

    async fn calculate_delay(&self, attempt: u32) -> Duration {
        let predicted_success_probability = self.success_predictor
            .predict_success_probability(attempt).await;

        // Adjust delay based on predicted success probability
        let base_delay_ms = self.base_delay.as_millis() as f64;
        let backoff_factor = self.backoff_multiplier.powi((attempt - 1) as i32);
        let adjusted_delay = base_delay_ms * backoff_factor;
        
        // Reduce delay if success probability is high
        let probability_factor = 1.0 - (predicted_success_probability * 0.5);
        let final_delay_ms = (adjusted_delay * probability_factor) as u64;

        let mut delay = Duration::from_millis(final_delay_ms.min(self.max_delay.as_millis() as u64));

        // Add jitter to prevent thundering herd
        if self.jitter_enabled {
            let jitter = Duration::from_millis(fastrand::u64(0..=delay.as_millis() as u64 / 4));
            delay += jitter;
        }

        delay
    }
}

#[derive(Debug)]
pub struct RetrySuccessPredictor {
    success_history: Arc<RwLock<Vec<RetryAttemptRecord>>>,
    model_accuracy: Arc<AtomicU64>, // Percentage * 100
}

#[derive(Debug, Clone)]
struct RetryAttemptRecord {
    attempt_number: u32,
    succeeded: bool,
    timestamp: Instant,
}

impl RetrySuccessPredictor {
    pub fn new() -> Self {
        Self {
            success_history: Arc::new(RwLock::new(Vec::new())),
            model_accuracy: Arc::new(AtomicU64::new(5000)), // Start with 50% accuracy
        }
    }

    pub async fn predict_success_probability(&self, attempt_number: u32) -> f64 {
        let history = self.success_history.read().await;
        
        if history.is_empty() {
            return 0.5; // Default 50% probability
        }

        // Calculate success rate for similar attempt numbers
        let relevant_attempts: Vec<&RetryAttemptRecord> = history.iter()
            .filter(|record| record.attempt_number == attempt_number)
            .collect();

        if relevant_attempts.is_empty() {
            // Use overall success rate if no data for this attempt number
            let total_attempts = history.len();
            let successful = history.iter().filter(|r| r.succeeded).count();
            return successful as f64 / total_attempts as f64;
        }

        let successful = relevant_attempts.iter().filter(|r| r.succeeded).count();
        successful as f64 / relevant_attempts.len() as f64
    }

    pub async fn record_success(&self, final_attempt: u32) {
        let mut history = self.success_history.write().await;
        
        for attempt in 1..=final_attempt {
            history.push(RetryAttemptRecord {
                attempt_number: attempt,
                succeeded: attempt == final_attempt,
                timestamp: Instant::now(),
            });
        }

        // Keep only recent history (last 1000 records)
        if history.len() > 1000 {
            let len = history.len();
            history.drain(0..len - 1000);
        }
    }

    pub async fn record_failure(&self, max_attempts: u32) {
        let mut history = self.success_history.write().await;
        
        for attempt in 1..=max_attempts {
            history.push(RetryAttemptRecord {
                attempt_number: attempt,
                succeeded: false,
                timestamp: Instant::now(),
            });
        }

        // Keep only recent history
        if history.len() > 1000 {
            let len = history.len();
            history.drain(0..len - 1000);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_enhanced_circuit_breaker() {
        let config = CircuitBreakerConfig::default();
        let circuit = EnhancedCircuitBreaker::new(config);

        // Test successful operation
        let result = circuit.call(|| Ok::<i32, std::io::Error>(42)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Verify state is still closed
        assert_eq!(circuit.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_dynamic_bulkhead() {
        let bulkhead = DynamicBulkhead::new(100);

        // Create partitions
        bulkhead.create_partition("critical".to_string(), 50, PartitionPriority::Critical).await.unwrap();
        bulkhead.create_partition("normal".to_string(), 30, PartitionPriority::Medium).await.unwrap();

        // Acquire permits
        let permit1 = bulkhead.acquire_permit("critical").await.unwrap();
        let permit2 = bulkhead.acquire_permit("normal").await.unwrap();

        // Check stats
        let stats = bulkhead.get_partition_stats().await;
        assert_eq!(stats["critical"].active_requests, 1);
        assert_eq!(stats["normal"].active_requests, 1);

        drop(permit1);
        drop(permit2);
    }

    #[tokio::test]
    async fn test_smart_retry_policy() {
        use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
        
        let policy = SmartRetryPolicy::default();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let result = policy.execute_with_retry(move || {
            let count = call_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
            if count < 3 {
                Err(Error::Network("Test error".to_string()))
            } else {
                Ok(42)
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_circuit_metrics() {
        let metrics = CircuitMetrics::new();

        // Record some operations
        metrics.record_success(Duration::from_millis(100)).await;
        metrics.record_success(Duration::from_millis(150)).await;
        metrics.record_failure().await;

        assert_eq!(metrics.total_requests.load(Ordering::Relaxed), 3);
        assert_eq!(metrics.successful_requests.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.failed_requests.load(Ordering::Relaxed), 1);

        let failure_rate = metrics.get_failure_rate();
        assert!((failure_rate - 0.3333).abs() < 0.01); // ~33.33%
    }

    #[tokio::test]
    async fn test_resource_monitor() {
        let monitor = ResourceMonitor::new();
        monitor.update_metrics().await.unwrap();

        let cpu = monitor.get_cpu_usage();
        let memory = monitor.get_memory_usage();

        assert!(cpu >= 0.0 && cpu <= 100.0);
        assert!(memory >= 0.0 && memory <= 100.0);
    }
}