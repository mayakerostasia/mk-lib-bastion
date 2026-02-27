use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use simian_base_api::transport::{AcpMessage, AgentId, Transport};
use simian_nats_streams::transport::NatsTransport;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

/// Message sent to visualization clients
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VizMessage {
    timestamp: i64,
    from_agent: String,
    to_agent: Option<String>,
    performative: String,
    message_id: String,
    conversation_id: String,
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<VizMessage>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("simian_viz_bridge=info,tower_http=debug")
        .init();

    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let ws_port = std::env::var("WS_PORT").unwrap_or_else(|_| "3030".to_string());
    let base_subject = std::env::var("BASE_SUBJECT").unwrap_or_else(|_| "agents".to_string());

    info!("Connecting to NATS at {} with base subject '{}'", nats_url, base_subject);
    
    // Create Transport using our proper abstraction
    let transport = NatsTransport::new(&nats_url, &base_subject).await?;
    info!("✅ Connected to NATS transport");

    // Broadcast channel for WebSocket clients
    let (tx, _rx) = broadcast::channel::<VizMessage>(1000);
    let state = AppState { tx: tx.clone() };

    // Spawn transport subscriber task
    tokio::spawn(transport_subscriber_task(transport, tx));

    // Build Axum app with WebSocket endpoint
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_handler))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", ws_port);
    info!("🚀 WebSocket server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Send task: forward broadcast messages to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Receive task: handle incoming WebSocket messages (pings, etc.)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                Message::Ping(_) | Message::Pong(_) => continue,
                _ => continue,
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    info!("WebSocket client disconnected");
}

/// Subscribe to agent messages using proper Transport abstraction
async fn transport_subscriber_task(
    transport: NatsTransport,
    tx: broadcast::Sender<VizMessage>,
) {
    info!("🔍 Subscribing as observer agent to receive all agent messages");
    
    // Create a special observer agent ID
    let observer_id = AgentId("__viz_observer__".to_string());
    
    match transport.subscribe(&observer_id).await {
        Ok(mut message_stream) => {
            info!("✅ Successfully subscribed to agent transport stream");
            
            // Process incoming ACP messages
            while let Some(acp_msg) = message_stream.next().await {
                match transform_acp_to_viz(&acp_msg) {
                    Ok(viz_msg) => {
                        info!(
                            "📨 Message: {} → {} ({})", 
                            viz_msg.from_agent, 
                            viz_msg.to_agent.as_deref().unwrap_or("broadcast"),
                            viz_msg.performative
                        );
                        
                        if let Err(e) = tx.send(viz_msg) {
                            error!("Failed to broadcast message to WebSocket clients: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to transform ACP message: {}", e);
                    }
                }
            }
            
            error!("Transport message stream ended unexpectedly");
        }
        Err(e) => {
            error!("❌ Failed to subscribe to transport: {}", e);
        }
    }
}

/// Transform AcpMessage from our transport to VizMessage for browser
fn transform_acp_to_viz(acp: &AcpMessage) -> Result<VizMessage> {
    Ok(VizMessage {
        timestamp: acp.timestamp,
        from_agent: acp.source.0.clone(),
        to_agent: acp.target.as_ref().map(|t| t.0.clone()),
        performative: format!("{:?}", acp.performative),
        message_id: acp.message_id.to_string(),
        conversation_id: acp.conversation_id.to_string(),
    })
}
