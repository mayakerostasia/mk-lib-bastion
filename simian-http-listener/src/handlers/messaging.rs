#[cfg(feature = "messaging")]
use axum::{extract::State, http::StatusCode, Json};
#[cfg(feature = "messaging")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "messaging")]
use simian_reactor::protocol::Frame;
#[cfg(feature = "messaging")]
use std::sync::Arc;
#[cfg(feature = "messaging")]
use tracing::{debug, error};

#[cfg(feature = "messaging")]
#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageRequest {
    pub frame_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[cfg(feature = "messaging")]
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageResponse {
    pub status: String,
    pub message_id: String,
}

#[cfg(feature = "messaging")]
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[cfg(feature = "messaging")]
#[async_trait::async_trait]
pub trait MessageSender: Send + Sync {
    async fn send(&self, frame: Frame) -> Result<(), anyhow::Error>;
}

#[cfg(feature = "messaging")]
pub async fn send_message_handler<T: MessageSender>(
    State(sender): State<Arc<T>>,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<SendMessageResponse>), (StatusCode, Json<ErrorResponse>)> {
    debug!("Received message request: {:?}", req);

    // Parse frame type and create Frame
    let frame = match req.frame_type.as_str() {
        "Msg" => Frame::Msg(req.content),
        "Bytes" => {
            // Accept raw string bytes
            Frame::Bytes(req.content.into_bytes().into_boxed_slice())
        }
        "Json" => {
            // Validate JSON
            let json_val: serde_json::Value = serde_json::from_str(&req.content).map_err(|e| {
                error!("Invalid JSON content: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid JSON content".to_string(),
                        details: Some(e.to_string()),
                    }),
                )
            })?;
            Frame::json(json_val)
        }
        "Exec" => {
            // Parse exec command - format: "command arg1 arg2..."
            let parts: Vec<&str> = req.content.split_whitespace().collect();
            if parts.is_empty() {
                error!("Empty exec command");
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Empty exec command".to_string(),
                        details: Some("Exec frame requires at least a command".to_string()),
                    }),
                ));
            }
            let cmd = parts[0];
            let args = parts[1..].to_vec();
            Frame::exec(cmd, args)
        }
        "Ping" => Frame::Ping,
        "Pong" => Frame::Pong,
        "Fin" => Frame::Fin,
        "Close" => Frame::Close,
        _ => {
            error!("Unsupported frame type: {}", req.frame_type);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid frame type".to_string(),
                    details: Some(format!(
                        "Supported types: Msg, Bytes, Json, Exec, Ping, Pong, Fin, Close"
                    )),
                }),
            ));
        }
    };

    // Send the frame
    sender.send(frame).await.map_err(|e| {
        error!("Failed to send frame: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to send message".to_string(),
                details: Some(e.to_string()),
            }),
        )
    })?;

    let message_id = uuid::Uuid::new_v4().to_string();
    debug!("Message sent successfully: {}", message_id);

    Ok((
        StatusCode::ACCEPTED,
        Json(SendMessageResponse {
            status: "queued".to_string(),
            message_id,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Clone)]
    struct MockSender {
        frames: Arc<Mutex<Vec<Frame>>>,
    }

    impl MockSender {
        fn new() -> Self {
            Self {
                frames: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_frames(&self) -> Vec<Frame> {
            self.frames.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl MessageSender for MockSender {
        async fn send(&self, frame: Frame) -> Result<(), anyhow::Error> {
            self.frames.lock().unwrap().push(frame);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_send_msg_frame() {
        let sender = Arc::new(MockSender::new());
        let req = SendMessageRequest {
            frame_type: "Msg".to_string(),
            content: "Hello, world!".to_string(),
            metadata: None,
        };

        let result = send_message_handler(State(sender.clone()), Json(req)).await;
        assert!(result.is_ok());

        let frames = sender.get_frames();
        assert_eq!(frames.len(), 1);
        match &frames[0] {
            Frame::Msg(s) => assert_eq!(s, "Hello, world!"),
            _ => panic!("Expected Msg frame"),
        }
    }

    #[tokio::test]
    async fn test_send_json_frame() {
        let sender = Arc::new(MockSender::new());
        let req = SendMessageRequest {
            frame_type: "Json".to_string(),
            content: r#"{"key": "value", "count": 42}"#.to_string(),
            metadata: None,
        };

        let result = send_message_handler(State(sender.clone()), Json(req)).await;
        assert!(result.is_ok());

        let frames = sender.get_frames();
        assert_eq!(frames.len(), 1);
        match &frames[0] {
            Frame::Json(json_val) => {
                let value: serde_json::Value = json_val.clone().into();
                assert_eq!(value["key"], "value");
                assert_eq!(value["count"], 42);
            }
            _ => panic!("Expected Json frame"),
        }
    }

    #[tokio::test]
    async fn test_send_exec_frame() {
        let sender = Arc::new(MockSender::new());
        let req = SendMessageRequest {
            frame_type: "Exec".to_string(),
            content: "ls -la /tmp".to_string(),
            metadata: None,
        };

        let result = send_message_handler(State(sender.clone()), Json(req)).await;
        assert!(result.is_ok());

        let frames = sender.get_frames();
        assert_eq!(frames.len(), 1);
        match &frames[0] {
            Frame::Exec(proc) => {
                assert_eq!(proc.cmd, "ls");
                assert_eq!(proc.args, vec!["-la", "/tmp"]);
            }
            _ => panic!("Expected Exec frame"),
        }
    }

    #[tokio::test]
    async fn test_invalid_frame_type() {
        let sender = Arc::new(MockSender::new());
        let req = SendMessageRequest {
            frame_type: "Invalid".to_string(),
            content: "test".to_string(),
            metadata: None,
        };

        let result = send_message_handler(State(sender.clone()), Json(req)).await;
        assert!(result.is_err());

        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_invalid_json_content() {
        let sender = Arc::new(MockSender::new());
        let req = SendMessageRequest {
            frame_type: "Json".to_string(),
            content: "not valid json {".to_string(),
            metadata: None,
        };

        let result = send_message_handler(State(sender.clone()), Json(req)).await;
        assert!(result.is_err());

        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_ping_pong_frames() {
        let sender = Arc::new(MockSender::new());

        // Test Ping
        let req = SendMessageRequest {
            frame_type: "Ping".to_string(),
            content: "".to_string(),
            metadata: None,
        };
        let result = send_message_handler(State(sender.clone()), Json(req)).await;
        assert!(result.is_ok());

        // Test Pong
        let req = SendMessageRequest {
            frame_type: "Pong".to_string(),
            content: "".to_string(),
            metadata: None,
        };
        let result = send_message_handler(State(sender.clone()), Json(req)).await;
        assert!(result.is_ok());

        let frames = sender.get_frames();
        assert_eq!(frames.len(), 2);
        assert!(matches!(frames[0], Frame::Ping));
        assert!(matches!(frames[1], Frame::Pong));
    }
}
