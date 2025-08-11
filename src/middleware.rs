//! Middleware for request/response processing, rate limiting, and metrics

use crate::error::{Error, Result};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Rate limiter for client requests
#[derive(Debug)]
pub struct RateLimiter {
    clients: Arc<RwLock<HashMap<String, ClientLimiter>>>,
    global_limit: u64,
    window_duration: Duration,
}

#[derive(Debug)]
struct ClientLimiter {
    requests: AtomicU64,
    window_start: Instant,
    blocked_until: Option<Instant>,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u64) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            global_limit: requests_per_minute,
            window_duration: Duration::from_secs(60),
        }
    }

    pub async fn check_rate_limit(&self, client_ip: &str) -> Result<bool> {
        let mut clients = self.clients.write().await;
        let now = Instant::now();

        let client_limiter =
            clients
                .entry(client_ip.to_string())
                .or_insert_with(|| ClientLimiter {
                    requests: AtomicU64::new(0),
                    window_start: now,
                    blocked_until: None,
                });

        // Check if client is still blocked
        if let Some(blocked_until) = client_limiter.blocked_until {
            if now < blocked_until {
                return Ok(false);
            }
        }

        // Reset window if needed
        if now.duration_since(client_limiter.window_start) >= self.window_duration {
            client_limiter.requests.store(0, Ordering::Relaxed);
            client_limiter.window_start = now;
            client_limiter.blocked_until = None;
        }

        let current_requests = client_limiter.requests.fetch_add(1, Ordering::Relaxed);

        if current_requests >= self.global_limit {
            client_limiter.blocked_until = Some(now + Duration::from_secs(60));
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn get_client_stats(&self, client_ip: &str) -> Option<(u64, Duration)> {
        let clients = self.clients.read().await;
        let client = clients.get(client_ip)?;
        let requests = client.requests.load(Ordering::Relaxed);
        let window_remaining = self
            .window_duration
            .saturating_sub(Instant::now().duration_since(client.window_start));
        Some((requests, window_remaining))
    }
}

/// Request metrics collector
#[derive(Debug)]
pub struct MetricsCollector {
    total_requests: AtomicU64,
    total_errors: AtomicU64,
    encryption_operations: AtomicU64,
    decryption_operations: AtomicU64,
    avg_response_time: AtomicU64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            encryption_operations: AtomicU64::new(0),
            decryption_operations: AtomicU64::new(0),
            avg_response_time: AtomicU64::new(0),
        }
    }

    pub fn increment_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_errors(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_encryptions(&self) {
        self.encryption_operations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_decryptions(&self) {
        self.decryption_operations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_response_time(&self, duration: Duration) {
        let millis = duration.as_millis() as u64;
        let current_avg = self.avg_response_time.load(Ordering::Relaxed);
        let new_avg = (current_avg + millis) / 2;
        self.avg_response_time.store(new_avg, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            encryption_operations: self.encryption_operations.load(Ordering::Relaxed),
            decryption_operations: self.decryption_operations.load(Ordering::Relaxed),
            avg_response_time_ms: self.avg_response_time.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub total_errors: u64,
    pub encryption_operations: u64,
    pub decryption_operations: u64,
    pub avg_response_time_ms: u64,
}

/// Security headers middleware
pub fn security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();

    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'".parse().unwrap(),
    );
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    response
}

/// Request validation middleware
pub async fn validate_request(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    // Validate Content-Type for POST requests
    if request.method() == "POST" {
        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.starts_with("application/json")
            && !content_type.starts_with("application/octet-stream")
        {
            log::warn!("Invalid content type: {}", content_type);
            return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
    }

    // Validate content length
    if let Some(content_length) = headers.get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > 10 * 1024 * 1024 {
                    // 10MB limit
                    log::warn!("Request too large: {} bytes", length);
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    let response = next.run(request).await;
    Ok(security_headers(response))
}

/// Authentication middleware (placeholder for actual auth)
pub async fn authenticate_request(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    // Check for API key in headers
    let auth_header = headers
        .get("authorization")
        .or_else(|| headers.get("x-api-key"));

    if auth_header.is_none() {
        log::warn!("Missing authentication header");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // For now, accept any non-empty auth header
    // In production, validate against stored API keys
    let response = next.run(request).await;
    Ok(response)
}

/// Privacy budget tracking middleware
#[derive(Debug)]
pub struct PrivacyBudgetTracker {
    user_budgets: Arc<RwLock<HashMap<String, UserPrivacyBudget>>>,
    default_epsilon: f64,
    default_delta: f64,
}

#[derive(Debug, Clone)]
pub struct UserPrivacyBudget {
    pub total_epsilon: f64,
    pub total_delta: f64,
    pub remaining_epsilon: f64,
    pub remaining_delta: f64,
    pub queries_count: u64,
    pub last_query: Instant,
}

impl PrivacyBudgetTracker {
    pub fn new(default_epsilon: f64, default_delta: f64) -> Self {
        Self {
            user_budgets: Arc::new(RwLock::new(HashMap::new())),
            default_epsilon,
            default_delta,
        }
    }

    pub async fn check_budget(
        &self,
        user_id: &str,
        epsilon_cost: f64,
        delta_cost: f64,
    ) -> Result<bool> {
        let mut budgets = self.user_budgets.write().await;
        let budget = budgets
            .entry(user_id.to_string())
            .or_insert_with(|| UserPrivacyBudget {
                total_epsilon: self.default_epsilon,
                total_delta: self.default_delta,
                remaining_epsilon: self.default_epsilon,
                remaining_delta: self.default_delta,
                queries_count: 0,
                last_query: Instant::now(),
            });

        if budget.remaining_epsilon < epsilon_cost || budget.remaining_delta < delta_cost {
            return Ok(false);
        }

        budget.remaining_epsilon -= epsilon_cost;
        budget.remaining_delta -= delta_cost;
        budget.queries_count += 1;
        budget.last_query = Instant::now();

        log::debug!(
            "Privacy budget consumed for user {}: ε={:.3}, δ={:.6}, remaining: ε={:.3}, δ={:.6}",
            user_id,
            epsilon_cost,
            delta_cost,
            budget.remaining_epsilon,
            budget.remaining_delta
        );

        Ok(true)
    }

    pub async fn get_budget_status(&self, user_id: &str) -> Option<UserPrivacyBudget> {
        let budgets = self.user_budgets.read().await;
        budgets.get(user_id).cloned()
    }

    pub async fn reset_budget(&self, user_id: &str) -> Result<()> {
        let mut budgets = self.user_budgets.write().await;
        if let Some(budget) = budgets.get_mut(user_id) {
            budget.remaining_epsilon = budget.total_epsilon;
            budget.remaining_delta = budget.total_delta;
            budget.queries_count = 0;
            log::info!("Reset privacy budget for user {}", user_id);
        }
        Ok(())
    }
}

/// Input sanitization utilities
pub fn sanitize_text_input(input: &str) -> String {
    // Remove potential injection patterns and normalize input
    input
        .chars()
        .filter(|c| c.is_ascii() && !c.is_control() || c.is_whitespace())
        .take(10_000) // Limit input size
        .collect::<String>()
        .trim()
        .to_string()
}

/// Validate UUID format
pub fn validate_uuid(uuid_str: &str) -> Result<Uuid> {
    uuid_str
        .parse::<Uuid>()
        .map_err(|_| Error::Validation(format!("Invalid UUID format: {}", uuid_str)))
}

/// Validate encryption parameters
pub fn validate_fhe_params(poly_degree: usize, security_level: u8) -> Result<()> {
    if !poly_degree.is_power_of_two() {
        return Err(Error::Validation(
            "Polynomial modulus degree must be a power of 2".to_string(),
        ));
    }

    if poly_degree < 1024 || poly_degree > 32768 {
        return Err(Error::Validation(
            "Polynomial modulus degree must be between 1024 and 32768".to_string(),
        ));
    }

    if security_level != 128 && security_level != 192 {
        return Err(Error::Validation(
            "Security level must be 128 or 192".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2); // 2 requests per minute
        let client_ip = "192.168.1.1";

        // First two requests should succeed
        assert!(limiter.check_rate_limit(client_ip).await.unwrap());
        assert!(limiter.check_rate_limit(client_ip).await.unwrap());

        // Third request should be blocked
        assert!(!limiter.check_rate_limit(client_ip).await.unwrap());

        // Check stats
        let (requests, _) = limiter.get_client_stats(client_ip).await.unwrap();
        assert_eq!(requests, 3); // Including the blocked request
    }

    #[tokio::test]
    async fn test_privacy_budget_tracker() {
        let tracker = PrivacyBudgetTracker::new(1.0, 1e-5);
        let user_id = "test_user";

        // Should allow query within budget
        assert!(tracker.check_budget(user_id, 0.1, 1e-6).await.unwrap());

        // Should block query that exceeds budget
        assert!(!tracker.check_budget(user_id, 1.0, 1e-5).await.unwrap());

        // Reset and try again
        tracker.reset_budget(user_id).await.unwrap();
        assert!(tracker.check_budget(user_id, 0.5, 5e-6).await.unwrap());
    }

    #[test]
    fn test_input_sanitization() {
        let malicious_input = "Hello\x00World\x1F\nValid text";
        let sanitized = sanitize_text_input(malicious_input);
        assert_eq!(sanitized, "HelloWorld\nValid text");

        // Test length limiting
        let long_input = "a".repeat(20000);
        let sanitized_long = sanitize_text_input(&long_input);
        assert_eq!(sanitized_long.len(), 10000);
    }

    #[test]
    fn test_uuid_validation() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(validate_uuid(valid_uuid).is_ok());

        let invalid_uuid = "not-a-uuid";
        assert!(validate_uuid(invalid_uuid).is_err());
    }

    #[test]
    fn test_fhe_params_validation() {
        // Valid parameters
        assert!(validate_fhe_params(16384, 128).is_ok());

        // Invalid polynomial degree (not power of 2)
        assert!(validate_fhe_params(15000, 128).is_err());

        // Invalid security level
        assert!(validate_fhe_params(16384, 100).is_err());
    }
}
