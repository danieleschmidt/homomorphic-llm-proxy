//! Error types for the FHE LLM Proxy

use std::fmt;

/// Result type for FHE operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur in the FHE proxy
#[derive(Debug)]
pub enum Error {
    /// Configuration errors
    Config(String),
    /// Network errors
    Network(std::io::Error),
    /// FHE operation errors
    Fhe(String),
    /// Provider API errors
    Provider(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Network(err) => write!(f, "Network error: {}", err),
            Error::Fhe(msg) => write!(f, "FHE error: {}", msg),
            Error::Provider(msg) => write!(f, "Provider error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Network(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Config(err.to_string())
    }
}