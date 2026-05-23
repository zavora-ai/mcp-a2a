# Changelog

## [1.0.0] - 2026-05-23

### Added

- **11 MCP tools** — resolve_agent_card, validate_agent_card, list_remote_agents, send_message, send_streaming_message, get_task, list_tasks, cancel_task, subscribe_to_task, configure_push_notifications, get_extended_agent_card
- **Agent card discovery** — fetch and validate /.well-known/agent.json
- **Task lifecycle** — create, monitor, cancel remote A2A tasks
- **Streaming support** — SSE-based task event subscription
- **Push notifications** — webhook configuration for task events
- **Auto-registration** — resolved agents automatically added to registry
- **Protocol version validation** — verify A2A compatibility
- **rmcp 1.7** — latest MCP protocol SDK
