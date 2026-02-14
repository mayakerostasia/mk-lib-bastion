# simian-llm Troubleshooting Guide

Common issues and their solutions.

## Connection Issues

### Error: Connection refused / Cannot connect to LMStudio

**Symptom:**
```
Error: reqwest::ClientError - connection refused
```

**Causes:**
- LMStudio not running
- Wrong URL (host/port)
- Firewall blocking
- LMStudio bound to wrong interface

**Solutions:**

1. **Verify LMStudio is running:**

   ```bash
   # Check if process exists
   ps aux | grep lmstudio
   
   # On Windows:
   tasklist | findstr /I lmstudio
   ```

2. **Check the endpoint:**

   ```rust
   // Wrong
   let config = LmStudioConfig::new("http://192.168.1.7:1234", "default");
   
   // Right - verify actual host and port
   let config = LmStudioConfig::new("http://localhost:1234", "default");
   ```

3. **Test connectivity manually:**

   ```bash
   # Linux/macOS
   curl http://localhost:1234/api/status
   
   # Windows PowerShell
   (curl http://localhost:1234/api/status).Content
   
   # Test with telnet
   telnet localhost 1234
   ```

4. **If remote machine, ensure it's accessible:**

   ```bash
   # From local machine, ping remote
   ping 192.168.1.7
   
   # Test specific port
   nmap -p 1234 192.168.1.7
   ```

5. **Check LMStudio binding address:**

   - Open LMStudio settings
   - Verify it's listening on `0.0.0.0` or your machine's IP (not just `127.0.0.1`)
   - Default is usually `http://0.0.0.0:1234`

---

## Authentication Issues

### Error: 401 Unauthorized

**Symptom:**
```
Error: HTTP 401 Unauthorized
```

**Causes:**
- API token missing
- API token incorrect
- Token expired
- LMStudio requires authentication

**Solutions:**

1. **Add API token (if using authenticated LMStudio):**

   ```rust
   let config = LmStudioConfig::new("http://localhost:1234", "default")
       .with_api_token("sk-lm-your-token-here");
   ```

2. **Verify token format:**

   - LMStudio tokens typically start with `sk-lm-`
   - Format: `sk-lm-{base}:{hash}`
   - Example: `sk-lm-gLUHl47J:Cc8UhKBjQJlro3pNutt0`

3. **Check LMStudio settings for authentication:**

   - Some LMStudio installations require authentication
   - Generate token from LMStudio UI if needed

4. **Test with curl:**

   ```bash
   # Without token (if auth optional)
   curl http://localhost:1234/v1/models
   
   # With token
   curl -H "Authorization: Bearer sk-lm-token" \
        http://localhost:1234/v1/models
   ```

---

## Endpoint/Path Issues

### Error: 404 Not Found / Invalid endpoints

**Symptom:**
```
HTTP 404: /completions not found
HTTP 404: /models not found
```

**Causes:**
- Missing `/v1/` prefix
- Typo in endpoint path
- LMStudio version mismatch

**Solutions:**

1. **Verify endpoints use `/v1/` prefix:**

   ```rust
   // Wrong - missing /v1/
   POST /completions
   GET /models
   
   // Right
   POST /v1/completions
   GET /v1/chat/completions
   GET /v1/models
   ```

2. **Check available endpoints:**

   ```bash
   # List models
   curl http://localhost:1234/v1/models
   
   # Health check
   curl http://localhost:1234/v1/models
   ```

3. **Verify LMStudio version:**

   - Use latest LMStudio version
   - Older versions might have different API paths

---

## Timeout Issues

### Error: Request timeout / Timed out

**Symptom:**
```
Error: Request timeout after 30s
```

**Causes:**
- LLM taking too long to respond
- Network latency
- Model size too large for available resources
- Timeout too short

**Solutions:**

1. **Increase timeout duration:**

   ```rust
   use std::time::Duration;
   
   let config = LmStudioConfig::new("http://localhost:1234", "default")
       .with_timeout(Duration::from_secs(120));  // 2 minutes
   ```

2. **Reduce response size:**

   ```rust
   let config = LmStudioConfig::new("http://localhost:1234", "default")
       .with_max_tokens(512);  // Smaller responses = faster
   ```

3. **Use faster model:**

   - Switch to smaller/faster model
   - Quantized models (e.g., Q4, Q5) usually faster

4. **Monitor server resources:**

   ```bash
   # Check CPU/memory on LMStudio machine
   top          # Linux
   htop         # Linux (better)
   Activity Monitor  # macOS
   Task Manager     # Windows
   ```

---

## NATS/Transport Issues

### Error: Failed to connect to NATS

**Symptom:**
```
Error: Failed to connect: Connection refused
```

**Causes:**
- NATS server not running
- Wrong NATS address
- Firewall blocking NATS port
- Network unreachable

**Solutions:**

1. **Start NATS server:**

   ```bash
   # macOS
   brew install nats-server
   nats-server
   
   # Linux
   nats-server
   
   # Docker
   docker run -d -p 4222:4222 nats:latest
   ```

2. **Verify NATS is listening:**

   ```bash
   # Check port 4222
   nc -zv 127.0.0.1 4222
   
   # Windows PowerShell
   Test-NetConnection -ComputerName 127.0.0.1 -Port 4222
   ```

3. **Check NATS URL:**

   ```rust
   // Wrong
   let nats = async_nats::connect("nats://192.168.0.1:4222").await?;
   
   // Right - verify actual address
   let nats = async_nats::connect("nats://127.0.0.1:4222").await?;
   ```

4. **Firewall configuration:**

   - Ensure port 4222 is open
   - If remote, allow traffic from client IP

---

## Message Routing Issues

### Agents don't receive messages

**Symptom:**
```
Agent sends message but recipient never receives it
No errors, but no response
```

**Causes:**
- Subscription not set up before sending
- Agent ID mismatch
- Subject mismatch
- Message timing issue

**Solutions:**

1. **Ensure subscription before messaging:**

   ```rust
   // WRONG: send before subscribing
   agent.request_completion("responder", "prompt").await?;
   // ^ Message might be lost if responder not listening yet
   
   // RIGHT: subscribe first
   let mut messages = transport.subscribe("responder").await?;
   // Now ready to receive
   spawn_listener(messages);  // Start listener in background
   agent.request_completion("responder", "prompt").await?;
   ```

2. **Verify agent IDs match:**

   ```rust
   // Create responder with matching ID
   let responder = LlmAgent::new("responder", transport, client);
   
   // Send to exact same ID
   requester.request_completion("responder", "prompt").await?;
   ```

3. **Check subject names:**

   ```rust
   // Use correct subjects
   use simian_llm::protocol::subjects;
   
   // Subjects should be:
   // - llm.reason.request
   // - llm.reason.response
   // - llm.chat.request
   // - llm.chat.response
   ```

4. **Add debug logging:**

   ```rust
   use tracing::debug;
   
   let mut messages = transport.subscribe(agent_id).await?;
   while let Some(msg) = messages.next().await {
       debug!(subject = msg.subject, "Received message");
       if let Err(e) = agent.handle_message(&msg).await {
           eprintln!("Error: {}", e);
       }
   }
   ```

---

## Serialization Issues

### Error: Cannot deserialize / JSON parse error

**Symptom:**
```
serde_json error: expected field `content` at line 1 column X
```

**Causes:**
- Message payload format mismatch
- Protocol version incompatibility
- Corrupted payload bytes

**Solutions:**

1. **Verify payload structure:**

   ```rust
   // Expected structure for LlmReasonResponse
   {
       "response": "string",
       "context": { ... } or null
   }
   
   // Ensure responses match this exactly
   ```

2. **Add debug logging for payloads:**

   ```rust
   use tracing::debug;
   
   let msg_str = String::from_utf8_lossy(&msg.payload);
   debug!(payload = %msg_str, "Received payload");
   ```

3. **Validate before deserialize:**

   ```rust
   let data: serde_json::Value = serde_json::from_slice(&msg.payload)?;
   debug!(data = %data, "Payload structure");
   
   // Then deserialize specific type
   let response: LlmReasonResponse = serde_json::from_value(data)?;
   ```

---

## Performance Issues

### Agent communication is slow

**Symptom:**
```
Messages take 5-10s to arrive
High latency between request and response
```

**Causes:**
- Network latency
- NATS server overloaded
- Message serialization overhead
- LLM processing time (not communication)

**Solutions:**

1. **Check network latency:**

   ```bash
   # Ping NATS server
   ping 127.0.0.1
   
   # For remote
   ping 192.168.1.10
   ```

2. **Monitor NATS performance:**

   ```bash
   # Connect to NATS admin interface
   nats-top  # if available
   
   # Or check NATS logs
   nats-server -DV
   ```

3. **Profile LLM time separately:**

   ```rust
   use std::time::Instant;
   
   let llm_start = Instant::now();
   let response = client.complete(prompt).await?;
   let llm_duration = llm_start.elapsed();
   
   println!("LLM took: {:?}", llm_duration);
   // If LLM is slow, that's not a communication issue
   ```

4. **Batch requests if possible:**

   Instead of sending 100 individual requests, send larger batches

---

## Build Issues

### Cargo build fails

**Symptom:**
```
error: cannot find crate `simian_llm`
error: cyclic dependency
```

**Solutions:**

1. **Dependency not in workspace:**

   ```toml
   # Ensure simian-llm is in root Cargo.toml members
   [workspace]
   members = [
       "simian-llm",
       "simian-base-api",
       # ...
   ]
   ```

2. **Cyclic dependency:**

   - Check for `A → B → A` dependency chains
   - See ARCHITECTURE.md for known resolved cycles

3. **Missing dependencies:**

   ```bash
   cargo tree -p simian-llm
   # Shows all transitive dependencies
   ```

---

## Model Issues

### Model not found / Invalid model name

**Symptom:**
```
Error: Model not found
```

**Causes:**
- Model name doesn't match available models
- Model not installed in LMStudio
- Typo in model name

**Solutions:**

1. **List available models:**

   ```bash
   curl http://localhost:1234/v1/models | jq
   ```

2. **Update config with correct model:**

   ```rust
   // Check available models first
   let response = client.client.get("http://localhost:1234/v1/models")
       .send()
       .await?
       .json::<ModelsResponse>()
       .await?;
   
   println!("Available: {:?}", response.data);
   
   // Use one of them
   let config = LmStudioConfig::new("http://localhost:1234", "neural-chat-7b");
   ```

3. **Install model in LMStudio:**

   - Open LMStudio GUI
   - Search for model
   - Download and install
   - Wait for completion

---

## Error Responses from Agent

### Agent returns Failure performative

**Symptom:**
```
{
  "performative": "Failure",
  "payload": { "error": "LLM timeout" }
}
```

**Causes:**
- LLM client operation failed
- Timeout exceeded
- Backend error

**Solutions:**

1. **Check payload for error details:**

   ```rust
   if msg.performative == Performative::Failure {
       let error_data: serde_json::Value = 
           serde_json::from_slice(&msg.payload)?;
       println!("Error: {}", error_data["error"]);
   }
   ```

2. **Implement retry logic:**

   ```rust
   for attempt in 0..3 {
       match agent.request_completion(target, prompt).await {
           Ok(()) => break,
           Err(e) => {
               eprintln!("Attempt {} failed: {}", attempt + 1, e);
               tokio::time::sleep(Duration::from_millis(100)).await;
           }
       }
   }
   ```

---

## Instrumentation Issues

### Tracing/metrics not appearing

**Symptom:**
```
No logs in output
Metrics not recorded
```

**Causes:**
- Tracing subscriber not initialized
- Log level too high
- Metrics not flushed

**Solutions:**

1. **Initialize tracing subscriber:**

   ```rust
   use tracing_subscriber;
   
   #[tokio::main]
   async fn main() -> anyhow::Result<()> {
       // Initialize tracing
       tracing_subscriber::fmt()
           .with_max_level(tracing::Level::INFO)
           .init();
       
       // Now logs will appear
       info!("Application started");
   }
   ```

2. **Set log level:**

   ```bash
   # Via environment variable
   export RUST_LOG=debug
   cargo run --example agent_llm_chat
   ```

3. **Check metrics setup:**

   - Ensure metrics crate is initialized
   - Check if metrics exporter is running

---

## Platform-Specific Issues

### Windows: cmake-sys build fails

**Symptom:**
```
error: CMAKE_GENERATOR not found
Visual Studio 18 2026 not installed
```

**Solution:**

Already fixed in `.cargo/config.toml`:

```toml
[env]
CMAKE_GENERATOR = "Visual Studio 17 2022"
```

---

## Getting Help

If you can't find the answer:

1. **Check existing documentation:**
   - [README.md](./README.md) - Quick reference
   - [ARCHITECTURE.md](./ARCHITECTURE.md) - Design overview
   - [PROTOCOL.md](./PROTOCOL.md) - Message format
   - [INTEGRATION_GUIDE.md](./INTEGRATION_GUIDE.md) - Usage guide
   - [BACKEND_GUIDE.md](./BACKEND_GUIDE.md) - Custom backends

2. **Enable debug logging:**

   ```bash
   export RUST_LOG=debug
   cargo run --example agent_llm_chat
   ```

3. **Verify prerequisites:**

   - Rust 1.75+
   - NATS running (if using agents)
   - LMStudio running (if using LMStudio backend)
   - Network connectivity

4. **File an issue:**

   Include:
   - Error message
   - Stack trace
   - Steps to reproduce
   - Environment (OS, Rust version, component versions)

---

## FAQ

**Q: Can I use simian-llm without NATS?**

A: Yes! Direct LLM calls don't need NATS. Only agent-to-agent communication requires a Transport.

**Q: What's the maximum context size?**

A: Depends on your LLM model. Configure `max_tokens` when creating the config.

**Q: Can I stream responses?**

A: Currently no. Streaming support is planned for a future version.

**Q: Can I use custom LLM backends?**

A: Yes! Implement the `LlmClient` trait. See BACKEND_GUIDE.md.

**Q: Does it support function calling?**

A: Not yet. This is on the roadmap.

**Q: How do I monitor metrics?**

A: See [INTEGRATION_GUIDE.md](./INTEGRATION_GUIDE.md) for instrumentation setup.

**Q: Can agents on different machines communicate?**

A: Yes, if they're all connected to the same NATS cluster and using compatible transports.

---

## Performance Tips

1. **Use connection pooling** - LmStudioClient reuses HTTP connections
2. **Batch requests** - Send multiple prompts in single request when possible
3. **Use faster models** - Quantized models are faster
4. **Increase timeouts for complex prompts** - Not all prompts are equal
5. **Monitor resource usage** - Watch CPU/memory on LLM machine
6. **Cache responses** - Don't repeat same prompt if possible (future feature)

---

## Additional Resources

- LMStudio Docs: https://lmstudio.ai/
- NATS Docs: https://docs.nats.io/
- OpenAI API: https://platform.openai.com/docs/api-reference
- Tokio Async Runtime: https://tokio.rs/
