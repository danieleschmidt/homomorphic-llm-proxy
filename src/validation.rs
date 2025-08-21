//! Enhanced validation and input sanitization for FHE operations

use crate::error::{Error, Result};
use crate::fhe::FheParams;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Comprehensive input validation framework
pub struct ValidationFramework {
    rules: HashMap<String, ValidationRule>,
    max_input_size: usize,
    allowed_patterns: Vec<regex::Regex>,
    blocked_patterns: Vec<regex::Regex>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub required: bool,
    pub pattern: Option<String>,
    pub custom_validator: Option<fn(&str) -> bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub sanitized_input: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub error_type: String,
    pub message: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl Default for ValidationFramework {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationFramework {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            max_input_size: 100_000, // 100KB max input
            allowed_patterns: vec![],
            blocked_patterns: vec![],
        }
    }

    pub fn with_fhe_defaults() -> Result<Self> {
        let mut framework = Self::new();

        // Add default FHE validation rules
        framework.add_rule(ValidationRule {
            field_name: "plaintext".to_string(),
            min_length: Some(1),
            max_length: Some(10_000),
            required: true,
            pattern: None,
            custom_validator: Some(validate_fhe_plaintext),
        });

        framework.add_rule(ValidationRule {
            field_name: "client_id".to_string(),
            min_length: Some(36),
            max_length: Some(36),
            required: true,
            pattern: Some(
                r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
                    .to_string(),
            ),
            custom_validator: None,
        });

        Ok(framework)
    }

    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.insert(rule.field_name.clone(), rule);
    }

    /// Comprehensive input validation with detailed reporting
    pub fn validate_input(&self, field: &str, input: &str) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check if field has validation rules
        let rule = match self.rules.get(field) {
            Some(rule) => rule,
            None => {
                warnings.push(format!("No validation rules found for field: {}", field));
                return ValidationReport {
                    is_valid: true,
                    errors,
                    warnings,
                    sanitized_input: Some(input.to_string()),
                };
            }
        };

        // Required field check
        if rule.required && input.is_empty() {
            errors.push(ValidationError {
                field: field.to_string(),
                error_type: "required".to_string(),
                message: format!("Field '{}' is required", field),
                severity: ErrorSeverity::Error,
            });
        }

        // Length validation
        if let Some(min_len) = rule.min_length {
            if input.len() < min_len {
                errors.push(ValidationError {
                    field: field.to_string(),
                    error_type: "min_length".to_string(),
                    message: format!("Field '{}' must be at least {} characters", field, min_len),
                    severity: ErrorSeverity::Error,
                });
            }
        }

        if let Some(max_len) = rule.max_length {
            if input.len() > max_len {
                errors.push(ValidationError {
                    field: field.to_string(),
                    error_type: "max_length".to_string(),
                    message: format!("Field '{}' must not exceed {} characters", field, max_len),
                    severity: ErrorSeverity::Error,
                });
            }
        }

        // Pattern validation
        if let Some(pattern) = &rule.pattern {
            match regex::Regex::new(pattern) {
                Ok(regex) => {
                    if !regex.is_match(input) {
                        errors.push(ValidationError {
                            field: field.to_string(),
                            error_type: "pattern".to_string(),
                            message: format!("Field '{}' does not match required pattern", field),
                            severity: ErrorSeverity::Error,
                        });
                    }
                }
                Err(e) => {
                    warnings.push(format!("Invalid regex pattern for field '{}': {}", field, e));
                }
            }
        }

        // Custom validator
        if let Some(validator) = rule.custom_validator {
            if !validator(input) {
                errors.push(ValidationError {
                    field: field.to_string(),
                    error_type: "custom".to_string(),
                    message: format!("Field '{}' failed custom validation", field),
                    severity: ErrorSeverity::Error,
                });
            }
        }

        // Security checks - blocked patterns
        for blocked_pattern in &self.blocked_patterns {
            if blocked_pattern.is_match(input) {
                errors.push(ValidationError {
                    field: field.to_string(),
                    error_type: "security".to_string(),
                    message: format!("Field '{}' contains blocked content", field),
                    severity: ErrorSeverity::Critical,
                });
            }
        }

        // Sanitize input
        let sanitized_input = if errors.is_empty() {
            Some(self.sanitize_input(input))
        } else {
            None
        };

        ValidationReport {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            sanitized_input,
        }
    }

    /// Sanitize input by removing/replacing dangerous characters
    fn sanitize_input(&self, input: &str) -> String {
        input
            .chars()
            .filter(|c| {
                c.is_ascii()
                && (!c.is_control() || c.is_whitespace())
                && *c != '\0' // Remove null bytes
                && !matches!(*c, '\x01'..='\x08' | '\x0B'..='\x0C' | '\x0E'..='\x1F' | '\x7F')
            })
            .collect()
    }

    /// Validate FHE parameters
    pub fn validate_fhe_params(&self, params: &FheParams) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate poly modulus degree
        if params.poly_modulus_degree == 0 {
            errors.push(ValidationError {
                field: "poly_modulus_degree".to_string(),
                error_type: "invalid_value".to_string(),
                message: "Polynomial modulus degree cannot be zero".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        if !params.poly_modulus_degree.is_power_of_two() {
            warnings.push("Polynomial modulus degree should be a power of two for optimal performance".to_string());
        }

        // Validate security level
        if params.security_level < 80 {
            errors.push(ValidationError {
                field: "security_level".to_string(),
                error_type: "insecure".to_string(),
                message: format!("Security level {} is too low (minimum 80 bits)", params.security_level),
                severity: ErrorSeverity::Critical,
            });
        }

        if params.security_level > 192 {
            warnings.push(format!("Security level {} may impact performance significantly", params.security_level));
        }

        // Validate coefficient modulus
        if params.coeff_modulus_bits.is_empty() {
            errors.push(ValidationError {
                field: "coeff_modulus_bits".to_string(),
                error_type: "required".to_string(),
                message: "Coefficient modulus bits cannot be empty".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        ValidationReport {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            sanitized_input: None,
        }
    }

    /// Validate base64 encoded ciphertext
    pub fn validate_ciphertext_data(&self, data: &str) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check if it's valid base64
        match base64::prelude::BASE64_STANDARD.decode(data) {
            Ok(decoded) => {
                if decoded.len() < 32 {
                    warnings.push("Ciphertext data seems unusually small".to_string());
                }
                if decoded.len() > 10_000_000 { // 10MB limit
                    errors.push(ValidationError {
                        field: "ciphertext_data".to_string(),
                        error_type: "too_large".to_string(),
                        message: "Ciphertext data exceeds maximum size (10MB)".to_string(),
                        severity: ErrorSeverity::Error,
                    });
                }
            }
            Err(e) => {
                errors.push(ValidationError {
                    field: "ciphertext_data".to_string(),
                    error_type: "invalid_encoding".to_string(),
                    message: format!("Invalid base64 encoding: {}", e),
                    severity: ErrorSeverity::Error,
                });
            }
        }

        ValidationReport {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            sanitized_input: None,
        }
    }

    /// Validate UUID string
    pub fn validate_uuid(&self, uuid_str: &str) -> ValidationReport {
        let mut errors = Vec::new();
        
        match Uuid::parse_str(uuid_str) {
            Ok(_) => {
                ValidationReport {
                    is_valid: true,
                    errors,
                    warnings: vec![],
                    sanitized_input: Some(uuid_str.to_lowercase()),
                }
            }
            Err(e) => {
                errors.push(ValidationError {
                    field: "uuid".to_string(),
                    error_type: "invalid_format".to_string(),
                    message: format!("Invalid UUID format: {}", e),
                    severity: ErrorSeverity::Error,
                });
                
                ValidationReport {
                    is_valid: false,
                    errors,
                    warnings: vec![],
                    sanitized_input: None,
                }
            }
        }
    }

    /// Detect potentially malicious patterns in input
    pub fn detect_security_threats(&self, input: &str) -> Vec<String> {
        let mut threats = Vec::new();
        let lower_input = input.to_lowercase();

        // SQL injection patterns
        let sql_patterns = [
            "select ", "union ", "insert ", "delete ", "drop ", "exec ", "script",
            "alter ", "create ", "truncate ", "grant ", "revoke ",
        ];

        for pattern in &sql_patterns {
            if lower_input.contains(pattern) {
                threats.push(format!("Potential SQL injection: {}", pattern));
            }
        }

        // XSS patterns
        let xss_patterns = [
            "<script", "javascript:", "onload=", "onerror=", "onclick=",
            "onmouseover=", "onfocus=", "onblur=", "onchange=", "onsubmit=",
        ];

        for pattern in &xss_patterns {
            if lower_input.contains(pattern) {
                threats.push(format!("Potential XSS: {}", pattern));
            }
        }

        // Command injection patterns
        let cmd_patterns = ["; ", "| ", "& ", "$(", "`", "exec(", "system(", "eval("];

        for pattern in &cmd_patterns {
            if lower_input.contains(pattern) {
                threats.push(format!("Potential command injection: {}", pattern));
            }
        }

        // Path traversal patterns
        let path_patterns = ["../", "..\\", "/etc/", "/proc/", "/sys/", "c:\\"];

        for pattern in &path_patterns {
            if lower_input.contains(pattern) {
                threats.push(format!("Potential path traversal: {}", pattern));
            }
        }

        threats
    }
}

/// Custom validator for FHE plaintext
fn validate_fhe_plaintext(input: &str) -> bool {
    // Check for valid UTF-8
    if !input.is_ascii() && std::str::from_utf8(input.as_bytes()).is_err() {
        return false;
    }

    // Check for reasonable content
    let char_count = input.chars().count();
    let byte_count = input.len();

    // Detect potential binary data disguised as text
    if byte_count > char_count * 4 {
        return false; // Likely binary data
    }

    // Check for excessive whitespace (potential padding attack)
    let whitespace_ratio =
        input.chars().filter(|c| c.is_whitespace()).count() as f64 / char_count as f64;
    if whitespace_ratio > 0.8 {
        return false;
    }

    true
}

/// Batch validation for multiple inputs
pub struct BatchValidator {
    framework: ValidationFramework,
    max_batch_size: usize,
}

impl BatchValidator {
    pub fn new(framework: ValidationFramework) -> Self {
        Self {
            framework,
            max_batch_size: 100,
        }
    }

    pub fn validate_batch(&self, inputs: Vec<(String, String)>) -> Result<Vec<ValidationReport>> {
        if inputs.len() > self.max_batch_size {
            return Err(Error::Validation(format!(
                "Batch size {} exceeds limit {}",
                inputs.len(),
                self.max_batch_size
            )));
        }

        let mut results = Vec::with_capacity(inputs.len());

        for (field, value) in inputs {
            let report = self.framework.validate_input(&field, &value);
            results.push(report);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_framework() {
        let framework = ValidationFramework::with_fhe_defaults().unwrap();

        // Test valid plaintext
        let report = framework.validate_input("plaintext", "Hello, world!");
        assert!(report.is_valid);
        assert!(report.errors.is_empty());

        // Test empty plaintext (should fail)
        let report = framework.validate_input("plaintext", "");
        assert!(!report.is_valid);
        assert!(!report.errors.is_empty());

        // Test too long plaintext
        let long_text = "a".repeat(20000);
        let report = framework.validate_input("plaintext", &long_text);
        assert!(!report.is_valid);
    }

    #[test]
    fn test_security_threat_detection() {
        let framework = ValidationFramework::new();

        let threats = framework.detect_security_threats("<script>alert('xss')</script>");
        assert!(!threats.is_empty());
        assert!(threats.iter().any(|t| t.contains("XSS")));

        let threats = framework.detect_security_threats("'; DROP TABLE users; --");
        assert!(!threats.is_empty());
        assert!(threats.iter().any(|t| t.contains("SQL injection")));
    }

    #[test]
    fn test_input_sanitization() {
        let framework = ValidationFramework::new();

        let input = "Hello\x00World\x1F\nValid text";
        let sanitized = framework.sanitize_input(input);
        assert!(!sanitized.contains('\x00'));
        assert!(!sanitized.contains('\x1F'));
    }

    #[test]
    fn test_uuid_validation() {
        let framework = ValidationFramework::new();

        // Valid UUID
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let report = framework.validate_uuid(valid_uuid);
        assert!(report.is_valid);

        // Invalid UUID
        let invalid_uuid = "not-a-uuid";
        let report = framework.validate_uuid(invalid_uuid);
        assert!(!report.is_valid);
    }

    #[test]
    fn test_base64_validation() {
        let framework = ValidationFramework::new();

        // Valid base64
        let valid_b64 = "SGVsbG8gV29ybGQ=";
        let report = framework.validate_ciphertext_data(valid_b64);
        assert!(report.is_valid);

        // Invalid base64
        let invalid_b64 = "This is not base64!";
        let report = framework.validate_ciphertext_data(invalid_b64);
        assert!(!report.is_valid);
    }
}