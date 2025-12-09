# WebSocket Protocol

**Purpose:** Real-time dashboard message format for live agent state updates.

---

### User Need

Understand how dashboard receives real-time updates from runtime and Control Panel.

### Core Idea

**Bidirectional WebSocket for dashboard live updates:**

```
Runtime/Control Panel → WebSocket → Dashboard
                        - STATE_UPDATE: Agent state changes
                        - AGENT_EVENT: LLM calls, tool usage
                        - COST_ALERT: Budget warnings
                        - HEARTBEAT: Connection keep-alive
```

### Message Types

**STATE_UPDATE** - Sent when agent state changes

```json
{
  "type": "STATE_UPDATE",
  "timestamp": "2025-12-09T09:00:00Z",
  "agent_id": "agent-001",
  "state": "running",
  "data": {
    "current_lead": 42,
    "budget_spent": 5.23
  }
}
```

**AGENT_EVENT** - Sent on LLM call or tool invocation

```json
{
  "type": "AGENT_EVENT",
  "timestamp": "2025-12-09T09:00:01Z",
  "agent_id": "agent-001",
  "event": "llm_call",
  "data": {
    "model": "gpt-4",
    "tokens": 500,
    "cost_usd": 0.015
  }
}
```

**COST_ALERT** - Sent when budget threshold exceeded

```json
{
  "type": "COST_ALERT",
  "timestamp": "2025-12-09T09:00:02Z",
  "agent_id": "agent-001",
  "severity": "warning",
  "data": {
    "budget_spent": 9.00,
    "budget_limit": 10.00,
    "percentage": 90
  }
}
```

**HEARTBEAT** - Sent every 30 seconds

```json
{
  "type": "HEARTBEAT",
  "timestamp": "2025-12-09T09:00:30Z"
}
```

### Connection Lifecycle

1. **Connect:** ws://localhost:8080/ws
2. **Authenticate:** First message contains IC Token
3. **Subscribe:** Receive all agent events
4. **Heartbeat:** Server sends every 30s
5. **Disconnect:** Client closes or timeout (60s)

### Reconnection Strategy

**On disconnect:**
- Retry after 1 second
- Exponential backoff (max 30s)
- Resume from last known state

---

*Related: [002_rest_api_protocol.md](002_rest_api_protocol.md) | [../architecture/005_service_integration.md](../architecture/005_service_integration.md)*
