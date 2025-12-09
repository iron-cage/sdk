# Protocol Collection

Communication protocols defining message formats, wire protocols, and version compatibility.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 002 | **002_rest_api_protocol.md** | Document HTTP REST API endpoint schemas for Control Panel operations (budget handshake, reporting, refresh, token management) |
| 003 | **003_websocket_protocol.md** | Specify real-time dashboard WebSocket message format (STATE_UPDATE, AGENT_EVENT, COST_ALERT, connection lifecycle, heartbeat) |
| 004 | **004_mcp_integration_protocol.md** | Define Model Context Protocol implementation (tool discovery, invocation, error mapping Iron Cage ↔ MCP) |
| 005 | **005_budget_control_protocol.md** | Document budget enforcement and token management protocol (two-token system, budget borrowing, token handshake) |

---

## Protocol Collection

| ID | Name | Purpose |
|----|------|---------|
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
- **Modules:** iron_api implements 002+003, iron_runtime uses 005
- **Architecture:** [architecture/004: Data Flow](../architecture/004_data_flow.md) uses budget control protocol
- **Capabilities:** [capabilities/002: LLM Access Control](../capabilities/002_llm_access_control.md) enabled by budget protocol (005)

---

**Last Updated:** 2025-12-09
**Note:** Follows Design Collections format per documentation.rulebook.md § protocol/ standards.
