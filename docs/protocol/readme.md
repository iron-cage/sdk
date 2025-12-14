# Protocol Collection

Communication protocols defining message formats, wire protocols, and version compatibility.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 002 | **002_rest_api_protocol.md** | Define cross-cutting REST API standards and master index for 23 Control Panel resources across 5 categories (Entity CRUD 6, Operation RPC-style 4, Analytics derived 8, Configuration admin-only 3, System public 2). Establish 5 universal standards: Audit Logging (mutations POST/PUT/DELETE with timestamp/user_id/action/resource_type/resource_id/changes, 90-day retention), CLI-API Parity (100% coverage 69 commands: 47 control + 22 token mapping to endpoints), System Resources (health /api/health 200/503, version /api/version public), Rate Limiting (auth 10/min, mutations 100/hour, reads 1000/hour, analytics 500/hour, admin unlimited), Example Data (canonical agent_a1b2c3d4, token_t9x8y7z6, user_u5v6w7x8 values). Reference 4 external standards (Pagination/Sorting/Filtering, Error Format, ID Format, Data Format via Standards docs). Document 3 authentication patterns (IC Token JWT agent auth Bearer ic_..., User Token JWT dashboard Bearer ..., None public). Provide Complete Protocol Reference table mapping resources to Protocol 006-017 with certainty classification (Certain/MUST-HAVE/NICE-TO-HAVE/POST-PILOT) |
| 003 | **003_websocket_protocol.md** | Define real-time WebSocket protocol for dashboard monitoring enabling sub-100ms latency push notifications of agent execution state changes, LLM operation tracking, and budget safety alerts. Specify 4 message types with JSON type discriminator format: STATE_UPDATE (agent running/idle/error state with current_lead progress and budget_spent tracking), AGENT_EVENT (LLM calls and tool invocations with model gpt-4, tokens 500, cost_usd 0.015 details), COST_ALERT (budget threshold warnings with severity warning/critical and percentage 90-100 consumed), HEARTBEAT (30-second keepalive preventing 60-second disconnect timeout). Document 5-step connection lifecycle (connect ws://localhost:8080/ws → authenticate IC Token first message → subscribe all agent events → receive server heartbeats → disconnect on timeout) with automatic exponential backoff reconnection strategy (1s initial retry, maximum 30s, resume from last known state). Eliminate HTTP polling inefficiencies (98% server load reduction from 60 req/min constant polling, sub-100ms vs 1-5s latency delay). Adhere to ID Format Standards (agent_<uuid>), Data Format Standards (ISO 8601 timestamps, decimal currency), Error Format Standards (connection failures). Complement REST API (002) request/response pattern with bidirectional persistent connection for time-sensitive operator intervention events (budget exhaustion, agent errors requiring pause within seconds) |
| 004 | **004_mcp_integration_protocol.md** | Define Model Context Protocol (MCP) integration enabling AI agents to discover and invoke tools through standard JSON-RPC 2.0 based wire protocol while adding Iron Cage governance layer for budget tracking, safety validation, and authorization enforcement. Specify 2 core MCP methods: tools/list (discovery returning tool array with name read_file, description, inputSchema object/properties/required fields for dynamic capability detection), tools/call (invocation accepting name, arguments params returning content array results). Document Iron Cage 7-step governance flow wrapping each tool invocation: (1) agent calls MCP tool via proxy, (2) validate tool authorization against agent permissions, (3) check budget before invocation preventing $10 limit breach, (4) invoke actual MCP tool with original params, (5) scan result for PII before returning, (6) track cost $0.015 per call against agent budget, (7) return result or governance error. Map Iron Cage safety errors to standard MCP error codes: BudgetExceeded → -32001, PiiDetected → -32002, Unauthorized → -32003, RateLimited → -32004. Eliminate custom tool integration complexity (no hardcoded tool-specific logic per capability, agents discover tools dynamically instead of manual configuration, standard wire format enables ecosystem tool reuse) while preventing three critical failures of unrestricted tool access (budget exhaustion from runaway loops $100+ costs, PII leakage violating GDPR, unauthorized dangerous tool access rm -rf). Adhere to ID Format Standards (agent_<uuid> governance), Data Format Standards (ISO 8601 timestamps, decimal cost 2 decimals), Error Format Standards (machine-readable TOOL_NOT_FOUND/UNAUTHORIZED/BUDGET_EXCEEDED codes). Reference external Model Context Protocol specification (https://modelcontextprotocol.io) for JSON-RPC 2.0 base protocol |
| 005 | **005_budget_control_protocol.md** | Document budget enforcement and token management protocol (two-token system, budget borrowing, token handshake) |
| 006 | **006_token_management_api.md** | Define RESTful CRUD API for IC Token lifecycle management enabling developers and admins to create, list, view, delete, and rotate IC Tokens with permission-based access controls. Specify 5 HTTP endpoints: GET /api/v1/tokens (list with pagination ?page=N&per_page=M default 50 max 200, filters for project_id/status/agent_id, permission scoping: developers see own tokens, admins see all), GET /api/v1/tokens/{id} (detail with usage_summary showing total_requests and total_cost_usd, 403 Forbidden for unauthorized access), POST /api/v1/tokens (create with agent_id/project_id, returns token value ONLY on creation with warning message, enforces 1:1 agent-token constraint via 409 Conflict error), DELETE /api/v1/tokens/{id} (immediate invalidation with 204 No Content, causes budget protocol calls to return 401 Unauthorized), PUT /api/v1/tokens/{id}/rotate (atomic operation generating new value, invalidating old, returning new token with rotated_at timestamp). Authenticate all requests with User Token (not IC Token) to avoid circular dependency where IC Tokens manage themselves, mapping to CLI flow: iron login → User Token → iron tokens create. Enforce permissions every request: developers access only owned agent tokens via agent ownership check, admins unrestricted access, 403 Forbidden for unauthorized. Protect token values: only in POST/PUT responses, NEVER in GET/LIST endpoints, preventing credential leakage from logs or audit trails. Adhere to ID Format Standards (token_<uuid>), Data Format Standards (ISO 8601 timestamps, JSON booleans), Error Format Standards (machine-readable codes: VALIDATION_ERROR, UNAUTHORIZED, NOT_FOUND, DUPLICATE_NAME, PERMISSION_DENIED, RESOURCE_CONFLICT), API Design Standards (standard REST conventions). Map to CLI commands via features/004 (24 operations: iron tokens list/get/create/delete/rotate). |
| 007 | **007_authentication_api.md** | Document User authentication API (login, logout, refresh, validate) with JWT-based token lifecycle management |
| 008 | **008_user_management_api.md** | Document User account management API (create, list, get, suspend, activate, delete, role change, password reset) with admin-only RBAC and audit logging |

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
| 008 | [User Management API](008_user_management_api.md) | Admin user account management endpoints |

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
