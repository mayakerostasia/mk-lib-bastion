# simian-llm Protocol Specification

This document defines the Agent Communication Protocol (ACP) extensions for LLM operations.

## Table of Contents

1. [Overview](#overview)
2. [Message Subjects](#message-subjects)
3. [Message Format](#message-format)
4. [Request-Response Flows](#request-response-flows)
5. [Error Handling](#error-handling)
6. [Context & Metadata](#context--metadata)
7. [Examples](#examples)

## Overview

The simian-llm protocol defines how agents communicate LLM requests and responses using the Agent Communication Protocol (ACP) standard.

### Key Principles

1. **Subject-Based Routing** - Messages route via ACP subject, not explicit protocol field
2. **Performative Semantics** - Performative enum indicates intent (Request, Inform, Failure)
3. **Conversation Tracking** - Conversation IDs correlate request-response pairs
4. **Bidirectional** - Both unicast (request-response) and broadcast messaging supported
5. **Error Transparency** - Failures produce Failure performatives with error details

## Message Subjects

### Completion Operations

```
llm.reason.request
├─ Performative: Request or Inform
├─ Sender: Requester agent
├─ Target: Specific agent (Request) or None (Inform/Broadcast)
├─ Payload: LlmReasonRequest (serialized to JSON)
└─ Purpose: Request text completion from LLM

llm.reason.response
├─ Performative: Inform or Failure
├─ Sender: LLM handler agent
├─ Target: Original requester
├─ Payload: LlmReasonResponse or error JSON
└─ Purpose: Return completion result or error
```

### Chat Operations

```
llm.chat.request
├─ Performative: Request or Inform
├─ Sender: Requester agent
├─ Target: Specific agent (Request) or None (Inform/Broadcast)
├─ Payload: LlmChatRequest (serialized to JSON)
└─ Purpose: Request multi-turn chat from LLM

llm.chat.response
├─ Performative: Inform or Failure
├─ Sender: LLM handler agent
├─ Target: Original requester
├─ Payload: LlmChatResponse or error JSON
└─ Purpose: Return chat result or error
```

## Message Format

### ACP Message Wrapper

All LLM protocol messages use the ACP envelope:

```rust
pub struct AcpMessage {
    pub message_id: Uuid,           // Unique message identifier
    pub source: AgentId,            // Sending agent
    pub target: Option<AgentId>,    // Receiving agent (None = broadcast)
    pub performative: Performative, // Request, Inform, Failure, etc.
    pub subject: String,            // "llm.reason.request", etc.
    pub conversation_id: Uuid,      // Correlates request-response
    pub payload: Bytes,             // Serialized protocol message
    pub timestamp: i64,             // UNIX timestamp
}
```

### LlmReasonRequest Payload

```json
{
  "prompt": "What is machine learning?",
  "context": {
    "domain": "education",
    "language": "en",
    "max_length": "one_paragraph"
  }
}
```

```rust
pub struct LlmReasonRequest {
    pub prompt: String,
    pub context: Option<HashMap<String, String>>,
}
```

**Fields:**
- `prompt` (required) - The text prompt for the LLM
- `context` (optional) - Arbitrary metadata as key-value pairs

### LlmReasonResponse Payload

```json
{
  "response": "Machine learning is a subset of artificial intelligence...",
  "model": "gemma-3-27b-it-abliterated",
  "tokens_used": {
    "prompt_tokens": 8,
    "completion_tokens": 45,
    "total_tokens": 53
  }
}
```

```rust
pub struct LlmReasonResponse {
    pub response: String,
    pub model: String,
    pub tokens_used: Option<TokenUsage>,
}

pub struct TokenUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}
```

**Fields:**
- `response` - The LLM-generated text response
- `model` - Name of the model that processed the request
- `tokens_used` - Token usage statistics (optional, backend-dependent)

### LlmChatRequest Payload

```json
{
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant specialized in Rust."
    },
    {
      "role": "user",
      "content": "Explain Rust ownership in 3 sentences."
    }
  ],
  "context": {
    "conversation_topic": "rust-education",
    "difficulty_level": "intermediate"
  }
}
```

```rust
pub struct LlmChatRequest {
    pub messages: Vec<ChatMessage>,
    pub context: Option<HashMap<String, String>>,
}

pub enum ChatRole {
    System,
    User,
    Assistant,
}

pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}
```

**Fields:**
- `messages` - Conversation history in chronological order
- `context` - Optional metadata about the conversation

### LlmChatResponse Payload

```json
{
  "message": {
    "role": "assistant",
    "content": "Rust ownership is the system that manages memory... Rust also has the borrow checker... Additionally, Rust enforces..."
  },
  "model": "gemma-3-27b-it-abliterated",
  "tokens_used": {
    "prompt_tokens": 35,
    "completion_tokens": 65,
    "total_tokens": 100
  }
}
```

```rust
pub struct LlmChatResponse {
    pub message: ChatMessage,
    pub model: String,
    pub tokens_used: Option<TokenUsage>,
}
```

**Fields:**
- `message` - The assistant's response (always role = "assistant")
- `model` - Name of the model
- `tokens_used` - Token statistics

### Error Payload

Both request and response can fail with error payload:

```json
{
  "error": "LLM request timed out after 60 seconds"
}
```

For Failure performatives, the payload is:
```rust
{ "error": String }
```

## Request-Response Flows

### Flow 1: Unicast Completion Request

```
┌─────────────────────────────────────────────────────────────┐
│ Agent-1 calls: agent1.request_completion("agent-2", prompt) │
└─────────────────────────────────────────────────────────────┘
           │
           ↓
   ┌──────────────────┐
   │ Build AcpMessage │
   ├──────────────────┤
   │ message_id: UUID │
   │ source: agent-1  │
   │ target: agent-2  │
   │ performative: Request
   │ subject: llm.reason.request
   │ conversation_id: UUID
   │ payload: LlmReasonRequest
   │ timestamp: now
   └──────────────────┘
           │
           ↓
   ┌─────────────────┐
   │ Publish via     │
   │ Transport       │
   └─────────────────┘
           │
           ↓
   ┌─────────────────────┐
   │ Message travels via │
   │ NATS/Iggy/etc       │
   └─────────────────────┘
           │
           ↓
   ┌──────────────────────┐
   │ Agent-2 subscribes   │
   │ to llm.reason.*      │
   │ receives message     │
   └──────────────────────┘
           │
           ↓
   ┌──────────────────────┐
   │ agent2.handle_message()
   ├──────────────────────┤
   │ 1. Deserialize       │
   │ 2. Call              │
   │    llm.complete()    │
   │ 3. Build response    │
   └──────────────────────┘
           │
           ├─── SUCCESS ───────────────────┐
           │                               │
           ↓                               ↓
   ┌──────────────────┐         ┌──────────────────┐
   │ Create Inform    │         │ Create Failure   │
   │ performative     │         │ performative     │
   │ subject:         │         │ subject:         │
   │  llm.reason.     │         │  llm.reason.     │
   │  response        │         │  response        │
   │ payload:         │         │ payload:         │
   │  LlmReason       │         │  { error: ... }  │
   │  Response        │         │                  │
   └──────────────────┘         └──────────────────┘
           │                               │
           └───────────┬───────────────────┘
                       │
                       ↓
           ┌──────────────────────┐
           │ Publish to           │
           │ Transport (back to   │
           │ agent-1's inbox)     │
           └──────────────────────┘
                       │
                       ↓
           ┌──────────────────────┐
           │ Agent-1 receives     │
           │ response on          │
           │ subscription         │
           │ (can correlate via   │
           │ conversation_id)     │
           └──────────────────────┘
```

### Flow 2: Broadcast Chat Request

```
Agent-1 calls: agent1.broadcast_completion("Explain AI")

Creates AcpMessage:
├─ performative: Inform           ← Broadcast, not Request
├─ target: None                   ← No specific target
├─ subject: llm.reason.request
└─ conversation_id: UUID

Published to Transport (NATS/Iggy)
           │
           ├─────────────→ Agent-2 (subscribed to llm.reason.*)
           │
           ├─────────────→ Agent-3 (subscribed to llm.reason.*)
           │
           └─────────────→ Agent-4 (subscribed to llm.reason.*)

Each agent independently:
1. Receives message
2. Calls handle_message()
3. Processes with their LlmClient
4. Optionally sends Inform response back to agent-1
   (or silently processes if no response desired)
```

### Flow 3: Error Handling

```
Agent-1 sends request to Agent-2

Agent-2 receives and processes:
├─ LLM client times out
├─ LLM client returns error
└─ JSON deserialization fails

Each case creates AcpMessage:
├─ performative: Failure           ← Indicates error
├─ subject: llm.reason.response
├─ payload: { "error": "description" }
└─ conversation_id: (matches request)

Publishes back to Agent-1

Agent-1 can correlate via conversation_id and handle error
```

## Error Handling

### Error Categories

| Category | Cause | Performative | Example |
|----------|-------|---|----------|
| Serialization | Request payload invalid JSON | Failure | `"Invalid UTF-8"` |
| LLM Failure | Backend returned error | Failure | `"API token invalid"` |
| Timeout | LLM didn't respond in time | Failure | `"Request timeout"` |
| Transport | Failed to publish message | Error (exception) | Connection lost |
| Context | Missing required context | Failure | `"Model not found"` |

### Error Response Format

```json
{
  "error": "LLM request failed: Connection refused at http://192.168.1.7:1234"
}
```

### Automatic Error Replies

When a request is received and processing fails:

```rust
// In LlmAgent::handle_reason_request()
match self.llm_client.complete(&request.prompt).await {
    Ok(response) => {
        // Send Inform with LlmReasonResponse
    }
    Err(e) => {
        // Automatically create Failure message
        let error_reply = AcpMessage {
            performative: Performative::Failure,
            subject: "llm.reason.response",
            payload: json!({ "error": e.to_string() }).into(),
            // ... other fields copied from request ...
        };
        self.transport.publish(error_reply).await?;
    }
}
```

## Context & Metadata

### Context HashMap

Both requests and responses can include optional context:

```rust
pub struct LlmReasonRequest {
    pub prompt: String,
    pub context: Option<HashMap<String, String>>,  // ← Context here
}
```

**Well-Known Context Keys:**

| Key | Value | Example | Used By |
|-----|-------|---------|---------|
| `task_type` | The type of task | `"summarization"`, `"qa"` | Application logic |
| `domain` | Subject domain | `"medical"`, `"finance"` | Application logic |
| `language` | Language code | `"en"`, `"es"`, `"ja"` | Application logic |
| `max_length` | Length limit | `"one_paragraph"`, `"brief"` | Application logic |
| `temperature` | Sampling parameter | `"0.7"`, `"0.9"` | LmStudioConfig override |
| `user_id` | Requesting user | `"user-123"` | Audit logging |
| `request_id` | External request ID | `"req-abc-123"` | Correlation |

### Metadata in AcpMessage

```rust
pub struct AcpMessage {
    pub timestamp: i64,              // When message was created
    pub conversation_id: Uuid,       // Correlate request-response
    pub message_id: Uuid,            // Unique message ID
    pub source: AgentId,             // Who sent it
    pub target: Option<AgentId>,     // Who it's for (None = broadcast)
}
```

### Conversation Tracking Example

```
Request arrives:
├─ conversation_id: "abc-123"
├─ message_id: "msg-1"
├─ timestamp: 1707897150

LLM processing happens...

Response:
├─ conversation_id: "abc-123"  ← Same ID
├─ message_id: "msg-2"         ← Different ID (new message)
└─ timestamp: 1707897155        ← Later timestamp

Receiver matches conversation_id to original request
```

## Examples

### Example 1: Simple Completion Request

**Agent-1 sends:**
```rust
agent1.request_completion("agent-2", "What is Rust?").await?;
```

**Generated AcpMessage:**
```json
{
  "message_id": "12345678-1234-1234-1234-123456789012",
  "source": "agent-1",
  "target": "agent-2",
  "performative": "Request",
  "subject": "llm.reason.request",
  "conversation_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
  "payload": {
    "prompt": "What is Rust?",
    "context": null
  },
  "timestamp": 1707897150
}
```

**Agent-2 processes and responds:**
```json
{
  "message_id": "87654321-4321-4321-4321-210987654321",
  "source": "agent-2",
  "target": "agent-1",
  "performative": "Inform",
  "subject": "llm.reason.response",
  "conversation_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
  "payload": {
    "response": "Rust is a systems programming language...",
    "model": "gemma-3-27b-it-abliterated",
    "tokens_used": {
      "prompt_tokens": 4,
      "completion_tokens": 12,
      "total_tokens": 16
    }
  },
  "timestamp": 1707897153
}
```

### Example 2: Multi-Turn Chat

**Agent-1 sends:**
```rust
let messages = vec![
    ChatMessage::system("You are a helpful assistant."),
    ChatMessage::user("What is 2+2?"),
    ChatMessage::assistant("2 + 2 = 4"),
    ChatMessage::user("What is 4+3?"),
];

agent1.request_chat("agent-2", &messages).await?;
```

**Generated AcpMessage:**
```json
{
  "subject": "llm.chat.request",
  "payload": {
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant."
      },
      {
        "role": "user",
        "content": "What is 2+2?"
      },
      {
        "role": "assistant",
        "content": "2 + 2 = 4"
      },
      {
        "role": "user",
        "content": "What is 4+3?"
      }
    ],
    "context": null
  }
}
```

### Example 3: Broadcast with Context

**Agent-1 sends:**
```rust
let agent = LlmAgent::new("agent-1", transport, client);

// Manually construct request with context
let request = LlmReasonRequest {
    prompt: "Summarize the latest AI trends".to_string(),
    context: Some({
        let mut m = HashMap::new();
        m.insert("task_type".to_string(), "summarization".to_string());
        m.insert("domain".to_string(), "technology".to_string());
        m.insert("max_length".to_string(), "two_paragraphs".to_string());
        m
    }),
};

agent.broadcast_completion(&request.prompt).await?;
```

**Generated AcpMessage (broadcast):**
```json
{
  "message_id": "...",
  "source": "agent-1",
  "target": null,             // ← Broadcast: no specific target
  "performative": "Inform",   // ← Broadcast uses Inform, not Request
  "subject": "llm.reason.request",
  "payload": {
    "prompt": "Summarize the latest AI trends",
    "context": {
      "task_type": "summarization",
      "domain": "technology",
      "max_length": "two_paragraphs"
    }
  }
}
```

---

**Next:** See [INTEGRATION_GUIDE.md](./INTEGRATION_GUIDE.md) for implementation examples.
