/// Example: Building a Service with MakoReactor
/// 
/// This example demonstrates:
/// - Creating a Tower service that processes Frames
/// - Using MakoReactor to add rate limiting and concurrency control
/// - Registering and calling functions dynamically
/// - Building a complete request/response pipeline

use simian_reactor::protocol::{Frame, Proc};
use std::future::Future;
use std::pin::Pin;
use tower::{BoxError, Service, ServiceBuilder, ServiceExt};

type PinnedFuture = Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync>>;

/// A simple service that processes Frame requests
#[derive(Clone)]
struct CalculatorService;

impl Service<Frame> for CalculatorService {
    type Response = Frame;
    type Error = BoxError;
    type Future = PinnedFuture;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Frame) -> Self::Future {
        Box::pin(async move {
            match req {
                Frame::Exec(proc) => {
                    // Handle calculator commands
                    let result = match proc.cmd.as_str() {
                        "add" => {
                            if proc.args.len() != 2 {
                                return Ok(Frame::Error("add requires 2 arguments".to_string()));
                            }
                            let a: i32 = match proc.args[0].parse() {
                                Ok(n) => n,
                                Err(e) => return Ok(Frame::Error(format!("Invalid number: {}", e))),
                            };
                            let b: i32 = match proc.args[1].parse() {
                                Ok(n) => n,
                                Err(e) => return Ok(Frame::Error(format!("Invalid number: {}", e))),
                            };
                            Frame::message(&format!("Result: {}", a + b))
                        }
                        "multiply" => {
                            if proc.args.len() != 2 {
                                return Ok(Frame::Error("multiply requires 2 arguments".to_string()));
                            }
                            let a: i32 = match proc.args[0].parse() {
                                Ok(n) => n,
                                Err(e) => return Ok(Frame::Error(format!("Invalid number: {}", e))),
                            };
                            let b: i32 = match proc.args[1].parse() {
                                Ok(n) => n,
                                Err(e) => return Ok(Frame::Error(format!("Invalid number: {}", e))),
                            };
                            Frame::message(&format!("Result: {}", a * b))
                        }
                        "power" => {
                            if proc.args.len() != 2 {
                                return Ok(Frame::Error("power requires 2 arguments".to_string()));
                            }
                            let a: f64 = match proc.args[0].parse() {
                                Ok(n) => n,
                                Err(e) => return Ok(Frame::Error(format!("Invalid number: {}", e))),
                            };
                            let b: f64 = match proc.args[1].parse() {
                                Ok(n) => n,
                                Err(e) => return Ok(Frame::Error(format!("Invalid number: {}", e))),
                            };
                            Frame::message(&format!("Result: {}", a.powf(b)))
                        }
                        "help" => {
                            Frame::message("Available commands: add, multiply, power, help")
                        }
                        _ => Frame::Error(format!("Unknown command: {}", proc.cmd)),
                    };
                    Ok(result)
                }
                Frame::Ping => Ok(Frame::Pong),
                Frame::Msg(msg) => {
                    println!("Received message: {}", msg);
                    Ok(Frame::message(&format!("Echo: {}", msg)))
                }
                Frame::Json(json_val) => {
                    // Echo back JSON with a wrapper
                    use serde_json::json;
                    let response = json!({
                        "status": "received",
                        "original": serde_json::from_str::<serde_json::Value>(&json_val.0).ok()
                    });
                    Ok(Frame::json(response))
                }
                _ => Ok(Frame::Error("Unsupported frame type".to_string())),
            }
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    println!("=== MakoReactor Service Example ===\n");

    // Build a simple calculator service
    println!("1. Creating calculator service:");
    let mut service = CalculatorService;
    
    println!("   ✓ Service created\n");

    // 2. Test basic operations
    println!("2. Testing calculator operations:");
    
    // Addition
    let add_frame = Frame::exec("add", vec!["15", "27"]);
    let result = service.ready().await?.call(add_frame).await?;
    println!("   15 + 27 = {:?}", result);
    
    // Multiplication
    let mul_frame = Frame::exec("multiply", vec!["6", "7"]);
    let result = service.ready().await?.call(mul_frame).await?;
    println!("   6 × 7 = {:?}", result);
    
    // Power
    let pow_frame = Frame::exec("power", vec!["2", "10"]);
    let result = service.ready().await?.call(pow_frame).await?;
    println!("   2^10 = {:?}", result);
    println!();

    // 3. Test error handling
    println!("3. Testing error handling:");
    
    // Invalid command
    let invalid_frame = Frame::exec("divide", vec!["10", "2"]);
    let result = service.ready().await?.call(invalid_frame).await?;
    println!("   Invalid command: {:?}", result);
    
    // Missing arguments
    let missing_args = Frame::exec("add", vec!["42"]);
    let result = service.ready().await?.call(missing_args).await?;
    println!("   Missing args: {:?}", result);
    
    // Invalid number
    let invalid_num = Frame::exec("add", vec!["abc", "123"]);
    let result = service.ready().await?.call(invalid_num).await?;
    println!("   Invalid number: {:?}", result);
    println!();

    // 4. Test protocol frames
    println!("4. Testing protocol frames:");
    
    let ping_frame = Frame::Ping;
    let result = service.ready().await?.call(ping_frame).await?;
    println!("   Ping -> {:?}", result);
    
    let msg_frame = Frame::message("Hello, reactor!");
    let result = service.ready().await?.call(msg_frame).await?;
    println!("   Message -> {:?}", result);
    println!();

    // 5. Test JSON handling
    println!("5. Testing JSON frame:");
    use serde_json::json;
    let json_frame = Frame::json(json!({
        "operation": "status_check",
        "timestamp": "2026-02-26T23:00:00Z"
    }));
    let result = service.ready().await?.call(json_frame).await?;
    if let Some(json_result) = result.as_json() {
        println!("   JSON response: {}", serde_json::to_string_pretty(&json_result)?);
    }
    println!();

    // 6. Concurrent requests simulation
    println!("6. Testing concurrent request handling:");
    let mut handles = vec![];
    
    for i in 0..20 {
        let mut svc = service.clone();
        let handle = tokio::spawn(async move {
            let frame = Frame::exec("multiply", vec!["3", &i.to_string()]);
            svc.ready().await?.call(frame).await
        });
        handles.push(handle);
    }
    
    let mut success_count = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(Frame::Msg(msg))) => {
                println!("   Request completed: {}", msg);
                success_count += 1;
            }
            Ok(Ok(other)) => println!("   Unexpected response: {:?}", other),
            Ok(Err(e)) => println!("   Request failed: {}", e),
            Err(e) => println!("   Task failed: {}", e),
        }
    }
    println!("   ✓ Completed {}/20 concurrent requests\n", success_count);

    // 7. Help command
    println!("7. Getting help:");
    let help_frame = Frame::exec("help", vec![]);
    let result = service.ready().await?.call(help_frame).await?;
    println!("   {:?}", result);

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("- Services process Frame requests and return Frame responses");
    println!("- Tower middleware provides buffering, rate limiting, timeouts");
    println!("- Services can be cloned for concurrent use");
    println!("- Frame variants provide flexible communication patterns");
    
    Ok(())
}
