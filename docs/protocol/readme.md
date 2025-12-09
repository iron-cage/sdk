# Protocol Collection

Communication protocols defining message formats, wire protocols, and version compatibility.

---

## Directory Responsibilities

| ID | Entity | Responsibility | Input → Output | Scope | Out of Scope |
|----|--------|----------------|----------------|-------|--------------|
| 001 | **001_ironlang_data_protocol.md** | Define NDJSON data source communication | Data protocol question → Message spec | IronMessage types (READ, WRITE, AUTH, LOG, ERROR), NDJSON streaming, error handling, versioning | NOT REST API (→ 002), NOT WebSocket (→ 003), NOT implementation (→ module/iron_lang/) |
| 002 | **002_rest_api_protocol.md** | Document HTTP endpoint schemas and contracts | REST question → API protocol | Endpoint schemas (tokens, usage, limits, traces), request/response formats, HTTP status codes, API versioning | NOT WebSocket (→ 003), NOT IronLang (→ 001), NOT implementation (→ module/iron_api/) |
| 003 | **003_websocket_protocol.md** | Specify real-time dashboard message format | WebSocket question → Message protocol | Message types (STATE_UPDATE, AGENT_EVENT, COST_ALERT), connection lifecycle, heartbeat, reconnection | NOT REST (→ 002), NOT IronLang (→ 001), NOT implementation (→ module/iron_api/) |
| 004 | **004_mcp_integration_protocol.md** | Define Model Context Protocol implementation | MCP question → Integration protocol | MCP message format, tool discovery, tool invocation, error mapping Iron Cage↔MCP | NOT IronLang (→ 001), NOT REST (→ 002), NOT capability overview (→ capabilities/006) |
| 005 | **005_budget_control_protocol.md** | Document budget enforcement and token management protocol | Budget control question → Protocol flow | IC Token vs IP Token, budget borrowing/leasing, token handshake, real-time tracking, budget refresh, provider token security | NOT capability overview (→ docs/capabilities/002), NOT implementation (→ module/iron_cost/spec.md), NOT other protocols (→ 001-004) |

---

## Protocol Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [IronLang Data Protocol](001_ironlang_data_protocol.md) | NDJSON agent-data source messages |
| 002 | [REST API Protocol](002_rest_api_protocol.md) | HTTP Control Panel endpoint schemas |
| 003 | [WebSocket Protocol](003_websocket_protocol.md) | Real-time dashboard message format |
| 004 | [MCP Integration Protocol](004_mcp_integration_protocol.md) | Model Context Protocol tool integration |
| 005 | [Budget Control Protocol](005_budget_control_protocol.md) | Two-token system (IC/IP), budget borrowing, token handshake |

---

## Cross-Collection Dependencies

**Protocols depend on:**
- **Capabilities:** Protocol implementations enable capabilities
- **Security:** Protocols follow security model
- **Architecture:** Protocols used in system architecture

**Protocols used by:**
- **Modules:** iron_lang implements 001, iron_api implements 002+003, iron_runtime uses 005
- **Architecture:** [architecture/004: Data Flow](../architecture/004_data_flow.md) uses budget control protocol
- **Capabilities:** [capabilities/002: LLM Access Control](../capabilities/002_llm_access_control.md) enabled by budget protocol (005)

---

**Last Updated:** 2025-12-09
**Note:** Follows Design Collections format per documentation.rulebook.md § protocol/ standards.
