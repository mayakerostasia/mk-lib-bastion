//! # Simian Agent Registry
//!
//! A long-lived service that tracks live agents and answers discovery queries.
//!
//! ## Protocol (all via ACP on NATS)
//!
//! | Performative | Subject           | Payload              | Action                        |
//! |---|---|---|---|
//! | Subscribe    | `registry`        | `AgentAnnouncement`  | Register agent                |
//! | Inform       | `registry`        | `AgentAnnouncement`  | Heartbeat (refresh last_seen) |
//! | Query        | `registry.query`  | `AgentQuery`         | Return matching agents        |
//!
//! ## Environment Variables
//! - `NATS_URL`       — NATS address (default: `nats://localhost:4222`)
//! - `AGENT_SUBJECT`  — ACP base subject (default: `agents`)
//! - `STALE_SECS`     — seconds before an agent is considered stale (default: `90`)
//! - `CLEANUP_SECS`   — cleanup interval in seconds (default: `30`)

use anyhow::Result;
use bytes::Bytes;
use futures::StreamExt;
use simian_agent_registry::{AgentQuery, AgentQueryResponse, AgentRegistry};
use simian_base_api::agent::SimianAgent;
use simian_base_api::transport::{AgentId, Performative};
use simian_llm::agent::AgentAnnouncement;
use simian_nats_streams::transport::NatsTransport;
use std::env;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let base_subject = env::var("AGENT_SUBJECT").unwrap_or_else(|_| "agents".to_string());
    let stale_secs: i64 = env::var("STALE_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(90);
    let cleanup_secs: u64 = env::var("CLEANUP_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);

    info!("🗂️  Agent Registry starting");
    info!("   NATS:    {}", nats_url);
    info!("   Subject: {}.registry.inbox", base_subject);
    info!("   Stale:   {}s", stale_secs);

    let registry = AgentRegistry::new();
    let transport = NatsTransport::new(&nats_url, &base_subject).await?;
    let agent = SimianAgent::new(AgentId::new("registry"), transport);

    // Background cleanup task
    let cleanup_registry = registry.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(cleanup_secs));
        loop {
            ticker.tick().await;
            let removed = cleanup_registry.remove_stale(stale_secs);
            if removed > 0 {
                info!("🧹 Removed {} stale agent(s); {} active", removed, cleanup_registry.len());
            }
        }
    });

    info!("✅ Registry listening");

    let mut stream = agent.listen().await?;

    while let Some(msg) = stream.next().await {
        match msg.performative {
            // Agent announcing itself on startup
            Performative::Subscribe => {
                match serde_json::from_slice::<AgentAnnouncement>(&msg.payload) {
                    Ok(ann) => {
                        info!(
                            agent_id = %ann.agent_id,
                            subjects = ?ann.subjects,
                            caps = ?ann.capabilities,
                            "📥 Registered"
                        );
                        registry.register(&ann);
                    }
                    Err(e) => warn!(error = %e, "Failed to parse Subscribe payload"),
                }
            }

            // Heartbeat — refresh last_seen, re-register if unknown
            Performative::Inform => {
                match serde_json::from_slice::<AgentAnnouncement>(&msg.payload) {
                    Ok(ann) => {
                        registry.register(&ann);
                        tracing::debug!(agent_id = %ann.agent_id, "💓 Heartbeat");
                    }
                    Err(e) => warn!(error = %e, "Failed to parse Inform payload"),
                }
            }

            // Discovery query — reply with matching agents
            Performative::Query => {
                match serde_json::from_slice::<AgentQuery>(&msg.payload) {
                    Ok(query) => {
                        let matches = registry.query(&query);
                        info!(
                            from = %msg.source.0,
                            count = matches.len(),
                            "🔍 Query answered"
                        );
                        let response = AgentQueryResponse { agents: matches };
                        match serde_json::to_vec(&response) {
                            Ok(bytes) => {
                                if let Err(e) = agent
                                    .reply(&msg, Performative::Inform, Bytes::from(bytes))
                                    .await
                                {
                                    warn!(error = %e, "Failed to send query response");
                                }
                            }
                            Err(e) => warn!(error = %e, "Failed to serialize query response"),
                        }
                    }
                    Err(e) => warn!(error = %e, "Failed to parse Query payload"),
                }
            }

            other => {
                tracing::debug!(performative = ?other, from = %msg.source.0, "Ignored message");
            }
        }
    }

    Ok(())
}
