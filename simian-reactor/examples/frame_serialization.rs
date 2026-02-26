/// Example: Frame Serialization with rkyv
/// 
/// This example demonstrates:
/// - Creating different Frame variants
/// - Encoding/decoding frames with rkyv
/// - Using the From<Frame> for Bytes trait
/// - Working with JSON frames
/// - Error handling in frame serialization

use bytes::Bytes;
use serde_json::json;
use simian_reactor::protocol::{Frame, Proc};
use tower::BoxError;

// Helper to convert BoxError to anyhow::Error
fn encode_frame(frame: &Frame) -> anyhow::Result<Vec<u8>> {
    frame.encode().map_err(|e| anyhow::anyhow!("{}", e))
}

fn decode_frame(data: &[u8]) -> anyhow::Result<Frame> {
    Frame::decode(data).map_err(|e| anyhow::anyhow!("{}", e))
}

fn main() -> anyhow::Result<()> {
    println!("=== Frame Serialization Example ===\n");

    // 1. Simple Message Frame
    println!("1. Creating and serializing a Message frame:");
    let msg_frame = Frame::message("Hello from simian-reactor!");
    let encoded = encode_frame(&msg_frame)?;
    println!("   Encoded size: {} bytes", encoded.len());
    
    let decoded = decode_frame(&encoded)?;
    println!("   Decoded: {:?}\n", decoded);

    // 2. JSON Frame
    println!("2. Creating and serializing a JSON frame:");
    let json_data = json!({
        "agent": "reactor-001",
        "task": "process_data",
        "priority": 5,
        "metadata": {
            "timestamp": "2026-02-26T23:00:00Z",
            "source": "example"
        }
    });
    
    let json_frame = Frame::json(json_data.clone());
    let encoded = &json_frame.encode().map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("   Encoded size: {} bytes", encoded.len());
    
    let decoded = decode_frame(&encoded)?;
    if let Some(extracted_json) = decoded.as_json() {
        println!("   Decoded JSON: {}", serde_json::to_string_pretty(&extracted_json)?);
    }
    println!();

    // 3. Exec Frame (command execution)
    println!("3. Creating and serializing an Exec frame:");
    let exec_frame = Frame::exec("process_task", vec!["arg1", "arg2", "--verbose"]);
    let encoded = &exec_frame.encode().map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("   Encoded size: {} bytes", encoded.len());
    
    let decoded = decode_frame(&encoded)?;
    if let Frame::Exec(proc) = decoded {
        println!("   Command: {}", proc.cmd);
        println!("   Args: {:?}", proc.args);
    }
    println!();

    // 4. Binary Data Frame
    println!("4. Creating and serializing a Bytes frame:");
    let binary_data = vec![0xFF, 0xAA, 0xBB, 0xCC, 0x01, 0x02, 0x03];
    let bytes_frame = Frame::bytes(binary_data.into_boxed_slice());
    let encoded = encode_frame(&bytes_frame)?;
    println!("   Encoded size: {} bytes", encoded.len());
    
    let decoded = decode_frame(&encoded)?;
    if let Frame::Bytes(data) = decoded {
        println!("   Binary data length: {} bytes", data.len());
        println!("   Data: {:02X?}", &data[..]);
    }
    println!();

    // 5. Using From<Frame> for Bytes trait
    println!("5. Using From trait for Bytes conversion:");
    let ping_frame = Frame::ping();
    let bytes: Bytes = ping_frame.clone().into();
    println!("   Frame::Ping -> Bytes: {} bytes", bytes.len());
    
    let decoded_frame: Frame = Frame::from(bytes);
    println!("   Bytes -> Frame: {:?}", decoded_frame);
    println!();

    // 6. Protocol Frames (Ping/Pong)
    println!("6. Protocol frames:");
    for frame in [Frame::Ping, Frame::Pong, Frame::Fin, Frame::Close] {
        let encoded = &frame.encode().map_err(|e| anyhow::anyhow!("{}", e))?;
        let decoded = decode_frame(&encoded)?;
        println!("   {:?} roundtrip successful ({} bytes)", decoded, encoded.len());
    }
    println!();

    // 7. Error Frame
    println!("7. Error frame:");
    let error_frame = Frame::Error("Something went wrong".to_string());
    let encoded = &error_frame.encode().map_err(|e| anyhow::anyhow!("{}", e))?;
    let decoded = decode_frame(&encoded)?;
    println!("   {:?}\n", decoded);

    // 8. SendBox Frame (message routing)
    println!("8. SendBox frame (for message routing):");
    use simian_reactor::protocol::SendBox;
    let sendbox = SendBox {
        from: "agent-001".to_string(),
        addr: "agent-002".to_string(),
        data: vec![1, 2, 3, 4, 5].into_boxed_slice(),
    };
    let sendbox_frame = Frame::SendBox(sendbox);
    let encoded = &sendbox_frame.encode().map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("   Encoded size: {} bytes", encoded.len());
    
    let decoded = decode_frame(&encoded)?;
    if let Frame::SendBox(sb) = decoded {
        println!("   From: {}", sb.from);
        println!("   To: {}", sb.addr);
        println!("   Payload: {} bytes", sb.data.len());
    }
    println!();

    // 9. Benchmarking encoding/decoding
    println!("9. Performance test (1000 iterations):");
    let test_frame = Frame::json(json!({
        "type": "benchmark",
        "data": vec![1, 2, 3, 4, 5],
        "nested": {
            "value": 42
        }
    }));
    
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let encoded = &test_frame.encode().map_err(|e| anyhow::anyhow!("{}", e))?;
        let _decoded = decode_frame(&encoded)?;
    }
    let elapsed = start.elapsed();
    println!("   1000 encode/decode cycles: {:?}", elapsed);
    println!("   Avg per cycle: {:?}", elapsed / 1000);

    println!("\n=== Example Complete ===");
    Ok(())
}
