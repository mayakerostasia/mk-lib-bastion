//! # Simian Agent Server
//!
//! A flexible, long-lived agent server that uses the Kong/Monkey pattern
//! for processing Frame messages over NATS.
//!
//! ## Features
//! - Built-in health endpoint for Docker/Kubernetes
//! - Role-based behavior (configurable via AGENT_ROLE env var)
//! - Tower service middleware support (buffering, rate limiting, timeouts)
//! - Graceful shutdown on SIGTERM/SIGINT
//!
//! ## Environment Variables
//! - `NATS_URL`: NATS server address (default: nats://localhost:4222)
//! - `AGENT_ID`: Unique agent identifier (default: agent-001)
//! - `AGENT_ROLE`: Agent role/behavior (default: echo)
//! - `AGENT_SUBJECT`: Base NATS subject (default: agents)
//! - `HEALTH_BIND`: Health check endpoint (default: 0.0.0.0:6660)
//! - `RUST_LOG`: Log level (default: info)
//!
//! ## Usage
//! ```bash
//! # Run with defaults (echo agent)
//! simian-agent
//!
//! # Run with specific role
//! AGENT_ROLE=processor simian-agent
//!
//! # Docker usage
//! docker run -e AGENT_ROLE=generator -e NATS_URL=nats://nats:4222 simian-agent
//! ```

use anyhow::Result;
use clap::Parser;
use simian_nats_streams::KingKong;
use tower::BoxError;
use tracing::{info, warn};

mod roles;

#[derive(Parser, Debug)]
#[command(version, about = "Simian Agent Server - Long-lived NATS Frame processor")]
struct Args {
    /// NATS server URL
    #[arg(long, env = "NATS_URL", default_value = "nats://localhost:4222")]
    nats_url: String,

    /// Agent unique identifier
    #[arg(long, env = "AGENT_ID", default_value = "agent-001")]
    agent_id: String,

    /// Agent role (determines behavior)
    #[arg(long, env = "AGENT_ROLE", default_value = "echo")]
    agent_role: String,

    /// Base NATS subject
    #[arg(long, env = "AGENT_SUBJECT", default_value = "agents")]
    agent_subject: String,

    /// Health check bind address
    #[arg(long, env = "HEALTH_BIND", default_value = "0.0.0.0:6660")]
    health_bind: String,
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Initialize tracing
    let _otel = simian_tracing::initialize()?;

    let args = Args::parse();

    // TODO: Should be taking advantage of custom tracing/logging fields perhaps?
    info!("🚀 Simian Agent Server starting...");
    info!("   Agent ID: {}", args.agent_id);
    info!("   Role: {}", args.agent_role);
    info!("   NATS URL: {}", args.nats_url);
    info!("   Subject: {}.{}", args.agent_subject, args.agent_id);
    info!("   Health: http://{}/healthz", args.health_bind);

    // Create KingKong with health endpoint
    let mut kkong = KingKong::new(
        &args.agent_subject,
        &args.nats_url,
        &args.health_bind,
    )
    .await;

    info!("✅ KingKong initialized with health endpoint");

    // Register handler based on role
    match args.agent_role.as_str() {
        "echo" => {
            info!("📢 Starting ECHO agent");
            kkong
                .new_future_kong(&args.agent_id, roles::echo_handler)
                .await?;
        }
        "processor" => {
            info!("⚙️  Starting PROCESSOR agent");
            kkong
                .new_future_kong(&args.agent_id, roles::processor_handler)
                .await?;
        }
        "generator" => {
            info!("🎲 Starting GENERATOR agent");
            kkong
                .new_future_kong(&args.agent_id, roles::generator_handler)
                .await?;
        }
        "verifier" => {
            info!("✓  Starting VERIFIER agent");
            kkong
                .new_future_kong(&args.agent_id, roles::verifier_handler)
                .await?;
        }
        "tower-example" => {
            info!("🏗️  Starting TOWER SERVICE agent (with middleware)");
            kkong
                .new_future_kong(&args.agent_id, roles::tower_service_handler)
                .await?;
        }
        unknown => {
            warn!("⚠️  Unknown role '{}', falling back to echo", unknown);
            kkong
                .new_future_kong(&args.agent_id, roles::echo_handler)
                .await?;
        }
    }

    info!("🎧 Agent listening on: {}.{}", args.agent_subject, args.agent_id);
    info!("🏥 Health check available at: http://{}/healthz", args.health_bind);
    info!("⏸️  Press Ctrl+C to shutdown gracefully");

    // Setup graceful shutdown
    let shutdown = tokio::spawn(async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl-c");
        info!("🛑 Shutdown signal received");
    });

    // Wait for either KingKong to exit or shutdown signal
    tokio::select! {
        result = kkong.wait() => {
            result?;
        }
        _ = shutdown => {
            info!("👋 Graceful shutdown initiated");
        }
    }

    info!("✅ Agent shutdown complete");
    Ok(())
}
