# Protocol Collection

Communication protocols defining message formats, wire protocols, and version compatibility.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 002 | **002_rest_api_protocol.md** | Document HTTP REST API overview (resource organization, authentication architecture, common patterns, status codes, versioning, CLI-API parity) |
| 003 | **003_websocket_protocol.md** | Specify real-time dashboard WebSocket message format (STATE_UPDATE, AGENT_EVENT, COST_ALERT, connection lifecycle, heartbeat) |
| 004 | **004_mcp_integration_protocol.md** | Define Model Context Protocol implementation (tool discovery, invocation, error mapping Iron Cage ↔ MCP) |
| 005 | **005_budget_control_protocol.md** | Document budget enforcement and token management protocol (two-token system, budget borrowing, token handshake) |
| 006 | **006_token_management_api.md** | Document IC Token lifecycle API (create, list, get, delete, rotate) with permission-based access (admin vs developer) |
| 007 | **007_authentication_api.md** | Document User authentication API (login, logout, refresh, validate) with JWT-based token lifecycle management |
| -draft_usage_analytics_api.md | **-draft_usage_analytics_api.md** | DRAFT: Usage analytics API (request counts, token usage, cost analysis) - Uncertain, not Pilot-critical |
| -draft_limits_management_api.md | **-draft_limits_management_api.md** | DRAFT: Agent Budget limits configuration API (view, update, bulk operations) - Uncertain, admin tooling |
| -draft_provider_management_api.md | **-draft_provider_management_api.md** | DRAFT: Inference Provider (IP) management API (add, configure, test, remove IPs) - Uncertain, vault integration pending |

---

## Protocol Collection

### Certain Protocols (✅ Required for Pilot)

| ID | Name | Purpose |
|----|------|---------|
| 002 | [REST API Protocol](002_rest_api_protocol.md) | HTTP REST API overview and common patterns |
| 003 | [WebSocket Protocol](003_websocket_protocol.md) | Real-time dashboard message format |
| 004 | [MCP Integration Protocol](004_mcp_integration_protocol.md) | Model Context Protocol tool integration |
| 005 | [Budget Control Protocol](005_budget_control_protocol.md) | Two-token system (IC/IP), budget borrowing, token handshake |
| 006 | [Token Management API](006_token_management_api.md) | IC Token CRUD endpoints |
| 007 | [Authentication API](007_authentication_api.md) | User login/logout/refresh endpoints |

### Uncertain Protocols (⚠️ Drafts, Not for Implementation)

| Name | Purpose | Reason for Uncertainty |
|------|---------|------------------------|
| [-draft_usage_analytics_api.md](-draft_usage_analytics_api.md) | Usage metrics and cost analysis | Not Pilot-critical, aggregation design pending |
| [-draft_limits_management_api.md](-draft_limits_management_api.md) | Agent Budget limits configuration | Admin tooling, grace period design pending |
| [-draft_provider_management_api.md](-draft_provider_management_api.md) | Inference Provider management | Vault integration pending, multi-token design unclear |

---

## Cross-Collection Dependencies

**Protocols depend on:**
- **Capabilities:** Protocol implementations enable capabilities
- **Security:** Protocols follow security model
- **Architecture:** Protocols used in system architecture

**Protocols used by:**
- **Modules:** iron_control_api implements 002+003+006+007, iron_runtime uses 005, iron_cli uses 006+007
- **Architecture:** [architecture/004: Data Flow](../architecture/004_data_flow.md) uses budget control protocol (005)
- **Architecture:** [architecture/009: Resource Catalog](../architecture/009_resource_catalog.md) documents all REST API resources
- **Capabilities:** [capabilities/002: LLM Access Control](../capabilities/002_llm_access_control.md) enabled by budget protocol (005)
- **Features:** [features/004: Token Management CLI-API Parity](../features/004_token_management_cli_api_parity.md) uses 006+007

---

**Last Updated:** 2025-12-09
**Note:** Follows Design Collections format per documentation.rulebook.md § protocol/ standards.
