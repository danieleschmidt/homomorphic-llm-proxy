```rust
//! Homomorphic LLM Proxy Library
//!
//! Core library for FHE-based LLM inference proxy.

pub mod api_test;
pub mod config;
pub mod deployment;
pub mod enhanced_logging;
pub mod enhanced_resilience;
pub mod error;
pub mod fhe;
pub mod global_config;
pub mod global_scaling;
pub mod health;
pub mod i18n;
pub mod middleware;
pub mod ml_optimizer;
pub mod monitoring;
pub mod observability;
pub mod performance;
pub mod performance_optimized;
pub mod performance_scaling;
pub mod proxy;
pub mod resilience;
pub mod scaling;
pub mod security;
pub mod security_enhanced;
pub mod validation;

pub use config::Config;
pub use error::{Error, Result};
```
