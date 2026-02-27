//! Health Monitoring Example
//!
//! This example demonstrates HTTP health checks with Kubernetes-style endpoints.
//! It shows how to:
//! - Configure multiple health checks (memory, uptime, custom)
//! - Integrate with health monitoring systems
//! - Handle graceful shutdown
//!
//! Run with:
//!   cargo run --example health_monitoring -p simian-http-listener
//!
//! Test endpoints:
//!   curl http://localhost:8080/healthz    # Liveness probe
//!   curl http://localhost:8080/readyz     # Readiness probe
//!   curl -s http://localhost:8080/healthz | jq  # Pretty print

use simian_http_listener::{
    checks::{CustomHealthCheck, MemoryHealthCheck, UptimeHealthCheck},
    HealthChecker, HealthStatus, Server,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Custom health check that simulates a service dependency
/// This could represent a database, cache, or external API
struct ServiceHealthCheck {
    /// Whether the simulated service is available
    available: Arc<AtomicBool>,
}

impl ServiceHealthCheck {
    fn new() -> Self {
        Self {
            available: Arc::new(AtomicBool::new(true)),
        }
    }

    fn set_available(&self, available: bool) {
        self.available.store(available, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl simian_http_listener::HealthCheck for ServiceHealthCheck {
    async fn check(&self) -> HealthStatus {
        if self.available.load(Ordering::Relaxed) {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: "External service unavailable".to_string(),
            }
        }
    }

    fn name(&self) -> &str {
        "external_service"
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for better observability
    simian_tracing::initialize()?;

    // Create custom health check for external service
    let service_check = Arc::new(ServiceHealthCheck::new());

    // Create health checker with multiple checks
    let health_checker = HealthChecker::new()
        // Memory check: alert if process uses more than 500MB
        .add_check(Arc::new(MemoryHealthCheck::new(500)))
        // Uptime check: service needs 5 seconds to warm up
        .add_check(Arc::new(UptimeHealthCheck::new(Duration::from_secs(5))))
        // Custom check: external service availability
        .add_check(service_check.clone())
        // Random health check for demonstration
        .add_check(Arc::new(CustomHealthCheck::new("random_check", || {
            // Simulate occasional degradation (10% chance)
            if rand::random::<f32>() < 0.1 {
                HealthStatus::Degraded {
                    reason: "Random degradation for demo".to_string(),
                }
            } else {
                HealthStatus::Healthy
            }
        })));

    // Create server with health checks
    let server = Server::with_health_checker("0.0.0.0:8080", health_checker);

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     Simian HTTP Health Monitoring Example                  ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("🚀 Server starting on http://0.0.0.0:8080");
    println!();
    println!("📊 Health Endpoints:");
    println!("  GET /healthz  → Kubernetes liveness probe (is service alive?)");
    println!("  GET /readyz   → Kubernetes readiness probe (is service ready?)");
    println!();
    println!("📋 Health Checks Configured:");
    println!("  ✓ Memory       - Alerts if > 500MB used");
    println!("  ✓ Uptime       - Requires 5s warm-up period");
    println!("  ✓ External API - Simulates external service health");
    println!("  ✓ Random       - Demonstrates degraded state");
    println!();
    println!("🧪 Try These Commands:");
    println!();
    println!("  # Check if service is alive (liveness probe)");
    println!("  $ curl http://localhost:8080/healthz");
    println!();
    println!("  # Check if service is ready (readiness probe)");
    println!("  $ curl http://localhost:8080/readyz");
    println!();
    println!("  # Pretty print health status");
    println!("  $ curl -s http://localhost:8080/healthz | jq .");
    println!();
    println!("  # Watch health status (Linux/Mac)");
    println!("  $ watch 'curl -s http://localhost:8080/healthz | jq .'");
    println!();
    println!("📋 Expected Health Status Responses:");
    println!();
    println!("  Healthy Response (200 OK):");
    println!("    {{ \"status\": \"healthy\" }}");
    println!();
    println!("  Degraded Response (200 OK, still serves traffic):");
    println!("    {{ \"status\": \"degraded\", \"reason\": \"check_name: issue description\" }}");
    println!();
    println!("  Unhealthy Response (503 Service Unavailable):");
    println!("    {{ \"status\": \"unhealthy\", \"reason\": \"external_service: External service unavailable\" }}");
    println!();
    println!("💡 Health Check Aggregation Rules:");
    println!("  1. Any Unhealthy check → returns 503");
    println!("  2. Any Degraded check (no Unhealthy) → returns 200 with degraded status");
    println!("  3. All Healthy → returns 200 with healthy status");
    println!();
    println!("🛑 Press Ctrl+C to shutdown gracefully");
    println!();

    // Spawn a task that simulates external service becoming unavailable
    let service_check_clone = service_check.clone();
    tokio::spawn(async move {
        // After 15 seconds, simulate service going down
        tokio::time::sleep(Duration::from_secs(15)).await;
        println!("\n⚠️  [Simulation] External service became unavailable!");
        service_check_clone.set_available(false);

        // After 10 more seconds, service recovers
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("✓ [Simulation] External service recovered!");
        service_check_clone.set_available(true);
    });

    // Start server with health endpoints
    // Parameters:
    // - None: no custom routes (health endpoints always included)
    // - 5000: header read timeout in milliseconds
    // - true: keep-alive enabled
    server.listen(None, 5000, true).await?;

    println!("\n✅ Server shutdown complete");
    Ok(())
}
