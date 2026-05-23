# API Reference

## resolve_agent_card

Fetch an agent's card from its well-known URL and register it in the local registry.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `base_url` | string | Yes | Base URL of the remote agent (e.g. `http://localhost:8003`) |

**Behavior:**
1. Tries `GET {base_url}/.well-known/agent-card.json` (standard path)
2. Falls back to `GET {base_url}/.well-known/agent.json` (legacy path)
3. Parses the agent card and registers it with status `connected`
4. Returns the full agent record including the generated `agent_id`

**Returns:**
```json
{
  "agent_id": "agent_a2a_quickstart_abc123",
  "name": "a2a-quickstart-agent",
  "base_url": "http://localhost:8003",
  "status": "connected",
  "card": {
    "name": "a2a-quickstart-agent",
    "description": "A minimal A2A-capable AI assistant built with ADK-Rust",
    "url": "http://localhost:8003/a2a",
    "version": "1.0.0",
    "protocolVersion": "0.2.6",
    "capabilities": { "streaming": false, "pushNotifications": false, "stateTransitionHistory": false },
    "skills": [{ "id": "general", "name": "General Assistant", "description": "Answer questions", "tags": ["reasoning"] }]
  },
  "auth_mode": null,
  "last_seen_at": "2026-05-23T16:45:00Z",
  "created_at": "2026-05-23T16:45:00Z"
}
```

**Errors:**
- `"No agent card found at {base_url}"` — neither endpoint returned a valid card
- `"Failed to resolve agent card: connection refused"` — agent not running

---

## validate_agent_card

Fetch and validate an agent card for protocol compliance without registering it.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `base_url` | string | Yes | Base URL of the remote agent |

**Returns:**
```json
{
  "name": "a2a-quickstart-agent",
  "url": "http://localhost:8003/a2a",
  "protocol_version": "0.2.6",
  "version_valid": true,
  "has_skills": true,
  "skill_count": 1,
  "streaming": false,
  "push_notifications": false,
  "validation": "passed"
}
```

---

## list_remote_agents

Return all agents discovered in this session.

**Parameters:** None.

**Returns:**
```json
[
  {
    "agent_id": "agent_a2a_quickstart_abc123",
    "name": "a2a-quickstart-agent",
    "base_url": "http://localhost:8003",
    "status": "connected"
  }
]
```

---

## send_message

Send a message to a registered remote agent via A2A `message/send`.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `agent_id` | string | Yes | Agent ID from `resolve_agent_card` |
| `text` | string | Yes | Message text to send |

**Protocol flow:**
1. Looks up the agent's URL from the registry
2. Sends JSON-RPC 2.0 `message/send` to the agent's URL
3. Parses the response (handles both `artifacts` and `history` formats)
4. Stores the task in the local registry

**Returns (ADK-Rust / Google ADK format):**
```json
{
  "id": "task-uuid",
  "status": { "state": "completed" },
  "artifacts": [{ "parts": [{ "kind": "text", "text": "The capital of Kenya is Nairobi." }] }]
}
```

**Returns (LangGraph format):**
```json
{
  "id": "task-uuid",
  "status": { "state": "completed" },
  "history": [
    { "role": "user", "parts": [{ "text": "What is the capital of France?" }] },
    { "role": "agent", "parts": [{ "text": "The capital of France is Paris." }] }
  ]
}
```

**Errors:**
- `"Agent not found: {agent_id}"` — call `resolve_agent_card` first
- `"Request failed: connection refused"` — agent is down
- `"Request failed: timeout"` — agent took longer than 60s

---

## send_streaming_message

Initiate a streaming task. Returns immediately with a task ID.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `agent_id` | string | Yes | Agent ID |
| `text` | string | Yes | Message text |

**Returns:**
```json
{
  "task_id": "a1b2c3d4-...",
  "status": { "state": "working" },
  "streaming": true,
  "message": "Task started. Use subscribe_to_task for streaming updates."
}
```

---

## get_task

Retrieve a task by ID from the local registry.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `task_id` | string | Yes | Task ID |

**Returns:** Full task object with status and history/artifacts.

---

## list_tasks

List all tasks in the local registry.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `limit` | integer | No | Max tasks to return |

**Returns:** Array of task objects, most recent first.

---

## cancel_task

Cancel a running task.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `task_id` | string | Yes | Task ID to cancel |

**Returns:**
```json
{ "task_id": "...", "status": "canceled" }
```

---

## subscribe_to_task

Subscribe to streaming events for a task.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `task_id` | string | Yes | Task ID |

**Returns:**
```json
{
  "task_id": "...",
  "subscription": "active",
  "current_state": "working",
  "message": "Subscribed to task events."
}
```

---

## configure_push_notifications

Create or update a webhook for receiving task events.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `agent_id` | string | Yes | Agent to receive notifications for |
| `webhook_url` | string | Yes | URL to POST events to |
| `events` | string[] | No | Event types (default: `["status_update", "artifact_update"]`) |

**Returns:**
```json
{
  "config_id": "push_ddc834c4...",
  "agent_id": "agent_...",
  "webhook_url": "https://hooks.example.com/a2a",
  "events": ["status_update"],
  "active": true
}
```

---

## get_extended_agent_card

Fetch the full agent record including auth configuration.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `agent_id` | string | Yes | Agent ID |

**Returns:** Full agent record with card, auth_mode, timestamps.

---

## Data Types

### AgentCard

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Agent display name |
| `description` | string | What the agent does |
| `url` | string | A2A endpoint URL |
| `version` | string | Agent version |
| `protocolVersion` | string | A2A protocol version |
| `capabilities` | object | streaming, pushNotifications, stateTransitionHistory |
| `skills` | array | List of AgentSkill objects |

### Task

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique task ID |
| `status` | object | `{ "state": "completed" }` |
| `artifacts` | array? | Response parts (ADK-Rust/Google ADK format) |
| `history` | array? | Conversation messages (LangGraph format) |

### TaskStatus States

`submitted` · `working` · `input_required` · `completed` · `failed` · `canceled`
