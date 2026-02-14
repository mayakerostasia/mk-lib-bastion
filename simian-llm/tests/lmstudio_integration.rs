//! Integration test: LMStudio client HTTP integration
//!
//! This test requires LMStudio running on http://192.168.1.7:1234
//! or http://localhost:1234. It will be skipped if not available.

use simian_llm::{ChatMessage, LlmClient, LmStudioClient, LmStudioConfig};
use std::time::Duration;

#[tokio::test]
async fn test_lmstudio_health_check() {
    // Try localhost first, then the IP
    let configs = vec![
        LmStudioConfig::new("http://localhost:1234", "default")
            .with_timeout(Duration::from_secs(5)),
        LmStudioConfig::new("http://192.168.1.7:1234", "default")
            .with_timeout(Duration::from_secs(5)),
    ];

    for config in configs {
        match LmStudioClient::new(config.clone()).await {
            Ok(client) => {
                match client.health_check().await {
                    Ok(true) => {
                        println!("✓ LMStudio health check passed at {}", config.base_url);
                        return; // Found a working instance
                    }
                    Ok(false) => {
                        println!("✗ LMStudio health check returned false at {}", config.base_url);
                    }
                    Err(e) => {
                        println!("✗ LMStudio health check error at {}: {}", config.base_url, e);
                    }
                }
            }
            Err(e) => {
                println!("✗ Failed to create LMStudio client for {}: {}", config.base_url, e);
            }
        }
    }

    println!("⚠ LMStudio not available - skipping detailed tests");
    println!("  To run these tests, start LMStudio at:");
    println!("    - http://localhost:1234, or");
    println!("    - http://192.168.1.7:1234");
}

#[tokio::test]
#[ignore] // Only run if LMStudio is available
async fn test_lmstudio_completion() {
    let config = LmStudioConfig::new("http://localhost:1234", "default")
        .with_timeout(Duration::from_secs(30))
        .with_max_tokens(100);

    let client = match LmStudioClient::new(config).await {
        Ok(c) => c,
        Err(_) => {
            println!("Skipping: LMStudio not available");
            return;
        }
    };

    match client.complete("Say 'hello' in one word:").await {
        Ok(response) => {
            println!("Completion response: {}", response.content);
            assert!(!response.content.is_empty());
            assert_eq!(response.model, "default");
        }
        Err(e) => {
            println!("Completion failed: {}", e);
            panic!("Completion should succeed if LMStudio is running");
        }
    }
}

#[tokio::test]
#[ignore] // Only run if LMStudio is available
async fn test_lmstudio_chat() {
    let config = LmStudioConfig::new("http://localhost:1234", "default")
        .with_timeout(Duration::from_secs(30))
        .with_max_tokens(100);

    let client = match LmStudioClient::new(config).await {
        Ok(c) => c,
        Err(_) => {
            println!("Skipping: LMStudio not available");
            return;
        }
    };

    let messages = vec![
        ChatMessage::system("You are a helpful assistant."),
        ChatMessage::user("What is 2+2?"),
    ];

    match client.chat(&messages).await {
        Ok(response) => {
            println!("Chat response: {}", response.content);
            assert!(!response.content.is_empty());
            assert_eq!(response.model, "default");
        }
        Err(e) => {
            println!("Chat failed: {}", e);
            panic!("Chat should succeed if LMStudio is running");
        }
    }
}
