//! Integration test: two SimianAgents exchange LLM protocol messages over NATS.
//!
//! Requires a NATS server running on 127.0.0.1:4222.

use bytes::Bytes;
use futures::StreamExt;
use simian_base_api::transport::{AgentId, Performative, Transport};
use simian_base_api::agent::SimianAgent;
use simian_llm::protocol::subjects;
use simian_llm::{LlmReasonRequest, LlmReasonResponse, TokenUsage};
use simian_nats_streams::transport::NatsTransport;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_llm_agent_nats_roundtrip() {
    // Connect two transports to NATS
    let transport_a = NatsTransport::new("nats://127.0.0.1:4222", "llmtest")
        .await
        .expect("Failed to connect transport A to NATS");
    let transport_b = NatsTransport::new("nats://127.0.0.1:4222", "llmtest")
        .await
        .expect("Failed to connect transport B to NATS");

    assert!(transport_a.is_connected().await, "Transport A not connected");
    assert!(transport_b.is_connected().await, "Transport B not connected");

    let requester = SimianAgent::new(AgentId::new("requester"), transport_a);
    let llm_agent = SimianAgent::new(AgentId::new("llm-agent"), transport_b);

    // LLM agent starts listening before the request is sent
    let mut llm_inbox = llm_agent.listen().await.expect("llm-agent failed to subscribe");

    // Small delay to let subscription propagate
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Requester sends an LlmReasonRequest
    let mut ctx = HashMap::new();
    ctx.insert("task".to_string(), "summarize".to_string());
    let reason_req = LlmReasonRequest {
        prompt: "Explain Rust ownership in one sentence.".to_string(),
        context: Some(ctx),
    };
    let payload = serde_json::to_vec(&reason_req).expect("serialize request");

    let conv_id = requester
        .request(
            AgentId::new("llm-agent"),
            subjects::LLM_REASON_REQUEST,
            Bytes::from(payload),
        )
        .await
        .expect("requester failed to send request");

    // LLM agent receives the request
    let incoming = tokio::time::timeout(Duration::from_secs(5), llm_inbox.next())
        .await
        .expect("llm-agent timed out waiting for message")
        .expect("llm-agent stream ended unexpectedly");

    assert_eq!(incoming.subject, subjects::LLM_REASON_REQUEST);
    assert_eq!(incoming.performative, Performative::Request);
    assert_eq!(incoming.conversation_id, conv_id);
    assert_eq!(incoming.source.0, "requester");

    let parsed_req: LlmReasonRequest =
        serde_json::from_slice(&incoming.payload).expect("deserialize request payload");
    assert_eq!(parsed_req.prompt, "Explain Rust ownership in one sentence.");
    assert_eq!(
        parsed_req.context.as_ref().unwrap().get("task").unwrap(),
        "summarize"
    );

    // LLM agent sends back a response
    let reason_resp = LlmReasonResponse {
        response: "Ownership ensures each value has a single owner, enabling memory safety without a garbage collector.".to_string(),
        model: "test-model".to_string(),
        tokens_used: Some(TokenUsage {
            prompt_tokens: Some(12),
            completion_tokens: Some(18),
            total_tokens: Some(30),
        }),
    };
    let resp_payload = serde_json::to_vec(&reason_resp).expect("serialize response");

    // Requester subscribes before the reply is sent
    let mut req_inbox = requester.listen().await.expect("requester failed to subscribe");
    tokio::time::sleep(Duration::from_millis(200)).await;

    llm_agent
        .reply(&incoming, Performative::Inform, Bytes::from(resp_payload))
        .await
        .expect("llm-agent failed to reply");

    // Requester receives the response
    let reply = tokio::time::timeout(Duration::from_secs(5), req_inbox.next())
        .await
        .expect("requester timed out waiting for reply")
        .expect("requester stream ended unexpectedly");

    assert_eq!(
        reply.subject,
        format!("{}.reply", subjects::LLM_REASON_REQUEST)
    );
    assert_eq!(reply.performative, Performative::Inform);
    assert_eq!(reply.conversation_id, conv_id);
    assert_eq!(reply.source.0, "llm-agent");

    let parsed_resp: LlmReasonResponse =
        serde_json::from_slice(&reply.payload).expect("deserialize response payload");
    assert!(parsed_resp.response.contains("Ownership"));
    assert_eq!(parsed_resp.model, "test-model");
    assert_eq!(parsed_resp.tokens_used.unwrap().total_tokens, Some(30));

    println!("✓ Full NATS roundtrip: requester -> llm-agent -> requester");
}
