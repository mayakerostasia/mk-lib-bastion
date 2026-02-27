# LLM Development Environment Setup

## Overview

Complete Docker Compose setup for testing real LLMs with simian-bastion agents.

## What You Get

```
┌─────────────┐
│  LMStudio   │  ← Your local LLM server (port 1234)
│  (external) │
└──────┬──────┘
       │ HTTP API
┌──────┴──────────────────────────────┐
│         Docker Network              │
│                                     │
│  ┌──────────┐    ┌──────────┐     │
│  │ Research │    │Summarize │     │
│  │ Agent    │    │  Agent   │     │
│  └────┬─────┘    └────┬─────┘     │
│       │               │            │
│       └───────┬───────┘            │
│               │                    │
│          ┌────┴────┐               │
│          │  NATS   │               │
│          └────┬────┘               │
│               │                    │
│      ┌────────┴────────┐           │
│      │   viz-bridge    │           │
│      └────────┬─────────           │
└───────────────┼─────────────────────┘
                │ WebSocket
         ┌──────┴──────┐
         │   Browser   │  ← http://localhost:5173
         │  Dashboard  │
         └─────────────┘
```

## Prerequisites

### 1. LMStudio (or similar)

**Option A: LMStudio** (Recommended for local testing)
- Download from: https://lmstudio.ai/
- Load a model (e.g., Llama 3, Mistral, etc.)
- Start local server (default: http://localhost:1234)
- Note: LMStudio has an OpenAI-compatible API

**Option B: Ollama**
```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model
ollama pull llama3

# Run server (default: http://localhost:11434)
ollama serve
```

**Option C: OpenAI API**
- Set `OPENAI_API_KEY` environment variable
- Change `LMSTUDIO_BASE_URL` to `https://api.openai.com/v1`

### 2. Docker & Docker Compose

```bash
# Verify installation
docker --version
docker-compose --version
```

## Quick Start

### Step 1: Start LMStudio

1. Open LMStudio
2. Load a model (Llama 3 8B recommended)
3. Start local server: **Developer** → **Start Server**
4. Verify: http://localhost:1234/v1/models

### Step 2: Create LLM Agent Binary

We need to create the actual LLM agent executable. Create this file:

**`simian-llm/src/bin/llm-agent.rs`**:
```rust
use anyhow::Result;
use simian_base_api::agent::SimianAgent;
use simian_base_api::transport::{AcpMessage, AgentId, Performative};
use simian_llm::client::LmStudioClient;
use simian_llm::LlmClient;
use simian_nats_streams::transport::NatsTransport;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Get config from environment
    let nats_url = env::var("NATS_URL")?;
    let agent_id = env::var("AGENT_ID")?;
    let system_prompt = env::var("SYSTEM_PROMPT")?;
    let lm_base_url = env::var("LMSTUDIO_BASE_URL")?;

    info!("🚀 Starting LLM Agent: {}", agent_id);

    // Create transport and agent
    let transport = NatsTransport::new(&nats_url, "agents").await?;
    let agent = SimianAgent::new(AgentId(agent_id.clone()), transport);

    // Create LLM client
    let llm = LmStudioClient::new(&lm_base_url)?;

    // Subscribe to messages
    let mut messages = agent.listen().await?;

    info!("✅ Agent {} listening...", agent_id);

    while let Some(msg) = messages.next().await {
        if msg.performative == Performative::Request {
            let prompt = String::from_utf8_lossy(&msg.payload);
            info!("📥 Received: {}", prompt);

            // Call LLM
            match llm.completion(&format!("{}\n\n{}", system_prompt, prompt)).await {
                Ok(response) => {
                    info!("📤 Responding: {}", response.text);
                    agent.reply(&msg, response.text.as_bytes()).await?;
                }
                Err(e) => {
                    info!("❌ LLM error: {}", e);
                }
            }
        }
    }

    Ok(())
}
```

### Step 3: Start Everything

```bash
# Start all services
docker-compose -f docker-compose.llm.yml up --build

# Watch logs
docker-compose -f docker-compose.llm.yml logs -f

# Scale agents
docker-compose -f docker-compose.llm.yml up --scale llm-agent-researcher=3
```

### Step 4: Test It

**Option A: Via Dashboard**
- Open http://localhost:5173
- Watch agents communicate in real-time

**Option B: Send test message**
```bash
# Install NATS CLI
go install github.com/nats-io/natscli/nats@latest

# Send a research request
nats pub "agents.researcher-001.inbox" \
  '{"message_id":"test-123","source":{"0":"tester"},"target":{"0":"researcher-001"},"performative":"Request","subject":"research","conversation_id":"conv-1","payload":"V2hhdCBpcyBSdXN0Pw==","timestamp":1234567890}'

# Watch responses
nats sub "agents.>"
```

**Option C: Simple test script**
```bash
#!/bin/bash
# test-llm.sh

# Send question to researcher
curl -X POST http://localhost:3030/message \
  -H "Content-Type: application/json" \
  -d '{
    "frame_type": "Msg",
    "content": "What is Rust programming language?",
    "to_agent": "researcher-001"
  }'
```

## Configuration

### Environment Variables

Each agent can be configured via docker-compose.llm.yml:

| Variable | Description | Example |
|----------|-------------|---------|
| `NATS_URL` | NATS server | `nats://nats:4222` |
| `AGENT_ID` | Unique agent ID | `researcher-001` |
| `AGENT_ROLE` | Agent role/type | `researcher` |
| `AGENT_SUBJECTS` | Subjects to listen | `research.*,analyze.*` |
| `SYSTEM_PROMPT` | LLM system prompt | `You are a research agent...` |
| `LMSTUDIO_BASE_URL` | LLM server URL | `http://host.docker.internal:1234/v1` |
| `LMSTUDIO_MODEL` | Model name | `llama-3-8b` |
| `LMSTUDIO_TEMPERATURE` | Creativity (0-1) | `0.7` |
| `LMSTUDIO_MAX_TOKENS` | Max response length | `2000` |

### Adding More Agents

Just add another service to `docker-compose.llm.yml`:

```yaml
llm-agent-translator:
  build:
    context: .
    dockerfile: docker/Dockerfile.llm-agent
  environment:
    - AGENT_ID=translator-001
    - AGENT_ROLE=translator
    - SYSTEM_PROMPT=You are a translation agent. Translate text to different languages.
    - LMSTUDIO_TEMPERATURE=0.1  # Low for accuracy
```

## Troubleshooting

### LMStudio Connection Issues

**Problem**: Agents can't reach LMStudio at `host.docker.internal`

**Solution 1**: Use host network
```yaml
network_mode: "host"
```

**Solution 2**: Find your IP
```bash
# Linux/Mac
ip addr show | grep inet

# Windows
ipconfig

# Then update LMSTUDIO_BASE_URL
environment:
  - LMSTUDIO_BASE_URL=http://192.168.1.100:1234/v1
```

### No Messages Appearing

1. Check NATS: http://localhost:8222
2. Check viz-bridge logs: `docker-compose logs viz-bridge`
3. Verify agents started: `docker-compose ps`
4. Check browser console for WebSocket errors

### Agents Not Responding

1. Check agent logs: `docker-compose logs llm-agent-researcher`
2. Verify LMStudio is running and accessible
3. Test LMStudio directly:
```bash
curl http://localhost:1234/v1/models
```

## Development Workflow

### 1. Develop Agent Logic
```bash
# Edit simian-llm/src/bin/llm-agent.rs
# ... make changes ...

# Rebuild and restart
docker-compose -f docker-compose.llm.yml up --build llm-agent-researcher
```

### 2. Test Locally (No Docker)
```bash
# Start NATS
docker run -p 4222:4222 -p 8222:8222 nats:2.10-alpine --http_port 8222

# Start LMStudio (external)

# Run agent locally
NATS_URL=nats://localhost:4222 \
AGENT_ID=test-001 \
SYSTEM_PROMPT="You are a test agent" \
LMSTUDIO_BASE_URL=http://localhost:1234/v1 \
cargo run --bin llm-agent
```

### 3. Monitor Everything
```bash
# Terminal 1: NATS monitoring
nats monitor -s localhost:8222

# Terminal 2: Docker logs
docker-compose -f docker-compose.llm.yml logs -f

# Terminal 3: Dashboard
open http://localhost:5173
```

## Next Steps

### Immediate
- [ ] Create `simian-llm/src/bin/llm-agent.rs`
- [ ] Test with LMStudio
- [ ] Verify dashboard shows agent communication

### Short-term
- [ ] Add conversation memory (store in NATS JetStream)
- [ ] Implement multi-turn conversations
- [ ] Add agent-to-agent delegation
- [ ] Rate limiting and error recovery

### Long-term
- [ ] Agent discovery service
- [ ] Dynamic agent spawning
- [ ] Vector store integration
- [ ] Function calling / tools
- [ ] Multi-model support (GPT-4, Claude, etc.)

## Example Scenarios

### Scenario 1: Research Pipeline
1. User sends question to **orchestrator**
2. Orchestrator delegates to **researcher**
3. Researcher analyzes and responds
4. Orchestrator sends to **summarizer**
5. Summarizer creates concise summary
6. Result returned to user

### Scenario 2: Content Creation
1. User sends topic to **writer**
2. Writer creates draft
3. Writer sends to **researcher** for fact-checking
4. Researcher provides corrections
5. Writer incorporates feedback
6. Final content returned

### Scenario 3: Multi-Agent Debate
1. Orchestrator poses question
2. Multiple agents respond with different perspectives
3. Orchestrator synthesizes responses
4. Agents discuss/debate
5. Consensus or summary provided

## Resources

- **LMStudio**: https://lmstudio.ai/docs
- **Ollama**: https://ollama.com/library
- **NATS**: https://docs.nats.io/
- **simian-llm README**: `simian-llm/README.md`
- **Visualization**: http://localhost:5173

---

**Ready to test real LLMs with your agent system!** 🚀
