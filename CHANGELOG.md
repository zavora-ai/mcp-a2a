# Changelog

## [1.3.0] - 2026-08-13

### Changed
- Upgraded to rmcp 3.1.2 and raised the minimum supported Rust version to 1.94.1.
- Added MCP 2026-07-28 stateless request handling while retaining MCP 2025-11-25 initialization compatibility.

### Added
- Per-request identity and protocol metadata, on-demand discovery/cache hints, and the configured Tasks and sealed MRTR approval policies.

## [1.2.0] - 2025-05-24

### Added
- HealthCheck trait implementation for registry monitoring
- `mcp-server.toml` manifest for ADK registry onboarding
- Structured tracing with `tracing-subscriber` (env-filter)

### Changed
- Edition upgraded to Rust 2024
- Added `adk-mcp-sdk` HealthCheck integration


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
