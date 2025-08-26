//! Enhanced Logging and Observability for FHE LLM Proxy

use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, span, warn, Level};
use uuid::Uuid;

/// Structured logging context for FHE operations
#[derive(Debug, Clone)]
pub struct LogContext {
    pub request_id: Uuid,
    pub client_id: Option<String>,
    pub operation: String,
    pub start_time: Instant,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl LogContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            client_id: None,
            operation: operation.into(),
            start_time: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    pub fn add_metadata<K, V>(mut self, key: K, value: V) -> Self 
    where
        K: Into<String>,
        V: serde::Serialize,
    {
        if let Ok(val) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), val);
        }
        self
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn log_completion(&self, result: &Result<(), crate::error::Error>) {
        let duration = self.elapsed();
        let duration_ms = duration.as_millis();

        match result {
            Ok(_) => {
                info!(
                    request_id = %self.request_id,
                    client_id = ?self.client_id,
                    operation = %self.operation,
                    duration_ms = %duration_ms,
                    status = "success",
                    metadata = ?self.metadata,
                    "Operation completed successfully"
                );
            }
            Err(e) => {
                error!(
                    request_id = %self.request_id,
                    client_id = ?self.client_id,
                    operation = %self.operation,
                    duration_ms = %duration_ms,
                    status = "error",
                    error = %e,
                    metadata = ?self.metadata,
                    "Operation failed"
                );
            }
        }
    }
}

/// Performance metrics collector for FHE operations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub duration: Duration,
    pub memory_used: usize,
    pub success: bool,
    pub timestamp: u64,
}

impl PerformanceMetrics {
    pub fn new(operation: impl Into<String>, duration: Duration, success: bool) -> Self {
        Self {
            operation: operation.into(),
            duration,
            memory_used: 0, // TODO: Implement actual memory tracking
            success,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "operation": self.operation,
            "duration_ms": self.duration.as_millis(),
            "memory_used_bytes": self.memory_used,
            "success": self.success,
            "timestamp": self.timestamp
        })
    }
}

/// Centralized metrics collector
#[derive(Debug)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<Vec<PerformanceMetrics>>>,
    max_metrics: usize,
}

impl MetricsCollector {
    pub fn new(max_metrics: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            max_metrics,
        }
    }

    pub async fn record(&self, metric: PerformanceMetrics) {
        let mut metrics = self.metrics.write().await;
        
        // Keep only the most recent metrics to prevent memory bloat
        if metrics.len() >= self.max_metrics {
            metrics.remove(0);
        }
        
        let operation = metric.operation.clone();
        let duration = metric.duration;
        let success = metric.success;
        
        metrics.push(metric);

        debug!(
            operation = %operation,
            duration_ms = %duration.as_millis(),
            success = %success,
            "Recorded performance metric"
        );
    }

    pub async fn get_summary(&self) -> MetricsSummary {
        let metrics = self.metrics.read().await;
        let total_operations = metrics.len();

        if total_operations == 0 {
            return MetricsSummary::default();
        }

        let successful_operations = metrics.iter().filter(|m| m.success).count();
        let total_duration: Duration = metrics.iter().map(|m| m.duration).sum();
        let avg_duration = total_duration / total_operations as u32;

        let mut operation_counts: HashMap<String, usize> = HashMap::new();
        for metric in metrics.iter() {
            *operation_counts.entry(metric.operation.clone()).or_insert(0) += 1;
        }

        MetricsSummary {
            total_operations,
            successful_operations,
            success_rate: successful_operations as f64 / total_operations as f64,
            average_duration: avg_duration,
            operation_counts,
        }
    }

    pub async fn export_prometheus(&self) -> String {
        let summary = self.get_summary().await;
        let mut output = String::new();

        // Total operations counter
        output.push_str(&format!(
            "# HELP fhe_operations_total Total number of FHE operations\n# TYPE fhe_operations_total counter\nfhe_operations_total {}\n\n",
            summary.total_operations
        ));

        // Success rate gauge
        output.push_str(&format!(
            "# HELP fhe_success_rate Success rate of FHE operations\n# TYPE fhe_success_rate gauge\nfhe_success_rate {:.4}\n\n",
            summary.success_rate
        ));

        // Average duration gauge
        output.push_str(&format!(
            "# HELP fhe_avg_duration_ms Average duration of FHE operations in milliseconds\n# TYPE fhe_avg_duration_ms gauge\nfhe_avg_duration_ms {}\n\n",
            summary.average_duration.as_millis()
        ));

        // Operation counts by type
        for (operation, count) in &summary.operation_counts {
            output.push_str(&format!(
                "fhe_operations_by_type{{operation=\"{}\"}} {}\n",
                operation, count
            ));
        }

        output
    }
}

/// Summary of collected metrics
#[derive(Debug, Clone, Default)]
pub struct MetricsSummary {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub success_rate: f64,
    pub average_duration: Duration,
    pub operation_counts: HashMap<String, usize>,
}

impl std::fmt::Display for MetricsSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "📊 FHE Performance Metrics Summary")?;
        writeln!(f, "=================================")?;
        writeln!(f, "Total Operations: {}", self.total_operations)?;
        writeln!(f, "Successful Operations: {}", self.successful_operations)?;
        writeln!(f, "Success Rate: {:.2}%", self.success_rate * 100.0)?;
        writeln!(f, "Average Duration: {:.2}ms", self.average_duration.as_millis())?;
        
        if !self.operation_counts.is_empty() {
            writeln!(f, "\nOperations by Type:")?;
            for (operation, count) in &self.operation_counts {
                writeln!(f, "  • {}: {}", operation, count)?;
            }
        }

        Ok(())
    }
}

/// Security event logger for audit trails
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub client_ip: Option<String>,
    pub user_id: Option<String>,
    pub request_id: Uuid,
    pub message: String,
    pub timestamp: u64,
    pub severity: SecuritySeverity,
}

#[derive(Debug, Clone)]
pub enum SecurityEventType {
    AuthenticationFailure,
    RateLimitExceeded,
    InvalidInput,
    SuspiciousActivity,
    PrivacyBudgetExceeded,
    UnauthorizedAccess,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SecuritySeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl SecurityEvent {
    pub fn new(
        event_type: SecurityEventType,
        message: impl Into<String>,
        severity: SecuritySeverity,
    ) -> Self {
        Self {
            event_type,
            client_ip: None,
            user_id: None,
            request_id: Uuid::new_v4(),
            message: message.into(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            severity,
        }
    }

    pub fn with_client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn log(&self) {
        let _log_level = match self.severity {
            SecuritySeverity::Info => Level::INFO,
            SecuritySeverity::Warning => Level::WARN,
            SecuritySeverity::Error => Level::ERROR,
            SecuritySeverity::Critical => Level::ERROR,
        };

        let span = span!(
            Level::ERROR,
            "security_event",
            event_type = ?self.event_type,
            severity = ?self.severity,
            request_id = %self.request_id,
            client_ip = ?self.client_ip,
            user_id = ?self.user_id,
        );

        let _guard = span.enter();

        match self.severity {
            SecuritySeverity::Info => info!("{}", self.message),
            SecuritySeverity::Warning => warn!("{}", self.message),
            SecuritySeverity::Error | SecuritySeverity::Critical => error!("{}", self.message),
        }
    }

    pub fn to_audit_log(&self) -> String {
        json!({
            "event_type": format!("{:?}", self.event_type),
            "severity": format!("{:?}", self.severity),
            "message": self.message,
            "client_ip": self.client_ip,
            "user_id": self.user_id,
            "request_id": self.request_id,
            "timestamp": self.timestamp
        }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_log_context_creation() {
        let ctx = LogContext::new("test_operation")
            .with_client_id("client123")
            .add_metadata("test_key", "test_value");

        assert_eq!(ctx.operation, "test_operation");
        assert_eq!(ctx.client_id, Some("client123".to_string()));
        assert!(ctx.metadata.contains_key("test_key"));
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new(10);
        
        let metric1 = PerformanceMetrics::new("encrypt", Duration::from_millis(100), true);
        let metric2 = PerformanceMetrics::new("decrypt", Duration::from_millis(150), false);

        collector.record(metric1).await;
        collector.record(metric2).await;

        let summary = collector.get_summary().await;
        assert_eq!(summary.total_operations, 2);
        assert_eq!(summary.successful_operations, 1);
        assert_eq!(summary.success_rate, 0.5);
        assert!(summary.operation_counts.contains_key("encrypt"));
        assert!(summary.operation_counts.contains_key("decrypt"));
    }

    #[test]
    fn test_security_event_creation() {
        let event = SecurityEvent::new(
            SecurityEventType::AuthenticationFailure,
            "Invalid API key provided",
            SecuritySeverity::Warning,
        )
        .with_client_ip("192.168.1.100")
        .with_user_id("user123");

        assert_eq!(event.severity, SecuritySeverity::Warning);
        assert_eq!(event.client_ip, Some("192.168.1.100".to_string()));
        assert_eq!(event.user_id, Some("user123".to_string()));
        assert_eq!(event.message, "Invalid API key provided");
    }

    #[tokio::test]
    async fn test_prometheus_export() {
        let collector = MetricsCollector::new(10);
        
        collector.record(PerformanceMetrics::new("encrypt", Duration::from_millis(100), true)).await;
        collector.record(PerformanceMetrics::new("decrypt", Duration::from_millis(150), true)).await;

        let prometheus = collector.export_prometheus().await;
        assert!(prometheus.contains("fhe_operations_total"));
        assert!(prometheus.contains("fhe_success_rate"));
        assert!(prometheus.contains("fhe_avg_duration_ms"));
    }
}