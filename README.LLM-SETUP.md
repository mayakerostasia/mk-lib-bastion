# External LMStudio Setup for Multi-Agent Testing

## Prerequisites

1. **LMStudio running on 192.168.1.7:1234**
   - Download from [lmstudio.ai](https://lmstudio.ai)
   - Load a model (e.g., Llama 3, Mistral, etc.)
   - Enable API server on port 1234
   - Note your API key

2. **Create `.env.docker` file** (never commit this):
   ```bash
   cp .env.docker.example .env.docker
   # Edit and add your API key:
   LMSTUDIO_API_KEY=your-actual-api-key-here
   ```

3. **Verify connectivity**:
   ```bash
   curl -H "Authorization: Bearer YOUR_KEY" http://192.168.1.7:1234/v1/models
   ```

## Quick Start

```bash
# Start the LLM agent environment
docker-compose -f docker-compose.llm.yml up --build

# In another terminal, view the dashboard
open http://localhost:5173

# Watch agent communication in real-time
```

## Agent Configuration

The system includes 3 pre-configured LLM agents:

- **researcher-001**: Research specialist (temp=0.3, analytical)
- **summarizer-001**: Summarization expert (temp=0.2, concise)
- **writer-001**: Creative writer (temp=0.9, engaging)

Each agent connects to your LMStudio instance at `192.168.1.7:1234/v1` using the API key from `.env.docker`.

## Testing

Send a test request to an agent:

```bash
# Install nats CLI if not already installed
brew install nats-io/nats-tools/nats  # macOS
# or download from https://github.com/nats-io/natscli

# Send a request
nats pub "agents.researcher-001.inbox" "Tell me about Rust async programming"

# Monitor all messages
nats sub "agents.>"
```

## Security

- ⚠️ **NEVER commit `.env.docker`** - it contains your API key
- ✅ `.gitignore` already excludes `.env.docker`
- ✅ Use `.env.docker.example` as a template

## Troubleshooting

### "Connection refused" errors
- Verify LMStudio is running on 192.168.1.7:1234
- Check firewall settings on the LMStudio machine
- Test with: `curl http://192.168.1.7:1234/v1/models`

### "401 Unauthorized" errors
- Verify API key in `.env.docker` matches LMStudio
- Check for extra whitespace in the key

### Agents not responding
- Check Docker logs: `docker logs researcher-001`
- Verify NATS connection: `docker logs nats`
- Check LMStudio model is loaded

## See Also

- Full documentation: `LLM_TESTING.md`
- Visualization setup: `viz-dashboard/README.md`
- Transport abstraction: `simian-viz-bridge/README.md`
