# A2A Integration Test Plan

## Goal

Verify that the `mcp-a2a` server can discover, communicate with, and manage tasks across agents built with three different frameworks — all speaking the A2A protocol.

## Test Agents

### 1. Google ADK Python Agent (Port 8001)
- **Framework:** `google-adk[a2a]`
- **Capability:** Prime number checking
- **Exposed via:** `adk api_server --a2a --port 8001`
- **Agent card:** `http://localhost:8001/.well-known/agent.json`

### 2. LangGraph Agent (Port 8002)
- **Framework:** `langgraph` + `a2a-sdk`
- **Capability:** Currency conversion
- **Exposed via:** Custom A2A server using `a2a-sdk`
- **Agent card:** `http://localhost:8002/.well-known/agent.json`

### 3. ADK-Rust Agent (Port 8003)
- **Framework:** `adk-rust` + `adk-server`
- **Capability:** Weather lookup
- **Exposed via:** `adk-server` with A2A enabled
- **Agent card:** `http://localhost:8003/.well-known/agent.json`

## Test Matrix

| Test | Description | Expected |
|------|-------------|----------|
| T1 | ADK Python agent responds to direct A2A message | Task completed |
| T2 | LangGraph agent responds to direct A2A message | Task completed |
| T3 | ADK-Rust agent responds to direct A2A message | Task completed |
| T4 | MCP `resolve_agent_card` discovers ADK Python | Card returned |
| T5 | MCP `resolve_agent_card` discovers LangGraph | Card returned |
| T6 | MCP `resolve_agent_card` discovers ADK-Rust | Card returned |
| T7 | MCP `validate_agent_card` validates all three | All pass |
| T8 | MCP `send_message` to ADK Python via A2A | Task result |
| T9 | MCP `send_message` to LangGraph via A2A | Task result |
| T10 | MCP `send_message` to ADK-Rust via A2A | Task result |
| T11 | MCP `list_remote_agents` shows all three | 3 agents |
| T12 | MCP `get_task` retrieves completed task | History present |
| T13 | MCP `cancel_task` cancels a running task | Canceled state |
| T14 | Cross-agent: ADK Python calls LangGraph via A2A | Works |
| T15 | Cross-agent: LangGraph calls ADK-Rust via A2A | Works |

## Prerequisites

```bash
# Python (Google ADK + LangGraph)
pip install google-adk[a2a] langgraph a2a-sdk langchain-google-genai

# Rust (ADK-Rust)
cd /path/to/adk-rust && cargo build --release

# MCP A2A Server
cd /path/to/mcp-a2a && cargo build --release
```

## Directory Structure

```
tests/a2a-integration/
├── README.md
├── run_all.sh              # Start all agents, run tests, stop
├── google-adk-agent/       # Python ADK agent
│   ├── agent.py
│   └── agent-card.json
├── langgraph-agent/        # LangGraph agent
│   ├── agent.py
│   └── requirements.txt
├── adk-rust-agent/         # ADK-Rust agent
│   └── (uses existing adk-server)
└── test_mcp_a2a.sh         # MCP tool call tests
```

## Implementation Steps

1. Create Google ADK Python agent with prime checking
2. Create LangGraph agent with currency conversion
3. Configure ADK-Rust agent with weather tool
4. Start all three on different ports
5. Run MCP A2A tools against each
6. Verify cross-agent communication
7. Document results
