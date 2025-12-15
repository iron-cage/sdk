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
- IronLang data protocol (archived, not in use for Control Panel API)
- Dashboard UI implementation (see `module/iron_dashboard/`)
- WebSocket server implementation (see `module/iron_control_api/spec.md` § FR-3)


### Purpose

**User Need**: Dashboard users (operators, developers, administrators) need real-time visibility into agent execution state (running/idle/completed/error), LLM operation details (model gpt-4, tokens 500, cost $0.015 per call), and budget safety alerts (90% threshold warnings, exhaustion events) with sub-100ms latency to enable immediate intervention (pause agent, increase budget, investigate errors) when agents approach budget limits ($9.00 spent of $10.00 limit) or encounter safety violations, without the inefficiencies of HTTP polling that creates 60+ requests/minute per dashboard (wasted bandwidth on "no changes" HTTP 304 responses), adds 1-5 second latency delays (preventing timely budget alert response), increases server load 98% unnecessarily (constant connections vs single WebSocket), and requires complex client-side retry/backoff/state reconciliation logic.

**Solution**: Persistent bidirectional WebSocket connection protocol (ws://localhost:8080/ws) defining 4 message types with JSON type discriminator format: STATE_UPDATE (agent state changes with current_lead progress and budget_spent tracking), AGENT_EVENT (LLM calls and tool invocations with model/tokens/cost_usd details), COST_ALERT (budget threshold warnings with severity warning/critical and percentage 90-100 consumed), HEARTBEAT (30-second keepalive preventing 60-second disconnect timeout). Establish 5-step connection lifecycle (connect → authenticate IC Token first message → subscribe to all agent events → receive server heartbeats every 30s → disconnect on timeout or client close) with automatic reconnection strategy (exponential backoff starting 1s, maximum 30s, resume from last known state). Adhere to Iron Cage standards: ID Format (agent_<uuid> identifiers), Data Format (ISO 8601 timestamps 2025-12-09T09:00:00Z, decimal currency 5.23), Error Format (connection failure codes).

**Key Insight**: WebSocket protocol eliminates HTTP polling trade-offs for Iron Cage dashboard real-time monitoring by replacing 60 requests/minute wasteful polling pattern (1-5 second latency, 98% unnecessary server load, complex retry logic) with single persistent bidirectional connection delivering sub-100ms event notifications (STATE_UPDATE agent state, AGENT_EVENT LLM tracking, COST_ALERT budget warnings 90%+ threshold, HEARTBEAT connection health) enabling immediate operator intervention on time-sensitive safety events (budget exhaustion requires pause within seconds not minutes), while maintaining simplicity through 4 message types with standardized JSON format and automatic exponential backoff reconnection (max 30s), complementing REST API (002) request/response pattern by providing push-based notifications for events requiring millisecond response times versus pull-based data retrieval for user-initiated actions.

---

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13
**Priority:** MUST-HAVE


### Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- All entity IDs use `prefix_uuid` format with underscore separator
- `agent_id`: `agent_<uuid>` (e.g., `agent_550e8400-e29b-41d4-a716-446655440000`)

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Currency: Decimal with exactly 2 decimal places (e.g., `5.23`, `0.015`)
- Percentages: Integer 0-100 (e.g., `90`)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- WebSocket error messages follow standard error response format
- Machine-readable error codes for connection failures


### Protocol Definition

#### Message Types

**STATE_UPDATE** - Sent when agent state changes

```json
{
  "type": "STATE_UPDATE",
  "timestamp": "2025-12-09T09:00:00Z",
  "agent_id": "agent_001",
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
  "agent_id": "agent_001",
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
  "agent_id": "agent_001",
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

#### Connection Lifecycle

1. **Connect:** ws://localhost:8080/ws
2. **Authenticate:** First message contains IC Token
3. **Subscribe:** Receive all agent events
4. **Heartbeat:** Server sends every 30s
5. **Disconnect:** Client closes or timeout (60s)

#### Reconnection Strategy

**On disconnect:**
- Retry after 1 second
- Exponential backoff (max 30s)
- Resume from last known state


### Cross-References

#### Related Principles Documents
- Design Philosophy - Real-time monitoring principle establishing push-based notifications for time-sensitive events (budget alerts, agent errors) vs pull-based REST API for user-initiated actions, WebSocket bidirectional persistent connection architecture decision, sub-100ms latency requirement for operator intervention scenarios
- Quality Attributes - Performance (sub-100ms event notification latency, 98% server load reduction vs HTTP polling 60 req/min), Reliability (automatic exponential backoff reconnection max 30s, heartbeat 30s keepalive preventing 60s timeout), Simplicity (4 message types only: STATE_UPDATE/AGENT_EVENT/COST_ALERT/HEARTBEAT), Observability (real-time agent execution tracking, LLM operation monitoring, budget consumption visibility)

#### Related Architecture Documents
- [Architecture: Service Integration](../architecture/005_service_integration.md) - WebSocket server integration pattern in Control Panel Gateway, persistent connection management, message broadcasting architecture

#### Used By
- [Capabilities: Observability](../capabilities/007_observability.md) - Real-time metrics streaming via STATE_UPDATE (agent state, current_lead, budget_spent) and AGENT_EVENT (LLM calls, model, tokens, cost_usd) messages
- [Protocol: Budget Control Protocol](005_budget_control_protocol.md) - Budget threshold warnings via COST_ALERT message type (severity warning/critical, percentage 90-100, budget_spent/budget_limit tracking)
- [Protocol: REST API Protocol](002_rest_api_protocol.md) - Complementary to REST request/response pattern, WebSocket provides push-based real-time notifications for time-sensitive events requiring sub-100ms response
- iron_dashboard - WebSocket client implementation receiving real-time agent updates (STATE_UPDATE), execution events (AGENT_EVENT), budget alerts (COST_ALERT), connection health (HEARTBEAT) via ws://localhost:8080/ws

#### Dependencies
- [Standards: ID Format Standards](../standards/id_format_standards.md) - Agent ID format `agent_<uuid>` used in STATE_UPDATE, AGENT_EVENT, COST_ALERT message types
- [Standards: Data Format Standards](../standards/data_format_standards.md) - ISO 8601 timestamp format (2025-12-09T09:00:00Z) in all message types, decimal currency format (5.23) for cost_usd and budget_spent fields
- [Standards: Error Format Standards](../standards/error_format_standards.md) - WebSocket connection failure error codes, authentication failure messages, standard error response format
- WebSocket Protocol Standard (RFC 6455) - Bidirectional persistent connection protocol over TCP, ws:// URI scheme, connection lifecycle (handshake, frames, close)
- JSON Standard (RFC 8259) - Message payload format with type discriminator field, object structure for STATE_UPDATE/AGENT_EVENT/COST_ALERT/HEARTBEAT messages

#### Implementation
- Module: `module/iron_control_api/` - WebSocket server implementation in Control Panel Gateway
- Source: `module/iron_control_api/src/websocket.rs` - WebSocket connection handler, message type serialization, broadcasting logic, heartbeat management
- Tests: `module/iron_control_api/tests/websocket_test.rs` - Connection lifecycle tests (connect/authenticate/subscribe/disconnect), message format validation (4 types), reconnection strategy tests (exponential backoff max 30s)
- Specification: `module/iron_control_api/spec.md` § FR-3 - WebSocket streaming functional requirement, message type schemas, connection timeout specifications
- Endpoints: ws://localhost:8080/ws - WebSocket server endpoint in Control Panel Gateway (port 8080)
- Authentication: IC Token sent in first WebSocket message for agent authentication, connection rejected on invalid token

