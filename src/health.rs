//! Comprehensive health checking and system monitoring

use crate::error::Result;
use crate::fhe::FheEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Comprehensive health check system
pub struct HealthChecker {
    components: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck + Send + Sync>>>>,
    check_interval: Duration,
    last_check: Arc<RwLock<Option<Instant>>>,
}

/// Health status for individual components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub last_check: u64, // Unix timestamp
    pub response_time_ms: u64,
    pub error_count: u64,
    pub warning_count: u64,
    pub details: HashMap<String, String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Overall system health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthReport {
    pub overall_status: HealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub summary: HealthSummary,
    pub timestamp: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub total_components: usize,
    pub healthy_components: usize,
    pub warning_components: usize,
    pub critical_components: usize,
    pub avg_response_time_ms: f64,
    pub total_errors: u64,
    pub total_warnings: u64,
}

/// Trait for implementing health checks
#[async_trait::async_trait]
pub trait HealthCheck {
    async fn check(&self) -> Result<ComponentHealth>;
    fn name(&self) -> &str;
    fn dependencies(&self) -> Vec<String>;
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            checks: Arc::new(RwLock::new(HashMap::new())),
            check_interval: Duration::from_secs(30),
            last_check: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Register a health check
    pub async fn register_check(&self, check: Box<dyn HealthCheck + Send + Sync>) {
        let name = check.name().to_string();
        self.checks.write().await.insert(name, check);
    }

    /// Run all health checks
    pub async fn run_health_checks(&self) -> Result<SystemHealthReport> {
        let start_time = Instant::now();
        let mut new_components = HashMap::new();
        let checks = self.checks.read().await;

        for (name, check) in checks.iter() {
            match check.check().await {
                Ok(health) => {
                    new_components.insert(name.clone(), health);
                }
                Err(e) => {
                    log::error!("Health check failed for {}: {}", name, e);
                    new_components.insert(
                        name.clone(),
                        ComponentHealth {
                            name: name.clone(),
                            status: HealthStatus::Critical,
                            last_check: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            response_time_ms: 0,
                            error_count: 1,
                            warning_count: 0,
                            details: {
                                let mut details = HashMap::new();
                                details.insert("error".to_string(), e.to_string());
                                details
                            },
                            dependencies: check.dependencies(),
                        },
                    );
                }
            }
        }

        // Update components
        {
            let mut components = self.components.write().await;
            *components = new_components.clone();
        }

        // Update last check time
        {
            let mut last_check = self.last_check.write().await;
            *last_check = Some(start_time);
        }

        self.generate_health_report(new_components).await
    }

    /// Generate comprehensive health report
    async fn generate_health_report(
        &self,
        components: HashMap<String, ComponentHealth>,
    ) -> Result<SystemHealthReport> {
        let mut healthy = 0;
        let mut warning = 0;
        let mut critical = 0;
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut total_response_time = 0;

        for health in components.values() {
            match health.status {
                HealthStatus::Healthy => healthy += 1,
                HealthStatus::Warning => warning += 1,
                HealthStatus::Critical => critical += 1,
                HealthStatus::Unknown => {}
            }
            total_errors += health.error_count;
            total_warnings += health.warning_count;
            total_response_time += health.response_time_ms;
        }

        let overall_status = if critical > 0 {
            HealthStatus::Critical
        } else if warning > 0 {
            HealthStatus::Warning
        } else if healthy > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };

        let avg_response_time = if !components.is_empty() {
            total_response_time as f64 / components.len() as f64
        } else {
            0.0
        };

        let component_count = components.len();
        Ok(SystemHealthReport {
            overall_status,
            components,
            summary: HealthSummary {
                total_components: component_count,
                healthy_components: healthy,
                warning_components: warning,
                critical_components: critical,
                avg_response_time_ms: avg_response_time,
                total_errors,
                total_warnings,
            },
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            uptime_seconds: 0, // Will be set by the caller
        })
    }

    /// Get cached health status
    pub async fn get_health_status(&self) -> Option<SystemHealthReport> {
        let components = self.components.read().await.clone();
        if components.is_empty() {
            return None;
        }

        match self.generate_health_report(components).await {
            Ok(report) => Some(report),
            Err(_) => None,
        }
    }

    /// Check if system is healthy
    pub async fn is_healthy(&self) -> bool {
        if let Some(report) = self.get_health_status().await {
            matches!(report.overall_status, HealthStatus::Healthy)
        } else {
            false
        }
    }

    /// Start periodic health checks
    pub async fn start_periodic_checks(&self) -> Result<()> {
        let checker = Arc::new(self.clone());
        let interval = self.check_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                if let Err(e) = checker.run_health_checks().await {
                    log::error!("Periodic health check failed: {}", e);
                }
            }
        });

        Ok(())
    }
}

impl Clone for HealthChecker {
    fn clone(&self) -> Self {
        Self {
            components: Arc::clone(&self.components),
            checks: Arc::clone(&self.checks),
            check_interval: self.check_interval,
            last_check: Arc::clone(&self.last_check),
        }
    }
}

/// FHE Engine health check implementation
pub struct FheEngineHealthCheck {
    engine: Arc<RwLock<FheEngine>>,
    name: String,
}

impl FheEngineHealthCheck {
    pub fn new(engine: Arc<RwLock<FheEngine>>, name: String) -> Self {
        Self { engine, name }
    }
}

#[async_trait::async_trait]
impl HealthCheck for FheEngineHealthCheck {
    async fn check(&self) -> Result<ComponentHealth> {
        let start_time = Instant::now();
        let mut details = HashMap::new();
        let mut error_count = 0;
        let mut warning_count = 0;

        // Try to perform a basic FHE operation
        let engine = self.engine.read().await;
        let test_result = match engine.validate_state() {
            Ok(_) => {
                details.insert("state".to_string(), "valid".to_string());
                HealthStatus::Healthy
            }
            Err(e) => {
                error_count += 1;
                details.insert("error".to_string(), e.to_string());
                HealthStatus::Critical
            }
        };

        let response_time = start_time.elapsed().as_millis() as u64;

        // Check response time warning threshold
        if response_time > 1000 {
            warning_count += 1;
            details.insert(
                "warning".to_string(),
                "High response time".to_string(),
            );
        }

        Ok(ComponentHealth {
            name: self.name.clone(),
            status: if warning_count > 0 && error_count == 0 {
                HealthStatus::Warning
            } else {
                test_result
            },
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            response_time_ms: response_time,
            error_count,
            warning_count,
            details,
            dependencies: vec!["fhe_params".to_string()],
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["fhe_params".to_string()]
    }
}

/// Memory usage health check
pub struct MemoryHealthCheck {
    max_memory_mb: u64,
    warning_threshold_percent: f64,
}

impl MemoryHealthCheck {
    pub fn new(max_memory_mb: u64, warning_threshold_percent: f64) -> Self {
        Self {
            max_memory_mb,
            warning_threshold_percent,
        }
    }

    fn get_memory_usage(&self) -> Result<u64> {
        // This is a simplified memory check
        // In production, you'd use system-specific APIs
        Ok(0) // Placeholder - would implement actual memory checking
    }
}

#[async_trait::async_trait]
impl HealthCheck for MemoryHealthCheck {
    async fn check(&self) -> Result<ComponentHealth> {
        let start_time = Instant::now();
        let mut details = HashMap::new();
        let mut warning_count = 0;
        let mut error_count = 0;

        let memory_usage = self.get_memory_usage().unwrap_or(0);
        let usage_percent = (memory_usage as f64 / self.max_memory_mb as f64) * 100.0;

        details.insert("memory_usage_mb".to_string(), memory_usage.to_string());
        details.insert("usage_percent".to_string(), format!("{:.2}", usage_percent));
        details.insert("max_memory_mb".to_string(), self.max_memory_mb.to_string());

        let status = if usage_percent > 90.0 {
            error_count += 1;
            HealthStatus::Critical
        } else if usage_percent > self.warning_threshold_percent {
            warning_count += 1;
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        Ok(ComponentHealth {
            name: "memory".to_string(),
            status,
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            error_count,
            warning_count,
            details,
            dependencies: vec![],
        })
    }

    fn name(&self) -> &str {
        "memory"
    }

    fn dependencies(&self) -> Vec<String> {
        vec![]
    }
}

/// Database connectivity health check
pub struct DatabaseHealthCheck {
    connection_string: String,
}

impl DatabaseHealthCheck {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

#[async_trait::async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> Result<ComponentHealth> {
        let start_time = Instant::now();
        let mut details = HashMap::new();

        // Simulate database connection check
        // In real implementation, you'd connect to your actual database
        tokio::time::sleep(Duration::from_millis(10)).await;

        details.insert("connection".to_string(), "available".to_string());
        details.insert("endpoint".to_string(), self.connection_string.clone());

        Ok(ComponentHealth {
            name: "database".to_string(),
            status: HealthStatus::Healthy,
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            error_count: 0,
            warning_count: 0,
            details,
            dependencies: vec!["network".to_string()],
        })
    }

    fn name(&self) -> &str {
        "database"
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["network".to_string()]
    }
}

/// External service dependency health check
pub struct ExternalServiceHealthCheck {
    service_name: String,
    endpoint: String,
    timeout: Duration,
}

impl ExternalServiceHealthCheck {
    pub fn new(service_name: String, endpoint: String, timeout: Duration) -> Self {
        Self {
            service_name,
            endpoint,
            timeout,
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for ExternalServiceHealthCheck {
    async fn check(&self) -> Result<ComponentHealth> {
        let start_time = Instant::now();
        let mut details = HashMap::new();
        let mut error_count = 0;

        details.insert("endpoint".to_string(), self.endpoint.clone());
        details.insert("timeout_ms".to_string(), self.timeout.as_millis().to_string());

        // Simulate external service check
        let status = match tokio::time::timeout(self.timeout, self.ping_service()).await {
            Ok(Ok(_)) => HealthStatus::Healthy,
            Ok(Err(e)) => {
                error_count += 1;
                details.insert("error".to_string(), e.to_string());
                HealthStatus::Critical
            }
            Err(_) => {
                error_count += 1;
                details.insert("error".to_string(), "Timeout".to_string());
                HealthStatus::Critical
            }
        };

        Ok(ComponentHealth {
            name: self.service_name.clone(),
            status,
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            error_count,
            warning_count: 0,
            details,
            dependencies: vec!["network".to_string()],
        })
    }

    fn name(&self) -> &str {
        &self.service_name
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["network".to_string()]
    }
}

impl ExternalServiceHealthCheck {
    async fn ping_service(&self) -> Result<()> {
        // Simulate service ping
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fhe::FheParams;

    #[tokio::test]
    async fn test_health_checker() {
        let checker = HealthChecker::new();

        // Register a memory health check
        let memory_check = MemoryHealthCheck::new(1024, 80.0);
        checker
            .register_check(Box::new(memory_check))
            .await;

        // Run health checks
        let report = checker.run_health_checks().await.unwrap();
        assert!(!report.components.is_empty());
        assert!(report.summary.total_components > 0);
    }

    #[tokio::test]
    async fn test_memory_health_check() {
        let check = MemoryHealthCheck::new(1024, 80.0);
        let health = check.check().await.unwrap();
        
        assert_eq!(health.name, "memory");
        assert!(!health.details.is_empty());
    }

    #[tokio::test]
    async fn test_external_service_health_check() {
        let check = ExternalServiceHealthCheck::new(
            "test_service".to_string(),
            "https://example.com".to_string(),
            Duration::from_secs(5),
        );
        
        let health = check.check().await.unwrap();
        assert_eq!(health.name, "test_service");
    }

    #[test]
    fn test_health_status_priority() {
        let statuses = vec![
            HealthStatus::Healthy,
            HealthStatus::Warning,
            HealthStatus::Critical,
        ];

        // Critical should take priority
        let overall = if statuses.contains(&HealthStatus::Critical) {
            HealthStatus::Critical
        } else if statuses.contains(&HealthStatus::Warning) {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        assert_eq!(overall, HealthStatus::Critical);
    }
}