# Protocol: WebSocket Protocol

### Scope

Real-time WebSocket message format for streaming agent state updates, events, and alerts to dashboard.

**In Scope:**
- Message types (STATE_UPDATE, AGENT_EVENT, COST_ALERT, HEARTBEAT)
- WebSocket connection lifecycle (connect, authenticate, subscribe, disconnect)
- Message format (JSON with type discriminator)
- Heartbeat protocol (30s interval)
- Reconnection strategy (exponential backoff, max 30s)
- Error handling and timeout behavior

**Out of Scope:**
- REST API protocol (see [002_rest_api_protocol.md](002_rest_api_protocol.md))
- IronLang data protocol (see [001_ironlang_data_protocol.md](001_ironlang_data_protocol.md))
- Dashboard UI implementation (see `module/iron_dashboard/`)
- WebSocket server implementation (see `module/iron_control_api/spec.md` § FR-3)

---

### Purpose

Enable real-time dashboard updates without polling, providing instant visibility into agent execution, budget usage, and safety events.

**Problem:**

HTTP polling for dashboard updates:
- High latency (1-5 second delay between updates)
- Server load (constant polling creates unnecessary requests)
- Wasted bandwidth (mostly "no changes" responses)
- Complex client logic (retry, backoff, state reconciliation)

**Solution:**

WebSocket streaming provides:
- Real-time updates (<100ms latency from event to dashboard)
- Low server load (single persistent connection per dashboard)
- Efficient bandwidth (only changed data sent)
- Simple client logic (receive message, update UI)
- Bidirectional communication (heartbeat, reconnection)

---

### Protocol Definition

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

### Cross-References

**Dependencies:**
- None (foundational real-time protocol)

**Used By:**
- [capabilities/007: Observability](../capabilities/007_observability.md) - Real-time metrics streaming
- [protocol/005: Budget Control](005_budget_control_protocol.md) - Budget warnings via WebSocket
- Dashboard implementations - Receive real-time updates

**Related:**
- [002: REST API Protocol](002_rest_api_protocol.md) - Complementary request/response protocol
- [architecture/005: Service Integration](../architecture/005_service_integration.md) - WebSocket server integration pattern

**Implementation:**
- Source: `module/iron_control_api/src/websocket.rs` - WebSocket handler
- Tests: `module/iron_control_api/tests/websocket_test.rs` - Connection and message tests
- Specification: `module/iron_control_api/spec.md` § FR-3 - WebSocket streaming specification

---

**Last Updated:** 2025-12-09
**Document Version:** 1.0
**Status:** Specification complete
