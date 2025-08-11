//! Configuration management for FHE LLM Proxy

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub encryption: EncryptionConfig,
    pub llm: LlmConfig,
    pub privacy: PrivacyConfig,
    pub gpu: GpuConfig,
    pub monitoring: MonitoringConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: u32,
    pub request_timeout_seconds: u64,
}

/// Encryption parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub poly_modulus_degree: usize,
    pub coeff_modulus_bits: Vec<u64>,
    pub scale_bits: u64,
    pub security_level: u8,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub endpoint: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub custom_providers: Vec<CustomProvider>,
}

/// Custom LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProvider {
    pub name: String,
    pub endpoint: String,
    pub api_key: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// GPU configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    pub enabled: bool,
    pub device_id: u32,
    pub batch_size: u32,
    pub kernel_optimization: String,
    pub memory_limit_gb: Option<u32>,
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub epsilon_per_query: f64,
    pub delta: f64,
    pub max_queries_per_user: u32,
    pub track_privacy_budget: bool,
    pub noise_multiplier: f64,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub trace_sampling_rate: f64,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: 4,
                max_connections: 1000,
                request_timeout_seconds: 300,
            },
            encryption: EncryptionConfig {
                poly_modulus_degree: 16384,
                coeff_modulus_bits: vec![60, 40, 40, 60],
                scale_bits: 40,
                security_level: 128,
            },
            llm: LlmConfig {
                provider: "openai".to_string(),
                endpoint: "https://api.openai.com/v1".to_string(),
                timeout_seconds: 300,
                max_retries: 3,
                openai_api_key: None,
                anthropic_api_key: None,
                custom_providers: vec![],
            },
            gpu: GpuConfig {
                enabled: false,
                device_id: 0,
                batch_size: 32,
                kernel_optimization: "aggressive".to_string(),
                memory_limit_gb: None,
            },
            privacy: PrivacyConfig {
                epsilon_per_query: 0.1,
                delta: 1e-5,
                max_queries_per_user: 1000,
                track_privacy_budget: true,
                noise_multiplier: 1.1,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_port: 9090,
                trace_sampling_rate: 0.1,
                log_level: "info".to_string(),
            },
        }
    }
}

impl Config {
    /// Load configuration from file, environment variables, or use defaults
    pub fn load() -> Result<Self> {
        let mut config = if let Ok(content) = fs::read_to_string("config.toml") {
            toml::from_str(&content).map_err(|e| Error::Config(e.to_string()))?
        } else {
            Self::default()
        };

        // Override with environment variables
        config.load_from_env();

        Ok(config)
    }

    /// Load configuration from environment variables
    pub fn load_from_env(&mut self) {
        if let Ok(host) = env::var("FHE_HOST") {
            self.server.host = host;
        }

        if let Ok(port) = env::var("FHE_PORT") {
            if let Ok(port) = port.parse() {
                self.server.port = port;
            }
        }

        if let Ok(openai_key) = env::var("OPENAI_API_KEY") {
            self.llm.openai_api_key = Some(openai_key);
        }

        if let Ok(anthropic_key) = env::var("ANTHROPIC_API_KEY") {
            self.llm.anthropic_api_key = Some(anthropic_key);
        }

        if let Ok(gpu_enabled) = env::var("FHE_GPU_ENABLED") {
            self.gpu.enabled = gpu_enabled.to_lowercase() == "true";
        }

        if let Ok(device_id) = env::var("FHE_GPU_DEVICE_ID") {
            if let Ok(device_id) = device_id.parse() {
                self.gpu.device_id = device_id;
            }
        }

        if let Ok(log_level) = env::var("RUST_LOG") {
            self.monitoring.log_level = log_level;
        }

        if let Ok(metrics_enabled) = env::var("FHE_METRICS_ENABLED") {
            self.monitoring.metrics_enabled = metrics_enabled.to_lowercase() == "true";
        }

        if let Ok(poly_degree) = env::var("FHE_POLY_MODULUS_DEGREE") {
            if let Ok(degree) = poly_degree.parse() {
                self.encryption.poly_modulus_degree = degree;
            }
        }

        if let Ok(security_level) = env::var("FHE_SECURITY_LEVEL") {
            if let Ok(level) = security_level.parse() {
                self.encryption.security_level = level;
            }
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(Error::Config("Invalid server port".to_string()));
        }

        if self.server.workers == 0 {
            return Err(Error::Config(
                "Worker count must be greater than 0".to_string(),
            ));
        }

        // Validate encryption parameters
        if !self.encryption.poly_modulus_degree.is_power_of_two() {
            return Err(Error::Config(
                "Poly modulus degree must be a power of 2".to_string(),
            ));
        }

        if self.encryption.coeff_modulus_bits.is_empty() {
            return Err(Error::Config(
                "Coefficient modulus bits cannot be empty".to_string(),
            ));
        }

        // Validate privacy parameters
        if self.privacy.epsilon_per_query <= 0.0 {
            return Err(Error::Config(
                "Epsilon per query must be positive".to_string(),
            ));
        }

        if self.privacy.delta <= 0.0 || self.privacy.delta >= 1.0 {
            return Err(Error::Config("Delta must be in (0, 1)".to_string()));
        }

        // Validate GPU configuration
        if self.gpu.enabled && self.gpu.batch_size == 0 {
            return Err(Error::Config(
                "GPU batch size must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get configuration summary for logging
    pub fn summary(&self) -> String {
        format!(
            "FHE Proxy Config - Server: {}:{}, GPU: {}, Security: {}",
            self.server.host,
            self.server.port,
            if self.gpu.enabled {
                "enabled"
            } else {
                "disabled"
            },
            self.encryption.security_level
        )
    }
}
