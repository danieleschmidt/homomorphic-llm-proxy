//! Homomorphic LLM Proxy
//!
//! GPU-accelerated gateway for fully homomorphic encryption (FHE) of LLM inference.
//! Process prompts on untrusted cloud infrastructure while maintaining complete privacy.

mod config;
mod error;
mod fhe;
mod middleware;
mod monitoring;
mod proxy;
mod scaling;

use config::Config;
use error::Result;
use proxy::ProxyServer;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging().await?;

    // Load and validate configuration
    let config = Config::load()?;
    config.validate()?;

    info!("🚀 Starting FHE LLM Proxy");
    info!("{}", config.summary());

    // Create and start the proxy server
    let server = ProxyServer::new(config)?;

    if let Err(e) = server.start().await {
        error!("Server failed to start: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

/// Initialize logging and tracing
async fn init_logging() -> Result<()> {
    // Set up tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🔐 FHE LLM Proxy starting up...");

    Ok(())
}
