//! Homomorphic LLM Proxy Library
//!
//! Core library for FHE-based LLM inference proxy.

pub mod api_test;
pub mod config;
// pub mod deployment; // Temporarily disabled due to compilation issues
pub mod enhanced_logging;
pub mod error;
pub mod fhe;
// pub mod global_scaling; // Temporarily disabled due to compilation issues  
pub mod health;
pub mod i18n;
pub mod middleware;
pub mod monitoring;
// pub mod observability; // Temporarily disabled due to compilation issues
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

pub use config::Config;
pub use error::{Error, Result};
