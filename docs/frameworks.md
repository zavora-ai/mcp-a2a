# Framework Integration Guide

## Overview

The A2A Remote Agent MCP server communicates with any agent implementing the A2A protocol. This guide covers setup for each verified framework.

## Protocol Requirements

Any A2A-compatible agent must implement:

1. **Agent card endpoint:** `GET /.well-known/agent-card.json` — returns agent metadata
2. **Message endpoint:** `POST <url>` — accepts JSON-RPC 2.0 `message/send`

### Minimum Agent Card

```json
{
  "name": "my_agent",
  "description": "What this agent does",
  "url": "http://localhost:8001",
  "version": "1.0.0",
  "protocolVersion": "0.2.6",
  "capabilities": { "streaming": false, "pushNotifications": false, "stateTransitionHistory": false },
  "skills": [{ "id": "my_skill", "name": "My Skill", "description": "What it does", "tags": ["category"] }]
}
```

### Minimum Message Handler

Request:
```json
{
  "jsonrpc": "2.0",
  "method": "message/send",
  "params": {
    "message": {
      "role": "user",
      "parts": [{"kind": "text", "text": "user input"}],
      "messageId": "uuid"
    }
  },
  "id": "request-uuid"
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "id": "task-uuid",
    "status": {"state": "completed"},
    "artifacts": [{"parts": [{"kind": "text", "text": "agent response"}]}]
  },
  "id": "request-uuid"
}
```

---

## ADK-Rust ✅ Verified

Native A2A support via `A2aServer` convenience API.

### Prerequisites

```bash
cargo add adk-rust --features standard
```

### Create an Agent

```rust
use adk_rust::prelude::*;
use adk_rust::server::A2aServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GOOGLE_API_KEY")?;
    let model = GeminiModel::new(api_key, "gemini-2.5-flash")?;

    let agent: Arc<dyn Agent> = Arc::new(
        LlmAgentBuilder::new("my_agent")
            .description("A helpful assistant")
            .instruction("Answer questions clearly and concisely.")
            .model(Arc::new(model))
            .build()?,
    );

    let server = A2aServer::builder()
        .agent(agent)
        .bind_addr("0.0.0.0:8003")
        .build()?;

    server.serve().await?;
    Ok(())
}
```

### Endpoints

- Card: `http://localhost:8003/.well-known/agent.json`
- A2A: `POST http://localhost:8003/a2a`

### Test

```bash
curl http://localhost:8003/.well-known/agent.json | jq .

curl -X POST http://localhost:8003/a2a \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"message/send","params":{"message":{"role":"user","parts":[{"kind":"text","text":"What is the capital of Kenya?"}],"messageId":"t1"}},"id":"r1"}'
```

**Verified response:** `"The capital of Kenya is Nairobi."`

### Connect from MCP

```
resolve_agent_card("http://localhost:8003")
send_message(agent_id, "What is the capital of Kenya?")
→ "The capital of Kenya is Nairobi."
```

### Scaffolding

```bash
cargo adk new my_a2a_agent --template a2a
cd my_a2a_agent
cargo run
```

---

## Google ADK (Python) ✅ Verified

### Prerequisites

```bash
pip install google-adk[a2a]
export GOOGLE_API_KEY=your-key
```

### Create an Agent

```python
# remote_agent/agent.py
from google.adk.agents.llm_agent import Agent
from google.adk.a2a.utils.agent_to_a2a import to_a2a

def check_prime(numbers: list[int]) -> dict:
    """Check if numbers are prime."""
    results = {}
    for n in numbers:
        if n < 2:
            results[str(n)] = False
        else:
            results[str(n)] = all(n % i != 0 for i in range(2, int(n**0.5) + 1))
    return {"results": results}

def get_weather(city: str) -> dict:
    """Get weather for a city."""
    return {"city": city, "temperature": "24°C", "condition": "partly cloudy"}

root_agent = Agent(
    model="gemini-2.5-flash",
    name="helper_agent",
    instruction="You are a helpful assistant with check_prime and get_weather tools.",
    tools=[check_prime, get_weather],
)

a2a_app = to_a2a(root_agent, port=8001)
```

### Run

```bash
uvicorn remote_agent.agent:a2a_app --host localhost --port 8001
```

### Endpoints

- Card: `http://localhost:8001/.well-known/agent-card.json`
- A2A: `POST http://localhost:8001`

### Test

```bash
curl http://localhost:8001/.well-known/agent-card.json | jq .

curl -X POST http://localhost:8001 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"message/send","params":{"message":{"role":"user","parts":[{"kind":"text","text":"Is 17 a prime number?"}],"messageId":"t1"}},"id":"r1"}'
```

**Verified response:** `"Yes, 17 is a prime number."`

### Connect from MCP

```
resolve_agent_card("http://localhost:8001")
send_message(agent_id, "Is 17 a prime number?")
→ "Yes, 17 is a prime number."
```

### Consuming Remote Agents in ADK

```python
from google.adk.agents.remote_a2a_agent import RemoteA2aAgent

remote = RemoteA2aAgent(
    name="helper",
    description="Remote helper agent",
    agent_card="http://localhost:8001/.well-known/agent-card.json",
)

orchestrator = Agent(
    model="gemini-2.5-flash",
    name="orchestrator",
    sub_agents=[remote],
)
```

---

## LangGraph (Python) ✅ Verified

### Prerequisites

```bash
pip install langgraph langchain-google-genai
export GOOGLE_API_KEY=your-key
```

### Create an Agent

```python
# langgraph-agent/agent.py
import json, uuid
from http.server import HTTPServer, BaseHTTPRequestHandler
from langgraph.graph import StateGraph, START, END
from langchain_google_genai import ChatGoogleGenerativeAI

llm = ChatGoogleGenerativeAI(model="gemini-2.5-flash")

AGENT_CARD = {
    "name": "langgraph_gemini_agent",
    "description": "A LangGraph agent powered by Gemini 2.5 Flash.",
    "url": "http://localhost:8002",
    "version": "1.0.0",
    "protocolVersion": "0.2.6",
    "capabilities": {"streaming": False, "pushNotifications": False, "stateTransitionHistory": False},
    "skills": [{"id": "reasoning", "name": "General Reasoning", "description": "Answer questions using Gemini", "tags": ["reasoning", "gemini", "langgraph"]}],
}

def invoke_llm(query):
    response = llm.invoke(query)
    return response.content

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path in ("/.well-known/agent-card.json", "/.well-known/agent.json"):
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(AGENT_CARD).encode())
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        text = body["params"]["message"]["parts"][0].get("text", body["params"]["message"]["parts"][0].get("text", ""))
        result = invoke_llm(text)
        task_id = str(uuid.uuid4())
        response = {
            "jsonrpc": "2.0",
            "result": {
                "id": task_id,
                "status": {"state": "completed"},
                "history": [
                    body["params"]["message"],
                    {"role": "agent", "parts": [{"text": result}], "messageId": str(uuid.uuid4()), "taskId": task_id}
                ]
            },
            "id": body["id"]
        }
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(response).encode())

    def log_message(self, format, *args): pass

HTTPServer(("0.0.0.0", 8002), Handler).serve_forever()
```

### Endpoints

- Card: `http://localhost:8002/.well-known/agent-card.json`
- A2A: `POST http://localhost:8002`

### Test

```bash
curl http://localhost:8002/.well-known/agent-card.json | jq .

curl -X POST http://localhost:8002 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"message/send","params":{"message":{"role":"user","parts":[{"text":"What is the capital of France?"}],"messageId":"t1"}},"id":"r1"}'
```

**Verified response:** `"The capital of France is Paris."`

### Connect from MCP

```
resolve_agent_card("http://localhost:8002")
send_message(agent_id, "What is the capital of France?")
→ "The capital of France is Paris."
```

---

## CrewAI (Python)

### Prerequisites

```bash
pip install crewai
```

### Wrap a Crew with A2A

CrewAI agents can be exposed via A2A using the same HTTP server pattern as LangGraph. Wrap your `crew.kickoff()` call inside a `do_POST` handler that returns a Task response.

---

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

---

## Troubleshooting

| Error | Fix |
|-------|-----|
| `"No agent card found"` | Ensure agent serves `/.well-known/agent-card.json` or `/.well-known/agent.json` |
| `"connection refused"` | Agent not running or wrong port |
| `"Agent not found: agent_xxx"` | Call `resolve_agent_card` first (registry is per-session) |
| Task state `"failed"` | Check agent logs, API key validity, input format |
