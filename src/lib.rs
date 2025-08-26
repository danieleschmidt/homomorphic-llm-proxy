//! Homomorphic LLM Proxy Library
//!
//! Core library for FHE-based LLM inference proxy.

pub mod config;
pub mod deployment;
pub mod error;
pub mod fhe;
pub mod global_scaling;  
pub mod health;
pub mod i18n;
pub mod middleware;
pub mod monitoring;
pub mod observability;
pub mod performance;
pub mod performance_optimized;
pub mod proxy;
pub mod resilience;
pub mod scaling;
pub mod security;
pub mod security_enhanced;
pub mod performance_scaling;
pub mod global_config;
pub mod validation;
pub mod enhanced_resilience;
pub mod ml_optimizer;

pub use config::Config;
pub use error::{Error, Result};
