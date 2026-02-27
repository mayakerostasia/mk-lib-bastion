use std::sync::Arc;
use tokio::sync::RwLock;

/// Health status of a service or component
#[derive(Clone, Debug, PartialEq)]
pub enum HealthStatus {
    /// Service is fully operational
    Healthy,
    /// Service is operational but experiencing issues
    Degraded { reason: String },
    /// Service is not operational
    Unhealthy { reason: String },
}

impl HealthStatus {
    /// Check if status is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Check if status is unhealthy
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, HealthStatus::Unhealthy { .. })
    }

    /// Check if status is degraded
    pub fn is_degraded(&self) -> bool {
        matches!(self, HealthStatus::Degraded { .. })
    }

    /// Get the reason if status is not healthy
    pub fn reason(&self) -> Option<&str> {
        match self {
            HealthStatus::Healthy => None,
            HealthStatus::Degraded { reason } | HealthStatus::Unhealthy { reason } => Some(reason),
        }
    }
}

/// Trait for implementing health checks
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform the health check
    async fn check(&self) -> HealthStatus;

    /// Name of this health check
    fn name(&self) -> &str;
}

/// Aggregates multiple health checks
pub struct HealthChecker {
    checks: Vec<Arc<dyn HealthCheck>>,
    cached_status: Arc<RwLock<Option<HealthStatus>>>,
}

impl std::fmt::Debug for HealthChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HealthChecker")
            .field("num_checks", &self.checks.len())
            .field("cached_status", &self.cached_status)
            .finish()
    }
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            cached_status: Arc::new(RwLock::new(None)),
        }
    }

    /// Add a health check
    pub fn add_check(mut self, check: Arc<dyn HealthCheck>) -> Self {
        self.checks.push(check);
        self
    }

    /// Run all health checks and aggregate results
    pub async fn check_health(&self) -> HealthStatus {
        if self.checks.is_empty() {
            return HealthStatus::Healthy;
        }

        let mut unhealthy_reasons = Vec::new();
        let mut degraded_reasons = Vec::new();

        for check in &self.checks {
            match check.check().await {
                HealthStatus::Unhealthy { reason } => {
                    unhealthy_reasons.push(format!("{}: {}", check.name(), reason));
                }
                HealthStatus::Degraded { reason } => {
                    degraded_reasons.push(format!("{}: {}", check.name(), reason));
                }
                HealthStatus::Healthy => {}
            }
        }

        let status = if !unhealthy_reasons.is_empty() {
            HealthStatus::Unhealthy {
                reason: unhealthy_reasons.join("; "),
            }
        } else if !degraded_reasons.is_empty() {
            HealthStatus::Degraded {
                reason: degraded_reasons.join("; "),
            }
        } else {
            HealthStatus::Healthy
        };

        // Cache the status
        *self.cached_status.write().await = Some(status.clone());

        status
    }

    /// Get cached status without running checks
    pub async fn get_cached_status(&self) -> Option<HealthStatus> {
        self.cached_status.read().await.clone()
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysHealthyCheck;

    #[async_trait::async_trait]
    impl HealthCheck for AlwaysHealthyCheck {
        async fn check(&self) -> HealthStatus {
            HealthStatus::Healthy
        }

        fn name(&self) -> &str {
            "always_healthy"
        }
    }

    struct AlwaysDegradedCheck;

    #[async_trait::async_trait]
    impl HealthCheck for AlwaysDegradedCheck {
        async fn check(&self) -> HealthStatus {
            HealthStatus::Degraded {
                reason: "test degraded".to_string(),
            }
        }

        fn name(&self) -> &str {
            "always_degraded"
        }
    }

    struct AlwaysUnhealthyCheck;

    #[async_trait::async_trait]
    impl HealthCheck for AlwaysUnhealthyCheck {
        async fn check(&self) -> HealthStatus {
            HealthStatus::Unhealthy {
                reason: "test unhealthy".to_string(),
            }
        }

        fn name(&self) -> &str {
            "always_unhealthy"
        }
    }

    #[tokio::test]
    async fn test_health_status() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Healthy.is_unhealthy());
        assert!(!HealthStatus::Healthy.is_degraded());
        assert_eq!(HealthStatus::Healthy.reason(), None);

        let degraded = HealthStatus::Degraded {
            reason: "test".to_string(),
        };
        assert!(!degraded.is_healthy());
        assert!(!degraded.is_unhealthy());
        assert!(degraded.is_degraded());
        assert_eq!(degraded.reason(), Some("test"));

        let unhealthy = HealthStatus::Unhealthy {
            reason: "error".to_string(),
        };
        assert!(!unhealthy.is_healthy());
        assert!(unhealthy.is_unhealthy());
        assert!(!unhealthy.is_degraded());
        assert_eq!(unhealthy.reason(), Some("error"));
    }

    #[tokio::test]
    async fn test_empty_checker() {
        let checker = HealthChecker::new();
        let status = checker.check_health().await;
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_all_healthy() {
        let checker = HealthChecker::new()
            .add_check(Arc::new(AlwaysHealthyCheck))
            .add_check(Arc::new(AlwaysHealthyCheck));

        let status = checker.check_health().await;
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_degraded() {
        let checker = HealthChecker::new()
            .add_check(Arc::new(AlwaysHealthyCheck))
            .add_check(Arc::new(AlwaysDegradedCheck));

        let status = checker.check_health().await;
        assert!(status.is_degraded());
        assert!(status.reason().unwrap().contains("test degraded"));
    }

    #[tokio::test]
    async fn test_unhealthy() {
        let checker = HealthChecker::new()
            .add_check(Arc::new(AlwaysHealthyCheck))
            .add_check(Arc::new(AlwaysUnhealthyCheck));

        let status = checker.check_health().await;
        assert!(status.is_unhealthy());
        assert!(status.reason().unwrap().contains("test unhealthy"));
    }

    #[tokio::test]
    async fn test_unhealthy_trumps_degraded() {
        let checker = HealthChecker::new()
            .add_check(Arc::new(AlwaysDegradedCheck))
            .add_check(Arc::new(AlwaysUnhealthyCheck));

        let status = checker.check_health().await;
        assert!(status.is_unhealthy());
    }

    #[tokio::test]
    async fn test_cached_status() {
        let checker = HealthChecker::new().add_check(Arc::new(AlwaysHealthyCheck));

        // No cached status initially
        assert!(checker.get_cached_status().await.is_none());

        // Check health
        let status = checker.check_health().await;
        assert_eq!(status, HealthStatus::Healthy);

        // Cached status should be available
        let cached = checker.get_cached_status().await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), HealthStatus::Healthy);
    }
}
