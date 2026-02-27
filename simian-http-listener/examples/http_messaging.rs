//! HTTP Messaging with Kong/Monkey Pattern Example
//!
//! This example demonstrates how to:
//! - Use HTTP POST /message endpoint to send Frame messages
//! - Integrate with Kong/Monkey pattern (NATS-based communication)
//! - Bridge HTTP clients to NATS services
//! - Use Monkey to send frames to Kong listeners
//!
//! Architecture:
//!
//!   HTTP Client
//!        |
//!        | POST /message (JSON)
//!        v
//!   HTTP Server (this example)
//!        |
//!        | MonkeySender (Frame)
//!        v
//!      NATS (Frame protocol)
//!        |
//!        | Kong listens on subject
//!        v
//!   Service Handler (processes Frame)
//!
//! Run with:
//!   cargo run --example http_messaging -p simian-http-listener --features messaging
//!
//! Test endpoints:
//!   curl -X POST http://localhost:8080/message \
//!     -H "Content-Type: application/json" \
//!     -d '{"frame_type":"Msg","content":"Hello Kong!"}'
//!
//! See Kong example in simian-nats-streams for listener implementation.

use axum::{routing::post, Router};
use simian_http_listener::{messaging, Server, MessageSender};
use simian_reactor::protocol::Frame;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Example MessageSender that demonstrates Frame handling.
/// In production, this would send frames to NATS via Monkey.
///
/// This implementation stores received frames for demonstration purposes.
#[derive(Clone)]
struct DemoMessageSender {
    /// Stores received frames for inspection
    messages: Arc<Mutex<Vec<Frame>>>,
}

impl DemoMessageSender {
    fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get all received frames (for testing)
    async fn get_messages(&self) -> Vec<Frame> {
        self.messages.lock().await.clone()
    }
}

/// Implement MessageSender trait for DemoMessageSender.
/// This is the interface that integrates with the HTTP handler.
#[async_trait::async_trait]
impl MessageSender for DemoMessageSender {
    async fn send(&self, frame: Frame) -> Result<(), anyhow::Error> {
        let frame_info = format_frame_info(&frame);
        println!("📨 Received Frame: {}", frame_info);
        self.messages.lock().await.push(frame);
        Ok(())
    }
}

/// Format frame information for display
fn format_frame_info(frame: &Frame) -> String {
    match frame {
        Frame::Msg(msg) => format!("Msg({})", msg),
        Frame::Bytes(bytes) => format!("Bytes({} bytes)", bytes.len()),
        Frame::Json(json) => format!("Json({})", json.0),
        Frame::Exec(proc) => format!("Exec({} {})", proc.cmd, proc.args.join(" ")),
        Frame::Ping => "Ping".to_string(),
        Frame::Pong => "Pong".to_string(),
        Frame::Fin => "Fin".to_string(),
        Frame::Close => "Close".to_string(),
        Frame::Error(e) => format!("Error({})", e),
        Frame::SendBox(sb) => format!("SendBox({:?})", sb),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    simian_tracing::initialize()?;

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║    Simian HTTP Messaging with Kong/Monkey Pattern          ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("🚀 HTTP Server starting on http://localhost:8080");
    println!();
    println!("📋 Available Endpoints:");
    println!("  GET  /healthz   → Health check (always available)");
    println!("  GET  /readyz    → Readiness check");
    println!("  POST /message   → Send Frame message (requires messaging feature)");
    println!();
    println!("🔄 Data Flow (Kong/Monkey Pattern):");
    println!("  1. HTTP Client sends JSON to POST /message");
    println!("  2. HTTP Server converts JSON to Frame");
    println!("  3. MessageSender receives Frame");
    println!("  4. [Production] Monkey sends Frame to NATS");
    println!("  5. [Production] Kong listener receives Frame");
    println!("  6. [Production] Service handler processes Frame");
    println!();

    // Create demo sender
    let sender = Arc::new(DemoMessageSender::new());

    // Create custom routes with message endpoint
    let routes = Router::new()
        .route("/message", post(messaging::send_message_handler))
        .with_state(sender.clone());

    // Create and start the server
    let server = Server::new("0.0.0.0:8080");

    println!("📤 Frame Types Supported:");
    println!();
    println!("  Msg - Simple text message");
    println!("    POST /message -d '{{ \"frame_type\": \"Msg\", \"content\": \"Hello!\" }}'");
    println!();
    println!("  Bytes - Raw bytes (content as string)");
    println!("    POST /message -d '{{ \"frame_type\": \"Bytes\", \"content\": \"binary\" }}'");
    println!();
    println!("  Json - JSON structured data");
    println!("    POST /message -d '{{ \"frame_type\": \"Json\", \"content\": \"{{\\\"key\\\":\\\"value\\\"}}\" }}'");
    println!();
    println!("  Exec - Command execution");
    println!("    POST /message -d '{{ \"frame_type\": \"Exec\", \"content\": \"ls -la /tmp\" }}'");
    println!();
    println!("  Ping - Connection keep-alive");
    println!("    POST /message -d '{{ \"frame_type\": \"Ping\", \"content\": \"\" }}'");
    println!();
    println!("  Pong - Ping response");
    println!("    POST /message -d '{{ \"frame_type\": \"Pong\", \"content\": \"\" }}'");
    println!();
    println!("  Fin - Graceful close");
    println!("    POST /message -d '{{ \"frame_type\": \"Fin\", \"content\": \"\" }}'");
    println!();
    println!("  Close - Hard close");
    println!("    POST /message -d '{{ \"frame_type\": \"Close\", \"content\": \"\" }}'");
    println!();
    println!("🧪 Try These Commands:");
    println!();

    println!("  # Send a simple message");
    println!("  $ curl -X POST http://localhost:8080/message \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!(r#"    -d '{{"frame_type":"Msg","content":"Hello from HTTP!"}}'  "#);
    println!();

    println!("  # Send structured JSON data");
    println!("  $ curl -X POST http://localhost:8080/message \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!(r#"    -d '{{"frame_type":"Json","content":"{{\"task\":\"process\",\"id\":123}}"}}'  "#);
    println!();

    println!("  # Send command to execute");
    println!("  $ curl -X POST http://localhost:8080/message \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!(r#"    -d '{{"frame_type":"Exec","content":"echo Hello World"}}'  "#);
    println!();

    println!("  # Keep-alive ping");
    println!("  $ curl -X POST http://localhost:8080/message \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!(r#"    -d '{{"frame_type":"Ping","content":""}}'  "#);
    println!();

    println!("  # Check health");
    println!("  $ curl http://localhost:8080/healthz");
    println!();

    println!("  # Pretty print response");
    println!("  $ curl -s -X POST http://localhost:8080/message \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!(r#"    -d '{{"frame_type":"Msg","content":"test"}}' | jq ."#);
    println!();

    println!("❌ Error Examples:");
    println!();

    println!("  # Invalid frame type");
    println!("  $ curl -X POST http://localhost:8080/message \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!(r#"    -d '{{"frame_type":"InvalidType","content":"test"}}'  "#);
    println!();

    println!("  # Invalid JSON content");
    println!("  $ curl -X POST http://localhost:8080/message \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!(r#"    -d '{{"frame_type":"Json","content":"not valid json {{"}}'  "#);
    println!();

    println!("📊 Response Codes:");
    println!("  202 Accepted       - Message queued successfully");
    println!("  400 Bad Request    - Invalid frame type or content");
    println!("  500 Server Error   - Failed to send frame");
    println!();

    println!("🔌 Integration with Kong (simian-nats-streams):");
    println!();
    println!("  This example uses a DemoMessageSender that prints frames.");
    println!("  In production, replace with MonkeySender:");
    println!();
    println!("    use simian_nats_streams::{{Monkey, MonkeySender}};");
    println!();
    println!("    let monkey = Monkey::new(\"my.subject\", \"nats://localhost:4222\").await;");
    println!("    let sender = Arc::new(MonkeySender::new(monkey));");
    println!();
    println!("  Then messages sent via HTTP POST /message are forwarded to:");
    println!("    - Kong listener on NATS subject \"my.subject\"");
    println!("    - Tower service stack for processing");
    println!("    - Other agents listening on NATS");
    println!();
    println!("🛑 Press Ctrl+C to shutdown");
    println!();

    // Start server with custom routes (health endpoints + message handler)
    server.listen(Some(routes), 5000, true).await?;

    println!("\n✅ Server shutdown complete");
    Ok(())
}
