//! Advanced Observability and Monitoring System
//!
//! Provides comprehensive telemetry, distributed tracing, metrics collection,
//! and real-time monitoring capabilities with AI-powered anomaly detection.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, span, warn, Level};
use uuid::Uuid;

/// Comprehensive observability orchestrator
#[derive(Debug)]
pub struct ObservabilitySystem {
    metrics_collector: Arc<AdvancedMetricsCollector>,
    trace_processor: Arc<DistributedTraceProcessor>,
    log_aggregator: Arc<StructuredLogAggregator>,
    alerting_engine: Arc<IntelligentAlertingEngine>,
    anomaly_detector: Arc<AiAnomalyDetector>,
    dashboard_engine: Arc<RealTimeDashboard>,
    performance_profiler: Arc<ContinuousProfiler>,
    sli_calculator: Arc<SliCalculator>,
    config: ObservabilityConfig,
}

/// Advanced metrics collection with custom dimensions
#[derive(Debug)]
pub struct AdvancedMetricsCollector {
    counter_metrics: Arc<RwLock<HashMap<String, CounterMetric>>>,
    gauge_metrics: Arc<RwLock<HashMap<String, GaugeMetric>>>,
    histogram_metrics: Arc<RwLock<HashMap<String, HistogramMetric>>>,
    summary_metrics: Arc<RwLock<HashMap<String, SummaryMetric>>>,
    custom_metrics: Arc<RwLock<HashMap<String, CustomMetric>>>,
    metric_registry: Arc<MetricRegistry>,
    export_manager: Arc<MetricExportManager>,
}

/// Distributed tracing with correlation and sampling
#[derive(Debug)]
pub struct DistributedTraceProcessor {
    active_spans: Arc<RwLock<HashMap<Uuid, ActiveSpan>>>,
    completed_traces: Arc<RwLock<VecDeque<CompletedTrace>>>,
    sampling_strategy: Arc<AdaptiveSamplingStrategy>,
    trace_correlator: Arc<TraceCorrelator>,
    baggage_processor: Arc<BaggageProcessor>,
    export_pipeline: Arc<TraceExportPipeline>,
}

/// Structured logging with correlation and enrichment
#[derive(Debug)]
pub struct StructuredLogAggregator {
    log_buffer: Arc<RwLock<VecDeque<LogEntry>>>,
    log_processors: Arc<RwLock<Vec<LogProcessor>>>,
    correlation_engine: Arc<LogCorrelationEngine>,
    search_index: Arc<LogSearchIndex>,
    retention_manager: Arc<LogRetentionManager>,
}

/// AI-powered intelligent alerting system
#[derive(Debug)]
pub struct IntelligentAlertingEngine {
    alert_rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    notification_channels: Arc<RwLock<HashMap<String, NotificationChannel>>>,
    escalation_policies: Arc<RwLock<HashMap<String, EscalationPolicy>>>,
    alert_correlator: Arc<AlertCorrelator>,
    suppression_manager: Arc<AlertSuppressionManager>,
    ai_predictor: Arc<AlertPredictor>,
}

/// Machine learning anomaly detection
#[derive(Debug)]
pub struct AiAnomalyDetector {
    detection_models: Arc<RwLock<HashMap<String, AnomalyModel>>>,
    feature_extractors: Arc<RwLock<HashMap<String, FeatureExtractor>>>,
    training_pipeline: Arc<ModelTrainingPipeline>,
    prediction_engine: Arc<AnomalyPredictionEngine>,
    feedback_loop: Arc<ModelFeedbackLoop>,
}

/// Real-time dashboard and visualization
#[derive(Debug)]
pub struct RealTimeDashboard {
    widget_registry: Arc<RwLock<HashMap<String, DashboardWidget>>>,
    data_sources: Arc<RwLock<HashMap<String, DataSource>>>,
    real_time_updates: Arc<RwLock<HashMap<String, broadcast::Sender<DashboardUpdate>>>>,
    layout_engine: Arc<AdaptiveLayoutEngine>,
    visualization_engine: Arc<VisualizationEngine>,
}

/// Continuous performance profiling
#[derive(Debug)]
pub struct ContinuousProfiler {
    profiling_sessions: Arc<RwLock<HashMap<String, ProfilingSession>>>,
    cpu_profiler: Arc<CpuProfiler>,
    memory_profiler: Arc<MemoryProfiler>,
    io_profiler: Arc<IoProfiler>,
    flame_graph_generator: Arc<FlameGraphGenerator>,
    performance_advisor: Arc<PerformanceAdvisor>,
}

/// Service Level Indicator calculator
#[derive(Debug)]
pub struct SliCalculator {
    sli_definitions: Arc<RwLock<HashMap<String, SliDefinition>>>,
    slo_targets: Arc<RwLock<HashMap<String, SloTarget>>>,
    error_budget_tracker: Arc<ErrorBudgetTracker>,
    burn_rate_calculator: Arc<BurnRateCalculator>,
    compliance_monitor: Arc<ComplianceMonitor>,
}

// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub metrics_collection_interval: Duration,
    pub trace_sampling_rate: f64,
    pub log_retention_days: u32,
    pub alerting_enabled: bool,
    pub anomaly_detection_enabled: bool,
    pub profiling_enabled: bool,
    pub dashboard_refresh_rate: Duration,
    pub export_endpoints: ExportEndpoints,
    pub storage_config: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEndpoints {
    pub prometheus_endpoint: Option<String>,
    pub jaeger_endpoint: Option<String>,
    pub elasticsearch_endpoint: Option<String>,
    pub grafana_endpoint: Option<String>,
    pub otel_collector_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub metrics_retention_days: u32,
    pub traces_retention_days: u32,
    pub logs_retention_days: u32,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

// Metric types
#[derive(Debug, Clone)]
pub struct CounterMetric {
    pub name: String,
    pub value: Arc<AtomicU64>,
    pub labels: HashMap<String, String>,
    pub description: String,
    pub created_at: Instant,
}

#[derive(Debug, Clone)]
pub struct GaugeMetric {
    pub name: String,
    pub value: Arc<AtomicU64>,
    pub labels: HashMap<String, String>,
    pub description: String,
    pub last_updated: Arc<Mutex<Instant>>,
}

#[derive(Debug, Clone)]
pub struct HistogramMetric {
    pub name: String,
    pub buckets: Arc<RwLock<Vec<HistogramBucket>>>,
    pub labels: HashMap<String, String>,
    pub description: String,
    pub total_count: Arc<AtomicU64>,
    pub total_sum: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
pub struct SummaryMetric {
    pub name: String,
    pub quantiles: Arc<RwLock<HashMap<f64, f64>>>,
    pub labels: HashMap<String, String>,
    pub description: String,
    pub count: Arc<AtomicU64>,
    pub sum: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
pub struct CustomMetric {
    pub name: String,
    pub metric_type: CustomMetricType,
    pub value: Arc<RwLock<serde_json::Value>>,
    pub labels: HashMap<String, String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum CustomMetricType {
    Business,
    Technical,
    Security,
    Compliance,
    UserExperience,
}

#[derive(Debug, Clone)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: Arc<AtomicU64>,
}

// Trace structures
#[derive(Debug, Clone)]
pub struct ActiveSpan {
    pub span_id: Uuid,
    pub trace_id: Uuid,
    pub parent_span_id: Option<Uuid>,
    pub operation_name: String,
    pub start_time: Instant,
    pub tags: HashMap<String, String>,
    pub baggage: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
    pub references: Vec<SpanReference>,
}

#[derive(Debug, Clone)]
pub struct CompletedTrace {
    pub trace_id: Uuid,
    pub spans: Vec<CompletedSpan>,
    pub duration: Duration,
    pub service_map: HashMap<String, ServiceInfo>,
    pub error_analysis: Option<ErrorAnalysis>,
    pub performance_metrics: TracePerformanceMetrics,
}

#[derive(Debug, Clone)]
pub struct CompletedSpan {
    pub span_id: Uuid,
    pub parent_span_id: Option<Uuid>,
    pub operation_name: String,
    pub start_time: Instant,
    pub finish_time: Instant,
    pub duration: Duration,
    pub tags: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
    pub references: Vec<SpanReference>,
    pub status: SpanStatus,
}

#[derive(Debug, Clone)]
pub struct SpanLog {
    pub timestamp: Instant,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct SpanReference {
    pub reference_type: ReferenceType,
    pub span_id: Uuid,
}

#[derive(Debug, Clone)]
pub enum ReferenceType {
    ChildOf,
    FollowsFrom,
}

#[derive(Debug, Clone, Copy)]
pub enum SpanStatus {
    Ok,
    Cancelled,
    Unknown,
    InvalidArgument,
    DeadlineExceeded,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    ResourceExhausted,
    FailedPrecondition,
    Aborted,
    OutOfRange,
    Unimplemented,
    Internal,
    Unavailable,
    DataLoss,
    Unauthenticated,
}

// Log structures
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub id: Uuid,
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub message: String,
    pub service: String,
    pub trace_id: Option<Uuid>,
    pub span_id: Option<Uuid>,
    pub fields: HashMap<String, serde_json::Value>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

// Alert structures
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub evaluation_interval: Duration,
    pub notification_channels: Vec<String>,
    pub escalation_policy: Option<String>,
    pub suppression_rules: Vec<SuppressionRule>,
}

#[derive(Debug, Clone)]
pub enum AlertCondition {
    ThresholdCondition {
        metric: String,
        operator: ComparisonOperator,
        threshold: f64,
        duration: Duration,
    },
    RateOfChangeCondition {
        metric: String,
        rate_threshold: f64,
        time_window: Duration,
    },
    AnomalyCondition {
        metric: String,
        sensitivity: f64,
        model_type: AnomalyModelType,
    },
    CompositeCondition {
        conditions: Vec<AlertCondition>,
        operator: LogicalOperator,
    },
}

#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct SuppressionRule {
    pub condition: SuppressionCondition,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub enum SuppressionCondition {
    TimeWindow { start: u8, end: u8 }, // Hours
    MaintenanceWindow,
    DependentServiceDown(String),
    AlertFlood { threshold: u32, window: Duration },
}

// Anomaly detection structures
#[derive(Debug, Clone)]
pub enum AnomalyModelType {
    IsolationForest,
    OneClassSvm,
    LocalOutlierFactor,
    AutoEncoder,
    StatisticalThreshold,
    SeasonalDecomposition,
}

#[derive(Debug)]
pub struct AnomalyModel {
    pub model_type: AnomalyModelType,
    pub model_data: Arc<RwLock<serde_json::Value>>,
    pub training_data: Arc<RwLock<VecDeque<DataPoint>>>,
    pub performance_metrics: Arc<ModelPerformanceMetrics>,
    pub last_trained: Arc<Mutex<Option<Instant>>>,
    pub prediction_accuracy: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub timestamp: Instant,
    pub features: HashMap<String, f64>,
    pub label: Option<bool>, // True for anomaly, False for normal
}

// Dashboard structures
#[derive(Debug, Clone)]
pub struct DashboardWidget {
    pub id: String,
    pub widget_type: WidgetType,
    pub title: String,
    pub data_source: String,
    pub query: String,
    pub refresh_interval: Duration,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub configuration: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum WidgetType {
    LineChart,
    BarChart,
    PieChart,
    Gauge,
    SingleStat,
    Table,
    Heatmap,
    WorldMap,
    FlameGraph,
    ServiceMap,
}

#[derive(Debug, Clone)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum DashboardUpdate {
    WidgetData {
        widget_id: String,
        data: serde_json::Value,
        timestamp: SystemTime,
    },
    WidgetAlert {
        widget_id: String,
        alert_type: AlertType,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub enum AlertType {
    DataError,
    ThresholdBreach,
    AnomalyDetected,
}

// SLI/SLO structures
#[derive(Debug, Clone)]
pub struct SliDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sli_type: SliType,
    pub good_events_query: String,
    pub valid_events_query: String,
    pub evaluation_interval: Duration,
}

#[derive(Debug, Clone)]
pub enum SliType {
    Availability,
    Latency,
    Throughput,
    ErrorRate,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct SloTarget {
    pub sli_id: String,
    pub target_percentage: f64,
    pub time_window: Duration,
    pub alert_burn_rate: f64,
}

// Performance metrics
#[derive(Debug, Clone)]
pub struct TracePerformanceMetrics {
    pub total_duration: Duration,
    pub service_durations: HashMap<String, Duration>,
    pub database_time: Duration,
    pub external_api_time: Duration,
    pub cpu_intensive_time: Duration,
    pub error_count: u32,
    pub warning_count: u32,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
    pub endpoint_count: u32,
    pub error_rate: f64,
    pub average_latency: Duration,
}

#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    pub error_type: String,
    pub error_message: String,
    pub error_span: Uuid,
    pub root_cause_analysis: Option<String>,
    pub similar_errors_count: u32,
}

#[derive(Debug)]
pub struct ModelPerformanceMetrics {
    pub accuracy: Arc<AtomicU64>,
    pub precision: Arc<AtomicU64>,
    pub recall: Arc<AtomicU64>,
    pub f1_score: Arc<AtomicU64>,
    pub false_positive_rate: Arc<AtomicU64>,
    pub false_negative_rate: Arc<AtomicU64>,
}

// Implementation
impl ObservabilitySystem {
    /// Create new observability system with comprehensive monitoring
    pub async fn new(config: ObservabilityConfig) -> Result<Self> {
        let metrics_collector = Arc::new(AdvancedMetricsCollector::new(&config).await?);
        let trace_processor = Arc::new(DistributedTraceProcessor::new(&config).await?);
        let log_aggregator = Arc::new(StructuredLogAggregator::new(&config).await?);
        let alerting_engine = Arc::new(IntelligentAlertingEngine::new(&config).await?);
        let anomaly_detector = Arc::new(AiAnomalyDetector::new(&config).await?);
        let dashboard_engine = Arc::new(RealTimeDashboard::new(&config).await?);
        let performance_profiler = Arc::new(ContinuousProfiler::new(&config).await?);
        let sli_calculator = Arc::new(SliCalculator::new(&config).await?);

        let system = Self {
            metrics_collector,
            trace_processor,
            log_aggregator,
            alerting_engine,
            anomaly_detector,
            dashboard_engine,
            performance_profiler,
            sli_calculator,
            config,
        };

        system.initialize_default_monitoring().await?;
        system.start_background_processors().await?;

        info!("📊 Advanced observability system initialized");
        Ok(system)
    }

    /// Initialize default monitoring configurations
    async fn initialize_default_monitoring(&self) -> Result<()> {
        // Initialize core FHE metrics
        self.metrics_collector
            .register_counter(
                "fhe_encryption_operations_total",
                "Total number of FHE encryption operations",
                vec!["operation_type", "client_id"],
            )
            .await?;

        self.metrics_collector
            .register_histogram(
                "fhe_operation_duration_seconds",
                "Duration of FHE operations in seconds",
                vec![0.001, 0.01, 0.1, 1.0, 10.0, 30.0, 60.0],
                vec!["operation", "success"],
            )
            .await?;

        self.metrics_collector
            .register_gauge(
                "fhe_active_sessions",
                "Current number of active FHE sessions",
                vec!["session_type"],
            )
            .await?;

        // Initialize system health metrics
        self.metrics_collector
            .register_gauge(
                "system_cpu_usage_percent",
                "Current CPU usage percentage",
                vec!["core"],
            )
            .await?;

        self.metrics_collector
            .register_gauge(
                "system_memory_usage_bytes",
                "Current memory usage in bytes",
                vec!["memory_type"],
            )
            .await?;

        self.metrics_collector
            .register_gauge(
                "gpu_utilization_percent",
                "GPU utilization percentage",
                vec!["gpu_id"],
            )
            .await?;

        // Initialize business metrics
        self.metrics_collector
            .register_counter(
                "user_requests_total",
                "Total number of user requests",
                vec!["endpoint", "method", "status_code"],
            )
            .await?;

        self.metrics_collector
            .register_histogram(
                "request_duration_seconds",
                "Request duration in seconds",
                vec![
                    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                ],
                vec!["endpoint", "method"],
            )
            .await?;

        // Initialize SLIs
        self.sli_calculator
            .register_sli(SliDefinition {
                id: "availability".to_string(),
                name: "Service Availability".to_string(),
                description: "Percentage of successful requests".to_string(),
                sli_type: SliType::Availability,
                good_events_query: "sum(rate(user_requests_total{status_code!~\"5.*\"}[5m]))"
                    .to_string(),
                valid_events_query: "sum(rate(user_requests_total[5m]))".to_string(),
                evaluation_interval: Duration::from_minutes(1),
            })
            .await?;

        self.sli_calculator
            .register_sli(SliDefinition {
                id: "latency".to_string(),
                name: "Request Latency".to_string(),
                description: "95th percentile request latency".to_string(),
                sli_type: SliType::Latency,
                good_events_query:
                    "histogram_quantile(0.95, request_duration_seconds_bucket) < 0.5".to_string(),
                valid_events_query: "sum(rate(request_duration_seconds_count[5m]))".to_string(),
                evaluation_interval: Duration::from_minutes(1),
            })
            .await?;

        // Initialize default alert rules
        self.alerting_engine.register_alert_rule(AlertRule {
            id: "high_error_rate".to_string(),
            name: "High Error Rate".to_string(),
            description: "Error rate exceeds 5% for 5 minutes".to_string(),
            condition: AlertCondition::ThresholdCondition {
                metric: "sum(rate(user_requests_total{status_code=~\"5.*\"}[5m])) / sum(rate(user_requests_total[5m]))".to_string(),
                operator: ComparisonOperator::GreaterThan,
                threshold: 0.05,
                duration: Duration::from_minutes(5),
            },
            severity: AlertSeverity::Critical,
            enabled: true,
            evaluation_interval: Duration::from_minutes(1),
            notification_channels: vec!["default".to_string()],
            escalation_policy: Some("default".to_string()),
            suppression_rules: vec![],
        }).await?;

        self.alerting_engine
            .register_alert_rule(AlertRule {
                id: "high_latency".to_string(),
                name: "High Request Latency".to_string(),
                description: "95th percentile latency exceeds 1s for 10 minutes".to_string(),
                condition: AlertCondition::ThresholdCondition {
                    metric: "histogram_quantile(0.95, request_duration_seconds_bucket)".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    threshold: 1.0,
                    duration: Duration::from_minutes(10),
                },
                severity: AlertSeverity::Warning,
                enabled: true,
                evaluation_interval: Duration::from_minutes(2),
                notification_channels: vec!["default".to_string()],
                escalation_policy: None,
                suppression_rules: vec![],
            })
            .await?;

        debug!("✅ Default monitoring configurations initialized");
        Ok(())
    }

    /// Start background processing tasks
    async fn start_background_processors(&self) -> Result<()> {
        // Metrics collection task
        let metrics_collector = Arc::clone(&self.metrics_collector);
        let collection_interval = self.config.metrics_collection_interval;
        tokio::spawn(async move {
            let mut interval = interval(collection_interval);
            loop {
                interval.tick().await;
                if let Err(e) = metrics_collector.collect_system_metrics().await {
                    error!("Failed to collect system metrics: {}", e);
                }
            }
        });

        // Trace processing task
        let trace_processor = Arc::clone(&self.trace_processor);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                if let Err(e) = trace_processor.process_completed_traces().await {
                    error!("Failed to process traces: {}", e);
                }
            }
        });

        // Log aggregation task
        let log_aggregator = Arc::clone(&self.log_aggregator);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                if let Err(e) = log_aggregator.process_log_buffer().await {
                    error!("Failed to process logs: {}", e);
                }
            }
        });

        // Alert evaluation task
        let alerting_engine = Arc::clone(&self.alerting_engine);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = alerting_engine.evaluate_alert_rules().await {
                    error!("Failed to evaluate alerts: {}", e);
                }
            }
        });

        // Anomaly detection task
        if self.config.anomaly_detection_enabled {
            let anomaly_detector = Arc::clone(&self.anomaly_detector);
            tokio::spawn(async move {
                let mut interval = interval(Duration::from_minutes(5));
                loop {
                    interval.tick().await;
                    if let Err(e) = anomaly_detector.run_anomaly_detection().await {
                        error!("Failed to run anomaly detection: {}", e);
                    }
                }
            });
        }

        // SLI calculation task
        let sli_calculator = Arc::clone(&self.sli_calculator);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_minutes(1));
            loop {
                interval.tick().await;
                if let Err(e) = sli_calculator.calculate_slis().await {
                    error!("Failed to calculate SLIs: {}", e);
                }
            }
        });

        // Continuous profiling task
        if self.config.profiling_enabled {
            let profiler = Arc::clone(&self.performance_profiler);
            tokio::spawn(async move {
                let mut interval = interval(Duration::from_minutes(15));
                loop {
                    interval.tick().await;
                    if let Err(e) = profiler.collect_profile().await {
                        error!("Failed to collect performance profile: {}", e);
                    }
                }
            });
        }

        info!("🚀 Background observability processors started");
        Ok(())
    }

    /// Record custom business metric
    pub async fn record_business_metric(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        self.metrics_collector
            .record_custom_metric(
                name,
                CustomMetricType::Business,
                serde_json::json!(value),
                labels,
            )
            .await
    }

    /// Start distributed trace
    pub async fn start_trace(
        &self,
        operation_name: &str,
        parent_span: Option<Uuid>,
    ) -> Result<Uuid> {
        self.trace_processor
            .start_span(operation_name, parent_span)
            .await
    }

    /// Finish distributed trace span
    pub async fn finish_trace_span(
        &self,
        span_id: Uuid,
        status: SpanStatus,
        tags: HashMap<String, String>,
    ) -> Result<()> {
        self.trace_processor
            .finish_span(span_id, status, tags)
            .await
    }

    /// Log structured message with correlation
    pub async fn log_structured(
        &self,
        level: LogLevel,
        message: &str,
        fields: HashMap<String, serde_json::Value>,
        correlation_id: Option<String>,
    ) -> Result<()> {
        self.log_aggregator
            .log_entry(LogEntry {
                id: Uuid::new_v4(),
                timestamp: SystemTime::now(),
                level,
                message: message.to_string(),
                service: "fhe-llm-proxy".to_string(),
                trace_id: None, // Could be extracted from context
                span_id: None,  // Could be extracted from context
                fields,
                correlation_id,
            })
            .await
    }

    /// Get real-time observability dashboard data
    pub async fn get_dashboard_data(&self, dashboard_id: &str) -> Result<serde_json::Value> {
        self.dashboard_engine.get_dashboard_data(dashboard_id).await
    }

    /// Get system health summary
    pub async fn get_health_summary(&self) -> ObservabilityHealthSummary {
        let metrics_health = self.metrics_collector.get_health_status().await;
        let trace_health = self.trace_processor.get_health_status().await;
        let log_health = self.log_aggregator.get_health_status().await;
        let alert_health = self.alerting_engine.get_health_status().await;

        let overall_status = if [&metrics_health, &trace_health, &log_health, &alert_health]
            .iter()
            .all(|h| matches!(h, HealthStatus::Healthy))
        {
            HealthStatus::Healthy
        } else if [&metrics_health, &trace_health, &log_health, &alert_health]
            .iter()
            .any(|h| matches!(h, HealthStatus::Unhealthy))
        {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        };

        ObservabilityHealthSummary {
            overall_status,
            metrics_status: metrics_health,
            tracing_status: trace_health,
            logging_status: log_health,
            alerting_status: alert_health,
            last_check: SystemTime::now(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ObservabilityHealthSummary {
    pub overall_status: HealthStatus,
    pub metrics_status: HealthStatus,
    pub tracing_status: HealthStatus,
    pub logging_status: HealthStatus,
    pub alerting_status: HealthStatus,
    pub last_check: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

// Implementation stubs for supporting structures
impl AdvancedMetricsCollector {
    pub async fn new(_config: &ObservabilityConfig) -> Result<Self> {
        Ok(Self {
            counter_metrics: Arc::new(RwLock::new(HashMap::new())),
            gauge_metrics: Arc::new(RwLock::new(HashMap::new())),
            histogram_metrics: Arc::new(RwLock::new(HashMap::new())),
            summary_metrics: Arc::new(RwLock::new(HashMap::new())),
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
            metric_registry: Arc::new(MetricRegistry::new()),
            export_manager: Arc::new(MetricExportManager::new()),
        })
    }

    pub async fn register_counter(
        &self,
        name: &str,
        description: &str,
        _labels: Vec<&str>,
    ) -> Result<()> {
        let metric = CounterMetric {
            name: name.to_string(),
            value: Arc::new(AtomicU64::new(0)),
            labels: HashMap::new(),
            description: description.to_string(),
            created_at: Instant::now(),
        };
        self.counter_metrics
            .write()
            .await
            .insert(name.to_string(), metric);
        debug!("📊 Counter metric '{}' registered", name);
        Ok(())
    }

    pub async fn register_gauge(
        &self,
        name: &str,
        description: &str,
        _labels: Vec<&str>,
    ) -> Result<()> {
        let metric = GaugeMetric {
            name: name.to_string(),
            value: Arc::new(AtomicU64::new(0)),
            labels: HashMap::new(),
            description: description.to_string(),
            last_updated: Arc::new(Mutex::new(Instant::now())),
        };
        self.gauge_metrics
            .write()
            .await
            .insert(name.to_string(), metric);
        debug!("📊 Gauge metric '{}' registered", name);
        Ok(())
    }

    pub async fn register_histogram(
        &self,
        name: &str,
        description: &str,
        buckets: Vec<f64>,
        _labels: Vec<&str>,
    ) -> Result<()> {
        let histogram_buckets: Vec<HistogramBucket> = buckets
            .into_iter()
            .map(|bound| HistogramBucket {
                upper_bound: bound,
                count: Arc::new(AtomicU64::new(0)),
            })
            .collect();

        let metric = HistogramMetric {
            name: name.to_string(),
            buckets: Arc::new(RwLock::new(histogram_buckets)),
            labels: HashMap::new(),
            description: description.to_string(),
            total_count: Arc::new(AtomicU64::new(0)),
            total_sum: Arc::new(AtomicU64::new(0)),
        };
        self.histogram_metrics
            .write()
            .await
            .insert(name.to_string(), metric);
        debug!("📊 Histogram metric '{}' registered", name);
        Ok(())
    }

    pub async fn record_custom_metric(
        &self,
        name: &str,
        metric_type: CustomMetricType,
        value: serde_json::Value,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        let metric = CustomMetric {
            name: name.to_string(),
            metric_type,
            value: Arc::new(RwLock::new(value)),
            labels,
            description: format!("Custom {} metric", name),
        };
        self.custom_metrics
            .write()
            .await
            .insert(name.to_string(), metric);
        Ok(())
    }

    pub async fn collect_system_metrics(&self) -> Result<()> {
        // Collect CPU usage
        if let Some(cpu_gauge) = self
            .gauge_metrics
            .read()
            .await
            .get("system_cpu_usage_percent")
        {
            // Simulate CPU collection - in real implementation, use system APIs
            let cpu_usage = 45; // Placeholder
            cpu_gauge.value.store(cpu_usage, Ordering::Relaxed);
            *cpu_gauge.last_updated.lock().await = Instant::now();
        }

        // Collect memory usage
        if let Some(memory_gauge) = self
            .gauge_metrics
            .read()
            .await
            .get("system_memory_usage_bytes")
        {
            let memory_usage = 2048 * 1024 * 1024; // 2GB placeholder
            memory_gauge.value.store(memory_usage, Ordering::Relaxed);
            *memory_gauge.last_updated.lock().await = Instant::now();
        }

        Ok(())
    }

    pub async fn get_health_status(&self) -> HealthStatus {
        HealthStatus::Healthy // Placeholder
    }
}

impl DistributedTraceProcessor {
    pub async fn new(_config: &ObservabilityConfig) -> Result<Self> {
        Ok(Self {
            active_spans: Arc::new(RwLock::new(HashMap::new())),
            completed_traces: Arc::new(RwLock::new(VecDeque::new())),
            sampling_strategy: Arc::new(AdaptiveSamplingStrategy::new()),
            trace_correlator: Arc::new(TraceCorrelator::new()),
            baggage_processor: Arc::new(BaggageProcessor::new()),
            export_pipeline: Arc::new(TraceExportPipeline::new()),
        })
    }

    pub async fn start_span(
        &self,
        operation_name: &str,
        parent_span_id: Option<Uuid>,
    ) -> Result<Uuid> {
        let span_id = Uuid::new_v4();
        let trace_id = if let Some(parent_id) = parent_span_id {
            // Find trace ID from parent span
            if let Some(parent_span) = self.active_spans.read().await.get(&parent_id) {
                parent_span.trace_id
            } else {
                Uuid::new_v4() // New trace if parent not found
            }
        } else {
            Uuid::new_v4() // New trace
        };

        let span = ActiveSpan {
            span_id,
            trace_id,
            parent_span_id,
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
            tags: HashMap::new(),
            baggage: HashMap::new(),
            logs: Vec::new(),
            references: Vec::new(),
        };

        self.active_spans.write().await.insert(span_id, span);
        debug!("🔍 Span '{}' started with ID {}", operation_name, span_id);
        Ok(span_id)
    }

    pub async fn finish_span(
        &self,
        span_id: Uuid,
        status: SpanStatus,
        tags: HashMap<String, String>,
    ) -> Result<()> {
        if let Some(active_span) = self.active_spans.write().await.remove(&span_id) {
            let completed_span = CompletedSpan {
                span_id: active_span.span_id,
                parent_span_id: active_span.parent_span_id,
                operation_name: active_span.operation_name,
                start_time: active_span.start_time,
                finish_time: Instant::now(),
                duration: active_span.start_time.elapsed(),
                tags,
                logs: active_span.logs,
                references: active_span.references,
                status,
            };

            // Store completed span for trace assembly
            // In real implementation, this would trigger trace completion logic
            debug!("🔍 Span {} finished with status {:?}", span_id, status);
        }
        Ok(())
    }

    pub async fn process_completed_traces(&self) -> Result<()> {
        // Process and export completed traces
        Ok(())
    }

    pub async fn get_health_status(&self) -> HealthStatus {
        HealthStatus::Healthy // Placeholder
    }
}

impl StructuredLogAggregator {
    pub async fn new(_config: &ObservabilityConfig) -> Result<Self> {
        Ok(Self {
            log_buffer: Arc::new(RwLock::new(VecDeque::new())),
            log_processors: Arc::new(RwLock::new(Vec::new())),
            correlation_engine: Arc::new(LogCorrelationEngine::new()),
            search_index: Arc::new(LogSearchIndex::new()),
            retention_manager: Arc::new(LogRetentionManager::new()),
        })
    }

    pub async fn log_entry(&self, entry: LogEntry) -> Result<()> {
        self.log_buffer.write().await.push_back(entry);
        Ok(())
    }

    pub async fn process_log_buffer(&self) -> Result<()> {
        let mut buffer = self.log_buffer.write().await;
        while let Some(entry) = buffer.pop_front() {
            // Process log entry - correlation, indexing, etc.
            debug!("Processing log entry: {}", entry.message);
        }
        Ok(())
    }

    pub async fn get_health_status(&self) -> HealthStatus {
        HealthStatus::Healthy // Placeholder
    }
}

impl IntelligentAlertingEngine {
    pub async fn new(_config: &ObservabilityConfig) -> Result<Self> {
        Ok(Self {
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            notification_channels: Arc::new(RwLock::new(HashMap::new())),
            escalation_policies: Arc::new(RwLock::new(HashMap::new())),
            alert_correlator: Arc::new(AlertCorrelator::new()),
            suppression_manager: Arc::new(AlertSuppressionManager::new()),
            ai_predictor: Arc::new(AlertPredictor::new()),
        })
    }

    pub async fn register_alert_rule(&self, rule: AlertRule) -> Result<()> {
        self.alert_rules
            .write()
            .await
            .insert(rule.id.clone(), rule.clone());
        info!("🚨 Alert rule '{}' registered", rule.name);
        Ok(())
    }

    pub async fn evaluate_alert_rules(&self) -> Result<()> {
        // Evaluate all alert rules
        let rules = self.alert_rules.read().await;
        for (id, rule) in rules.iter() {
            if rule.enabled {
                // Evaluate rule condition - placeholder implementation
                debug!("Evaluating alert rule: {}", id);
            }
        }
        Ok(())
    }

    pub async fn get_health_status(&self) -> HealthStatus {
        HealthStatus::Healthy // Placeholder
    }
}

impl AiAnomalyDetector {
    pub async fn new(_config: &ObservabilityConfig) -> Result<Self> {
        Ok(Self {
            detection_models: Arc::new(RwLock::new(HashMap::new())),
            feature_extractors: Arc::new(RwLock::new(HashMap::new())),
            training_pipeline: Arc::new(ModelTrainingPipeline::new()),
            prediction_engine: Arc::new(AnomalyPredictionEngine::new()),
            feedback_loop: Arc::new(ModelFeedbackLoop::new()),
        })
    }

    pub async fn run_anomaly_detection(&self) -> Result<()> {
        // Run anomaly detection on current metrics
        debug!("Running anomaly detection");
        Ok(())
    }
}

impl RealTimeDashboard {
    pub async fn new(_config: &ObservabilityConfig) -> Result<Self> {
        Ok(Self {
            widget_registry: Arc::new(RwLock::new(HashMap::new())),
            data_sources: Arc::new(RwLock::new(HashMap::new())),
            real_time_updates: Arc::new(RwLock::new(HashMap::new())),
            layout_engine: Arc::new(AdaptiveLayoutEngine::new()),
            visualization_engine: Arc::new(VisualizationEngine::new()),
        })
    }

    pub async fn get_dashboard_data(&self, _dashboard_id: &str) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            "widgets": []
        }))
    }
}

impl ContinuousProfiler {
    pub async fn new(_config: &ObservabilityConfig) -> Result<Self> {
        Ok(Self {
            profiling_sessions: Arc::new(RwLock::new(HashMap::new())),
            cpu_profiler: Arc::new(CpuProfiler::new()),
            memory_profiler: Arc::new(MemoryProfiler::new()),
            io_profiler: Arc::new(IoProfiler::new()),
            flame_graph_generator: Arc::new(FlameGraphGenerator::new()),
            performance_advisor: Arc::new(PerformanceAdvisor::new()),
        })
    }

    pub async fn collect_profile(&self) -> Result<()> {
        // Collect performance profile
        debug!("Collecting performance profile");
        Ok(())
    }
}

impl SliCalculator {
    pub async fn new(_config: &ObservabilityConfig) -> Result<Self> {
        Ok(Self {
            sli_definitions: Arc::new(RwLock::new(HashMap::new())),
            slo_targets: Arc::new(RwLock::new(HashMap::new())),
            error_budget_tracker: Arc::new(ErrorBudgetTracker::new()),
            burn_rate_calculator: Arc::new(BurnRateCalculator::new()),
            compliance_monitor: Arc::new(ComplianceMonitor::new()),
        })
    }

    pub async fn register_sli(&self, sli: SliDefinition) -> Result<()> {
        self.sli_definitions
            .write()
            .await
            .insert(sli.id.clone(), sli.clone());
        info!("📈 SLI '{}' registered", sli.name);
        Ok(())
    }

    pub async fn calculate_slis(&self) -> Result<()> {
        // Calculate SLI values
        debug!("Calculating SLIs");
        Ok(())
    }
}

// Missing type definitions for observability.rs
#[derive(Debug)]
pub struct MetricRegistry {
    pub metrics: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct MetricExportManager {
    pub exporters: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct AdaptiveSamplingStrategy {
    pub sampling_rules: HashMap<String, f64>, // Placeholder
}

#[derive(Debug)]
pub struct TraceCorrelator {
    pub correlation_rules: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct BaggageProcessor {
    pub baggage_items: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct TraceExportPipeline {
    pub pipeline_stages: Vec<String>, // Placeholder
}

#[derive(Debug)]
pub struct LogCorrelationEngine {
    pub correlation_models: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct LogSearchIndex {
    pub indices: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct LogRetentionManager {
    pub retention_policies: HashMap<String, Duration>, // Placeholder
}

#[derive(Debug)]
pub struct LogProcessor {
    pub processor_id: String,
    pub processing_rules: Vec<String>, // Placeholder
}

#[derive(Debug)]
pub struct NotificationChannel {
    pub channel_id: String,
    pub channel_type: String,
    pub config: HashMap<String, String>,
}

#[derive(Debug)]
pub struct EscalationPolicy {
    pub policy_id: String,
    pub escalation_steps: Vec<String>, // Placeholder
}

#[derive(Debug)]
pub struct AlertCorrelator {
    pub correlation_rules: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct AlertSuppressionManager {
    pub suppression_rules: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct AlertPredictor {
    pub prediction_models: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct FeatureExtractor {
    pub extractor_id: String,
    pub features: Vec<String>, // Placeholder
}

#[derive(Debug)]
pub struct ModelTrainingPipeline {
    pub pipeline_id: String,
    pub training_stages: Vec<String>, // Placeholder
}

#[derive(Debug)]
pub struct AnomalyPredictionEngine {
    pub prediction_models: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct ModelFeedbackLoop {
    pub feedback_data: VecDeque<String>, // Placeholder
}

#[derive(Debug)]
pub struct DataSource {
    pub source_id: String,
    pub source_type: String,
    pub connection_config: HashMap<String, String>,
}

#[derive(Debug)]
pub struct AdaptiveLayoutEngine {
    pub layout_templates: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct VisualizationEngine {
    pub chart_renderers: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct ProfilingSession {
    pub session_id: String,
    pub profiling_data: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct CpuProfiler {
    pub profiling_config: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct MemoryProfiler {
    pub memory_snapshots: Vec<String>, // Placeholder
}

#[derive(Debug)]
pub struct IoProfiler {
    pub io_metrics: HashMap<String, f64>, // Placeholder
}

#[derive(Debug)]
pub struct FlameGraphGenerator {
    pub graph_templates: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct PerformanceAdvisor {
    pub advisor_rules: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct ErrorBudgetTracker {
    pub budget_definitions: HashMap<String, f64>, // Placeholder
}

#[derive(Debug)]
pub struct BurnRateCalculator {
    pub calculation_models: HashMap<String, String>, // Placeholder
}

#[derive(Debug)]
pub struct ComplianceMonitor {
    pub compliance_rules: HashMap<String, String>, // Placeholder
}

// Placeholder implementations for supporting types
macro_rules! impl_new_for_placeholder {
    ($($type:ty),*) => {
        $(
            impl $type {
                pub fn new() -> Self { Self }
            }
        )*
    };
}

// Implementation for missing types in observability.rs
impl MetricRegistry {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }
}

impl MetricExportManager {
    pub fn new() -> Self {
        Self {
            exporters: HashMap::new(),
        }
    }
}

impl AdaptiveSamplingStrategy {
    pub fn new() -> Self {
        Self {
            sampling_rules: HashMap::new(),
        }
    }
}

impl TraceCorrelator {
    pub fn new() -> Self {
        Self {
            correlation_rules: HashMap::new(),
        }
    }
}

impl BaggageProcessor {
    pub fn new() -> Self {
        Self {
            baggage_items: HashMap::new(),
        }
    }
}

impl TraceExportPipeline {
    pub fn new() -> Self {
        Self {
            pipeline_stages: Vec::new(),
        }
    }
}

impl LogCorrelationEngine {
    pub fn new() -> Self {
        Self {
            correlation_models: HashMap::new(),
        }
    }
}

impl LogSearchIndex {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }
}

impl LogRetentionManager {
    pub fn new() -> Self {
        Self {
            retention_policies: HashMap::new(),
        }
    }
}

impl LogProcessor {
    pub fn new() -> Self {
        Self {
            processor_id: Uuid::new_v4().to_string(),
            processing_rules: Vec::new(),
        }
    }
}

impl NotificationChannel {
    pub fn new() -> Self {
        Self {
            channel_id: Uuid::new_v4().to_string(),
            channel_type: "default".to_string(),
            config: HashMap::new(),
        }
    }
}

impl EscalationPolicy {
    pub fn new() -> Self {
        Self {
            policy_id: Uuid::new_v4().to_string(),
            escalation_steps: Vec::new(),
        }
    }
}

impl AlertCorrelator {
    pub fn new() -> Self {
        Self {
            correlation_rules: HashMap::new(),
        }
    }
}

impl AlertSuppressionManager {
    pub fn new() -> Self {
        Self {
            suppression_rules: HashMap::new(),
        }
    }
}

impl AlertPredictor {
    pub fn new() -> Self {
        Self {
            prediction_models: HashMap::new(),
        }
    }
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            extractor_id: Uuid::new_v4().to_string(),
            features: Vec::new(),
        }
    }
}

impl ModelTrainingPipeline {
    pub fn new() -> Self {
        Self {
            pipeline_id: Uuid::new_v4().to_string(),
            training_stages: Vec::new(),
        }
    }
}

impl AnomalyPredictionEngine {
    pub fn new() -> Self {
        Self {
            prediction_models: HashMap::new(),
        }
    }
}

impl ModelFeedbackLoop {
    pub fn new() -> Self {
        Self {
            feedback_data: VecDeque::new(),
        }
    }
}

impl DataSource {
    pub fn new() -> Self {
        Self {
            source_id: Uuid::new_v4().to_string(),
            source_type: "default".to_string(),
            connection_config: HashMap::new(),
        }
    }
}

impl AdaptiveLayoutEngine {
    pub fn new() -> Self {
        Self {
            layout_templates: HashMap::new(),
        }
    }
}

impl VisualizationEngine {
    pub fn new() -> Self {
        Self {
            chart_renderers: HashMap::new(),
        }
    }
}

impl ProfilingSession {
    pub fn new() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            profiling_data: HashMap::new(),
        }
    }
}

impl CpuProfiler {
    pub fn new() -> Self {
        Self {
            profiling_config: HashMap::new(),
        }
    }
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            memory_snapshots: Vec::new(),
        }
    }
}

impl IoProfiler {
    pub fn new() -> Self {
        Self {
            io_metrics: HashMap::new(),
        }
    }
}

impl FlameGraphGenerator {
    pub fn new() -> Self {
        Self {
            graph_templates: HashMap::new(),
        }
    }
}

impl PerformanceAdvisor {
    pub fn new() -> Self {
        Self {
            advisor_rules: HashMap::new(),
        }
    }
}

impl ErrorBudgetTracker {
    pub fn new() -> Self {
        Self {
            budget_definitions: HashMap::new(),
        }
    }
}

impl BurnRateCalculator {
    pub fn new() -> Self {
        Self {
            calculation_models: HashMap::new(),
        }
    }
}

impl ComplianceMonitor {
    pub fn new() -> Self {
        Self {
            compliance_rules: HashMap::new(),
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_collection_interval: Duration::from_secs(15),
            trace_sampling_rate: 0.1,
            log_retention_days: 30,
            alerting_enabled: true,
            anomaly_detection_enabled: true,
            profiling_enabled: false, // Disabled by default due to overhead
            dashboard_refresh_rate: Duration::from_secs(30),
            export_endpoints: ExportEndpoints {
                prometheus_endpoint: Some("http://localhost:9090".to_string()),
                jaeger_endpoint: Some("http://localhost:14268".to_string()),
                elasticsearch_endpoint: Some("http://localhost:9200".to_string()),
                grafana_endpoint: Some("http://localhost:3000".to_string()),
                otel_collector_endpoint: Some("http://localhost:4317".to_string()),
            },
            storage_config: StorageConfig {
                metrics_retention_days: 90,
                traces_retention_days: 7,
                logs_retention_days: 30,
                compression_enabled: true,
                encryption_enabled: true,
            },
        }
    }
}

// Extension trait for Duration
trait DurationExt {
    fn from_minutes(minutes: u64) -> Duration;
}

impl DurationExt for Duration {
    fn from_minutes(minutes: u64) -> Duration {
        Duration::from_secs(minutes * 60)
    }
}
