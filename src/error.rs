//! Error types for the FHE LLM Proxy

use thiserror::Error as ThisError;

/// Result type for FHE operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur in the FHE proxy
#[derive(Debug, ThisError)]
pub enum Error {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),

    /// FHE operation errors
    #[error("FHE error: {0}")]
    Fhe(String),

    /// Provider API errors
    #[error("Provider error: {0}")]
    Provider(String),

    /// HTTP server errors
    #[error("HTTP error: {0}")]
    Http(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Request errors
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Privacy budget errors
    #[error("Privacy budget error: {0}")]
    PrivacyBudget(String),

    /// Timeout errors
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// Security violations
    #[error("Security violation: {0}")]
    Security(String),

    /// Resource exhaustion
    #[error("Resource exhausted: {0}")]
    ResourceExhaustion(String),

    /// Concurrent access errors
    #[error("Concurrency error: {0}")]
    Concurrency(String),

    /// Data corruption detected
    #[error("Data corruption: {0}")]
    DataCorruption(String),

    /// Encryption/Decryption errors
    #[error("Cryptographic error: {0}")]
    Cryptographic(String),
}

impl Error {
    /// Get error severity level for monitoring
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Error::Config(_) => ErrorSeverity::High,
            Error::Network(_) => ErrorSeverity::Medium,
            Error::Fhe(_) => ErrorSeverity::High,
            Error::Provider(_) => ErrorSeverity::Medium,
            Error::Http(_) => ErrorSeverity::Low,
            Error::Serialization(_) => ErrorSeverity::Low,
            Error::Request(_) => ErrorSeverity::Low,
            Error::Auth(_) => ErrorSeverity::High,
            Error::Validation(_) => ErrorSeverity::Medium,
            Error::RateLimit(_) => ErrorSeverity::Low,
            Error::PrivacyBudget(_) => ErrorSeverity::High,
            Error::Timeout(_) => ErrorSeverity::Medium,
            Error::Internal(_) => ErrorSeverity::Critical,
            Error::Security(_) => ErrorSeverity::Critical,
            Error::ResourceExhaustion(_) => ErrorSeverity::High,
            Error::Concurrency(_) => ErrorSeverity::Medium,
            Error::DataCorruption(_) => ErrorSeverity::Critical,
            Error::Cryptographic(_) => ErrorSeverity::Critical,
        }
    }

    /// Get error category for metrics
    pub fn category(&self) -> &'static str {
        match self {
            Error::Config(_) => "configuration",
            Error::Network(_) | Error::Http(_) | Error::Request(_) => "network",
            Error::Fhe(_) | Error::Cryptographic(_) => "cryptography",
            Error::Provider(_) => "external_service",
            Error::Serialization(_) => "data_format",
            Error::Auth(_) | Error::Security(_) => "security",
            Error::Validation(_) => "validation",
            Error::RateLimit(_) => "rate_limiting",
            Error::PrivacyBudget(_) => "privacy",
            Error::Timeout(_) => "performance",
            Error::Internal(_) => "internal",
            Error::ResourceExhaustion(_) => "resources",
            Error::Concurrency(_) => "concurrency",
            Error::DataCorruption(_) => "data_integrity",
        }
    }

    /// Check if error should trigger immediate alert
    pub fn requires_immediate_alert(&self) -> bool {
        matches!(
            self.severity(),
            ErrorSeverity::Critical | ErrorSeverity::High
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Error context for enhanced debugging
#[derive(Debug)]
pub struct ErrorContext {
    pub error: Error,
    pub context: String,
    pub timestamp: std::time::SystemTime,
    pub client_id: Option<uuid::Uuid>,
    pub request_id: Option<String>,
    pub stack_trace: Option<String>,
}

impl ErrorContext {
    pub fn new(error: Error, context: String) -> Self {
        Self {
            error,
            context,
            timestamp: std::time::SystemTime::now(),
            client_id: None,
            request_id: None,
            stack_trace: None,
        }
    }

    pub fn with_client_id(mut self, client_id: uuid::Uuid) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Config(err.to_string())
    }
}

impl From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Self {
        Error::Config(err.to_string())
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Error::Validation(format!("Base64 decode error: {}", err))
    }
}
