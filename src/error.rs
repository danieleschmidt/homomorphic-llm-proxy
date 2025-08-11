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
