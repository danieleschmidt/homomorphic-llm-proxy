//! Simple API endpoint test functionality for FHE LLM Proxy

use crate::config::Config;
use crate::error::Result;
// use crate::fhe::FheEngine;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

/// Simple API client for testing the FHE proxy
pub struct ApiTestClient {
    client: Client,
    base_url: String,
}

impl ApiTestClient {
    /// Create a new API test client
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url,
        }
    }

    /// Test health endpoint
    pub async fn test_health(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        
        match timeout(Duration::from_secs(5), self.client.get(&url).send()).await {
            Ok(Ok(response)) => Ok(response.status().is_success()),
            Ok(Err(e)) => {
                tracing::error!("HTTP error during health check: {}", e);
                Ok(false)
            },
            Err(_) => {
                tracing::error!("Health check timed out");
                Ok(false)
            }
        }
    }

    /// Test basic FHE encryption endpoint
    pub async fn test_encrypt(&self, plaintext: &str) -> Result<String> {
        let url = format!("{}/v1/encrypt", self.base_url);
        let payload = json!({
            "plaintext": plaintext,
            "security_level": 128
        });

        let response = timeout(
            Duration::from_secs(10),
            self.client.post(&url)
                .json(&payload)
                .send()
        ).await
        .map_err(|_| crate::error::Error::Timeout("Encryption request timed out".into()))?
        .map_err(|e| crate::error::Error::Request(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::error::Error::Http(
                format!("Server returned status: {}", response.status())
            ));
        }

        let result: serde_json::Value = response.json().await
            .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;
        
        Ok(result.get("ciphertext")
            .and_then(|v| v.as_str())
            .unwrap_or("empty")
            .to_string())
    }

    /// Test metrics endpoint
    pub async fn test_metrics(&self) -> Result<bool> {
        let url = format!("{}/metrics", self.base_url);
        
        match timeout(Duration::from_secs(5), self.client.get(&url).send()).await {
            Ok(Ok(response)) => {
                let is_success = response.status().is_success();
                if is_success {
                    let text = response.text().await.unwrap_or_default();
                    tracing::info!("Metrics endpoint responded with {} bytes", text.len());
                }
                Ok(is_success)
            },
            Ok(Err(e)) => {
                tracing::error!("HTTP error during metrics check: {}", e);
                Ok(false)
            },
            Err(_) => {
                tracing::error!("Metrics check timed out");
                Ok(false)
            }
        }
    }
}

/// Run a comprehensive API test suite
pub async fn run_api_tests(config: &Config) -> Result<ApiTestReport> {
    let base_url = format!("http://{}:{}", config.server.host, config.server.port);
    let client = ApiTestClient::new(base_url);
    
    let mut report = ApiTestReport::new();
    
    // Test 1: Health check
    tracing::info!("Running health check test...");
    match client.test_health().await {
        Ok(true) => {
            report.tests_passed += 1;
            tracing::info!("✅ Health check passed");
        },
        Ok(false) => {
            report.tests_failed += 1;
            report.failures.push("Health check returned unhealthy status".to_string());
            tracing::error!("❌ Health check failed");
        },
        Err(e) => {
            report.tests_failed += 1;
            report.failures.push(format!("Health check error: {}", e));
            tracing::error!("❌ Health check error: {}", e);
        }
    }

    // Test 2: Basic encryption
    tracing::info!("Running encryption test...");
    match client.test_encrypt("Hello, FHE World!").await {
        Ok(ciphertext) => {
            if !ciphertext.is_empty() && ciphertext != "empty" {
                report.tests_passed += 1;
                tracing::info!("✅ Encryption test passed (ciphertext length: {})", ciphertext.len());
            } else {
                report.tests_failed += 1;
                report.failures.push("Encryption returned empty result".to_string());
                tracing::error!("❌ Encryption test failed: empty result");
            }
        },
        Err(e) => {
            report.tests_failed += 1;
            report.failures.push(format!("Encryption test error: {}", e));
            tracing::error!("❌ Encryption test error: {}", e);
        }
    }

    // Test 3: Metrics
    tracing::info!("Running metrics test...");
    match client.test_metrics().await {
        Ok(true) => {
            report.tests_passed += 1;
            tracing::info!("✅ Metrics test passed");
        },
        Ok(false) => {
            report.tests_failed += 1;
            report.failures.push("Metrics endpoint returned error status".to_string());
            tracing::error!("❌ Metrics test failed");
        },
        Err(e) => {
            report.tests_failed += 1;
            report.failures.push(format!("Metrics test error: {}", e));
            tracing::error!("❌ Metrics test error: {}", e);
        }
    }

    report.total_tests = report.tests_passed + report.tests_failed;
    Ok(report)
}

#[derive(Debug, Clone)]
pub struct ApiTestReport {
    pub total_tests: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub failures: Vec<String>,
}

impl ApiTestReport {
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            tests_passed: 0,
            tests_failed: 0,
            failures: Vec::new(),
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            1.0
        } else {
            self.tests_passed as f64 / self.total_tests as f64
        }
    }

    pub fn is_successful(&self) -> bool {
        self.tests_failed == 0
    }
}

impl std::fmt::Display for ApiTestReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "🧪 API Test Report")?;
        writeln!(f, "==================")?;
        writeln!(f, "Total Tests: {}", self.total_tests)?;
        writeln!(f, "Passed: {}", self.tests_passed)?;
        writeln!(f, "Failed: {}", self.tests_failed)?;
        writeln!(f, "Success Rate: {:.1}%", self.success_rate() * 100.0)?;
        
        if !self.failures.is_empty() {
            writeln!(f, "\n❌ Failures:")?;
            for failure in &self.failures {
                writeln!(f, "  • {}", failure)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_client_creation() {
        let client = ApiTestClient::new("http://localhost:8080".to_string());
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_api_report_creation() {
        let report = ApiTestReport::new();
        assert_eq!(report.total_tests, 0);
        assert_eq!(report.tests_passed, 0);
        assert_eq!(report.tests_failed, 0);
        assert!(report.failures.is_empty());
        assert!(report.is_successful());
        assert_eq!(report.success_rate(), 1.0);
    }

    #[test]
    fn test_api_report_success_rate() {
        let mut report = ApiTestReport::new();
        report.total_tests = 10;
        report.tests_passed = 8;
        report.tests_failed = 2;
        
        assert_eq!(report.success_rate(), 0.8);
        assert!(!report.is_successful());
    }
}