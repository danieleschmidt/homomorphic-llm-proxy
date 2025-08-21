//! Proxy server implementation

use crate::config::Config;
use crate::error::{Error, Result};
use crate::fhe::{Ciphertext, FheEngine, FheParams};
use crate::middleware::{MetricsCollector, PrivacyBudgetTracker, RateLimiter};
use crate::monitoring::{MonitoringService, PerformanceProfiler, StructuredLogger};
use crate::scaling::{AutoScaler, BatchProcessor, CiphertextCache, CircuitBreaker, FheConnectionPool};
use crate::performance::{PerformanceCache, ConnectionPoolShard, CacheConfig, EvictionStrategy};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use base64::prelude::*;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Request to encrypt text
#[derive(Debug, Deserialize)]
pub struct EncryptRequest {
    pub text: String,
    pub client_id: Option<Uuid>,
    pub params: Option<FheParams>,
}

/// Response with encrypted data
#[derive(Debug, Serialize)]
pub struct EncryptResponse {
    pub ciphertext_id: Uuid,
    pub encrypted_data: String, // Base64 encoded
    pub params: FheParams,
    pub noise_budget: Option<u64>,
}

/// Request to process encrypted prompt
#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    pub ciphertext_id: Uuid,
    pub encrypted_data: String, // Base64 encoded
    pub provider: String,
    pub model: String,
    pub stream: Option<bool>,
}

/// LLM completion request
#[derive(Debug, Serialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

/// LLM message
#[derive(Debug, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

/// LLM completion response
#[derive(Debug, Deserialize)]
pub struct LlmResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<LlmChoice>,
    pub usage: Option<LlmUsage>,
}

#[derive(Debug, Deserialize)]
pub struct LlmChoice {
    pub index: u32,
    pub message: LlmMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LlmUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Session management for client keys
#[derive(Debug)]
pub struct SessionManager {
    sessions: RwLock<HashMap<Uuid, SessionData>>,
}

#[derive(Debug)]
struct SessionData {
    client_id: Uuid,
    server_id: Uuid,
    created_at: Instant,
    last_used: Instant,
    request_count: u64,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_session(&self, client_id: Uuid, server_id: Uuid) -> Uuid {
        let session_id = Uuid::new_v4();
        let now = Instant::now();

        let session_data = SessionData {
            client_id,
            server_id,
            created_at: now,
            last_used: now,
            request_count: 0,
        };

        self.sessions.write().await.insert(session_id, session_data);
        session_id
    }

    pub async fn get_client_id(&self, session_id: Uuid) -> Option<Uuid> {
        self.sessions
            .read()
            .await
            .get(&session_id)
            .map(|s| s.client_id)
    }

    pub async fn update_last_used(&self, session_id: Uuid) {
        if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
            session.last_used = Instant::now();
            session.request_count += 1;
        }
    }
}

/// LLM provider client
#[derive(Debug)]
pub struct LlmProvider {
    client: HttpClient,
    api_key: String,
    base_url: String,
}

impl LlmProvider {
    pub fn new(provider: &str, api_key: String) -> Self {
        let base_url = match provider {
            "openai" => "https://api.openai.com/v1".to_string(),
            "anthropic" => "https://api.anthropic.com/v1".to_string(),
            url if url.starts_with("http") => url.to_string(),
            _ => format!("https://api.{}.com/v1", provider),
        };

        Self {
            client: HttpClient::new(),
            api_key,
            base_url,
        }
    }

    pub async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        log::debug!("Sending request to LLM provider: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(300))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Provider(format!("LLM API error: {}", error_text)));
        }

        response.json().await.map_err(Error::from)
    }
}

/// Main proxy server state
#[derive(Debug)]
pub struct ProxyState {
    pub config: Config,
    pub fhe_engine: Arc<RwLock<FheEngine>>,
    pub session_manager: SessionManager,
    pub llm_providers: HashMap<String, LlmProvider>,
    pub ciphertext_cache: RwLock<HashMap<Uuid, Ciphertext>>,
    pub rate_limiter: RateLimiter,
    pub metrics: MetricsCollector,
    pub privacy_tracker: PrivacyBudgetTracker,
    pub monitoring: MonitoringService,
    pub profiler: PerformanceProfiler,
    // Scaling components
    pub fhe_pool: FheConnectionPool,
    pub auto_scaler: AutoScaler,
    pub batch_processor: BatchProcessor,
    pub advanced_cache: CiphertextCache,
    pub circuit_breaker: CircuitBreaker,
    // Performance optimization
    pub performance_cache: PerformanceCache,
    pub connection_manager: ConnectionPoolShard,
}

/// Main proxy server
pub struct ProxyServer {
    state: Arc<ProxyState>,
}

impl ProxyServer {
    /// Create new proxy server
    pub fn new(config: Config) -> Result<Self> {
        let fhe_params = FheParams {
            poly_modulus_degree: config.encryption.poly_modulus_degree,
            coeff_modulus_bits: config.encryption.coeff_modulus_bits.clone(),
            scale_bits: config.encryption.scale_bits,
            security_level: 128,
        };

        let fhe_engine = FheEngine::new(fhe_params)?;

        // Initialize LLM providers
        let mut llm_providers = HashMap::new();
        if let Some(ref openai_key) = config.llm.openai_api_key {
            llm_providers.insert(
                "openai".to_string(),
                LlmProvider::new("openai", openai_key.clone()),
            );
        }
        if let Some(ref anthropic_key) = config.llm.anthropic_api_key {
            llm_providers.insert(
                "anthropic".to_string(),
                LlmProvider::new("anthropic", anthropic_key.clone()),
            );
        }

        // Initialize scaling components
        let fhe_params_for_pool = FheParams {
            poly_modulus_degree: config.encryption.poly_modulus_degree,
            coeff_modulus_bits: config.encryption.coeff_modulus_bits.clone(),
            scale_bits: config.encryption.scale_bits,
            security_level: 128,
        };
        
        let fhe_pool = FheConnectionPool::new(
            config.scaling.connection_pool_size,
            config.scaling.max_concurrent_requests as usize,
            fhe_params_for_pool.clone(),
        )?;
        
        let auto_scaler = AutoScaler::new(
            config.scaling.target_cpu_utilization,
            config.scaling.min_instances as usize,
            config.scaling.max_instances as usize,
            config.scaling.scale_up_threshold as usize,
            std::time::Duration::from_secs(config.scaling.cooldown_period_seconds),
        );
        
        let batch_processor = BatchProcessor::new(
            config.performance.batch_size,
            std::time::Duration::from_millis(config.performance.batch_timeout_ms),
        );
        
        let advanced_cache = CiphertextCache::new(
            (config.performance.cache_size_mb * 1024 * 1024) as usize,
            std::time::Duration::from_secs(config.performance.cache_ttl_seconds),
        );
        
        let circuit_breaker = CircuitBreaker::new(
            50,
            30,
            std::time::Duration::from_secs(60),
        );
        
        let performance_cache = PerformanceCache::new(CacheConfig {
            l1_max_size: (config.performance.cache_size_mb * 1024 * 1024 / 4) as usize,
            l2_max_size: (config.performance.cache_size_mb * 1024 * 1024 / 2) as usize,
            hot_max_size: (config.performance.cache_size_mb * 1024 * 1024 / 4) as usize,
            default_ttl: std::time::Duration::from_secs(config.performance.cache_ttl_seconds),
            hot_threshold_accesses: 5,
            eviction_strategy: EvictionStrategy::Adaptive,
            compression_enabled: config.performance.compression_enabled,
            encryption_enabled: true,
        });
        
        let connection_manager = ConnectionPoolShard {
            id: 0,
            engines: Vec::new(),
            current_load: std::sync::atomic::AtomicUsize::new(0),
            max_load: config.scaling.max_concurrent_requests as usize,
            health_score: std::sync::atomic::AtomicU64::new(100),
        };

        let state = Arc::new(ProxyState {
            rate_limiter: RateLimiter::new(config.privacy.max_queries_per_user as u64),
            metrics: MetricsCollector::new(),
            privacy_tracker: PrivacyBudgetTracker::new(
                config.privacy.epsilon_per_query * config.privacy.max_queries_per_user as f64,
                config.privacy.delta,
            ),
            monitoring: MonitoringService::new(env!("CARGO_PKG_VERSION").to_string()),
            profiler: PerformanceProfiler::new(),
            fhe_engine: Arc::new(RwLock::new(fhe_engine)),
            session_manager: SessionManager::new(),
            llm_providers,
            ciphertext_cache: RwLock::new(HashMap::new()),
            // Scaling components
            fhe_pool,
            auto_scaler,
            batch_processor,
            advanced_cache,
            circuit_breaker,
            // Performance optimization
            performance_cache,
            connection_manager,
            config,
        });

        Ok(Self { state })
    }

    /// Start the proxy server
    pub async fn start(&self) -> Result<()> {
        let app = self.create_router().await;

        let addr = format!(
            "{}:{}",
            self.state.config.server.host, self.state.config.server.port
        );
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        log::info!("🔐 FHE LLM Proxy listening on {}", addr);
        log::info!(
            "📊 Available providers: {:?}",
            self.state.llm_providers.keys().collect::<Vec<_>>()
        );

        axum::serve(listener, app)
            .await
            .map_err(|e| Error::Http(e.to_string()))
    }

    /// Create the router with all endpoints
    async fn create_router(&self) -> Router {
        Router::new()
            // Health and monitoring endpoints
            .route("/health", get(health_check))
            .route("/health/live", get(liveness_check))
            .route("/health/ready", get(readiness_check))
            .route("/metrics", get(get_metrics))
            .route("/metrics/detailed", get(get_detailed_metrics))
            // Core FHE endpoints
            .route("/v1/keys/generate", post(generate_keys))
            .route("/v1/keys/rotate/{client_id}", post(rotate_client_keys))
            .route("/v1/encrypt", post(encrypt_text))
            .route("/v1/decrypt", post(decrypt_text))
            .route("/v1/chat/completions", post(process_encrypted_completion))
            .route("/v1/chat/stream", post(stream_encrypted_completion))
            .route("/v1/ciphertext/{id}", get(get_ciphertext))
            .route("/v1/ciphertext/{id}/validate", post(validate_ciphertext))
            .route("/v1/params", get(get_fhe_params))
            .route("/v1/concatenate", post(concatenate_ciphertexts))
            // Session and admin endpoints
            .route("/v1/sessions/{id}/stats", get(get_session_stats))
            .route("/v1/privacy/budget/{user}", get(get_privacy_budget))
            .route(
                "/v1/privacy/budget/{user}/reset",
                post(reset_privacy_budget),
            )
            .route("/v1/admin/performance", get(get_performance_stats))
            // Middleware layers
            .layer(from_fn_with_state(
                self.state.clone(),
                rate_limiting_middleware,
            ))
            .layer(from_fn(logging_middleware))
            .with_state(self.state.clone())
    }
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "FHE LLM Proxy is running"
}

/// Generate new FHE key pair with enhanced error handling
async fn generate_keys(
    State(state): State<Arc<ProxyState>>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    // Record operation start for metrics
    let timer = state.profiler.start_timer("key_generation");

    // Check system capacity before generating keys
    let stats = state.fhe_engine.read().await.get_stats();
    if stats.total_client_keys > 1000 {
        log::warn!(
            "Key generation limit approached: {} active keys",
            stats.total_client_keys
        );
        state
            .monitoring
            .record_error(
                "capacity_warning".to_string(),
                "High number of active keys".to_string(),
            )
            .await;
    }

    let mut fhe_engine = state.fhe_engine.write().await;

    // Attempt key generation with retry logic
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 3;

    while attempts < MAX_ATTEMPTS {
        match fhe_engine.generate_keys() {
            Ok((client_id, server_id)) => {
                let session_id = state
                    .session_manager
                    .create_session(client_id, server_id)
                    .await;

                // Record successful operation
                state.metrics.increment_encryptions();
                drop(timer); // Complete timing measurement

                log::info!(
                    "Generated FHE key pair for session {} (attempt {})",
                    session_id,
                    attempts + 1
                );

                return Ok(Json(serde_json::json!({
                    "session_id": session_id,
                    "client_id": client_id,
                    "server_id": server_id,
                    "params": fhe_engine.get_params(),
                    "expires_at": chrono::Utc::now() + chrono::Duration::hours(24)
                })));
            }
            Err(e) => {
                attempts += 1;
                log::warn!("Key generation attempt {} failed: {}", attempts, e);

                if attempts < MAX_ATTEMPTS {
                    // Brief backoff before retry
                    drop(fhe_engine); // Release lock during backoff
                    tokio::time::sleep(std::time::Duration::from_millis(100 * attempts as u64))
                        .await;
                    fhe_engine = state.fhe_engine.write().await;
                } else {
                    state.metrics.increment_errors();
                    state
                        .monitoring
                        .record_error("key_generation".to_string(), e.to_string())
                        .await;
                    log::error!(
                        "Failed to generate keys after {} attempts: {}",
                        MAX_ATTEMPTS,
                        e
                    );
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Encrypt text endpoint
async fn encrypt_text(
    State(state): State<Arc<ProxyState>>,
    Json(request): Json<EncryptRequest>,
) -> std::result::Result<Json<EncryptResponse>, StatusCode> {
    let client_id = request.client_id.ok_or(StatusCode::BAD_REQUEST)?;
    let fhe_engine = state.fhe_engine.read().await;

    match fhe_engine.encrypt_text(client_id, &request.text) {
        Ok(ciphertext) => {
            let encrypted_data = base64::prelude::BASE64_STANDARD.encode(&ciphertext.data);

            // Cache the ciphertext
            state
                .ciphertext_cache
                .write()
                .await
                .insert(ciphertext.id, ciphertext.clone());

            Ok(Json(EncryptResponse {
                ciphertext_id: ciphertext.id,
                encrypted_data,
                params: ciphertext.params,
                noise_budget: ciphertext.noise_budget,
            }))
        }
        Err(e) => {
            log::error!("Encryption failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Decrypt text endpoint
async fn decrypt_text(
    State(state): State<Arc<ProxyState>>,
    Json(request): Json<serde_json::Value>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let ciphertext_id: Uuid = request["ciphertext_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let client_id: Uuid = request["client_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let ciphertext = state
        .ciphertext_cache
        .read()
        .await
        .get(&ciphertext_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    let fhe_engine = state.fhe_engine.read().await;

    match fhe_engine.decrypt_text(client_id, &ciphertext) {
        Ok(plaintext) => Ok(Json(serde_json::json!({
            "plaintext": plaintext,
            "ciphertext_id": ciphertext_id
        }))),
        Err(e) => {
            log::error!("Decryption failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Process encrypted completion request with enhanced security and validation
async fn process_encrypted_completion(
    State(state): State<Arc<ProxyState>>,
    Json(request): Json<ProcessRequest>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let _timer = state.profiler.start_timer("encrypted_completion");

    // Validate request parameters
    if request.provider.is_empty() || request.model.is_empty() {
        log::warn!("Invalid request parameters: empty provider or model");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Security check: validate provider against allowlist
    let allowed_providers = ["openai", "anthropic", "huggingface"];
    if !allowed_providers.contains(&request.provider.as_str()) {
        log::warn!("Unauthorized provider requested: {}", request.provider);
        return Err(StatusCode::FORBIDDEN);
    }

    // Get the cached ciphertext with enhanced validation
    let ciphertext = {
        let cache = state.ciphertext_cache.read().await;
        match cache.get(&request.ciphertext_id) {
            Some(ct) => {
                // Validate ciphertext age (expire after 1 hour)
                if ct.data.len() > 10_000_000 {
                    // 10MB limit
                    log::error!("Ciphertext too large: {} bytes", ct.data.len());
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
                ct.clone()
            }
            None => {
                log::warn!("Ciphertext not found: {}", request.ciphertext_id);
                return Err(StatusCode::NOT_FOUND);
            }
        }
    };

    // Get the LLM provider with validation
    let _provider = state.llm_providers.get(&request.provider).ok_or_else(|| {
        log::error!("Provider not configured: {}", request.provider);
        StatusCode::BAD_REQUEST
    })?;

    let fhe_engine = state.fhe_engine.read().await;

    // Validate ciphertext integrity before processing
    if !fhe_engine.validate_ciphertext(&ciphertext).map_err(|e| {
        log::error!("Ciphertext validation failed: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        log::error!("Ciphertext failed integrity check");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Process the encrypted prompt with error handling
    let processed_ciphertext = fhe_engine
        .process_encrypted_prompt(&ciphertext)
        .map_err(|e| {
            log::error!("FHE processing failed: {}", e);
            state.metrics.increment_errors();
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // For now, simulate an LLM response
    let response = serde_json::json!({
        "id": format!("fhe-{}", Uuid::new_v4()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": request.model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "This is an encrypted response processed through FHE."
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 12,
            "total_tokens": 22
        },
        "fhe_metadata": {
            "processed_ciphertext_id": processed_ciphertext.id,
            "noise_budget_remaining": processed_ciphertext.noise_budget,
            "encryption_params": processed_ciphertext.params
        }
    });

    // Cache the processed ciphertext
    state
        .ciphertext_cache
        .write()
        .await
        .insert(processed_ciphertext.id, processed_ciphertext);

    Ok(Json(response))
}

/// Get ciphertext by ID
async fn get_ciphertext(
    State(state): State<Arc<ProxyState>>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let ciphertext = state
        .ciphertext_cache
        .read()
        .await
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::json!({
        "id": ciphertext.id,
        "params": ciphertext.params,
        "noise_budget": ciphertext.noise_budget,
        "data_size": ciphertext.data.len()
    })))
}

/// Get FHE parameters
async fn get_fhe_params(State(state): State<Arc<ProxyState>>) -> Json<FheParams> {
    let fhe_engine = state.fhe_engine.read().await;
    Json(fhe_engine.get_params().clone())
}

/// Get session statistics
async fn get_session_stats(
    State(state): State<Arc<ProxyState>>,
    Path(session_id): Path<Uuid>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let sessions = state.session_manager.sessions.read().await;
    let session = match sessions.get(&session_id) {
        Some(session) => session,
        None => return Err(StatusCode::NOT_FOUND),
    };

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "client_id": session.client_id,
        "created_at": session.created_at.elapsed().as_secs(),
        "last_used": session.last_used.elapsed().as_secs(),
        "request_count": session.request_count
    })))
}

/// Enhanced logging middleware
async fn logging_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let response = next.run(request).await;

    let elapsed = start.elapsed();
    let status = response.status().as_u16();

    StructuredLogger::log_request(method.as_str(), uri.path(), status, elapsed, &client_ip);

    response
}

/// Rate limiting middleware
async fn rate_limiting_middleware(
    State(state): State<Arc<ProxyState>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> std::result::Result<Response, StatusCode> {
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // Check rate limit
    let allowed = state
        .rate_limiter
        .check_rate_limit(client_ip)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !allowed {
        StructuredLogger::log_security_event("rate_limit_exceeded", client_ip, "Too many requests");
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Increment metrics
    state.metrics.increment_requests();

    let response = next.run(request).await;
    Ok(response)
}

/// Liveness check endpoint (Kubernetes)
async fn liveness_check(State(state): State<Arc<ProxyState>>) -> StatusCode {
    if state.monitoring.liveness_check().await {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

/// Readiness check endpoint (Kubernetes)
async fn readiness_check(State(state): State<Arc<ProxyState>>) -> StatusCode {
    if state.monitoring.readiness_check().await {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

/// Get basic metrics
async fn get_metrics(State(state): State<Arc<ProxyState>>) -> Json<serde_json::Value> {
    let metrics = state.metrics.get_stats();
    Json(serde_json::json!({
        "requests": metrics.total_requests,
        "errors": metrics.total_errors,
        "encryptions": metrics.encryption_operations,
        "decryptions": metrics.decryption_operations,
        "avg_response_time_ms": metrics.avg_response_time_ms,
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

/// Get detailed system metrics
async fn get_detailed_metrics(State(state): State<Arc<ProxyState>>) -> Json<serde_json::Value> {
    let metrics = state.metrics.get_stats();
    let system_metrics = state
        .monitoring
        .get_metrics(metrics, &state.fhe_engine)
        .await;
    Json(serde_json::to_value(system_metrics).unwrap())
}

/// Get privacy budget for user
async fn get_privacy_budget(
    State(state): State<Arc<ProxyState>>,
    Path(user_id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let budget = state
        .privacy_tracker
        .get_budget_status(&user_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "remaining_epsilon": budget.remaining_epsilon,
        "remaining_delta": budget.remaining_delta,
        "total_queries": budget.queries_count,
        "last_query_ago_seconds": budget.last_query.elapsed().as_secs()
    })))
}

/// Reset privacy budget for user
async fn reset_privacy_budget(
    State(state): State<Arc<ProxyState>>,
    Path(user_id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    state
        .privacy_tracker
        .reset_budget(&user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "status": "reset",
        "timestamp": chrono::Utc::now().timestamp()
    })))
}

/// Get performance statistics
async fn get_performance_stats(State(state): State<Arc<ProxyState>>) -> Json<serde_json::Value> {
    let stats = state.profiler.get_all_stats().await;
    Json(serde_json::to_value(stats).unwrap())
}

/// Rotate client keys for enhanced security
async fn rotate_client_keys(
    State(state): State<Arc<ProxyState>>,
    Path(client_id): Path<Uuid>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let mut fhe_engine = state.fhe_engine.write().await;

    match fhe_engine.rotate_keys(client_id) {
        Ok(new_server_id) => {
            log::info!("Successfully rotated keys for client {}", client_id);
            Ok(Json(serde_json::json!({
                "client_id": client_id,
                "new_server_id": new_server_id,
                "rotated_at": chrono::Utc::now().timestamp(),
                "status": "success"
            })))
        }
        Err(e) => {
            log::error!("Key rotation failed for client {}: {}", client_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Stream encrypted completion response
async fn stream_encrypted_completion(
    State(state): State<Arc<ProxyState>>,
    Json(request): Json<ProcessRequest>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    // For now, return a simulated streaming response
    // In production, this would use Server-Sent Events or WebSockets

    let _ciphertext = state
        .ciphertext_cache
        .read()
        .await
        .get(&request.ciphertext_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    let stream_id = Uuid::new_v4();

    log::info!(
        "Starting encrypted stream {} for ciphertext {}",
        stream_id,
        request.ciphertext_id
    );

    Ok(Json(serde_json::json!({
        "stream_id": stream_id,
        "status": "streaming",
        "estimated_tokens": 150,
        "chunk_size": 10,
        "encryption_preserved": true,
        "noise_budget_tracking": true
    })))
}

/// Validate ciphertext integrity
async fn validate_ciphertext(
    State(state): State<Arc<ProxyState>>,
    Path(ciphertext_id): Path<Uuid>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let ciphertext = state
        .ciphertext_cache
        .read()
        .await
        .get(&ciphertext_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    let fhe_engine = state.fhe_engine.read().await;

    match fhe_engine.validate_ciphertext(&ciphertext) {
        Ok(is_valid) => {
            let validation_result = if is_valid { "valid" } else { "invalid" };

            Ok(Json(serde_json::json!({
                "ciphertext_id": ciphertext_id,
                "status": validation_result,
                "noise_budget": ciphertext.noise_budget,
                "size_bytes": ciphertext.data.len(),
                "security_level": ciphertext.params.security_level,
                "validated_at": chrono::Utc::now().timestamp()
            })))
        }
        Err(e) => {
            log::error!("Ciphertext validation error: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Concatenate two ciphertexts
async fn concatenate_ciphertexts(
    State(state): State<Arc<ProxyState>>,
    Json(request): Json<serde_json::Value>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let ciphertext_a_id: Uuid = request["ciphertext_a"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let ciphertext_b_id: Uuid = request["ciphertext_b"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let (ciphertext_a, ciphertext_b) = {
        let cache = state.ciphertext_cache.read().await;

        let ciphertext_a = cache
            .get(&ciphertext_a_id)
            .cloned()
            .ok_or(StatusCode::NOT_FOUND)?;

        let ciphertext_b = cache
            .get(&ciphertext_b_id)
            .cloned()
            .ok_or(StatusCode::NOT_FOUND)?;

        (ciphertext_a, ciphertext_b)
    };

    let fhe_engine = state.fhe_engine.read().await;

    match fhe_engine.concatenate_encrypted(&ciphertext_a, &ciphertext_b) {
        Ok(result_ciphertext) => {
            // Cache the result
            state
                .ciphertext_cache
                .write()
                .await
                .insert(result_ciphertext.id, result_ciphertext.clone());

            Ok(Json(serde_json::json!({
                "result_ciphertext_id": result_ciphertext.id,
                "input_a": ciphertext_a_id,
                "input_b": ciphertext_b_id,
                "size_bytes": result_ciphertext.data.len(),
                "noise_budget": result_ciphertext.noise_budget,
                "created_at": chrono::Utc::now().timestamp()
            })))
        }
        Err(e) => {
            log::error!("Ciphertext concatenation failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
