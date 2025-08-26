//! Resilience patterns for FHE LLM Proxy

use crate::error::{Error, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Circuit breaker implementation for fault tolerance
#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    failure_threshold: usize,
    recovery_timeout: Duration,
    state: Arc<RwLock<CircuitBreakerState>>,
}

#[derive(Debug)]
struct CircuitBreakerState {
    failure_count: usize,
    last_failure_time: Option<Instant>,
    is_open: bool,
}

impl CircuitBreaker {
    pub fn new(name: String, failure_threshold: usize, recovery_timeout: Duration) -> Self {
        Self {
            name,
            failure_threshold,
            recovery_timeout,
            state: Arc::new(RwLock::new(CircuitBreakerState {
                failure_count: 0,
                last_failure_time: None,
                is_open: false,
            })),
        }
    }

    /// Execute operation with circuit breaker protection
    pub async fn call<F, T>(&self, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // Check if circuit is open
        {
            let mut state = self.state.write().await;
            
            if state.is_open {
                // Check if recovery timeout has passed
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        info!(circuit_breaker = %self.name, "Attempting recovery - half-open state");
                        state.is_open = false;
                        state.failure_count = 0;
                    } else {
                        warn!(circuit_breaker = %self.name, "Circuit breaker is OPEN, rejecting request");
                        return Err(Error::CircuitBreakerOpen(format!(
                            "Circuit breaker '{}' is open", self.name
                        )));
                    }
                }
            }
        }

        // Execute operation
        let result = operation.await;

        // Update state based on result
        let mut state = self.state.write().await;
        match result {
            Ok(ref _value) => {
                // Reset on success
                state.failure_count = 0;
                state.is_open = false;
                debug!(circuit_breaker = %self.name, "Operation succeeded");
            }
            Err(ref error) => {
                state.failure_count += 1;
                state.last_failure_time = Some(Instant::now());
                
                if state.failure_count >= self.failure_threshold {
                    state.is_open = true;
                    warn!(
                        circuit_breaker = %self.name,
                        failure_count = %state.failure_count,
                        threshold = %self.failure_threshold,
                        "Circuit breaker OPENED due to failures"
                    );
                }
                
                error!(circuit_breaker = %self.name, error = %error, "Operation failed");
            }
        }

        result
    }
}

/// Retry policy with exponential backoff
#[derive(Debug)]
pub struct RetryPolicy {
    max_attempts: usize,
    initial_delay: Duration,
    max_delay: Duration,
}

impl RetryPolicy {
    pub fn new(max_attempts: usize, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay,
        }
    }

    pub async fn execute<F, T, E, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, E>>,
        E: Into<Error>,
    {
        let mut last_error: Option<Error> = None;

        for attempt in 1..=self.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Retry succeeded after {} attempts", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    let error = e.into();
                    last_error = Some(error.clone());

                    if attempt < self.max_attempts {
                        let delay = self.calculate_delay(attempt);
                        warn!(
                            attempt = %attempt,
                            delay_ms = %delay.as_millis(),
                            "Operation failed, retrying"
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::Internal("Retry exhausted".to_string())
        }))
    }

    fn calculate_delay(&self, attempt: usize) -> Duration {
        let delay = self.initial_delay * 2_u32.pow((attempt - 1) as u32);
        std::cmp::min(delay, self.max_delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let cb = CircuitBreaker::new("test".to_string(), 3, Duration::from_millis(100));
        
        let result = cb.call(async { Ok::<i32, Error>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_policy() {
        let retry = RetryPolicy::new(3, Duration::from_millis(1), Duration::from_millis(100));
        
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        
        let attempt = Arc::new(AtomicUsize::new(0));
        let attempt_clone = attempt.clone();
        
        let result = retry.execute(move || {
            let attempt = attempt_clone.clone();
            async move {
                let current = attempt.fetch_add(1, Ordering::SeqCst) + 1;
                if current < 3 {
                    Err(Error::Internal("test".to_string()))
                } else {
                    Ok(42)
                }
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt.load(Ordering::SeqCst), 3);
    }
}