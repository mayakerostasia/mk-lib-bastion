//! Phase 5d: Agent Autonomy - Discovering and Communicating
//!
//! Demonstrates agents discovering each other via registry and routing messages by subject.

use simian_llm::{AgentRegistry, AgentSpawner, SpawnConfig, AgentStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Phase 5d: Agent Autonomy ===\n");

    // Shared registry for agent discovery
    let registry = AgentRegistry::new();
    let spawner = AgentSpawner::new(registry.clone());

    // Create 3 specialized agents
    println!("Creating agents...\n");

    let _a1 = spawner.spawn(
        SpawnConfig::new()
            .with_id("analyzer".to_string())
            .with_system_prompt("You analyze data and find insights.".to_string())
            .with_subjects(vec!["analysis.*".to_string()])
            .persistent(true),
        async { tokio::time::sleep(tokio::time::Duration::from_secs(10)).await },
    )
    .await?;

    let _a2 = spawner.spawn(
        SpawnConfig::new()
            .with_id("writer".to_string())
            .with_system_prompt("You write clear summaries.".to_string())
            .with_subjects(vec!["writing.summary".to_string()])
            .persistent(true),
        async { tokio::time::sleep(tokio::time::Duration::from_secs(10)).await },
    )
    .await?;

    let _a3 = spawner.spawn(
        SpawnConfig::new()
            .with_id("critic".to_string())
            .with_system_prompt("You provide critical feedback.".to_string())
            .with_subjects(vec!["review.*".to_string()])
            .persistent(true),
        async { tokio::time::sleep(tokio::time::Duration::from_secs(10)).await },
    )
    .await?;

    // Give agents time to register
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Discovery patterns
    println!("Discovered agents:\n");

    let all = registry.list_agents().await;
    for agent in &all {
        println!("  {}: {:?}", agent.id, agent.subjects);
    }

    println!("\nAgent Discovery by Subject:\n");

    // Find agents handling specific subjects
    let analysis = registry.find_agents_by_subject("analysis.request").await;
    println!("  'analysis.request' → {}", analysis[0].id);

    let writing = registry.find_agents_by_subject("writing.summary").await;
    println!("  'writing.summary' → {}", writing[0].id);

    let review = registry.find_agents_by_subject("review.draft").await;
    println!("  'review.draft' (wildcard) → {}", review[0].id);

    // Update agent status
    println!("\nAgent Status Updates:\n");
    registry.update_status("analyzer", AgentStatus::Running).await?;
    registry.update_status("writer", AgentStatus::Running).await?;
    registry.update_status("critic", AgentStatus::Running).await?;

    let updated = registry.get_agent("analyzer").await.unwrap();
    println!("  Analyzer status: {:?}", updated.status);

    // Capability discovery
    println!("\nCapability-Based Discovery:\n");

    let analysts = registry.find_agents_by_capability("analyze").await;
    let analyst_ids: Vec<String> = analysts.iter().map(|a| a.id.clone()).collect();
    println!("  Agents with 'analyze' capability: {}", analyst_ids.join(", "));

    let writers = registry.find_agents_by_capability("write").await;
    let writer_ids: Vec<String> = writers.iter().map(|a| a.id.clone()).collect();
    println!("  Agents with 'write' capability: {}", writer_ids.join(", "));

    // Summary
    println!("\nAgent Network Summary:");
    println!("  Total agents: {}", registry.agent_count().await);
    println!("  Total subscriptions: {}", all.iter().map(|a| a.subjects.len()).sum::<usize>());

    println!("\n✅ Phase 5d Complete!");
    println!("Key capabilities demonstrated:");
    println!("  • Registry-based agent discovery");
    println!("  • Subject-based routing (exact + wildcard)");
    println!("  • Capability-based discovery");
    println!("  • Agent status lifecycle management");
    println!("  • Agent specialization via system prompts");

    Ok(())
}
