# Security & Governance

## Threat Model

| Threat | Risk | Mitigation |
|--------|------|-----------|
| Malicious agent card | Agent impersonation | Validate card signatures, pin known agents |
| Man-in-the-middle | Message interception | Use HTTPS for all A2A communication |
| Prompt injection via A2A | Remote agent manipulates local agent | Sanitize responses, scope trust |
| Denial of service | Remote agent floods with events | Rate limiting per agent |
| Data exfiltration | Sensitive data sent to untrusted agent | Scope what data reaches remote agents |
| Stale agent cards | Connecting to decommissioned agents | TTL on cached cards, periodic re-validation |

## Enterprise Governance Features

### Tenant Isolation

In ADK-Rust Enterprise, all A2A messages include isolation headers:

```json
{
  "X-Tenant-Id": "tenant_123",
  "X-Workspace-Id": "ws_prod",
  "X-Environment-Id": "env_prod_us",
  "X-Trace-Id": "trace_abc123"
}
```

### Rate Limiting

Per-agent rate limits prevent abuse:

```json
{
  "requests_per_minute": 60,
  "tokens_per_minute": 100000,
  "concurrent_connections": 5
}
```

### Agent Card Validation

Before trusting a remote agent:

1. **Protocol version** — must be compatible (0.x or 1.x)
2. **Skills declared** — agent must declare what it can do
3. **URL reachable** — endpoint must respond
4. **Auth mode** — if specified, credentials must be valid
5. **Signature** (enterprise) — card signed by trusted authority

### Audit Trail

All A2A interactions are logged:

- Agent discovery events
- Messages sent and received
- Task state transitions
- Push notification deliveries
- Errors and denials

### Access Control

| Action | Required Permission |
|--------|-------------------|
| `resolve_agent_card` | Any authenticated user |
| `validate_agent_card` | Any authenticated user |
| `list_remote_agents` | Workspace read access |
| `send_message` | Agent communication permission |
| `send_streaming_message` | Agent communication permission |
| `get_task` | Task read access |
| `list_tasks` | Task read access |
| `cancel_task` | Task write access or task owner |
| `subscribe_to_task` | Task read access |
| `configure_push_notifications` | Admin or agent owner |
| `get_extended_agent_card` | Agent admin access |

## Best Practices

1. **Always validate before trusting** — use `validate_agent_card` before sending sensitive data
2. **Use HTTPS in production** — never send A2A messages over plain HTTP in production
3. **Scope agent access** — only resolve agents your workflow actually needs
4. **Monitor task states** — watch for stuck `working` tasks (may indicate agent issues)
5. **Set up push notifications** — don't poll; let events come to you
6. **Rotate auth tokens** — if agents use bearer auth, rotate regularly via Credentials Vault
7. **Log everything** — A2A interactions should flow to your audit system
