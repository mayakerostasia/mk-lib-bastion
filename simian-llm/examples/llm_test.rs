/// Manual test script to interact with LMStudio on 192.168.1.7:1234
///
/// Run with: cargo run --example llm_test --release

use simian_llm::{ChatMessage, LlmClient, LmStudioClient, LmStudioConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 LMStudio Client Test\n");

    let config = LmStudioConfig::new("http://192.168.1.7:1234", "default")
        .with_timeout(Duration::from_secs(60))
        .with_temperature(0.7)
        .with_max_tokens(256)
        .with_api_token("sk-lm-gLUHl47J:Cc8UhKBjQJlro3pNutt0");

    println!("📋 Config:");
    println!("  URL: {}", config.base_url);
    println!("  Model: {}", config.model);
    println!("  Temperature: {}", config.temperature);
    println!("  Max tokens: {}", config.max_tokens);
    println!("  Timeout: {:?}\n", config.timeout);

    // Create client
    println!("🔌 Connecting to LMStudio...");
    let client = LmStudioClient::new(config).await?;
    println!("✓ Client created\n");

    // Health check
    println!("💓 Health Check:");
    match client.health_check().await {
        Ok(true) => println!("✓ LMStudio is healthy\n"),
        Ok(false) => {
            println!("✗ LMStudio returned unhealthy status\n");
            return Ok(());
        }
        Err(e) => {
            println!("✗ Health check failed: {}\n", e);
            return Err(e);
        }
    }

    // Test completion
    println!("📝 Testing Completion...");
    let prompt = "What is the capital of France? Answer in one word:";
    println!("  Prompt: {}", prompt);
    match client.complete(prompt).await {
        Ok(response) => {
            println!("  ✓ Response: {}", response.content.trim());
            if let Some(usage) = response.usage {
                println!("    Tokens: prompt={:?}, completion={:?}, total={:?}",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
            }
            println!("    Model: {}\n", response.model);
        }
        Err(e) => {
            println!("  ✗ Error: {}\n", e);
        }
    }

    // Test chat
    println!("💬 Testing Chat...");
    let messages = vec![
        ChatMessage::system("You are a helpful assistant. Keep responses brief."),
        ChatMessage::user("What is the largest planet in our solar system?"),
    ];
    println!("  System: You are a helpful assistant. Keep responses brief.");
    println!("  User: What is the largest planet in our solar system?");
    
    match client.chat(&messages).await {
        Ok(response) => {
            println!("  ✓ Response: {}", response.content.trim());
            if let Some(usage) = response.usage {
                println!("    Tokens: prompt={:?}, completion={:?}, total={:?}",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
            }
            println!("    Model: {}\n", response.model);
        }
        Err(e) => {
            println!("  ✗ Error: {}\n", e);
        }
    }

    // Test multi-turn chat
    println!("🔄 Testing Multi-turn Chat...");
    let messages = vec![
        ChatMessage::system("You are a math tutor."),
        ChatMessage::user("What is 2 + 2?"),
        ChatMessage::assistant("2 + 2 equals 4."),
        ChatMessage::user("What is 4 + 3?"),
    ];
    println!("  System: You are a math tutor.");
    println!("  User: What is 2 + 2?");
    println!("  Assistant: 2 + 2 equals 4.");
    println!("  User: What is 4 + 3?");
    
    match client.chat(&messages).await {
        Ok(response) => {
            println!("  ✓ Response: {}", response.content.trim());
            if let Some(usage) = response.usage {
                println!("    Tokens: prompt={:?}, completion={:?}, total={:?}",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
            }
            println!("    Model: {}\n", response.model);
        }
        Err(e) => {
            println!("  ✗ Error: {}\n", e);
        }
    }

    println!("✅ All tests completed!");
    Ok(())
}
