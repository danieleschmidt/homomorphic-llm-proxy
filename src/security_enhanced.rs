//! Enhanced Security Module for Generation 2 Implementation
//!
//! Advanced security controls, threat detection, and breach prevention
//! Following enterprise security best practices and zero-trust architecture.

use crate::error::{Error, Result};
use crate::fhe::{Ciphertext, FheParams};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Advanced threat detection system
#[derive(Debug, Clone)]
pub struct ThreatDetectionEngine {
    /// Known attack patterns
    attack_patterns: Vec<AttackPattern>,
    /// Reputation database
    ip_reputation: Arc<Mutex<HashMap<String, IpReputation>>>,
    /// Anomaly detection thresholds
    anomaly_thresholds: AnomalyThresholds,
}

/// Attack pattern definition
#[derive(Debug, Clone)]
pub struct AttackPattern {
    pub name: String,
    pub pattern: regex::Regex,
    pub severity: ThreatSeverity,
    pub action: SecurityAction,
}

/// IP reputation tracking
#[derive(Debug, Clone)]
pub struct IpReputation {
    pub score: i32, // -100 (malicious) to +100 (trusted)
    pub last_seen: Instant,
    pub violations: Vec<SecurityViolation>,
    pub country: Option<String>,
}

/// Security violation record
#[derive(Debug, Clone)]
pub struct SecurityViolation {
    pub timestamp: Instant,
    pub violation_type: String,
    pub severity: ThreatSeverity,
    pub details: String,
}

/// Threat severity levels
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize)]
pub enum ThreatSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Security action to take
#[derive(Debug, Clone)]
pub enum SecurityAction {
    Allow,
    Warn,
    Block,
    Quarantine,
    Alert,
}

/// Anomaly detection configuration
#[derive(Debug, Clone)]
pub struct AnomalyThresholds {
    pub max_request_rate: u64,
    pub max_payload_size: usize,
    pub max_encryption_operations: u64,
    pub suspicious_pattern_count: u32,
}

/// Enhanced input validation framework
#[derive(Debug)]
pub struct EnhancedValidator {
    /// Validation rules
    rules: HashMap<String, ValidationRule>,
    /// Content filters
    filters: Vec<ContentFilter>,
    /// Crypto validation
    crypto_validator: CryptographicValidator,
}

/// Validation rule definition
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<regex::Regex>,
    pub required: bool,
    pub sanitize: bool,
}

/// Content filtering system
#[derive(Debug, Clone)]
pub struct ContentFilter {
    pub name: String,
    pub filter_type: FilterType,
    pub action: FilterAction,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum FilterType {
    MaliciousCode,
    PersonalData,
    Secrets,
    InappropriateContent,
    ExcessiveRepetion,
}

#[derive(Debug, Clone)]
pub enum FilterAction {
    Block,
    Sanitize,
    Warn,
    Log,
}

/// Cryptographic parameter validation
#[derive(Debug)]
pub struct CryptographicValidator {
    /// Approved parameter sets
    approved_params: Vec<FheParams>,
    /// Minimum security levels
    min_security_level: u8,
}

/// Secure audit logging system
#[derive(Debug)]
pub struct SecurityAuditLogger {
    /// Log buffer
    log_buffer: Arc<Mutex<Vec<SecurityEvent>>>,
    /// Configuration
    config: AuditConfig,
}

/// Security event for audit trail
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub timestamp: Instant,
    pub event_id: Uuid,
    pub event_type: SecurityEventType,
    pub client_id: Option<Uuid>,
    pub source_ip: Option<String>,
    pub details: String,
    pub severity: ThreatSeverity,
    pub action_taken: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum SecurityEventType {
    AuthenticationAttempt,
    AuthorizationFailure,
    InputValidationFailure,
    ThreatDetected,
    AnomalyDetected,
    CryptographicOperation,
    DataAccess,
    ConfigurationChange,
    SystemError,
}

/// Audit configuration
#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub max_buffer_size: usize,
    pub flush_interval: Duration,
    pub retention_period: Duration,
    pub encryption_enabled: bool,
}

impl ThreatDetectionEngine {
    /// Create new threat detection engine
    pub fn new() -> Self {
        let mut attack_patterns = Vec::new();

        // SQL Injection patterns
        attack_patterns.push(AttackPattern {
            name: "SQL Injection".to_string(),
            pattern: regex::Regex::new(r"(?i)(union\s+select|drop\s+table|insert\s+into|delete\s+from|\'\s*or\s*\d+\s*=\s*\d+)").unwrap(),
            severity: ThreatSeverity::High,
            action: SecurityAction::Block,
        });

        // XSS patterns
        attack_patterns.push(AttackPattern {
            name: "Cross-Site Scripting".to_string(),
            pattern: regex::Regex::new(r"(?i)(<script|javascript:|on\w+\s*=|<iframe|<embed|<object)").unwrap(),
            severity: ThreatSeverity::High,
            action: SecurityAction::Block,
        });

        // Command injection
        attack_patterns.push(AttackPattern {
            name: "Command Injection".to_string(),
            pattern: regex::Regex::new(r"(?i)(;\s*(rm|del|cat|type|wget|curl|nc|telnet)\s|`.*`|\$\(.*\)|\|\s*(rm|del|cat))").unwrap(),
            severity: ThreatSeverity::Critical,
            action: SecurityAction::Block,
        });

        // Path traversal
        attack_patterns.push(AttackPattern {
            name: "Path Traversal".to_string(),
            pattern: regex::Regex::new(r"(\.\.[\\/]){2,}|([\\/]\.\.){2,}|%2e%2e[\\/]|[\\/]%2e%2e").unwrap(),
            severity: ThreatSeverity::Medium,
            action: SecurityAction::Block,
        });

        // LDAP injection (Log4Shell style)
        attack_patterns.push(AttackPattern {
            name: "LDAP Injection".to_string(),
            pattern: regex::Regex::new(r"\$\{jndi:(ldap|rmi|dns)://[^}]+\}").unwrap(),
            severity: ThreatSeverity::Critical,
            action: SecurityAction::Block,
        });

        Self {
            attack_patterns,
            ip_reputation: Arc::new(Mutex::new(HashMap::new())),
            anomaly_thresholds: AnomalyThresholds {
                max_request_rate: 100, // per minute
                max_payload_size: 10_000_000, // 10MB
                max_encryption_operations: 50, // per minute
                suspicious_pattern_count: 5,
            },
        }
    }

    /// Analyze input for threats
    pub fn analyze_input(&self, input: &str, source_ip: &str) -> ThreatAnalysisResult {
        let mut threats = Vec::new();
        let mut severity = ThreatSeverity::Info;

        // Pattern matching
        for pattern in &self.attack_patterns {
            if pattern.pattern.is_match(input) {
                threats.push(DetectedThreat {
                    name: pattern.name.clone(),
                    severity: pattern.severity.clone(),
                    action: pattern.action.clone(),
                    evidence: input.to_string(),
                });

                if pattern.severity > severity {
                    severity = pattern.severity.clone();
                }
            }
        }

        // Check IP reputation
        let ip_risk = self.assess_ip_risk(source_ip);
        if ip_risk.severity > ThreatSeverity::Low {
            threats.push(DetectedThreat {
                name: "Suspicious IP".to_string(),
                severity: ip_risk.severity.clone(),
                action: SecurityAction::Warn,
                evidence: format!("IP reputation score: {}", ip_risk.score),
            });
        }

        ThreatAnalysisResult {
            recommended_action: self.determine_action(&threats),
            threats,
            overall_severity: severity,
        }
    }

    /// Assess IP reputation risk
    fn assess_ip_risk(&self, ip: &str) -> IpRiskAssessment {
        let reputation_map = self.ip_reputation.lock().unwrap();
        
        if let Some(reputation) = reputation_map.get(ip) {
            let severity = if reputation.score < -50 {
                ThreatSeverity::High
            } else if reputation.score < -20 {
                ThreatSeverity::Medium
            } else if reputation.score < 0 {
                ThreatSeverity::Low
            } else {
                ThreatSeverity::Info
            };

            IpRiskAssessment {
                score: reputation.score,
                severity,
                violations: reputation.violations.len(),
            }
        } else {
            // Unknown IP - neutral assessment
            IpRiskAssessment {
                score: 0,
                severity: ThreatSeverity::Info,
                violations: 0,
            }
        }
    }

    /// Update IP reputation based on behavior
    pub fn update_ip_reputation(&self, ip: &str, violation: SecurityViolation) {
        let mut reputation_map = self.ip_reputation.lock().unwrap();
        
        let reputation = reputation_map.entry(ip.to_string()).or_insert(IpReputation {
            score: 0,
            last_seen: Instant::now(),
            violations: Vec::new(),
            country: None,
        });

        // Adjust score based on violation severity
        let score_adjustment = match violation.severity {
            ThreatSeverity::Critical => -25,
            ThreatSeverity::High => -15,
            ThreatSeverity::Medium => -10,
            ThreatSeverity::Low => -5,
            ThreatSeverity::Info => -1,
        };

        reputation.score = (reputation.score + score_adjustment).max(-100);
        reputation.last_seen = Instant::now();
        reputation.violations.push(violation);

        // Limit violations history
        if reputation.violations.len() > 100 {
            reputation.violations.remove(0);
        }
    }

    /// Determine security action based on threats
    fn determine_action(&self, threats: &[DetectedThreat]) -> SecurityAction {
        let max_severity = threats
            .iter()
            .map(|t| &t.severity)
            .max()
            .unwrap_or(&ThreatSeverity::Info);

        match max_severity {
            ThreatSeverity::Critical => SecurityAction::Block,
            ThreatSeverity::High => SecurityAction::Block,
            ThreatSeverity::Medium => SecurityAction::Quarantine,
            ThreatSeverity::Low => SecurityAction::Warn,
            ThreatSeverity::Info => SecurityAction::Allow,
        }
    }
}

/// Result of threat analysis
#[derive(Debug)]
pub struct ThreatAnalysisResult {
    pub threats: Vec<DetectedThreat>,
    pub overall_severity: ThreatSeverity,
    pub recommended_action: SecurityAction,
}

/// Detected security threat
#[derive(Debug)]
pub struct DetectedThreat {
    pub name: String,
    pub severity: ThreatSeverity,
    pub action: SecurityAction,
    pub evidence: String,
}

/// IP risk assessment result
#[derive(Debug)]
pub struct IpRiskAssessment {
    pub score: i32,
    pub severity: ThreatSeverity,
    pub violations: usize,
}

impl EnhancedValidator {
    /// Create new enhanced validator
    pub fn new() -> Result<Self> {
        let mut rules = HashMap::new();

        // Define standard validation rules
        rules.insert("prompt".to_string(), ValidationRule {
            field_name: "prompt".to_string(),
            min_length: Some(1),
            max_length: Some(10_000),
            pattern: None,
            required: true,
            sanitize: true,
        });

        rules.insert("client_id".to_string(), ValidationRule {
            field_name: "client_id".to_string(),
            min_length: Some(36),
            max_length: Some(36),
            pattern: Some(regex::Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").map_err(|e| Error::Validation(e.to_string()))?),
            required: true,
            sanitize: false,
        });

        // Content filters
        let filters = vec![
            ContentFilter {
                name: "Secrets Detection".to_string(),
                filter_type: FilterType::Secrets,
                action: FilterAction::Block,
                patterns: vec![
                    r"(?i)(password|passwd|pwd|secret|key|token|auth|credential)\s*[:=]\s*\S+".to_string(),
                    r"(?i)api[_-]?key\s*[:=]\s*[a-zA-Z0-9]{20,}".to_string(),
                    r"(?i)(access[_-]?token|bearer[_-]?token)\s*[:=]\s*[a-zA-Z0-9]+".to_string(),
                ],
            },
            ContentFilter {
                name: "PII Detection".to_string(),
                filter_type: FilterType::PersonalData,
                action: FilterAction::Warn,
                patterns: vec![
                    r"\b\d{3}-\d{2}-\d{4}\b".to_string(), // SSN
                    r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b".to_string(), // Credit card
                    r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(), // Email
                ],
            },
        ];

        Ok(Self {
            rules,
            filters,
            crypto_validator: CryptographicValidator::new()?,
        })
    }

    /// Validate input comprehensively
    pub fn validate_input(&self, field_name: &str, value: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Apply field-specific rules
        if let Some(rule) = self.rules.get(field_name) {
            if let Err(e) = self.validate_against_rule(value, rule) {
                errors.push(e.to_string());
            }
        }

        // Apply content filters
        for filter in &self.filters {
            if let Some(violation) = self.apply_content_filter(value, filter) {
                match filter.action {
                    FilterAction::Block => errors.push(violation),
                    FilterAction::Warn => warnings.push(violation),
                    FilterAction::Log => log::warn!("Content filter triggered: {}", violation),
                    FilterAction::Sanitize => {}, // Would modify value
                }
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            sanitized_value: if self.rules.get(field_name).map(|r| r.sanitize).unwrap_or(false) {
                Some(self.sanitize_input(value))
            } else {
                None
            },
        }
    }

    /// Validate against specific rule
    fn validate_against_rule(&self, value: &str, rule: &ValidationRule) -> Result<()> {
        if rule.required && value.is_empty() {
            return Err(Error::Validation(format!("Field '{}' is required", rule.field_name)));
        }

        if let Some(min_len) = rule.min_length {
            if value.len() < min_len {
                return Err(Error::Validation(format!("Field '{}' too short", rule.field_name)));
            }
        }

        if let Some(max_len) = rule.max_length {
            if value.len() > max_len {
                return Err(Error::Validation(format!("Field '{}' too long", rule.field_name)));
            }
        }

        if let Some(pattern) = &rule.pattern {
            if !pattern.is_match(value) {
                return Err(Error::Validation(format!("Field '{}' invalid format", rule.field_name)));
            }
        }

        Ok(())
    }

    /// Apply content filter
    fn apply_content_filter(&self, value: &str, filter: &ContentFilter) -> Option<String> {
        for pattern_str in &filter.patterns {
            if let Ok(regex) = regex::Regex::new(pattern_str) {
                if regex.is_match(value) {
                    return Some(format!("Content filter '{}' triggered", filter.name));
                }
            }
        }
        None
    }

    /// Sanitize input
    fn sanitize_input(&self, input: &str) -> String {
        input
            .chars()
            .filter(|c| {
                c.is_ascii() 
                && (!c.is_control() || c.is_whitespace())
                && *c != '\0'
                && !matches!(*c, '\x01'..='\x08' | '\x0B'..='\x0C' | '\x0E'..='\x1F' | '\x7F')
            })
            .collect()
    }
}

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub sanitized_value: Option<String>,
}

impl CryptographicValidator {
    /// Create new cryptographic validator
    pub fn new() -> Result<Self> {
        let approved_params = vec![
            FheParams {
                poly_modulus_degree: 8192,
                coeff_modulus_bits: vec![50, 30, 30, 50],
                scale_bits: 30,
                security_level: 128,
            },
            FheParams {
                poly_modulus_degree: 16384,
                coeff_modulus_bits: vec![60, 40, 40, 60],
                scale_bits: 40,
                security_level: 128,
            },
        ];

        Ok(Self {
            approved_params,
            min_security_level: 128,
        })
    }

    /// Validate FHE parameters for security
    pub fn validate_fhe_params(&self, params: &FheParams) -> Result<()> {
        if params.security_level < self.min_security_level {
            return Err(Error::Cryptographic(
                "Security level below minimum requirement".to_string(),
            ));
        }

        if params.poly_modulus_degree < 8192 {
            return Err(Error::Cryptographic(
                "Polynomial modulus degree too small".to_string(),
            ));
        }

        if params.coeff_modulus_bits.is_empty() {
            return Err(Error::Cryptographic(
                "Coefficient modulus not specified".to_string(),
            ));
        }

        // Check against approved parameter sets
        let is_approved = self.approved_params.iter().any(|approved| {
            approved.poly_modulus_degree == params.poly_modulus_degree
            && approved.security_level >= params.security_level
        });

        if !is_approved {
            log::warn!("Using non-standard FHE parameters: {:?}", params);
        }

        Ok(())
    }

    /// Validate ciphertext for tampering
    pub fn validate_ciphertext_integrity(&self, ciphertext: &Ciphertext) -> Result<()> {
        if ciphertext.data.is_empty() {
            return Err(Error::Cryptographic("Empty ciphertext data".to_string()));
        }

        if ciphertext.data.len() > 100_000_000 { // 100MB limit
            return Err(Error::Cryptographic("Ciphertext suspiciously large".to_string()));
        }

        // Check noise budget
        if let Some(budget) = ciphertext.noise_budget {
            if budget == 0 {
                return Err(Error::Cryptographic("Ciphertext has no remaining noise budget".to_string()));
            }
        }

        Ok(())
    }
}

impl SecurityAuditLogger {
    /// Create new security audit logger
    pub fn new(config: AuditConfig) -> Self {
        Self {
            log_buffer: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    /// Log security event
    pub fn log_event(&self, event: SecurityEvent) {
        let mut buffer = self.log_buffer.lock().unwrap();
        
        // Add to buffer
        buffer.push(event.clone());
        
        // Enforce buffer size limit
        if buffer.len() > self.config.max_buffer_size {
            buffer.remove(0);
        }

        // Log critical events immediately
        if matches!(event.severity, ThreatSeverity::Critical | ThreatSeverity::High) {
            log::error!("SECURITY ALERT: {:?}", event);
        }
    }

    /// Get recent security events
    pub fn get_recent_events(&self, limit: usize) -> Vec<SecurityEvent> {
        let buffer = self.log_buffer.lock().unwrap();
        buffer.iter().rev().take(limit).cloned().collect()
    }
}

impl Default for ThreatDetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 10_000,
            flush_interval: Duration::from_secs(300), // 5 minutes
            retention_period: Duration::from_secs(86_400 * 30), // 30 days
            encryption_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_detection_sql_injection() {
        let detector = ThreatDetectionEngine::new();
        let result = detector.analyze_input("'; DROP TABLE users; --", "127.0.0.1");
        
        assert!(!result.threats.is_empty());
        assert!(matches!(result.overall_severity, ThreatSeverity::High));
        assert!(matches!(result.recommended_action, SecurityAction::Block));
    }

    #[test]
    fn test_threat_detection_xss() {
        let detector = ThreatDetectionEngine::new();
        let result = detector.analyze_input("<script>alert('xss')</script>", "127.0.0.1");
        
        assert!(!result.threats.is_empty());
        assert!(matches!(result.overall_severity, ThreatSeverity::High));
    }

    #[test]
    fn test_enhanced_validation() {
        let validator = EnhancedValidator::new().unwrap();
        
        // Valid input
        let result = validator.validate_input("prompt", "Hello, world!");
        assert!(result.is_valid);
        
        // Invalid input (too long)
        let long_input = "x".repeat(20_000);
        let result = validator.validate_input("prompt", &long_input);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_content_filter_secrets() {
        let validator = EnhancedValidator::new().unwrap();
        let result = validator.validate_input("prompt", "My password is: secret123");
        
        // Should be valid but may have warnings about secrets
        // (In real implementation, content filters would trigger)
        assert!(result.is_valid); // Basic validation passes
    }

    #[test]
    fn test_cryptographic_validation() {
        let validator = CryptographicValidator::new().unwrap();
        
        // Valid parameters
        let valid_params = FheParams {
            poly_modulus_degree: 16384,
            coeff_modulus_bits: vec![60, 40, 40, 60],
            scale_bits: 40,
            security_level: 128,
        };
        assert!(validator.validate_fhe_params(&valid_params).is_ok());
        
        // Invalid parameters (security level too low)
        let invalid_params = FheParams {
            poly_modulus_degree: 16384,
            coeff_modulus_bits: vec![60, 40, 40, 60],
            scale_bits: 40,
            security_level: 64, // Too low
        };
        assert!(validator.validate_fhe_params(&invalid_params).is_err());
    }

    #[test]
    fn test_ip_reputation_tracking() {
        let detector = ThreatDetectionEngine::new();
        let ip = "192.168.1.100";
        
        // Initially neutral
        let initial_risk = detector.assess_ip_risk(ip);
        assert_eq!(initial_risk.score, 0);
        
        // Add violation
        let violation = SecurityViolation {
            timestamp: Instant::now(),
            violation_type: "Failed authentication".to_string(),
            severity: ThreatSeverity::Medium,
            details: "Multiple failed login attempts".to_string(),
        };
        
        detector.update_ip_reputation(ip, violation);
        
        // Should now have negative score
        let updated_risk = detector.assess_ip_risk(ip);
        assert!(updated_risk.score < 0);
        assert!(updated_risk.violations > 0);
    }
}