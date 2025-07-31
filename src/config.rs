//! Configuration management for FHE LLM Proxy

use serde::{Deserialize, Serialize};
use std::fs;
use crate::error::{Error, Result};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub encryption: EncryptionConfig,
    pub llm: LlmConfig,
    pub privacy: PrivacyConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

/// Encryption parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub poly_modulus_degree: u32,
    pub coeff_modulus_bits: Vec<u8>,
    pub scale_bits: u8,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub endpoint: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub epsilon_per_query: f64,
    pub delta: f64,
    pub max_queries_per_user: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: 4,
            },
            encryption: EncryptionConfig {
                poly_modulus_degree: 16384,
                coeff_modulus_bits: vec![60, 40, 40, 60],
                scale_bits: 40,
            },
            llm: LlmConfig {
                provider: "openai".to_string(),
                endpoint: "https://api.openai.com/v1".to_string(),
                timeout_seconds: 300,
                max_retries: 3,
            },
            privacy: PrivacyConfig {
                epsilon_per_query: 0.1,
                delta: 1e-5,
                max_queries_per_user: 1000,
            },
        }
    }
}

impl Config {
    /// Load configuration from file or use defaults
    pub fn load() -> Result<Self> {
        if let Ok(content) = fs::read_to_string("config.toml") {
            toml::from_str(&content).map_err(|e| Error::Config(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }
}