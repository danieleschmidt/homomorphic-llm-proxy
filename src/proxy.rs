//! Proxy server implementation

use crate::error::{Error, Result};
use crate::fhe::{FheEngine, FheParams, Ciphertext};
use crate::config::Config;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use reqwest::Client as HttpClient;
use tower::ServiceBuilder;
use axum::middleware::from_fn;
use std::time::{Duration, Instant};
use base64::prelude::*;

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
        self.sessions.read().await.get(&session_id).map(|s| s.client_id)
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
        
        let response = self.client
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
            llm_providers.insert("openai".to_string(), LlmProvider::new("openai", openai_key.clone()));
        }
        if let Some(ref anthropic_key) = config.llm.anthropic_api_key {
            llm_providers.insert("anthropic".to_string(), LlmProvider::new("anthropic", anthropic_key.clone()));
        }
        
        let state = Arc::new(ProxyState {
            config,
            fhe_engine: Arc::new(RwLock::new(fhe_engine)),
            session_manager: SessionManager::new(),
            llm_providers,
            ciphertext_cache: RwLock::new(HashMap::new()),
        });
        
        Ok(Self { state })
    }
    
    /// Start the proxy server
    pub async fn start(&self) -> Result<()> {
        let app = self.create_router().await;
        
        let addr = format!("{}:{}", self.state.config.server.host, self.state.config.server.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        
        log::info!("🔐 FHE LLM Proxy listening on {}", addr);
        log::info!("📊 Available providers: {:?}", self.state.llm_providers.keys().collect::<Vec<_>>());
        
        axum::serve(listener, app).await.map_err(|e| Error::Http(e.to_string()))
    }
    
    /// Create the router with all endpoints
    async fn create_router(&self) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/v1/keys/generate", post(generate_keys))
            .route("/v1/encrypt", post(encrypt_text))
            .route("/v1/decrypt", post(decrypt_text))
            .route("/v1/chat/completions", post(process_encrypted_completion))
            .route("/v1/ciphertext/:id", get(get_ciphertext))
            .route("/v1/params", get(get_fhe_params))
            .route("/v1/sessions/:id/stats", get(get_session_stats))
            .layer(ServiceBuilder::new()
                .layer(from_fn(logging_middleware))
                .into_inner())
            .with_state(self.state.clone())
    }
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "FHE LLM Proxy is running"
}

/// Generate new FHE key pair
async fn generate_keys(
    State(state): State<Arc<ProxyState>>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let mut fhe_engine = state.fhe_engine.write().await;
    
    match fhe_engine.generate_keys() {
        Ok((client_id, server_id)) => {
            let session_id = state.session_manager.create_session(client_id, server_id).await;
            
            Ok(Json(serde_json::json!({
                "session_id": session_id,
                "client_id": client_id,
                "server_id": server_id,
                "params": fhe_engine.get_params()
            })))
        }
        Err(e) => {
            log::error!("Failed to generate keys: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
            state.ciphertext_cache.write().await.insert(ciphertext.id, ciphertext.clone());
            
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
    
    let ciphertext = state.ciphertext_cache.read().await
        .get(&ciphertext_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let fhe_engine = state.fhe_engine.read().await;
    
    match fhe_engine.decrypt_text(client_id, &ciphertext) {
        Ok(plaintext) => {
            Ok(Json(serde_json::json!({
                "plaintext": plaintext,
                "ciphertext_id": ciphertext_id
            })))
        }
        Err(e) => {
            log::error!("Decryption failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Process encrypted completion request
async fn process_encrypted_completion(
    State(state): State<Arc<ProxyState>>,
    Json(request): Json<ProcessRequest>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    // Get the cached ciphertext
    let ciphertext = state.ciphertext_cache.read().await
        .get(&request.ciphertext_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Get the LLM provider
    let _provider = state.llm_providers.get(&request.provider)
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // For demonstration, we'll simulate processing the encrypted request
    // In a real implementation, this would involve complex FHE operations
    let fhe_engine = state.fhe_engine.read().await;
    
    // Process the encrypted prompt
    let processed_ciphertext = fhe_engine.process_encrypted_prompt(&ciphertext)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
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
    state.ciphertext_cache.write().await.insert(processed_ciphertext.id, processed_ciphertext);
    
    Ok(Json(response))
}

/// Get ciphertext by ID
async fn get_ciphertext(
    State(state): State<Arc<ProxyState>>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let ciphertext = state.ciphertext_cache.read().await
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
async fn get_fhe_params(
    State(state): State<Arc<ProxyState>>,
) -> Json<FheParams> {
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

/// Logging middleware
async fn logging_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    let response = next.run(request).await;
    
    let elapsed = start.elapsed();
    log::info!("{} {} - {} - {:?}", method, uri, response.status(), elapsed);
    
    response
}