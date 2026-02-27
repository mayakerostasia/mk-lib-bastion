//! Agent Role Implementations
//!
//! This module contains the various agent roles and their frame handling logic.
//! Each role demonstrates different patterns and use cases.

use simian_nats_streams::Frame;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{BoxError, Service, ServiceBuilder};
use tracing::{debug, info, warn};

/// Type alias for frame processing futures
type FrameFuture = Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send>>;

// ============================================================================
// Role: ECHO - Simple echo server that returns messages
// ============================================================================

/// Echo handler - returns the received frame
pub async fn echo_handler(frame: Frame) -> Result<Frame, BoxError> {
    debug!("Echo received: {:?}", frame);
    match frame {
        Frame::Ping => Ok(Frame::Pong),
        Frame::Msg(ref msg) => {
            info!("Echo message: {}", msg);
            Ok(frame)
        }
        _ => {
            warn!("Bad Command for Echo Handler, we accept Ping or Msg");
            // Return anyways so we stay running!
            Ok(Frame::Error("Invalid Command! Try Ping or Msg".to_string()))
        }
        
    }
}

// ============================================================================
// Role: PROCESSOR - Process and transform messages
// ============================================================================

/// Processor handler - transforms messages with processing metadata
pub async fn processor_handler(frame: Frame) -> Result<Frame, BoxError> {
    debug!("Processor received: {:?}", frame);
    
    match frame {
        Frame::Ping => Ok(Frame::Pong),
        
        Frame::Msg(msg) => {
            info!("Processing message: {}", msg);
            let processed = format!("[PROCESSED] {}", msg);
            Ok(Frame::message(&processed))
        }
        
        Frame::Exec(proc) => {
            info!("Processing command: {} {:?}", proc.cmd, proc.args);
            let result = match proc.cmd.as_str() {
                "uppercase" => {
                    let text = proc.args.join(" ");
                    Frame::message(&text.to_uppercase())
                }
                "lowercase" => {
                    let text = proc.args.join(" ");
                    Frame::message(&text.to_lowercase())
                }
                "reverse" => {
                    let text = proc.args.join(" ");
                    let reversed: String = text.chars().rev().collect();
                    Frame::message(&reversed)
                }
                "count" => {
                    let text = proc.args.join(" ");
                    Frame::message(&format!("Character count: {}", text.len()))
                }
                _ => Frame::Error(format!("Unknown command: {}", proc.cmd)),
            };
            Ok(result)
        }
        
        Frame::Json(json) => {
            info!("Processing JSON: {}", json.0);
            // Add processing metadata
            let mut val: serde_json::Value = serde_json::from_str(&json.0)?;
            if let Some(obj) = val.as_object_mut() {
                obj.insert("processed_by".to_string(), serde_json::json!("processor"));
                obj.insert("timestamp".to_string(), serde_json::json!(chrono::Local::now().to_rfc3339()));
            }
            Ok(Frame::json(val))
        }
        
        _ => {
            warn!("Bad Command for Echo Handler, we accept Ping or Msg");
            // Return anyways so we stay running!
            Ok(Frame::Error("Invalid Command! Try Ping or Msg".to_string()))
        }
        // other => Ok(other),
    }
}

// ============================================================================
// Role: GENERATOR - Generate data/events
// ============================================================================

/// Generator handler - generates responses based on requests
pub async fn generator_handler(frame: Frame) -> Result<Frame, BoxError> {
    debug!("Generator received: {:?}", frame);
    
    match frame {
        Frame::Ping => Ok(Frame::Pong),
        
        Frame::Exec(proc) => {
            info!("Generating: {} {:?}", proc.cmd, proc.args);
            let result = match proc.cmd.as_str() {
                "uuid" => {
                    let id = uuid::Uuid::new_v4();
                    Frame::message(&id.to_string())
                }
                "random" => {
                    let num = if proc.args.is_empty() {
                        rand::random::<u32>()
                    } else {
                        let max: u32 = proc.args[0].parse().unwrap_or(100);
                        rand::random::<u32>() % max
                    };
                    Frame::message(&num.to_string())
                }
                "timestamp" => {
                    let ts = chrono::Local::now().to_rfc3339();
                    Frame::message(&ts)
                }
                "sequence" => {
                    let count: usize = proc.args.get(0)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(5);
                    let sequence: Vec<i32> = (1..=count as i32).collect();
                    Frame::json(serde_json::json!({ "sequence": sequence }))
                }
                _ => Frame::Error(format!("Unknown generator command: {}", proc.cmd)),
            };
            Ok(result)
        }
        
        _ => {
            warn!("Bad Command for Echo Handler, we accept Ping or Msg");
            // Return anyways so we stay running!
            Ok(Frame::Error("Invalid Command! Try Ping or Msg".to_string()))
        }
    }
}

// ============================================================================
// Role: VERIFIER - Validate and verify data
// ============================================================================

/// Verifier handler - validates messages and data
pub async fn verifier_handler(frame: Frame) -> Result<Frame, BoxError> {
    debug!("Verifier received: {:?}", frame);
    
    match frame {
        Frame::Ping => Ok(Frame::Pong),
        
        Frame::Msg(msg) => {
            info!("Verifying message: {}", msg);
            let is_valid = !msg.is_empty() && msg.len() < 1000;
            let status = if is_valid { "VALID" } else { "INVALID" };
            let reason = if !is_valid && msg.is_empty() {
                "empty message"
            } else if !is_valid {
                "too long"
            } else {
                "ok"
            };
            Ok(Frame::json(serde_json::json!({
                "status": status,
                "reason": reason,
                "length": msg.len()
            })))
        }
        
        Frame::Json(json) => {
            info!("Verifying JSON: {}", json.0);
            match serde_json::from_str::<serde_json::Value>(&json.0) {
                Ok(val) => {
                    Ok(Frame::json(serde_json::json!({
                        "status": "VALID",
                        "type": match &val {
                            serde_json::Value::Object(_) => "object",
                            serde_json::Value::Array(_) => "array",
                            serde_json::Value::String(_) => "string",
                            serde_json::Value::Number(_) => "number",
                            serde_json::Value::Bool(_) => "boolean",
                            serde_json::Value::Null => "null",
                        }
                    })))
                }
                Err(e) => {
                    Ok(Frame::Error(format!("Invalid JSON: {}", e)))
                }
            }
        }
        
        Frame::Exec(proc) => {
            info!("Verifying command: {} {:?}", proc.cmd, proc.args);
            // Verify command structure
            let is_valid = !proc.cmd.is_empty();
            Ok(Frame::json(serde_json::json!({
                "status": if is_valid { "VALID" } else { "INVALID" },
                "command": proc.cmd,
                "arg_count": proc.args.len()
            })))
        }
        
        other => Ok(other),
    }
}

// ============================================================================
// Tower Service Example - Demonstrates service pattern
// ============================================================================

/// Tower service handler - demonstrates async processing patterns
pub async fn tower_service_handler(frame: Frame) -> Result<Frame, BoxError> {
    debug!("Tower service processing: {:?}", frame);
    
    match frame {
        Frame::Ping => Ok(Frame::Pong),
        Frame::Msg(msg) => {
            // Simulate some async processing
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok(Frame::message(&format!("[TOWER] {}", msg)))
        }
        Frame::Exec(proc) => {
            info!("Tower executing: {}", proc.cmd);
            // Simulate command execution
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            Ok(Frame::message(&format!("Executed: {}", proc.cmd)))
        }
        other => Ok(other),
    }
}
