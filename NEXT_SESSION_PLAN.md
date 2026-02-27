# Next Session Plan - Agent Autonomy & Intelligence

**Based On:** Current successful visualization + LLM infrastructure  
**Goal:** Enable agents to discover, spawn, and delegate to each other autonomously

---

## 🎯 Session Objectives

Make agents truly autonomous by adding:
1. **Discovery** - Agents can find each other by capability
2. **Spawning** - Agents can create new agents on demand
3. **Delegation** - Agents can delegate subtasks intelligently
4. **Memory** - Agents remember conversation context

---

## Phase 5: Agent Autonomy (from FUTURE_REFACTORING.md)

### Phase 5a: AgentRegistry - Discovery System
**Goal:** Let agents find each other by capabilities, not hardcoded IDs

**Tasks:**

#### 5a.1: Design AgentRegistry
- **What:** Registry service that tracks live agents and their capabilities
- **How:**
  - Subscribe to special `agents.registry` subject
  - Agents announce themselves on startup (Subscribe performative)
  - Agents send heartbeats every 30s
  - Registry maintains map of AgentId → {capabilities, subjects, last_seen}
  - Support wildcard matching: "data.*" matches "data.import", "data.export"
- **Output:** Design document with message protocol
- **Size:** Small (2-3 hours)
- **Files:** `docs/agent-registry-design.md`

#### 5a.2: Implement AgentRegistry
- **What:** Rust binary that implements the registry service
- **How:**
  - New crate: `simian-agent-registry/`
  - Listens on `agents.registry` for announcements
  - Responds to Query performatives with matching agents
  - Implements timeout for stale agents (remove after 90s no heartbeat)
  - Uses HashMap<AgentId, AgentMetadata>
- **Output:** Working registry binary
- **Size:** Medium (half day)
- **Files:** `simian-agent-registry/src/main.rs`

#### 5a.3: Add Registry Client to LlmAgent
- **What:** LlmAgent can query registry to find other agents
- **How:**
  - Add `discover_agents(capability: &str)` method
  - Sends Query to registry with capability filter
  - Returns Vec<AgentId> of matching agents
  - Cache results for 60s to reduce registry load
- **Output:** LlmAgent with discovery capability
- **Size:** Small
- **Files:** `simian-llm/src/agent.rs`

#### 5a.4: Add Auto-Registration
- **What:** Agents automatically register on startup
- **How:**
  - Add `register()` method called in agent initialization
  - Sends Subscribe message to `agents.registry`
  - Includes subjects array and capability tags
  - Starts heartbeat task (tokio::spawn)
- **Output:** Self-registering agents
- **Size:** Small
- **Files:** `simian-llm/src/agent.rs`, `simian-base-api/src/agent.rs`

---

### Phase 5b: System Prompts & Subject Management
**Goal:** Give LlmAgent personality and multi-subject handling

**Tasks:**

#### 5b.1: Add System Prompt Support
- **What:** LlmAgent has customizable system prompt
- **How:**
  - Add `system_prompt: Arc<String>` field
  - Add `with_system_prompt()` constructor
  - Prepend system prompt to all LLM calls (chat: system message, completion: delimiter)
  - Document best practices (role, constraints, output format, tone)
- **Output:** LlmAgent with configurable personality
- **Size:** Small
- **Files:** `simian-llm/src/agent.rs`

#### 5b.2: Add Subject Management
- **What:** LlmAgent can handle multiple subjects dynamically
- **How:**
  - Add `subjects: Arc<RwLock<Vec<String>>>` field
  - Add `add_subject()` / `remove_subject()` methods
  - Add `handles_subject()` for routing checks
  - Update registry announcement when subjects change
- **Output:** Multi-subject agent support
- **Size:** Small
- **Files:** `simian-llm/src/agent.rs`

#### 5b.3: Implement Subject Matching Algorithm
- **What:** Proper wildcard matching for subjects
- **How:**
  - "exact match": "analysis.request" == "analysis.request"
  - "wildcard match": "data.*" matches "data.import", "data.export"
  - "no recursive": "data.*" does NOT match "data.export.csv"
  - Use suffix check: `subject.starts_with(prefix) && subject[prefix.len()..].matches('.')`.count() == 0`
- **Output:** Correct subject routing
- **Size:** Tiny
- **Files:** `simian-base-api/src/transport.rs` (utility function)

---

### Phase 5c: Observability Stack (Feature Branch)
**Goal:** Production-grade monitoring with Grafana Alloy

**Note:** This should be done in `feature/phase-5c-observability` branch as it's a large change.

**Tasks:**

#### 5c.1: Add Grafana Alloy to docker-compose
- **What:** Observability collector that replaces individual exporters
- **How:**
  - Add `alloy` service to docker-compose.yml
  - Configure for Prometheus, Loki, Tempo
  - Single collector for all telemetry
  - Exports to local Grafana or cloud
- **Output:** docker-compose with Alloy
- **Size:** Medium
- **Files:** `docker-compose.observability.yml`

#### 5c.2: Configure simian-tracing for Alloy
- **What:** Direct export to Alloy instead of separate exporters
- **How:**
  - Update simian-tracing to use Alloy endpoints
  - Remove old OTLP exporter config
  - Add Alloy-specific config options
- **Output:** Simplified tracing config
- **Size:** Small
- **Files:** `simian-tracing/src/lib.rs`

#### 5c.3: Add Grafana Dashboard
- **What:** Pre-built dashboard for agent monitoring
- **How:**
  - Create JSON dashboard for Grafana
  - Panels: message rate, agent count, LLM latency, error rate
  - Use Prometheus metrics + Loki logs
  - Add to docker-compose as provisioned dashboard
- **Output:** Working Grafana setup
- **Size:** Small
- **Files:** `docker/grafana/dashboards/agents.json`

---

### Phase 5d: Agent Spawning & Delegation
**Goal:** Agents can create and manage other agents

**Tasks:**

#### 5d.1: Design Agent Spawner
- **What:** Service that can spawn new agent containers/processes
- **How:**
  - Listens for "spawn_agent" requests
  - Creates new agent with specified binary/config
  - Options: ephemeral (dies when done) vs persistent (stays running)
  - Returns new AgentId to requester
  - Manages agent lifecycle
- **Output:** Design document
- **Size:** Small
- **Files:** `docs/agent-spawner-design.md`

#### 5d.2: Implement AgentSpawner
- **What:** Spawner service
- **How:**
  - New binary: `simian-agent-spawner`
  - Uses Docker API to spawn containers OR
  - Uses process spawning for local development
  - Tracks spawned agents in internal map
  - Cleans up ephemeral agents after completion
- **Output:** Working spawner
- **Size:** Large (full day)
- **Files:** `simian-agent-spawner/src/main.rs`

#### 5d.3: Add spawn_agent() to LlmAgent
- **What:** LlmAgent can request new agents
- **How:**
  - Add `spawn_agent(binary: &str, config: AgentConfig)` method
  - Sends Request to spawner service
  - Returns AgentId of new agent
  - Example: Orchestrator spawns researcher on demand
- **Output:** Spawn capability in agents
- **Size:** Small
- **Files:** `simian-llm/src/agent.rs`

#### 5d.4: Implement Task Delegation Pattern
- **What:** High-level API for delegating work
- **How:**
  - Add `delegate(task: &str, to_capability: &str)` method
  - Discovers capable agents via registry
  - If none found, spawns one
  - Sends Request with task
  - Waits for response or timeout
  - Returns result
- **Output:** Easy delegation API
- **Size:** Medium
- **Files:** `simian-llm/src/delegation.rs` (new module)

---

### Phase 5e: MCP Server Integration
**Goal:** Expose agent system to Claude Desktop and other MCP clients

**Tasks:**

#### 5e.1: Design MCP Server Interface
- **What:** MCP server that bridges to agent system
- **How:**
  - Implements MCP protocol (stdio or HTTP)
  - Exposes tools: `send_message`, `discover_agents`, `spawn_agent`
  - Exposes prompts: agent system prompts, conversation templates
  - Exposes resources: agent list, message history, capabilities
- **Output:** Design document
- **Size:** Small
- **Files:** `docs/mcp-server-design.md`

#### 5e.2: Implement MCP Server
- **What:** MCP server binary
- **How:**
  - New crate: `simian-mcp-server/`
  - Uses `mcp-sdk` or implement protocol directly
  - Connects to NATS as special agent
  - Translates MCP tool calls to ACP messages
  - Returns responses in MCP format
- **Output:** Working MCP server
- **Size:** Large
- **Files:** `simian-mcp-server/src/main.rs`

#### 5e.3: Add Claude Desktop Config
- **What:** Easy setup for Claude Desktop users
- **How:**
  - Create config JSON for Claude Desktop
  - Add to docs with installation instructions
  - Example workflows for common tasks
- **Output:** Working Claude integration
- **Size:** Tiny
- **Files:** `docs/mcp-claude-setup.md`, `claude-desktop-config.json`

---

### Phase 5f: Integration & Documentation
**Goal:** Polish and document everything

**Tasks:**

#### 5f.1: Create End-to-End Example
- **What:** Complete example showing all features
- **How:**
  - Orchestrator agent receives complex request
  - Discovers available agents via registry
  - Spawns researcher if needed
  - Delegates subtasks
  - Aggregates results
  - Returns final answer
- **Output:** Working example
- **Size:** Small
- **Files:** `examples/autonomous_research.rs`

#### 5f.2: Write Phase 5 Documentation
- **What:** Complete docs for autonomous features
- **How:**
  - Architecture guide: How discovery works
  - API guide: Using registry, spawner, delegation
  - Best practices: When to spawn vs reuse
  - Examples: Common patterns
- **Output:** Comprehensive docs
- **Size:** Medium
- **Files:** `docs/AUTONOMOUS_AGENTS.md`

#### 5f.3: Update docker-compose.llm.yml
- **What:** Include registry and spawner in LLM setup
- **How:**
  - Add `agent-registry` service
  - Add `agent-spawner` service
  - Configure agents to auto-register
  - Update viz-dashboard to show registry data
- **Output:** Complete autonomous environment
- **Size:** Small
- **Files:** `docker-compose.llm.yml`

---

## 🗺️ Roadmap

### Week 1: Discovery (Phase 5a)
- Day 1: Design & implement AgentRegistry
- Day 2: Add discovery to LlmAgent
- Day 3: Test with multiple agents

### Week 2: Intelligence (Phase 5b)
- Day 1: System prompts + subject management
- Day 2: Subject matching algorithm
- Day 3: Test multi-subject routing

### Week 3: Observability (Phase 5c) - Optional Feature Branch
- Day 1: Add Alloy to docker-compose
- Day 2: Configure tracing + Grafana
- Day 3: Create dashboards

### Week 4: Autonomy (Phase 5d)
- Day 1-2: Design & implement AgentSpawner
- Day 3: Add spawn capability to agents
- Day 4: Implement delegation pattern

### Week 5: Integration (Phase 5e + 5f)
- Day 1-2: MCP server implementation
- Day 3: End-to-end example
- Day 4-5: Documentation

---

## 🎯 Success Criteria

### Phase 5a Complete When:
- ✅ Registry tracks all live agents
- ✅ Agents can discover by capability
- ✅ Stale agents removed automatically
- ✅ Visualization shows registry state

### Phase 5b Complete When:
- ✅ LlmAgent has system prompt
- ✅ Agents handle multiple subjects
- ✅ Subject matching works correctly
- ✅ Registry updated on subject changes

### Phase 5c Complete When:
- ✅ Alloy collecting all telemetry
- ✅ Grafana dashboard showing metrics
- ✅ Logs and traces correlated
- ✅ Performance acceptable

### Phase 5d Complete When:
- ✅ Spawner can create new agents
- ✅ Ephemeral agents cleaned up
- ✅ Delegation API easy to use
- ✅ Example shows full workflow

### Phase 5e Complete When:
- ✅ MCP server working with Claude Desktop
- ✅ Can send messages from Claude
- ✅ Can discover agents from Claude
- ✅ Can spawn agents from Claude

### Phase 5f Complete When:
- ✅ Complete example working
- ✅ Documentation comprehensive
- ✅ docker-compose includes all services
- ✅ Easy 5-minute quickstart

---

## 📦 Deliverables

### Code
- [ ] `simian-agent-registry/` - Discovery service
- [ ] `simian-agent-spawner/` - Agent lifecycle management
- [ ] `simian-mcp-server/` - MCP protocol bridge
- [ ] `simian-llm/` - Enhanced with autonomy features
- [ ] `examples/autonomous_research.rs` - End-to-end demo

### Documentation
- [ ] `docs/agent-registry-design.md`
- [ ] `docs/agent-spawner-design.md`
- [ ] `docs/mcp-server-design.md`
- [ ] `docs/AUTONOMOUS_AGENTS.md`
- [ ] `docs/mcp-claude-setup.md`

### Docker
- [ ] `docker-compose.observability.yml` - Grafana Alloy stack
- [ ] Updated `docker-compose.llm.yml` with registry + spawner
- [ ] Grafana dashboards

---

## 🚀 Getting Started (Next Session)

### Prerequisites
- ✅ Current session's visualization working
- ✅ LLM agents communicating
- ✅ Docker infrastructure solid

### First Task
Start with Phase 5a.1 (Design AgentRegistry):
```bash
# 1. Read current architecture
cat ARCHITECTURE.md
cat simian-llm/src/agent.rs

# 2. Design registry protocol
# - What messages for announce/discover/heartbeat?
# - What data structure for agent metadata?
# - How to handle wildcards?

# 3. Create design doc
# Write: docs/agent-registry-design.md
```

### Quick Wins
1. System prompts (5b.1) - Very easy, immediate value
2. Subject matching (5b.3) - Small utility, foundational
3. Auto-registration (5a.4) - Makes other features possible

---

## 🎓 Key Insights for Next Session

### Architecture Principles
1. **Agents are agnostic** - No parent-child tracking, all peers
2. **Discovery over hardcoding** - Find agents by capability, not ID
3. **Ephemeral by default** - Spawn when needed, clean up when done
4. **Observable everything** - Every action logged and traced

### Design Patterns
1. **Registry as heart** - Central discovery, but stateless
2. **Spawner as factory** - Creates agents but doesn't control them
3. **MCP as interface** - Entry point for external systems
4. **Delegation as abstraction** - High-level task assignment

### Technical Considerations
1. **Heartbeats prevent stale data** - Registry needs cleanup
2. **Caching reduces load** - Don't query registry every message
3. **Timeouts are essential** - Tasks can fail, handle gracefully
4. **Wildcards need care** - "data.*" should not match "data.x.y"

---

This plan builds directly on the solid foundation we built this session. The visualization will be invaluable for debugging autonomous behavior! 🎉
