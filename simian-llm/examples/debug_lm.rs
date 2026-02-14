/// Debug script to test LMStudio connectivity and endpoints

use reqwest::Client;
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 LMStudio Debug Test\n");

    let base_url = "http://192.168.1.7:1234";
    let token = "sk-lm-gLUHl47J:Cc8UhKBjQJlro3pNutt0";
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    println!("Testing connectivity to: {}\n", base_url);

    // Test 1: Health check endpoint
    println!("1️⃣ Testing /v1/models endpoint:");
    match client
        .get(format!("{}/v1/models", base_url))
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) => {
            println!("   Status: {}", response.status());
            match response.text().await {
                Ok(text) => {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        println!("   Parsed JSON:\n{}\n", serde_json::to_string_pretty(&json)?);
                    } else {
                        println!("   Response (raw):\n{}\n", text);
                    }
                }
                Err(e) => println!("   Error reading response: {}\n", e),
            }
        }
        Err(e) => println!("   Error: {} (likely not reachable)\n", e),
    }

    // Test 2: Completions endpoint
    println!("2️⃣ Testing /v1/completions endpoint:");
    let completion_body = json!({
        "model": "default",
        "prompt": "Hello",
        "temperature": 0.7,
        "max_tokens": 50
    });

    match client
        .post(format!("{}/v1/completions", base_url))
        .bearer_auth(token)
        .json(&completion_body)
        .send()
        .await
    {
        Ok(response) => {
            println!("   Status: {}", response.status());
            match response.text().await {
                Ok(text) => {
                    println!("   Raw response length: {}", text.len());
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        println!("   Parsed JSON:\n{}\n", serde_json::to_string_pretty(&json)?);
                    } else {
                        println!("   Response (raw):\n{}\n", text);
                    }
                }
                Err(e) => println!("   Error reading response: {}\n", e),
            }
        }
        Err(e) => println!("   Error: {}\n", e),
    }

    // Test 3: Chat endpoint
    println!("3️⃣ Testing /v1/chat/completions endpoint:");
    let chat_body = json!({
        "model": "default",
        "messages": [
            {"role": "user", "content": "Hello"}
        ],
        "temperature": 0.7,
        "max_tokens": 50
    });

    match client
        .post(format!("{}/v1/chat/completions", base_url))
        .bearer_auth(token)
        .json(&chat_body)
        .send()
        .await
    {
        Ok(response) => {
            println!("   Status: {}", response.status());
            match response.text().await {
                Ok(text) => {
                    println!("   Raw response length: {}", text.len());
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        println!("   Parsed JSON:\n{}\n", serde_json::to_string_pretty(&json)?);
                    } else {
                        println!("   Response (raw):\n{}\n", text);
                    }
                }
                Err(e) => println!("   Error reading response: {}\n", e),
            }
        }
        Err(e) => println!("   Error: {}\n", e),
    }

    println!("✅ Debug test complete");
    Ok(())
}

