use crate::health::{HealthCheck, HealthStatus};
use std::time::{Duration, Instant};

/// Check memory usage
pub struct MemoryHealthCheck {
    threshold_mb: u64,
}

impl MemoryHealthCheck {
    /// Create a new memory health check
    ///
    /// # Arguments
    /// * `threshold_mb` - Memory threshold in megabytes for degraded status
    pub fn new(threshold_mb: u64) -> Self {
        Self { threshold_mb }
    }
}

#[async_trait::async_trait]
impl HealthCheck for MemoryHealthCheck {
    async fn check(&self) -> HealthStatus {
        #[cfg(target_os = "linux")]
        {
            match std::fs::read_to_string("/proc/self/status") {
                Ok(contents) => {
                    // Parse VmRSS (resident set size)
                    for line in contents.lines() {
                        if line.starts_with("VmRSS:") {
                            if let Some(kb_str) = line.split_whitespace().nth(1) {
                                if let Ok(kb) = kb_str.parse::<u64>() {
                                    let mb = kb / 1024;
                                    if mb > self.threshold_mb {
                                        return HealthStatus::Degraded {
                                            reason: format!(
                                                "Memory usage {}MB exceeds threshold {}MB",
                                                mb, self.threshold_mb
                                            ),
                                        };
                                    }
                                    return HealthStatus::Healthy;
                                }
                            }
                        }
                    }
                    HealthStatus::Healthy
                }
                Err(e) => HealthStatus::Degraded {
                    reason: format!("Failed to read memory info: {}", e),
                },
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            // On non-Linux systems, always report healthy
            HealthStatus::Healthy
        }
    }

    fn name(&self) -> &str {
        "memory"
    }
}

/// Check uptime
pub struct UptimeHealthCheck {
    started_at: Instant,
    min_uptime: Duration,
}

impl UptimeHealthCheck {
    /// Create a new uptime health check
    ///
    /// # Arguments
    /// * `min_uptime` - Minimum uptime before service is considered healthy
    pub fn new(min_uptime: Duration) -> Self {
        Self {
            started_at: Instant::now(),
            min_uptime,
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for UptimeHealthCheck {
    async fn check(&self) -> HealthStatus {
        let uptime = self.started_at.elapsed();
        if uptime < self.min_uptime {
            HealthStatus::Degraded {
                reason: format!(
                    "Service warming up ({:.1}s < {:.1}s)",
                    uptime.as_secs_f64(),
                    self.min_uptime.as_secs_f64()
                ),
            }
        } else {
            HealthStatus::Healthy
        }
    }

    fn name(&self) -> &str {
        "uptime"
    }
}

/// Custom health check using a closure
pub struct CustomHealthCheck {
    name: String,
    check_fn: Box<dyn Fn() -> HealthStatus + Send + Sync>,
}

impl CustomHealthCheck {
    /// Create a new custom health check
    ///
    /// # Arguments
    /// * `name` - Name of the health check
    /// * `check_fn` - Function to perform the health check
    pub fn new<F>(name: impl Into<String>, check_fn: F) -> Self
    where
        F: Fn() -> HealthStatus + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            check_fn: Box::new(check_fn),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for CustomHealthCheck {
    async fn check(&self) -> HealthStatus {
        (self.check_fn)()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_check() {
        let check = MemoryHealthCheck::new(999999); // Very high threshold
        let status = check.check().await;
        assert!(status.is_healthy() || status.is_degraded());
    }

    #[tokio::test]
    async fn test_uptime_check_warming() {
        let check = UptimeHealthCheck::new(Duration::from_secs(10));
        let status = check.check().await;
        assert!(status.is_degraded());
        assert!(status.reason().unwrap().contains("warming up"));
    }

    #[tokio::test]
    async fn test_uptime_check_ready() {
        let check = UptimeHealthCheck::new(Duration::from_millis(1));
        tokio::time::sleep(Duration::from_millis(10)).await;
        let status = check.check().await;
        assert!(status.is_healthy());
    }

    #[tokio::test]
    async fn test_custom_check() {
        let check = CustomHealthCheck::new("test", || HealthStatus::Healthy);
        let status = check.check().await;
        assert!(status.is_healthy());
        assert_eq!(check.name(), "test");
    }

    #[tokio::test]
    async fn test_custom_check_unhealthy() {
        let check = CustomHealthCheck::new("test", || HealthStatus::Unhealthy {
            reason: "custom failure".to_string(),
        });
        let status = check.check().await;
        assert!(status.is_unhealthy());
        assert_eq!(status.reason(), Some("custom failure"));
    }
}
