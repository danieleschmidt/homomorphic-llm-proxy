//! Security utilities and authentication

use crate::error::{Error, Result};
use base64::prelude::*;
use ring::digest;
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// API key management
#[derive(Debug)]
pub struct ApiKeyManager {
    keys: HashMap<String, ApiKeyInfo>,
    master_secret: Secret<String>,
}

#[derive(Debug, Clone)]
struct ApiKeyInfo {
    user_id: String,
    permissions: Vec<Permission>,
    created_at: Instant,
    expires_at: Option<Instant>,
    rate_limit: u32,
    last_used: Option<Instant>,
    usage_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    Encrypt,
    Decrypt,
    ProcessLLM,
    ViewMetrics,
    Admin,
}

impl ApiKeyManager {
    pub fn new(master_secret: SecretString) -> Self {
        Self {
            keys: HashMap::new(),
            master_secret: master_secret,
        }
    }

    /// Generate a new API key
    pub fn generate_api_key(&mut self, user_id: String, permissions: Vec<Permission>) -> String {
        let key_data = format!("{}-{}-{}", 
            user_id, 
            Uuid::new_v4(), 
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
        
        // Hash the key data with master secret
        let key_hash = self.hash_with_secret(&key_data);
        let api_key = BASE64_STANDARD.encode(&key_hash);
        
        let key_info = ApiKeyInfo {
            user_id,
            permissions,
            created_at: Instant::now(),
            expires_at: Some(Instant::now() + Duration::from_secs(86400 * 30)), // 30 days
            rate_limit: 1000,
            last_used: None,
            usage_count: 0,
        };
        
        self.keys.insert(api_key.clone(), key_info);
        api_key
    }

    /// Validate API key and return user permissions
    pub fn validate_api_key(&mut self, api_key: &str) -> Result<Vec<Permission>> {
        let key_info = self.keys.get_mut(api_key)
            .ok_or_else(|| Error::Auth("Invalid API key".to_string()))?;

        // Check expiration
        if let Some(expires_at) = key_info.expires_at {
            if Instant::now() > expires_at {
                return Err(Error::Auth("API key expired".to_string()));
            }
        }

        // Update usage statistics
        key_info.last_used = Some(Instant::now());
        key_info.usage_count += 1;

        Ok(key_info.permissions.clone())
    }

    /// Check if user has specific permission
    pub fn has_permission(&self, api_key: &str, permission: &Permission) -> bool {
        if let Some(key_info) = self.keys.get(api_key) {
            key_info.permissions.contains(permission) || 
            key_info.permissions.contains(&Permission::Admin)
        } else {
            false
        }
    }

    /// Revoke an API key
    pub fn revoke_api_key(&mut self, api_key: &str) -> Result<()> {
        self.keys.remove(api_key)
            .ok_or_else(|| Error::Auth("API key not found".to_string()))?;
        log::info!("Revoked API key: {}", &api_key[..8]);
        Ok(())
    }

    /// Get usage statistics for API key
    pub fn get_key_stats(&self, api_key: &str) -> Option<(u64, Option<Instant>)> {
        self.keys.get(api_key)
            .map(|info| (info.usage_count, info.last_used))
    }

    fn hash_with_secret(&self, data: &str) -> Vec<u8> {
        let mut context = digest::Context::new(&digest::SHA256);
        context.update(self.master_secret.expose_secret().as_bytes());
        context.update(data.as_bytes());
        context.finish().as_ref().to_vec()
    }
}

/// Input sanitization and validation
pub struct InputValidator;

impl InputValidator {
    /// Sanitize text input for FHE operations
    pub fn sanitize_text(input: &str) -> Result<String> {
        // Check length limits
        if input.is_empty() {
            return Err(Error::Validation("Input cannot be empty".to_string()));
        }
        
        if input.len() > 10_000 {
            return Err(Error::Validation("Input too long (max 10,000 characters)".to_string()));
        }

        // Remove control characters but preserve whitespace
        let sanitized: String = input
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect();

        // Check for potential injection patterns
        if Self::contains_suspicious_patterns(&sanitized) {
            return Err(Error::Validation("Input contains suspicious patterns".to_string()));
        }

        // Validate UTF-8
        if !sanitized.is_ascii() {
            // Allow Unicode but validate it's properly formed
            match std::str::from_utf8(sanitized.as_bytes()) {
                Ok(_) => {},
                Err(_) => return Err(Error::Validation("Invalid UTF-8 sequence".to_string())),
            }
        }

        Ok(sanitized)
    }

    /// Validate UUID format
    pub fn validate_uuid(uuid_str: &str) -> Result<Uuid> {
        uuid_str.parse::<Uuid>()
            .map_err(|_| Error::Validation(format!("Invalid UUID format: {}", uuid_str)))
    }

    /// Validate FHE parameters
    pub fn validate_fhe_params(poly_degree: usize, security_level: u8) -> Result<()> {
        if !poly_degree.is_power_of_two() {
            return Err(Error::Validation("Polynomial modulus degree must be power of 2".to_string()));
        }

        if !(1024..=32768).contains(&poly_degree) {
            return Err(Error::Validation("Poly modulus degree must be 1024-32768".to_string()));
        }

        if ![128, 192].contains(&security_level) {
            return Err(Error::Validation("Security level must be 128 or 192".to_string()));
        }

        Ok(())
    }

    /// Validate base64 encoded data
    pub fn validate_base64(data: &str) -> Result<Vec<u8>> {
        BASE64_STANDARD.decode(data)
            .map_err(|e| Error::Validation(format!("Invalid base64 data: {}", e)))
    }

    fn contains_suspicious_patterns(input: &str) -> bool {
        let suspicious_patterns = [
            "javascript:", "data:", "vbscript:", "onload=", "onerror=",
            "<script", "</script>", "eval(", "document.cookie",
            "window.location", "alert(", "confirm(", "prompt(",
            "DROP TABLE", "DELETE FROM", "UPDATE SET", "INSERT INTO",
            "UNION SELECT", "OR 1=1", "AND 1=1", "--", "/*", "*/"
        ];

        let input_lower = input.to_lowercase();
        suspicious_patterns.iter().any(|pattern| input_lower.contains(pattern))
    }
}

/// Security audit logger
pub struct SecurityAuditor;

impl SecurityAuditor {
    /// Log authentication events
    pub fn log_auth_event(event_type: &str, user_id: &str, ip_address: &str, success: bool) {
        log::warn!(
            target: "security_audit",
            "auth_event={} user_id={} ip={} success={} timestamp={}",
            event_type,
            user_id,
            ip_address,
            success,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
    }

    /// Log data access events
    pub fn log_data_access(operation: &str, user_id: &str, resource_id: &str, ip_address: &str) {
        log::info!(
            target: "security_audit",
            "data_access operation={} user_id={} resource_id={} ip={} timestamp={}",
            operation,
            user_id,
            resource_id,
            ip_address,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
    }

    /// Log security violations
    pub fn log_security_violation(violation_type: &str, details: &str, ip_address: &str) {
        log::error!(
            target: "security_audit",
            "security_violation type={} details='{}' ip={} timestamp={}",
            violation_type,
            details,
            ip_address,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
    }

    /// Log privilege escalation attempts
    pub fn log_privilege_escalation(user_id: &str, requested_permission: &str, ip_address: &str) {
        log::error!(
            target: "security_audit", 
            "privilege_escalation user_id={} permission='{}' ip={} timestamp={}",
            user_id,
            requested_permission,
            ip_address,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
    }
}

/// Content Security Policy (CSP) helper
pub struct ContentSecurityPolicy;

impl ContentSecurityPolicy {
    pub fn default_policy() -> &'static str {
        "default-src 'self'; \
         script-src 'self' 'unsafe-inline'; \
         style-src 'self' 'unsafe-inline'; \
         img-src 'self' data:; \
         connect-src 'self'; \
         font-src 'self'; \
         object-src 'none'; \
         media-src 'self'; \
         frame-src 'none'; \
         base-uri 'self'; \
         form-action 'self'"
    }
    
    pub fn api_policy() -> &'static str {
        "default-src 'none'; \
         connect-src 'self'"
    }
}

/// Rate limiting implementation with adaptive thresholds
#[derive(Debug)]
pub struct AdaptiveRateLimiter {
    client_buckets: HashMap<String, TokenBucket>,
    global_bucket: TokenBucket,
    suspicious_ips: HashMap<String, SuspiciousActivity>,
}

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

#[derive(Debug, Clone)]
struct SuspiciousActivity {
    violation_count: u32,
    last_violation: Instant,
    blocked_until: Option<Instant>,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }
}

impl AdaptiveRateLimiter {
    pub fn new(global_capacity: f64, global_refill_rate: f64) -> Self {
        Self {
            client_buckets: HashMap::new(),
            global_bucket: TokenBucket::new(global_capacity, global_refill_rate),
            suspicious_ips: HashMap::new(),
        }
    }

    /// Check if request should be allowed
    pub fn check_request(&mut self, client_ip: &str, tokens_requested: f64) -> Result<bool> {
        // Check if IP is currently blocked
        if let Some(suspicious) = self.suspicious_ips.get(client_ip) {
            if let Some(blocked_until) = suspicious.blocked_until {
                if Instant::now() < blocked_until {
                    SecurityAuditor::log_security_violation(
                        "blocked_ip_request",
                        &format!("Request from blocked IP: {}", client_ip),
                        client_ip
                    );
                    return Ok(false);
                }
            }
        }

        // Check global rate limit first
        if !self.global_bucket.try_consume(tokens_requested) {
            self.record_violation(client_ip, "global_rate_limit");
            return Ok(false);
        }

        // Get or create client bucket with adaptive limits
        let client_capacity = self.calculate_adaptive_capacity(client_ip);
        let client_bucket = self.client_buckets
            .entry(client_ip.to_string())
            .or_insert_with(|| TokenBucket::new(client_capacity, client_capacity / 60.0)); // refill over 1 minute

        if client_bucket.try_consume(tokens_requested) {
            Ok(true)
        } else {
            self.record_violation(client_ip, "client_rate_limit");
            Ok(false)
        }
    }

    fn calculate_adaptive_capacity(&self, client_ip: &str) -> f64 {
        if let Some(suspicious) = self.suspicious_ips.get(client_ip) {
            // Reduce capacity for suspicious IPs
            match suspicious.violation_count {
                0..=2 => 100.0,
                3..=5 => 50.0,
                6..=10 => 20.0,
                _ => 5.0,
            }
        } else {
            100.0 // Default capacity
        }
    }

    fn record_violation(&mut self, client_ip: &str, violation_type: &str) {
        let suspicious = self.suspicious_ips
            .entry(client_ip.to_string())
            .or_insert_with(|| SuspiciousActivity {
                violation_count: 0,
                last_violation: Instant::now(),
                blocked_until: None,
            });

        suspicious.violation_count += 1;
        suspicious.last_violation = Instant::now();

        // Escalate blocking based on violation count
        let block_duration = match suspicious.violation_count {
            1..=3 => None, // Warning only
            4..=6 => Some(Duration::from_secs(60)), // 1 minute
            7..=10 => Some(Duration::from_secs(300)), // 5 minutes
            11..=20 => Some(Duration::from_secs(3600)), // 1 hour
            _ => Some(Duration::from_secs(86400)), // 24 hours
        };

        if let Some(duration) = block_duration {
            suspicious.blocked_until = Some(Instant::now() + duration);
            SecurityAuditor::log_security_violation(
                "ip_temporarily_blocked",
                &format!("IP {} blocked for {} seconds due to {} violations",
                        client_ip, duration.as_secs(), suspicious.violation_count),
                client_ip
            );
        }

        SecurityAuditor::log_security_violation(
            violation_type,
            &format!("Rate limit violation #{}", suspicious.violation_count),
            client_ip
        );
    }

    /// Clean up old entries to prevent memory leaks
    pub fn cleanup_old_entries(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(3600); // Clean up entries older than 1 hour

        self.suspicious_ips.retain(|_, suspicious| {
            suspicious.last_violation > cutoff ||
            suspicious.blocked_until.map_or(false, |until| Instant::now() < until)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_validation() {
        // Valid input
        assert!(InputValidator::sanitize_text("Hello world!").is_ok());

        // Empty input
        assert!(InputValidator::sanitize_text("").is_err());

        // Too long input
        let long_input = "a".repeat(20000);
        assert!(InputValidator::sanitize_text(&long_input).is_err());

        // Suspicious patterns
        assert!(InputValidator::sanitize_text("<script>alert('xss')</script>").is_err());
        assert!(InputValidator::sanitize_text("'; DROP TABLE users; --").is_err());
    }

    #[test]
    fn test_uuid_validation() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(InputValidator::validate_uuid(valid_uuid).is_ok());

        let invalid_uuid = "not-a-uuid";
        assert!(InputValidator::validate_uuid(invalid_uuid).is_err());
    }

    #[test]
    fn test_fhe_params_validation() {
        // Valid params
        assert!(InputValidator::validate_fhe_params(16384, 128).is_ok());

        // Invalid polynomial degree
        assert!(InputValidator::validate_fhe_params(15000, 128).is_err());

        // Invalid security level
        assert!(InputValidator::validate_fhe_params(16384, 100).is_err());
    }

    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10.0, 1.0); // 10 tokens capacity, 1 token/sec refill

        // Should be able to consume tokens initially
        assert!(bucket.try_consume(5.0));
        assert!(bucket.try_consume(5.0));
        
        // Should be empty now
        assert!(!bucket.try_consume(1.0));

        // Wait and refill (simulated)
        std::thread::sleep(std::time::Duration::from_millis(100));
        bucket.refill();
        
        // Should have some tokens now
        assert!(bucket.try_consume(0.1));
    }

    #[test]
    fn test_api_key_generation() {
        let master_secret = SecretString::new("test-secret-key".to_string());
        let mut manager = ApiKeyManager::new(master_secret);

        let permissions = vec![Permission::Encrypt, Permission::Decrypt];
        let api_key = manager.generate_api_key("test-user".to_string(), permissions.clone());

        assert!(!api_key.is_empty());
        assert!(manager.validate_api_key(&api_key).is_ok());
        assert!(manager.has_permission(&api_key, &Permission::Encrypt));
        assert!(!manager.has_permission(&api_key, &Permission::Admin));
    }
}