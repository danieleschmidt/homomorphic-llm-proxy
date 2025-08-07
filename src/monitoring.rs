//! Monitoring, health checks, and observability

use crate::error::{Error, Result};
use crate::middleware::MetricsSnapshot;
use crate::fhe::FheEngine;
// Axum imports removed as they're not used in this module
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: HashMap<String, ComponentHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub last_check: u64,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
}

/// Detailed system metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub requests: MetricsSnapshot,
    pub fhe_operations: FheMetrics,
    pub system_resources: ResourceMetrics,
    pub errors: ErrorMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FheMetrics {
    pub active_sessions: u64,
    pub cached_ciphertexts: u64,
    pub key_generations: u64,
    pub encryption_time_avg_ms: f64,
    pub decryption_time_avg_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_mb: f64,
    pub network_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub error_rate_per_minute: f64,
    pub error_types: HashMap<String, u64>,
    pub last_error: Option<String>,
    pub last_error_timestamp: Option<u64>,
}

/// Health check and monitoring service
#[derive(Debug)]
pub struct MonitoringService {
    start_time: Instant,
    version: String,
    components: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    error_tracker: Arc<RwLock<Vec<ErrorEvent>>>,
}

#[derive(Debug, Clone)]
struct ErrorEvent {
    error_type: String,
    message: String,
    timestamp: Instant,
}

impl MonitoringService {
    pub fn new(version: String) -> Self {
        Self {
            start_time: Instant::now(),
            version,
            components: Arc::new(RwLock::new(HashMap::new())),
            error_tracker: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Perform comprehensive health check
    pub async fn health_check(&self) -> HealthStatus {
        let mut components = HashMap::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check FHE engine health
        components.insert("fhe_engine".to_string(), self.check_fhe_engine().await);
        
        // Check database/cache health (simulated)
        components.insert("cache".to_string(), self.check_cache().await);
        
        // Check network connectivity
        components.insert("network".to_string(), self.check_network().await);

        // Determine overall status
        let overall_status = if components.values().all(|c| c.status == "healthy") {
            "healthy"
        } else if components.values().any(|c| c.status == "critical") {
            "critical"
        } else {
            "degraded"
        };

        HealthStatus {
            status: overall_status.to_string(),
            timestamp: now,
            version: self.version.clone(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            components,
        }
    }

    async fn check_fhe_engine(&self) -> ComponentHealth {
        let start = Instant::now();
        
        // Simulate FHE engine health check
        let status = match self.simulate_fhe_check().await {
            Ok(_) => "healthy",
            Err(e) => {
                log::error!("FHE engine health check failed: {}", e);
                "unhealthy"
            }
        };

        ComponentHealth {
            status: status.to_string(),
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
            error_message: if status == "unhealthy" { 
                Some("FHE operations failing".to_string()) 
            } else { 
                None 
            },
        }
    }

    async fn check_cache(&self) -> ComponentHealth {
        ComponentHealth {
            status: "healthy".to_string(),
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: Some(1),
            error_message: None,
        }
    }

    async fn check_network(&self) -> ComponentHealth {
        ComponentHealth {
            status: "healthy".to_string(),
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: Some(5),
            error_message: None,
        }
    }

    async fn simulate_fhe_check(&self) -> Result<()> {
        // Simulate basic FHE operations
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    /// Record an error for tracking
    pub async fn record_error(&self, error_type: String, message: String) {
        let mut errors = self.error_tracker.write().await;
        errors.push(ErrorEvent {
            error_type,
            message,
            timestamp: Instant::now(),
        });

        // Keep only errors from the last hour
        let one_hour_ago = Instant::now() - Duration::from_secs(3600);
        errors.retain(|e| e.timestamp > one_hour_ago);
    }

    /// Get detailed system metrics
    pub async fn get_metrics(&self, 
        requests_metrics: MetricsSnapshot,
        fhe_engine: &Arc<RwLock<FheEngine>>
    ) -> SystemMetrics {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let errors = self.error_tracker.read().await;
        
        // Calculate error metrics
        let recent_errors: Vec<_> = errors.iter()
            .filter(|e| e.timestamp > Instant::now() - Duration::from_secs(60))
            .collect();
        
        let mut error_types = HashMap::new();
        for error in &recent_errors {
            *error_types.entry(error.error_type.clone()).or_insert(0) += 1;
        }

        let last_error = errors.last().map(|e| e.message.clone());
        let last_error_timestamp = errors.last()
            .map(|e| e.timestamp.elapsed().as_secs());

        SystemMetrics {
            timestamp: now,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            requests: requests_metrics,
            fhe_operations: self.get_fhe_metrics(fhe_engine).await,
            system_resources: self.get_resource_metrics().await,
            errors: ErrorMetrics {
                error_rate_per_minute: recent_errors.len() as f64,
                error_types,
                last_error,
                last_error_timestamp,
            },
        }
    }

    async fn get_fhe_metrics(&self, _fhe_engine: &Arc<RwLock<FheEngine>>) -> FheMetrics {
        FheMetrics {
            active_sessions: 10, // Simulated
            cached_ciphertexts: 25, // Simulated
            key_generations: 5, // Simulated
            encryption_time_avg_ms: 45.2,
            decryption_time_avg_ms: 38.7,
        }
    }

    async fn get_resource_metrics(&self) -> ResourceMetrics {
        ResourceMetrics {
            memory_usage_mb: 256.5, // Simulated
            cpu_usage_percent: 15.3,
            disk_usage_mb: 1024.0,
            network_connections: 12,
        }
    }

    /// Get readiness status (for Kubernetes readiness probes)
    pub async fn readiness_check(&self) -> bool {
        let health = self.health_check().await;
        health.status == "healthy" || health.status == "degraded"
    }

    /// Get liveness status (for Kubernetes liveness probes)
    pub async fn liveness_check(&self) -> bool {
        // Basic liveness check - server is running
        true
    }
}

/// Performance profiler for critical operations
#[derive(Debug)]
pub struct PerformanceProfiler {
    operations: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            operations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start timing an operation
    pub fn start_timer(&self, operation: &str) -> OperationTimer {
        OperationTimer {
            operation: operation.to_string(),
            start_time: Instant::now(),
            profiler: self.operations.clone(),
        }
    }

    /// Get performance statistics for an operation
    pub async fn get_stats(&self, operation: &str) -> Option<PerformanceStats> {
        let operations = self.operations.read().await;
        let timings = operations.get(operation)?;

        if timings.is_empty() {
            return None;
        }

        let total: Duration = timings.iter().sum();
        let avg = total / timings.len() as u32;
        let min = *timings.iter().min()?;
        let max = *timings.iter().max()?;

        // Calculate percentiles
        let mut sorted = timings.clone();
        sorted.sort();
        let p50 = sorted[sorted.len() / 2];
        let p95 = sorted[(sorted.len() as f32 * 0.95) as usize];
        let p99 = sorted[(sorted.len() as f32 * 0.99) as usize];

        Some(PerformanceStats {
            operation: operation.to_string(),
            total_calls: timings.len(),
            avg_duration: avg,
            min_duration: min,
            max_duration: max,
            p50_duration: p50,
            p95_duration: p95,
            p99_duration: p99,
        })
    }

    /// Get all performance statistics
    pub async fn get_all_stats(&self) -> HashMap<String, PerformanceStats> {
        let operations = self.operations.read().await;
        let mut stats = HashMap::new();

        for operation in operations.keys() {
            if let Some(stat) = self.get_stats(operation).await {
                stats.insert(operation.clone(), stat);
            }
        }

        stats
    }
}

#[derive(Debug)]
pub struct OperationTimer {
    operation: String,
    start_time: Instant,
    profiler: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        let operation = self.operation.clone();
        let profiler = self.profiler.clone();
        
        tokio::spawn(async move {
            let mut operations = profiler.write().await;
            let timings = operations.entry(operation.clone()).or_insert_with(Vec::new);
            timings.push(duration);
            
            // Keep only the last 1000 measurements
            if timings.len() > 1000 {
                timings.remove(0);
            }
        });
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceStats {
    pub operation: String,
    pub total_calls: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub p50_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
}

/// Structured logging helper
pub struct StructuredLogger;

impl StructuredLogger {
    pub fn log_request(method: &str, path: &str, status: u16, duration: Duration, client_ip: &str) {
        log::info!(
            target: "http_requests",
            "method={} path={} status={} duration_ms={} client_ip={}",
            method, path, status, duration.as_millis(), client_ip
        );
    }

    pub fn log_fhe_operation(operation: &str, client_id: Uuid, duration: Duration, success: bool) {
        if success {
            log::info!(
                target: "fhe_operations",
                "operation={} client_id={} duration_ms={} success={}",
                operation, client_id, duration.as_millis(), success
            );
        } else {
            log::warn!(
                target: "fhe_operations",
                "operation={} client_id={} duration_ms={} success={}",
                operation, client_id, duration.as_millis(), success
            );
        }
    }

    pub fn log_security_event(event_type: &str, client_ip: &str, details: &str) {
        log::warn!(
            target: "security_events",
            "event_type={} client_ip={} details={}",
            event_type, client_ip, details
        );
    }

    pub fn log_error(error: &Error, context: &str) {
        log::error!(
            target: "application_errors",
            "error_type={:?} context={} message={}",
            std::mem::discriminant(error),
            context,
            error
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_monitoring_service() {
        let service = MonitoringService::new("test-1.0.0".to_string());
        
        // Test health check
        let health = service.health_check().await;
        assert_eq!(health.version, "test-1.0.0");
        assert!(!health.components.is_empty());

        // Test error recording
        service.record_error("TestError".to_string(), "Test message".to_string()).await;
        
        let errors = service.error_tracker.read().await;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, "TestError");
    }

    #[tokio::test]
    async fn test_performance_profiler() {
        let profiler = PerformanceProfiler::new();
        
        // Simulate some operations
        for _ in 0..10 {
            let _timer = profiler.start_timer("test_operation");
            sleep(Duration::from_millis(1)).await;
        }

        // Wait for async operations to complete
        sleep(Duration::from_millis(10)).await;

        let stats = profiler.get_stats("test_operation").await;
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.total_calls, 10);
        assert!(stats.avg_duration > Duration::from_millis(0));
    }
}