use simian_http_listener::{Server, messaging};
use simian_reactor::protocol::Frame;
use axum::{routing::post, Router};
use std::sync::Arc;
use tokio::sync::Mutex;

// Mock sender for demonstration purposes
#[derive(Clone)]
struct ConsoleSender {
    messages: Arc<Mutex<Vec<Frame>>>,
}

impl ConsoleSender {
    fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl simian_http_listener::MessageSender for ConsoleSender {
    async fn send(&self, frame: Frame) -> Result<(), anyhow::Error> {
        println!("📨 Received Frame: {:?}", frame);
        self.messages.lock().await.push(frame);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    simian_tracing::initialize()?;

    println!("🚀 Starting HTTP Messaging Server on http://localhost:8080");
    println!("\nEndpoints:");
    println!("  - GET  http://localhost:8080/healthz");
    println!("  - GET  http://localhost:8080/readyz");
    println!("  - POST http://localhost:8080/message\n");

    println!("Try these curl commands:");
    println!("\n  # Send a simple message:");
    println!(r#"  curl -X POST http://localhost:8080/message \"#);
    println!(r#"    -H "Content-Type: application/json" \"#);
    println!(r#"    -d '{{"frame_type":"Msg","content":"Hello from HTTP!"}}'"#);
    
    println!("\n  # Send JSON data:");
    println!(r#"  curl -X POST http://localhost:8080/message \"#);
    println!(r#"    -H "Content-Type: application/json" \"#);
    println!(r#"    -d '{{"frame_type":"Json","content":"{{\"key\":\"value\"}}"}}'#);
    
    println!("\n  # Send Ping:");
    println!(r#"  curl -X POST http://localhost:8080/message \"#);
    println!(r#"    -H "Content-Type: application/json" \"#);
    println!(r#"    -d '{{"frame_type":"Ping","content":""}}'"#);
    
    println!("\n");

    // Create a console sender to print messages
    let sender = Arc::new(ConsoleSender::new());

    // Create custom routes with message endpoint
    let routes = Router::new()
        .route("/message", post(messaging::send_message_handler))
        .with_state(sender);

    // Create and start the server
    let server = Server::new("0.0.0.0:8080");
    server.listen(Some(routes), 5000, true).await?;

    Ok(())
}
