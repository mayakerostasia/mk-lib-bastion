use simian_http_listener::{
    checks::{CustomHealthCheck, MemoryHealthCheck, UptimeHealthCheck},
    HealthChecker, HealthStatus, Server,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    simian_tracing::initialize()?;

    // Create health checker with multiple checks
    let health_checker = HealthChecker::new()
        // Check memory usage (alert if over 500MB)
        .add_check(Arc::new(MemoryHealthCheck::new(500)))
        // Check uptime (warm up period of 5 seconds)
        .add_check(Arc::new(UptimeHealthCheck::new(Duration::from_secs(5))))
        // Custom health check
        .add_check(Arc::new(CustomHealthCheck::new("custom_check", || {
            // Example: check some condition
            if rand::random::<bool>() {
                HealthStatus::Healthy
            } else {
                HealthStatus::Degraded {
                    reason: "Random degradation for demo".to_string(),
                }
            }
        })));

    // Create server with health checks
    let server = Server::with_health_checker("0.0.0.0:8080", health_checker);

    println!("Starting health check server on http://0.0.0.0:8080");
    println!("Try:");
    println!("  curl http://localhost:8080/healthz");
    println!("  curl http://localhost:8080/readyz");

    // Start server
    server.listen(None, 5000, true).await?;

    Ok(())
}
