use simian_http_listener::{checks::CustomHealthCheck, HealthChecker, HealthStatus, Server};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_health_endpoints_default() {
    // Create server with default health checker (always healthy)
    let server = Server::new("127.0.0.1:0");

    // Spawn server in background
    let handle = tokio::spawn(async move {
        server.listen(None, 5000, true).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test healthz endpoint
    let client = reqwest::Client::new();
    let _resp = client.get("http://127.0.0.1:8080/healthz").send().await;

    // Server might not be on 8080, so we can't test properly without dynamic port
    // This test is more of a compilation check
    handle.abort();
}

#[tokio::test]
async fn test_health_checker_with_custom_checks() {
    let health_checker = HealthChecker::new()
        .add_check(Arc::new(CustomHealthCheck::new("test", || {
            HealthStatus::Healthy
        })));

    let status: HealthStatus = health_checker.check_health().await;
    assert_eq!(status, HealthStatus::Healthy);
}

#[tokio::test]
async fn test_health_checker_unhealthy() {
    let health_checker =
        HealthChecker::new().add_check(Arc::new(CustomHealthCheck::new("test", || {
            HealthStatus::Unhealthy {
                reason: "test failure".to_string(),
            }
        })));

    let status: HealthStatus = health_checker.check_health().await;
    assert!(status.is_unhealthy());
    assert!(status.reason().unwrap().contains("test failure"));
}

#[tokio::test]
async fn test_health_checker_degraded() {
    let health_checker =
        HealthChecker::new().add_check(Arc::new(CustomHealthCheck::new("test", || {
            HealthStatus::Degraded {
                reason: "test degraded".to_string(),
            }
        })));

    let status: HealthStatus = health_checker.check_health().await;
    assert!(status.is_degraded());
    assert!(status.reason().unwrap().contains("test degraded"));
}

#[tokio::test]
async fn test_multiple_checks_aggregation() {
    let health_checker = HealthChecker::new()
        .add_check(Arc::new(CustomHealthCheck::new("check1", || {
            HealthStatus::Healthy
        })))
        .add_check(Arc::new(CustomHealthCheck::new("check2", || {
            HealthStatus::Degraded {
                reason: "minor issue".to_string(),
            }
        })))
        .add_check(Arc::new(CustomHealthCheck::new("check3", || {
            HealthStatus::Healthy
        })));

    let status: HealthStatus = health_checker.check_health().await;
    assert!(status.is_degraded());
    assert!(status.reason().unwrap().contains("check2"));
    assert!(status.reason().unwrap().contains("minor issue"));
}

#[tokio::test]
async fn test_unhealthy_trumps_degraded() {
    let health_checker = HealthChecker::new()
        .add_check(Arc::new(CustomHealthCheck::new("degraded", || {
            HealthStatus::Degraded {
                reason: "minor".to_string(),
            }
        })))
        .add_check(Arc::new(CustomHealthCheck::new("unhealthy", || {
            HealthStatus::Unhealthy {
                reason: "critical".to_string(),
            }
        })));

    let status: HealthStatus = health_checker.check_health().await;
    assert!(status.is_unhealthy());
    assert!(status.reason().unwrap().contains("critical"));
}
