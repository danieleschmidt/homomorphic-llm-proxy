//! Homomorphic LLM Proxy Library
//!
//! Core library for FHE-based LLM inference proxy.

pub mod config;
pub mod error;
pub mod fhe;
pub mod health;
pub mod i18n;
pub mod middleware;
pub mod monitoring;
pub mod performance;
pub mod proxy;
pub mod scaling;
pub mod security;
pub mod validation;

pub use config::Config;
pub use error::{Error, Result};
