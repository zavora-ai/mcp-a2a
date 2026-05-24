# A2A Remote Agent MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-a2a.svg)](https://crates.io/crates/mcp-a2a)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Agent-to-Agent protocol management for [ADK-Rust Enterprise](https://enterprise.adk-rust.com). Discover remote agents, send tasks, stream results, and manage push notifications — across any framework that implements the [A2A protocol](https://github.com/google/A2A).

## What is A2A?

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-a2a/main/docs/architecture.svg" alt="A2A Remote Agent MCP Architecture" width="800"/>
</p>

The Agent-to-Agent (A2A) protocol is an open standard enabling communication between AI agents regardless of their underlying framework. This MCP server lets you discover, communicate with, and manage remote A2A agents from any MCP client.

## Tools (11)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `resolve_agent_card` | Fetch agent card from a remote agent | Read-only |
| `validate_agent_card` | Verify protocol version, capabilities, and skills | Read-only |
| `list_remote_agents` | Show all discovered/configured remote agents | Read-only |
| `send_message` | Create or continue a remote A2A task | External write |
| `send_streaming_message` | Start remote task with streaming updates | External write |
| `get_task` | Fetch task state and history | Read-only |
| `list_tasks` | Page and filter remote task registry | Read-only |
| `cancel_task` | Cancel a running remote task | External write |
| `subscribe_to_task` | Open SSE subscription for task events | Read-only |
| `configure_push_notifications` | Create/update/delete webhook configs | Internal write |
| `get_extended_agent_card` | Fetch private agent capabilities | Read-only |

## Verified: Three Frameworks via MCP

Tested end-to-end with real Gemini 2.5 Flash LLM calls:

### ADK-Rust Agent

```
> resolve_agent_card("http://localhost:8003")

{
  "agent_id": "agent_a2a_quickstart...",
  "name": "a2a-quickstart-agent",
  "status": "connected",
  "skills": [{ "name": "General Assistant", "tags": ["reasoning", "conversation"] }]
}

> send_message(agent_id, "What is the capital of Kenya?")

{
  "status": { "state": "completed" },
  "artifacts": [{ "parts": [{ "kind": "text", "text": "The capital of Kenya is Nairobi." }] }]
}
```

### Google ADK Agent

```
> resolve_agent_card("http://localhost:8001")

{
  "name": "helper_agent",
  "skills": [
    { "name": "check_prime", "description": "Check if numbers are prime." },
    { "name": "get_weather", "description": "Get current weather for a city." }
  ]
}

> send_message(agent_id, "Is 17 a prime number?")

{
  "status": { "state": "completed" },
  "artifacts": [{ "parts": [{ "kind": "text", "text": "Yes, 17 is a prime number." }] }]
}
```

### LangGraph Agent

```
> resolve_agent_card("http://localhost:8002")

{
  "name": "langgraph_gemini_agent",
  "skills": [{ "name": "General Reasoning", "tags": ["reasoning", "gemini", "langgraph"] }]
}

> send_message(agent_id, "What is the capital of France?")

{
  "status": { "state": "completed" },
  "history": [
    { "role": "user", "parts": [{ "text": "What is the capital of France?" }] },
    { "role": "agent", "parts": [{ "text": "The capital of France is Paris." }] }
  ]
}
```

## Compatible Frameworks

### ADK-Rust (Native)

```rust
use adk_rust::prelude::*;
use adk_rust::server::A2aServer;

let agent = LlmAgentBuilder::new("my_agent")
    .model(Arc::new(GeminiModel::new(api_key, "gemini-2.5-flash")?))
    .instruction("You are a helpful assistant.")
    .build()?;

let server = A2aServer::builder()
    .agent(Arc::new(agent))
    .bind_addr("0.0.0.0:8003")
    .build()?;

server.serve().await?;
```

Card: `http://localhost:8003/.well-known/agent.json` · Endpoint: `POST http://localhost:8003/a2a`

### Google ADK (Python)

```python
from google.adk.agents.llm_agent import Agent
from google.adk.a2a.utils.agent_to_a2a import to_a2a

root_agent = Agent(
    model="gemini-2.5-flash",
    name="helper_agent",
    instruction="You are a helpful assistant.",
    tools=[check_prime, get_weather],
)

a2a_app = to_a2a(root_agent, port=8001)
# Run: uvicorn my_agent:a2a_app --host localhost --port 8001
```

Card: `http://localhost:8001/.well-known/agent-card.json` · Endpoint: `POST http://localhost:8001`

### LangGraph (Python)

```python
from langgraph.graph import StateGraph
from langchain_google_genai import ChatGoogleGenerativeAI

llm = ChatGoogleGenerativeAI(model="gemini-2.5-flash")
# Wrap in A2A HTTP server (see docs/frameworks.md for full example)
```

Card: `http://localhost:8002/.well-known/agent-card.json` · Endpoint: `POST http://localhost:8002`

### Any Framework

Any agent serving these endpoints is compatible:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/.well-known/agent-card.json` | GET | Agent card discovery |
| Custom URL (from card) | POST | JSON-RPC 2.0 (`message/send`) |

## Multi-Agent Orchestration

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  ADK-Rust   │     │ Google ADK  │     │  LangGraph  │
│  :8003      │     │  :8001      │     │  :8002      │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌──────▼──────┐
                    │   mcp-a2a   │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ MCP Client  │
                    └─────────────┘
```

```
resolve_agent_card("http://localhost:8003")  → ADK-Rust agent
resolve_agent_card("http://localhost:8001")  → Google ADK agent
resolve_agent_card("http://localhost:8002")  → LangGraph agent
list_remote_agents()                        → 3 agents

send_message(rust_id, "What is 2+2?")       → "4"
send_message(adk_id, "Is 97 prime?")        → "Yes, 97 is a prime number."
send_message(lg_id, "Capital of France?")   → "The capital of France is Paris."
```

## Installation

```bash
git clone https://github.com/zavora-ai/mcp-a2a
cd mcp-a2a
cargo build --release
```

### MCP Client Config

```json
{
  "mcpServers": {
    "a2a": {
      "command": "/path/to/mcp-a2a"
    }
  }
}
```

Works with Claude Desktop, Kiro, Codex, Cursor, Windsurf, and any MCP-compatible client.

## Documentation

| Document | Description |
|----------|-------------|
| [API Reference](docs/api-reference.md) | All 11 tools with parameters, returns, errors |
| [Framework Guide](docs/frameworks.md) | Setup for ADK-Rust, Google ADK, LangGraph, CrewAI |
| [Security & Governance](docs/security.md) | Threat model, tenant isolation, rate limits |
| [CHANGELOG.md](CHANGELOG.md) | Version history |

## A2A Protocol Reference

### Task States

| State | Meaning |
|-------|---------|
| `submitted` | Task received, not yet started |
| `working` | Agent is processing |
| `input_required` | Agent needs more input |
| `completed` | Task finished successfully |
| `failed` | Task failed |
| `canceled` | Task was canceled |

### JSON-RPC Methods

| Method | Purpose |
|--------|---------|
| `message/send` | Send a message, get a task back |
| `message/stream` | Send a message, get SSE stream |
| `tasks/get` | Get task by ID |
| `tasks/cancel` | Cancel a task |

## Governance

- **Protocol version validation** — reject incompatible agents
- **Tenant-scoped auth** — isolation headers for multi-tenant deployments
- **Remote task limits** — rate limiting per agent
- **Audit logging** — all A2A interactions recorded

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — async health probe for registry monitoring
- **mcp-server.toml** — manifest declaring tools, risk classes, and credentials
- **Structured tracing** — `RUST_LOG` env-filter for observability

