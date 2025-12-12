# Protocol 002: REST API Protocol

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-11
**Priority:** MUST-HAVE

---

### Scope

HTTP REST API endpoint schemas for all Control Panel operations organized by resource type.

**In Scope:**
- **Entity Resources:** IC Token CRUD, Project management, IP management
- **Operation Resources:** Authentication (login/logout), Budget protocol (handshake/report/refresh)
- **Analytics Resources:** Analytics (usage metrics, spending analysis, performance data)
- **Configuration Resources:** Budget limits, system settings
- **System Resources:** Health check, API version
- HTTP status codes and error responses
- Request/response JSON schemas
- Authentication types: IC Token (agents), User Token (users), None (public endpoints)
- Common patterns: Pagination, filtering, sorting, search
- API versioning strategy

**Out of Scope:**
- WebSocket protocol (see [003_websocket_protocol.md](003_websocket_protocol.md))
- IronLang data protocol (archived, not in use for Control Panel API)
- Implementation details (see `module/iron_control_api/spec.md`)
- Individual resource-specific protocols (see [006_token_management_api.md](006_token_management_api.md), [007_authentication_api.md](007_authentication_api.md))

---

### Purpose

Define cross-cutting REST API standards that apply universally across all resource-specific protocols and serve as the master index for all REST API documentation.

**Cross-Cutting Standards Defined in This Document:**

1. **Audit Logging Standard** - What operations are logged, standard fields, retention policy (mutation operations only)
2. **CLI-API Parity Standard** - Ensuring 100% coverage of user-facing endpoints in CLI (69 commands total: 47 control + 22 token management)
3. **System Resources** - Health check and API version endpoints (operational/discovery endpoints)
4. **Rate Limiting Standard** - Cross-endpoint rate limit policies by category
5. **Example Data Standards** - Consistent example values across all documentation

**Cross-Cutting Standards Defined in Separate Files:**

- **Pagination, Sorting, Filtering:** See [API Design Standards](../standards/api_design_standards.md)
- **Error Response Format:** See [Error Format Standards](../standards/error_format_standards.md)
- **ID Format:** See [ID Format Standards](../standards/id_format_standards.md)
- **Data Format (Timestamps, Currency, etc.):** See [Data Format Standards](../standards/data_format_standards.md)

**Resource-Specific API Endpoints:**

See Protocol files 006-017 for individual resource endpoint specifications:
- Agents (010), Providers (011), Analytics (012)
- API Tokens (014), Projects (015), Users (008)
- Budget Requests (017), Settings (016 - POST-PILOT)

**Note:** This document consolidates standards that span multiple resources and don't fit in individual resource protocols. Standards comprehensive enough to warrant separate normative files are maintained independently and referenced here.

---

### How to Use This Document

**For Cross-Cutting Standards (Defined in This Document):**
- **Audit Logging:** See [Audit Logging Standard](#audit-logging-standard) section below
- **CLI-API Parity:** See [CLI-API Parity Standard](#cli-api-parity-standard) section below
- **System Resources:** See [System Resources](#system-resources) section below (health check, API version)
- **Rate Limiting:** See [Rate Limiting](#rate-limiting) section below
- **Example Data:** See [Example Data Standards](#example-data-standards) section below

**For Resource-Specific Endpoints:**
- **Resource Taxonomy:** See [Resource Organization](#resource-organization) for 5-category classification
- **Master Index:** See [Complete Protocol Reference](#complete-protocol-reference) for full endpoint catalog
- **Individual Resources:** See protocol files 006-017 for detailed endpoint specifications

**For Standards Defined Elsewhere:**
- **Pagination:** [API Design Standards - Pagination](../standards/api_design_standards.md#pagination)
- **Sorting:** [API Design Standards - Sorting](../standards/api_design_standards.md#sorting)
- **Filtering:** [API Design Standards - Filtering](../standards/api_design_standards.md#filtering)
- **Errors:** [Error Format Standards](../standards/error_format_standards.md)
- **IDs:** [ID Format Standards](../standards/id_format_standards.md)
- **Data Formats:** [Data Format Standards](../standards/data_format_standards.md)

**Quick Start:** If you're looking for a specific endpoint, jump to [Complete Protocol Reference](#complete-protocol-reference) table to find the right protocol file.

---

### Standards Compliance

All REST API endpoints and protocols adhere to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- All entity IDs use `prefix_uuid` format with underscore separator
- Examples: `agent_<uuid>`, `provider_<uuid>`, `token_<uuid>`, `user_<uuid>`
- See standard for complete entity prefix list and validation rules

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Currency: Decimal with exactly 2 decimal places (e.g., `100.50`)
- Booleans: JSON boolean `true`/`false` (not strings)
- Nulls: Omit optional fields when empty (not `null`)
- Arrays: Empty array `[]` when no items (not `null`)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes (e.g., `VALIDATION_ERROR`, `UNAUTHORIZED`)
- HTTP status codes: 200, 201, 400, 401, 403, 404, 409, 429, 500, 503
- Field-level validation details in `error.fields` object

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- Pagination: Offset-based with `?page=N&per_page=M` (default 50 items/page)
- Sorting: Optional `?sort=field` (ascending) or `?sort=-field` (descending)
- Filtering: Resource-specific query parameters
- Versioning: URL-based `/api/v1/`, `/api/v2/`
- Deprecation: 6-month notice with `X-API-Deprecation` headers

**Note:** Resource-specific protocols (006-017) inherit these standards and add domain-specific constraints.

---

### Cross-Cutting Standards

The following standards apply universally across ALL REST API endpoints unless explicitly documented otherwise in resource-specific protocols.

#### Universal Pagination Standard

**See:** [API Design Standards - Pagination](../standards/api_design_standards.md#pagination)

All list endpoints adhere to the universal pagination standard defined in API Design Standards.

**Standard Features:**
- **Method:** Offset-based with `?page=N&per_page=M` query parameters
- **Defaults:** page=1, per_page=50
- **Limits:** page >= 1, per_page 1-100 (max 100)
- **Response Structure:** `{"data": [...], "pagination": {"page": N, "per_page": M, "total": X, "total_pages": Y}}`
- **Validation:** 400 Bad Request for invalid parameters (page < 1, per_page > 100)
- **Edge Cases:** Empty results return `data: []` with pagination metadata

**Endpoints with Pagination (7 categories):**
- `GET /api/v1/agents` - List agents
- `GET /api/v1/providers` - List providers
- `GET /api/v1/analytics/*` - All analytics endpoints (8 total)
- `GET /api/v1/api-tokens` - List API tokens
- `GET /api/v1/projects` - List projects
- `GET /api/v1/budget-requests` - List budget requests
- `GET /api/v1/users` - List users

**Complete Specification:** See [API Design Standards - Pagination](../standards/api_design_standards.md#pagination) for:
- Detailed query parameter validation rules
- Complete response format with all fields
- Edge case handling (empty results, page overflow, invalid params)
- Error response examples with field-level validation details
- Implementation notes (SQL OFFSET/LIMIT, performance considerations)
- Testing guidance and examples

---

#### Audit Logging Standard

**Applies To:** All mutation operations (POST, PUT, DELETE) that modify system state

**Scope:** Mutation operations only (read operations NOT logged to avoid excessive volume)

**Operations Logged:**

| HTTP Method | Operations | Examples |
|-------------|-----------|----------|
| POST | Create resources | Create agent, create provider, create API token, create budget request |
| PUT | Update resources | Update agent budget, approve budget request, suspend user, change role |
| DELETE | Delete resources | Delete agent, delete provider, revoke API token, delete user |

**NOT Logged:**
- GET operations (read-only, no state change)
- Health checks (`GET /api/health`)
- Version queries (`GET /api/version`)
- Failed authentication attempts (handled separately in security audit)

**Audit Log Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Audit log entry ID (prefix: `audit_`) |
| `timestamp` | string | ISO 8601 timestamp with Z suffix |
| `operation` | string | Operation type (e.g., AGENT_CREATED, BUDGET_UPDATED, USER_DELETED) |
| `resource_type` | string | Resource affected (agent, provider, user, token, budget_request) |
| `resource_id` | string | ID of affected resource |
| `user_id` | string | User who performed operation (null for system operations) |
| `user_role` | string | Role at time of operation (admin, user, viewer) |
| `ip_address` | string | Source IP address |
| `user_agent` | string | HTTP User-Agent header |
| `request_id` | string | Request correlation ID for tracing |
| `changes` | object | Before/after state for updates (null for create/delete) |
| `metadata` | object | Operation-specific details (e.g., justification, review notes) |

**Audit Log Schema Example:**

```json
{
  "id": "audit_abc123xyz789",
  "timestamp": "2025-12-11T10:30:45.123Z",
  "operation": "AGENT_BUDGET_UPDATED",
  "resource_type": "agent",
  "resource_id": "agent_abc123",
  "user_id": "user_xyz789",
  "user_role": "admin",
  "ip_address": "192.168.1.100",
  "user_agent": "iron-cli/1.0.0",
  "request_id": "req_def456",
  "changes": {
    "before": {"budget": 100.00},
    "after": {"budget": 150.00}
  },
  "metadata": {
    "justification": "Emergency budget increase for production agent",
    "force_flag": true
  }
}
```

**Retention Policy:**
- **Pilot:** 90 days rolling retention
- **POST-PILOT:** Configurable retention (90 days, 1 year, indefinite) with audit export

**Sensitive Data Handling:**
- **Never Log:** Passwords, API token values, IC token values, provider credentials
- **Sanitize:** Replace sensitive fields with `[REDACTED]` in audit log
- **Hash:** Store user IP as SHA-256 hash for privacy (POST-PILOT option)

**Query Operations (POST-PILOT):**
- `GET /api/v1/audit-logs` - List audit logs (admin-only, paginated)
- Filters: `user_id`, `resource_type`, `operation`, `start_date`, `end_date`
- Export: CSV/JSON download for compliance

**Operation Types:**

| Category | Operations |
|----------|-----------|
| Agent | AGENT_CREATED, AGENT_UPDATED, AGENT_DELETED, AGENT_PROVIDERS_UPDATED, AGENT_PROVIDER_REMOVED |
| Provider | PROVIDER_CREATED, PROVIDER_UPDATED, PROVIDER_DELETED |
| User | USER_CREATED, USER_SUSPENDED, USER_ACTIVATED, USER_DELETED, USER_ROLE_CHANGED, USER_PASSWORD_RESET |
| Budget | BUDGET_UPDATED, BUDGET_REQUEST_CREATED, BUDGET_REQUEST_APPROVED, BUDGET_REQUEST_REJECTED, BUDGET_REQUEST_CANCELLED |
| Token | API_TOKEN_CREATED, API_TOKEN_REVOKED, IC_TOKEN_REGENERATED |
| Project | PROJECT_CREATED, PROJECT_UPDATED, PROJECT_DELETED (POST-PILOT) |

**Database Schema:**

```sql
CREATE TABLE audit_logs (
  id VARCHAR(50) PRIMARY KEY,
  timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
  operation VARCHAR(100) NOT NULL,
  resource_type VARCHAR(50) NOT NULL,
  resource_id VARCHAR(50) NOT NULL,
  user_id VARCHAR(50),
  user_role VARCHAR(20),
  ip_address VARCHAR(45),
  user_agent VARCHAR(500),
  request_id VARCHAR(50),
  changes JSONB,
  metadata JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_operation ON audit_logs(operation);
```

**Implementation Notes:**
- **Asynchronous:** Write audit logs asynchronously (don't block API responses)
- **Guaranteed:** Use message queue (Redis/RabbitMQ) to ensure no log loss
- **Correlation:** Include `request_id` in all responses for tracing
- **Testing:** Verify all mutation endpoints generate audit logs

---

#### CLI-API Parity Standard

**Principle:** All user-facing REST API endpoints MUST have corresponding CLI commands with identical functionality

**Scope:** User-facing endpoints only (excludes system/internal endpoints like budget handshake protocol)

**Enforcement:** Test suite verifies 1:1 parity between CLI commands and API endpoints

**Parity Requirements:**

| API Endpoint | CLI Command | Parity Level |
|--------------|-------------|--------------|
| POST /api/v1/agents | iron .agent.create | FULL (all parameters) |
| GET /api/v1/agents | iron .agent.list | FULL (all filters, pagination) |
| GET /api/v1/agents/{id} | iron .agent.get id::<id> | FULL (identical output) |
| PUT /api/v1/agents/{id} | iron .agent.update id::<id> | FULL (all fields) |
| PUT /api/v1/agents/{id}/providers | iron .agent.assign_providers id::<id> | FULL |
| GET /api/v1/agents/{id}/providers | iron .agent.list_providers id::<id> | FULL |
| DELETE /api/v1/agents/{id}/providers/{pid} | iron .agent.remove_provider id::<id> provider_id::<pid> | FULL |

**Exclusions (No CLI Required):**

| Endpoint | Reason |
|----------|--------|
| POST /api/budget/handshake | Internal protocol (iron_runtime only) |
| POST /api/budget/report | Internal protocol (iron_runtime only) |
| POST /api/budget/refresh | Internal protocol (iron_runtime only) |
| GET /api/health | System endpoint (curl sufficient) |
| GET /api/version | System endpoint (curl sufficient) |

**CLI Architecture:**

**Native CLI (`iron_cli` - Rust):**
- Owns responsibility for all user-facing commands
- Calls REST API endpoints directly (no business logic duplication)
- Returns API responses in formatted tables or JSON (format::json parameter)
- Handles authentication (token storage, refresh)

**Python Wrapper (`iron_cli_py` - Python):**
- Wraps `iron_cli` commands (calls `iron` binary)
- Provides Python-friendly interface for scripting
- No direct API calls (delegates to native CLI)

---

**Framework Requirements:**

**Mandatory Framework:** All CLI implementations MUST use the `unilang` crate exclusively. The `clap` crate is strictly forbidden.

**Unilang Architecture:**
```
YAML Command Definitions → build.rs (compile-time) → Static Registry → Pipeline API
```

**Components:**

1. **Command Definitions** (`commands/*.yaml`):
   - One YAML file per command
   - Defines parameters, help text, validation rules
   - Example: `commands/agent/list.yaml`

2. **Build-Time Generation** (`build.rs`):
   - Parses YAML files at compile time
   - Generates static command registry (binary search array)
   - Type-safe handler signatures
   - Zero runtime overhead

3. **Handler Layer** (pure functions):
   - No I/O operations
   - No async code
   - Deterministic transformations only
   - Signature: `fn handler(params: Params) -> HandlerResult`

4. **Adapter Layer** (async I/O):
   - Bridges handlers to API clients
   - Handles HTTP requests
   - Formats output (table, JSON, YAML)
   - Signature: `async fn adapter(result: HandlerResult) -> CliOutput`

**Performance Characteristics:**
- Command resolution: <100ns (50x faster than HashMap)
- Startup time: <50ms
- Memory usage: <10MB typical
- Binary size: ~2-3MB (with compression)

**Benefits:**
- Compile-time safety: Invalid commands can't compile
- Zero overhead: Static dispatch, no dynamic lookups
- Maintainability: YAML changes don't require code changes
- Consistency: All CLIs use same framework

**Example YAML Definition:**
```yaml
- name: ".agent.list"
  namespace: ""
  description: "List all agents"
  hint: "Shows all configured AI agents"
  status: "stable"
  version: "1.0.0"
  idempotent: true
  arguments:
    - name: "v"
      kind: "Integer"
      description: "Verbosity level (0-5)"
      hint: "Higher values show more details"
      attributes:
        optional: true
        default: "2"
        multiple: false
        interactive: false
        sensitive: false
      validation_rules: []
      aliases: []
      tags: []
    - name: "format"
      kind: "String"
      description: "Output format"
      hint: "table, json, or yaml"
      attributes:
        optional: true
        default: "table"
        multiple: false
        interactive: false
        sensitive: false
      validation_rules: []
      aliases: []
      tags: []
  examples: []
  aliases: []
  tags: []
```

**Command Naming Convention:**

```
iron .<resource>.<action> [param::value...]
```

**Format Rules:**
- Dot-prefix mandatory: Commands MUST start with `.`
- Noun-verb order: Resource before action (`.agent.create` not `.create.agent`)
- Parameter format: Use `param::value` (NOT `--param value`)
- Underscore for multi-word: `.agent.assign_providers` not `.agent.assign-providers`

**Examples:**
- `iron .agent.list` → `GET /api/v1/agents`
- `iron .agent.create name::"Agent1" budget::100` → `POST /api/v1/agents`
- `iron .provider.delete id::ip_openai_001` → `DELETE /api/v1/providers/ip_openai_001`
- `iron .budget_request.approve id::breq-abc123` → `PUT /api/v1/budget-requests/breq-abc123/approve`

**Output Format:**

**Default (Table with verbosity v::2):**
```
$ iron .agent.list v::2

ID               NAME        BUDGET    SPENT   PROVIDERS   STATUS
agent_abc123     Agent 1     $100.00   $45.67  2           active
agent_def456     Agent 2     $50.00    $12.34  1           active
```

**JSON Mode (format::json):**
```
$ iron .agent.list format::json

{
  "data": [
    {"id": "agent_abc123", "name": "Agent 1", "budget": 100.00, "spent": 45.67, "providers_count": 2, "status": "active"},
    {"id": "agent_def456", "name": "Agent 2", "budget": 50.00, "spent": 12.34, "providers_count": 1, "status": "active"}
  ],
  "pagination": {"page": 1, "per_page": 50, "total": 2, "total_pages": 1}
}
```

**Help System:**

The CLI provides three ways to access help:

1. **Quick Help (`?`)** - One-line summary:
   ```
   $ iron .agent.list ?
   List all agents (GET /api/v1/agents) - Params: v, format, page, limit
   ```

2. **Detailed Help (`??`)** - Full command documentation:
   ```
   $ iron .agent.list ??

   Command: iron .agent.list
   API: GET /api/v1/agents
   Description: List all agents with optional filtering and pagination

   Parameters:
     v::N           Verbosity level (0-5, default: 2)
     format::STR    Output format (table|json|yaml, default: table)
     page::N        Page number (default: 1)
     limit::N       Results per page (default: 50)

   Examples:
     iron .agent.list                         # List all agents (table)
     iron .agent.list v::0                   # Silent mode
     iron .agent.list format::json           # JSON output
     iron .agent.list page::2 limit::100     # Pagination
   ```

3. **Global Help (`.help`)** - All commands:
   ```
   $ iron .help

   Available Commands:

   Agents:
     .agent.list                List all agents
     .agent.create              Create new agent
     .agent.get                 Get agent details
     .agent.update              Update agent
     .agent.delete              Delete agent

   Providers:
     .provider.list             List all providers
     .provider.create           Create provider
     ...

   Use 'iron <command> ??' for detailed help on any command
   ```

**Help Implementation:**
- Help text stored in YAML command definitions
- Generated at compile time (no runtime overhead)
- Supports filtering by resource: `iron .help resource::agent`
- Supports search: `iron .help search::budget`

---

**Verbosity Control:**

**Levels (0-5):**
```
$ iron .agent.list v::0          # Silent - exit code only (for scripting)
$ iron .agent.list v::1          # Minimal - count only ("2 agents")
$ iron .agent.list v::2          # Normal - table (default, human-readable)
$ iron .agent.list v::3          # Detailed - includes metadata, timestamps
$ iron .agent.list v::4          # Verbose - includes request/response details
$ iron .agent.list v::5          # Debug - full API response with timing, headers
```

**Key Principle:** Verbosity controls DISPLAY only, not computation or API calls.

**Example - Query Execution:**
```rust
// ALWAYS execute the API call (even at v::0)
let agents = api_client.get("/api/v1/agents").await?;
let count = agents.len();

// Verbosity only affects display
match verbosity {
  0 => {}, // Silent - no output
  1 => println!("{} agents", count),
  2 => print_table(&agents),
  3 => print_table_with_metadata(&agents),
  4 => print_verbose(&agents, &request_details),
  5 => print_debug(&agents, &full_response),
  _ => {}, // Invalid level
}

// Metrics ALWAYS recorded (even at v::0)
metrics.record("agent_count", count);
```

**Benefits:**
- Consistent behavior across verbosity levels
- Reliable for automation (`v::0` for scripts, check exit code)
- Debugging support (`v::5` shows full API interaction)
- No hidden side effects from verbosity changes

---

**Dry Run Mode:**

**Syntax:** `dry::1` parameter (default: `dry::0` for real execution)

**Behavior:**
```
$ iron .agent.create name::"Test Agent" budget::100 dry::1

DRY RUN MODE - No changes will be made

Would execute:
  API: POST /api/v1/agents
  Body: {"name": "Test Agent", "budget": 100}

Expected response:
  Status: 201 Created
  Agent ID: (generated on real execution)

To execute for real, remove 'dry::1' parameter
```

**Implementation:**
- Client-side validation before API call
- Shows what WOULD happen (endpoint, method, payload)
- Validates all parameters (same validation as real execution)
- Safe testing for destructive operations (delete, update)

**Example - Delete with Dry Run:**
```
$ iron .agent.delete id::agent_abc123 dry::1

DRY RUN MODE - No changes will be made

Would execute:
  API: DELETE /api/v1/agents/agent_abc123

Expected response:
  Status: 204 No Content

Verification: Run 'iron .agent.get id::agent_abc123' to confirm existence
To execute for real, remove 'dry::1' parameter
```

**Mutation Commands:** All mutation commands (create, update, delete, approve, etc.) MUST support `dry::1`

**Read-Only Commands:** Dry run has no effect on read-only commands (list, get) - always executes normally

---

**Error Handling:**

CLI propagates API errors with consistent format:

```
$ iron .agent.delete id::agent_xyz999

Error: Agent not found
Code: AGENT_NOT_FOUND
Status: 404

Run 'iron .agent.list' to see available agents.
```

**Testing Requirements:**

**Parity Test Suite:**
- Located: `iron_cli/tests/parity/`
- Verifies: Every user-facing API endpoint has CLI command
- Verifies: CLI command parameters match API endpoint parameters
- Verifies: CLI output includes all API response fields (in table or JSON mode)
- CI/CD: Runs on every API protocol change

**Example Test:**
```rust
#[test]
fn test_agents_list_parity() {
  // Verify CLI command exists
  let cli_result = run_cli(&[".agent.list", "format::json"]);

  // Verify API endpoint exists
  let api_result = http_get("/api/v1/agents");

  // Verify responses match
  assert_eq!(cli_result.data, api_result.data);
  assert_eq!(cli_result.pagination, api_result.pagination);
}
```

**Documentation:**

- CLI help text (`iron .agent ?` or `iron .agent ??`) MUST reference API endpoint
- API protocol docs MUST include CLI command examples
- CLI architecture doc maintains Responsibility Matrix (see [features/001_cli_architecture.md](../features/001_cli_architecture.md))
- Note: Token management CLI (`iron-token`) is documented separately in [004_token_management_cli_api_parity.md](../features/004_token_management_cli_api_parity.md)

**Coverage:**

| Resource | API Endpoints | CLI Commands | Parity |
|----------|---------------|--------------|--------|
| Agents | 8 | 8 | ✅ 100% |
| Providers | 8 | 8 | ✅ 100% |
| Analytics | 8 | 8 | ✅ 100% |
| Budget Limits | 2 | 2 | ✅ 100% |
| API Tokens | 4 | 4 | ✅ 100% |
| Projects | 2 | 2 | ✅ 100% |
| Budget Requests | 6 | 6 | ✅ 100% |
| Users | 8 | 8 | ✅ 100% |
| **Total User-Facing** | **46** | **46** | **✅ 100%** |

---

**CLI Command Responsibility Table:**

| # | CLI Command | API Endpoint | Responsibility | Category |
|---|-------------|--------------|----------------|----------|
| 1 | `.agent.list` | `GET /api/v1/agents` | List all agents with optional filtering/pagination | Agents |
| 2 | `.agent.create` | `POST /api/v1/agents` | Create new agent with name and budget | Agents |
| 3 | `.agent.get` | `GET /api/v1/agents/{id}` | Get agent details by ID | Agents |
| 4 | `.agent.update` | `PUT /api/v1/agents/{id}` | Update agent name or budget | Agents |
| 5 | `.agent.delete` | `DELETE /api/v1/agents/{id}` | Delete agent (cascade deletes associated leases) | Agents |
| 6 | `.agent.assign_providers` | `PUT /api/v1/agents/{id}/providers` | Assign LLM providers to agent | Agents |
| 7 | `.agent.list_providers` | `GET /api/v1/agents/{id}/providers` | List providers assigned to agent | Agents |
| 8 | `.agent.remove_provider` | `DELETE /api/v1/agents/{id}/providers/{pid}` | Remove provider assignment from agent | Agents |
| 9 | `.provider.list` | `GET /api/v1/providers` | List all LLM providers | Providers |
| 10 | `.provider.create` | `POST /api/v1/providers` | Create new LLM provider | Providers |
| 11 | `.provider.get` | `GET /api/v1/providers/{id}` | Get provider details by ID | Providers |
| 12 | `.provider.update` | `PUT /api/v1/providers/{id}` | Update provider configuration | Providers |
| 13 | `.provider.delete` | `DELETE /api/v1/providers/{id}` | Delete provider (fails if agents assigned) | Providers |
| 14 | `.provider.assign_agents` | `PUT /api/v1/providers/{id}/agents` | Assign agents to provider | Providers |
| 15 | `.provider.list_agents` | `GET /api/v1/providers/{id}/agents` | List agents using this provider | Providers |
| 16 | `.provider.remove_agent` | `DELETE /api/v1/providers/{id}/agents/{aid}` | Remove agent from provider | Providers |
| 17 | `.analytics.usage` | `GET /api/v1/analytics/usage` | Get usage statistics (tokens, requests) | Analytics |
| 18 | `.analytics.spending` | `GET /api/v1/analytics/spending` | Get spending statistics by agent/provider | Analytics |
| 19 | `.analytics.metrics` | `GET /api/v1/analytics/metrics` | Get performance metrics (latency, errors) | Analytics |
| 20 | `.analytics.usage_by_agent` | `GET /api/v1/analytics/usage/by-agent` | Usage breakdown by agent | Analytics |
| 21 | `.analytics.usage_by_provider` | `GET /api/v1/analytics/usage/by-provider` | Usage breakdown by provider | Analytics |
| 22 | `.analytics.spending_by_period` | `GET /api/v1/analytics/spending/by-period` | Spending breakdown by time period | Analytics |
| 23 | `.analytics.export_usage` | `GET /api/v1/analytics/export/usage` | Export usage data (CSV/JSON) | Analytics |
| 24 | `.analytics.export_spending` | `GET /api/v1/analytics/export/spending` | Export spending data (CSV/JSON) | Analytics |
| 25 | `.budget_limit.get` | `GET /api/v1/budget/limit` | Get current budget limit (admin only) | Budget Limits |
| 26 | `.budget_limit.set` | `PUT /api/v1/budget/limit` | Set budget limit (admin only) | Budget Limits |
| 27 | `.api_token.list` | `GET /api/v1/api-tokens` | List all API tokens | API Tokens |
| 28 | `.api_token.create` | `POST /api/v1/api-tokens` | Create new API token | API Tokens |
| 29 | `.api_token.get` | `GET /api/v1/api-tokens/{id}` | Get API token details | API Tokens |
| 30 | `.api_token.revoke` | `DELETE /api/v1/api-tokens/{id}` | Revoke API token | API Tokens |
| 31 | `.project.list` | `GET /api/v1/projects` | List all projects | Projects |
| 32 | `.project.get` | `GET /api/v1/projects/{id}` | Get project details | Projects |
| 33 | `.budget_request.list` | `GET /api/v1/budget-requests` | List budget change requests | Budget Requests |
| 34 | `.budget_request.create` | `POST /api/v1/budget-requests` | Create budget increase request | Budget Requests |
| 35 | `.budget_request.get` | `GET /api/v1/budget-requests/{id}` | Get budget request details | Budget Requests |
| 36 | `.budget_request.approve` | `PUT /api/v1/budget-requests/{id}/approve` | Approve budget request (admin only) | Budget Requests |
| 37 | `.budget_request.reject` | `PUT /api/v1/budget-requests/{id}/reject` | Reject budget request (admin only) | Budget Requests |
| 38 | `.budget_request.cancel` | `DELETE /api/v1/budget-requests/{id}` | Cancel pending budget request | Budget Requests |
| 39 | `.user.list` | `GET /api/v1/users` | List all users | Users |
| 40 | `.user.create` | `POST /api/v1/users` | Create new user | Users |
| 41 | `.user.get` | `GET /api/v1/users/{id}` | Get user details | Users |
| 42 | `.user.update` | `PUT /api/v1/users/{id}` | Update user profile | Users |
| 43 | `.user.delete` | `DELETE /api/v1/users/{id}` | Delete user | Users |
| 44 | `.user.set_role` | `PUT /api/v1/users/{id}/role` | Set user role (admin/user) | Users |
| 45 | `.user.reset_password` | `POST /api/v1/users/{id}/reset-password` | Trigger password reset | Users |
| 46 | `.user.get_permissions` | `GET /api/v1/users/{id}/permissions` | Get user permissions | Users |

**Table Notes:**
- All commands use unilang framework
- All commands support `v::N` (verbosity 0-5)
- All mutation commands support `dry::1` (dry run mode)
- All commands support `format::json` for machine-readable output
- All commands support `?` (quick help) and `??` (detailed help)

---

**Implementation Notes:**
- **Single Source of Truth:** API is canonical, CLI wraps API (no business logic in CLI)
- **Consistency:** CLI uses exact API error codes/messages
- **Authentication:** CLI stores tokens in `~/.iron/config.toml`, automatically refreshes
- **Scripting:** JSON mode (format::json) enables shell scripting

---

### Resource Organization

**Four Resource Categories:**

| Category | Definition | Auth Type | Examples | Certainty |
|----------|-----------|-----------|----------|-----------|
| **Entity Resources** | CRUD operations on domain entities | User Token | `/api/tokens`, `/api/projects`, `/api/providers` | Tokens: ✅ Certain, Projects/IPs: ⚠️ Uncertain |
| **Operation Resources** | RPC-style operations spanning entities | IC Token or User Token | `/api/auth`, `/api/budget/*` | ✅ Certain (required for Pilot) |
| **Analytics Resources** | Read-only derived/aggregated data | User Token | `/api/usage`, `/api/spending`, `/api/metrics` | ⚠️ Uncertain (not Pilot-critical) |
| **Configuration Resources** | System-level configuration | User Token (Admin) | `/api/limits`, `/api/settings` | ⚠️ Uncertain (admin tooling) |
| **System Resources** | Health/version endpoints | None | `/api/health`, `/api/version` | ✅ Certain (standard endpoints) |

**Certainty Classification:**
- ✅ **Certain:** Required for Pilot, design complete, documented in permanent protocol docs
- ⚠️ **Uncertain:** Not Pilot-critical or design pending, specifications deferred

**Complete Resource Inventory:** See [architecture/009_resource_catalog.md](../architecture/009_resource_catalog.md)

**Resource-Specific Protocols:**
- [006_token_management_api.md](006_token_management_api.md) - IC Token CRUD (✅ Certain, Pilot)
- [007_authentication_api.md](007_authentication_api.md) - User authentication (✅ Certain, Pilot)
- [008_user_management_api.md](008_user_management_api.md) - User management (✅ Certain, Pilot)
- [010_agents_api.md](010_agents_api.md) - Agent management (✅ MUST-HAVE)
- [011_providers_api.md](011_providers_api.md) - Provider management (✅ MUST-HAVE)
- [012_analytics_api.md](012_analytics_api.md) - Usage and spending analytics (✅ MUST-HAVE)
- [013_budget_limits_api.md](013_budget_limits_api.md) - Budget modification, admin-only (✅ MUST-HAVE)
- [014_api_tokens_api.md](014_api_tokens_api.md) - API token management (✅ MUST-HAVE)
- [015_projects_api.md](015_projects_api.md) - Project access (✅ NICE-TO-HAVE, Pilot: read-only)
- [016_settings_api.md](016_settings_api.md) - Settings management (📋 POST-PILOT)
- [017_budget_requests_api.md](017_budget_requests_api.md) - Budget change requests workflow (✅ MUST-HAVE)

---

### Complete Protocol Reference

| Resource Category | Resources | Protocol Document | Status | Pilot |
|-------------------|-----------|-------------------|--------|-------|
| **Entity Resources** | | | | |
| IC Tokens | `/api/tokens/*` | [006_token_management_api.md](006_token_management_api.md) | ✅ Certain | Yes |
| Agents | `/api/agents/*` | [010_agents_api.md](010_agents_api.md) | ✅ MUST-HAVE | Yes |
| Providers | `/api/providers/*` | [011_providers_api.md](011_providers_api.md) | ✅ MUST-HAVE | Yes |
| Projects | `/api/projects/*` | [015_projects_api.md](015_projects_api.md) | ✅ NICE-TO-HAVE | Yes (read-only) |
| **Operation Resources** | | | | |
| Authentication | `/api/auth/*` | [007_authentication_api.md](007_authentication_api.md) | ✅ Certain | Yes |
| User Management | `/api/users/*` | [008_user_management_api.md](008_user_management_api.md) | ✅ Certain | Yes |
| Budget Protocol | `/api/budget/*` | [005_budget_control_protocol.md](005_budget_control_protocol.md) | ✅ Certain | Yes |
| API Tokens | `/api/api-tokens/*` | [014_api_tokens_api.md](014_api_tokens_api.md) | ✅ MUST-HAVE | Yes |
| **Analytics Resources** | | | | |
| Analytics | `/api/analytics/*` | [012_analytics_api.md](012_analytics_api.md) | ✅ MUST-HAVE | Yes |
| **Configuration Resources** | | | | |
| Budget Limits | `/api/limits/*` | [013_budget_limits_api.md](013_budget_limits_api.md) | ✅ MUST-HAVE | Yes |
| Budget Requests | `/api/budget-requests/*` | [017_budget_requests_api.md](017_budget_requests_api.md) | ✅ MUST-HAVE | Yes |
| System Settings | `/api/settings/*` | [016_settings_api.md](016_settings_api.md) | 📋 POST-PILOT | No |
| **System Resources** | | | | |
| Health & Version | `/api/health`, `/api/version` | [002_rest_api_protocol.md](#system-resources) | ✅ Certain | Yes |

**Legend:**
- ✅ **Certain:** Required for Pilot, specification complete
- ✅ **MUST-HAVE:** Critical for production operation, specification complete
- ✅ **NICE-TO-HAVE:** Enhances user experience, specification complete
- 📋 **POST-PILOT:** Future implementation, specification prepared

---

### Authentication Architecture

**Three Authentication Types:**

#### 1. IC Token (Agent Authentication)

**Format:** JWT with `ic_` prefix

**Characteristics:**
- 1:1 relationship with agent (one agent = one IC Token)
- Lifetime: Until agent deleted (long-lived, no auto-expiration)
- Regeneration: Developer can regenerate own, admin can regenerate any
- Used by: iron_runtime for agent operations

**Resources:**
- `POST /api/budget/handshake` - Negotiate budget
- `POST /api/budget/report` - Report usage
- `POST /api/budget/refresh` - Refresh budget

**Header Format:**
```http
Authorization: Bearer ic_abc123def456...
```

#### 2. User Token (Control Panel Access)

**Format:** JWT

**Characteristics:**
- Multiple tokens per user allowed
- Lifetime: 30 days (configurable, refreshable)
- Scope: User + accessible projects
- Used by: iron_cli, web dashboard

**Resources:**
- All entity resources (`/api/tokens`, `/api/projects`, `/api/providers`)
- All analytics resources (`/api/usage`, `/api/spending`, `/api/metrics`)
- All configuration resources (`/api/limits`, `/api/settings`)
- Authentication operations (`POST /api/auth/login`, `POST /api/auth/refresh`)

**Header Format:**
```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### 3. No Authentication (Public Endpoints)

**Resources:**
- `GET /api/health` - Health check
- `GET /api/version` - API version

**No Header Required**

**Authentication Protocol:** See [005_budget_control_protocol.md](005_budget_control_protocol.md) for IC Token handshake details.

---

### Common Patterns

**Pagination (List Endpoints):**

```http
GET /api/tokens?page=1&per_page=50

Response:
{
  "data": [...],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 250,
    "total_pages": 5
  }
}
```

**Filtering:**

```http
GET /api/tokens?project_id=proj_123&status=active
```

**Sorting:**

```http
GET /api/tokens?sort_by=created_at&order=desc
```

**Search:**

```http
GET /api/tokens?search=agent_name
```

**Field Selection (Future):**

```http
GET /api/tokens?fields=id,name,status
```

---

### Example Data Standards

**Use these standard values in all protocol documentation examples for consistency:**

**IDs:**
- Agent ID: `agent_abc123`
- IC Token ID: `ic_def456`
- Project ID: `proj_ghi789`
- Provider ID: `ip_openai_001`, `ip_anthropic_001`
- User ID: `user_jkl012`
- User Token ID: `ut_mno345`
- API Token ID: `at_pqr678`

**Tokens:**
- IC Token: `ic_abc123def456ghi789...`
- User Token: `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`
- API Token: `apitok_abc123def456ghi789...`

**Names:**
- Agent: `Production Agent 1`, `Development Agent 2`
- Project: `My Project`, `Production Project`
- User: `john.doe@example.com`, `admin@example.com`

**Timestamps:**
- Created: `2025-12-01T09:00:00Z`
- Updated: `2025-12-09T14:00:00Z`
- Last Used: `2025-12-09T12:30:00Z`

**Costs:**
- Budget Allocated: `$100.00`
- Budget Spent: `$42.35`
- Budget Remaining: `$57.65`
- Budget Portion: `$10.00`

**Request IDs:**
- Request ID: `req-xyz789`

**Purpose:** Consistent examples help readers recognize patterns across documentation and make cross-references clearer.

---

### System Resources

System resources provide operational endpoints for health checks and API discovery. These endpoints require no authentication and are publicly accessible.

#### Health Check

```http
GET /api/health

Response: 200 OK
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2025-12-09T14:00:00Z",
  "services": {
    "database": "healthy",
    "redis": "healthy",
    "vault": "healthy"
  },
  "uptime_seconds": 86400
}

Response: 503 Service Unavailable
{
  "status": "unhealthy",
  "version": "1.0.0",
  "timestamp": "2025-12-09T14:00:00Z",
  "services": {
    "database": "healthy",
    "redis": "unhealthy",
    "vault": "healthy"
  },
  "errors": [
    {
      "service": "redis",
      "message": "Connection timeout"
    }
  ]
}
```

**Purpose:**
- Load balancer health checks
- Monitoring system integration
- Operational readiness verification

**Response Codes:**
- `200 OK`: All services healthy
- `503 Service Unavailable`: One or more services unhealthy

#### API Version

```http
GET /api/version

Response: 200 OK
{
  "current_version": "v1",
  "supported_versions": ["v1"],
  "deprecated_versions": [],
  "latest_endpoint": "/api/v1",
  "build": {
    "commit": "76dfb54",
    "timestamp": "2025-12-09T12:00:00Z",
    "environment": "production"
  }
}
```

**Purpose:**
- API version discovery
- Client compatibility checks
- Build information for debugging

**Response Codes:**
- `200 OK`: Always succeeds

---

### Error Response Format

**See:** [Error Format Standards](../standards/error_format_standards.md)

All error responses adhere to the standard error format defined in Error Format Standards.

**Standard Structure:**
- **Simple JSON format:** `{"error": {"code": "...", "message": "...", "fields": {...}}}`
- **Machine-readable codes:** UPPER_SNAKE_CASE format (e.g., `VALIDATION_ERROR`, `BUDGET_EXHAUSTED`, `UNAUTHORIZED`)
- **Field-level validation:** `error.fields` object maps field names to specific error messages
- **Batch error reporting:** All validation errors returned at once (not fail-fast)

**HTTP Status Codes:**
- **2xx Success:** 200 (OK), 201 (Created), 204 (No Content)
- **4xx Client Errors:** 400 (Validation), 401 (Auth), 403 (Permissions), 404 (Not Found), 409 (Conflict), 429 (Rate Limit)
- **5xx Server Errors:** 500 (Internal Error), 503 (Service Unavailable)

**Complete Specification:** See [Error Format Standards](../standards/error_format_standards.md) for:
- Detailed error structure and required/optional fields
- Complete HTTP status code mapping with usage guidelines
- Validation error examples with field-level details
- Design rationale (why simple format instead of RFC 7807)
- Error code categories and naming conventions

---

### Rate Limiting

**Standard Rate Limits:**

| Endpoint Category | Limit | Window | Scope |
|-------------------|-------|--------|-------|
| Authentication | 5 attempts | 5 minutes | Per IP address |
| Token Create/Delete | 10 operations | 1 hour | Per user |
| Token Rotate | 5 operations | 1 hour | Per token |
| List/Get Operations | 100 requests | 1 minute | Per user |
| Analytics Queries | 20 requests | 1 minute | Per user |
| Settings Updates | 30 operations | 1 hour | Per user (admin) |

**Rate Limit Response:**

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 300
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1733754300

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded (max 100 req/min)",
    "details": {
      "limit": 100,
      "window_seconds": 60,
      "retry_after_seconds": 300,
      "reset_at": "2025-12-09T14:05:00Z"
    },
    "timestamp": "2025-12-09T14:00:00Z",
    "request_id": "req_xyz789"
  }
}
```

**Headers:**
- `X-RateLimit-Limit`: Maximum requests in window
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Reset`: Unix timestamp when limit resets

**Implementation:**
- Token bucket algorithm with distributed tracking (Redis)
- Per-user tracking for authenticated endpoints
- Per-IP tracking for public endpoints (health, version, auth)

---

### Budget Protocol Summary

Budget protocol endpoints enable agent runtime to negotiate and report LLM usage. These are agent_facing endpoints (not exposed via CLI).

**Endpoints:**

| Endpoint | Purpose | Request | Response |
|----------|---------|---------|----------|
| `POST /api/v1/budget/handshake` | Negotiate budget and get IP Token | `{requested_budget: float}` | `{ip_token, budget_granted, lease_id}` |
| `POST /api/v1/budget/report` | Report usage (tokens, cost) | `{lease_id, tokens, cost_usd}` | 204 No Content |
| `POST /api/v1/budget/refresh` | Refresh budget during execution | `{lease_id, requested_budget}` | `{budget_granted, lease_id}` |

**Authentication:** IC Token (agent authentication)

**Complete Specification:** See [protocol/005: Budget Control Protocol](005_budget_control_protocol.md) for:
- Full request/response schemas
- Error responses and retry logic
- Implementation variants (per-request vs batched reporting)
- Sequence diagrams and state transitions
- Security considerations

---

### HTTP Status Codes

**Success Codes:**

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful GET, POST (with response body), PUT |
| 201 | Created | Successful POST (resource created) |
| 204 | No Content | Successful DELETE, POST (no response body) |

**Client Error Codes:**

| Code | Meaning | Usage |
|------|---------|-------|
| 400 | Bad Request | Invalid request body, malformed JSON |
| 401 | Unauthorized | Missing or invalid authentication token |
| 403 | Forbidden | Valid token but insufficient permissions, budget exhausted |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Resource already exists, state conflict |
| 422 | Unprocessable Entity | Validation errors |
| 429 | Too Many Requests | Rate limit exceeded |

**Server Error Codes:**

| Code | Meaning | Usage |
|------|---------|-------|
| 500 | Internal Server Error | Unexpected server error |
| 503 | Service Unavailable | Service temporarily unavailable |

**Example Response Bodies:**

```json
// 200 OK (GET /api/tokens/tok-123)
{
  "id": "tok-123",
  "agent_id": "agent_abc",
  "status": "active",
  "created_at": "2025-12-09T09:00:00Z"
}

// 201 Created (POST /api/tokens)
{
  "id": "tok-456",
  "token": "ic_abc123...",
  "agent_id": "agent_xyz",
  "created_at": "2025-12-09T10:00:00Z"
}

// 204 No Content (DELETE /api/tokens/tok-123)
(empty response body)

// 400 Bad Request
{
  "error": {
    "code": "VALIDATION_REQUIRED_FIELD",
    "message": "Required field 'agent_id' missing",
    "details": {"field": "agent_id"}
  }
}

// 401 Unauthorized
{
  "error": {
    "code": "AUTH_INVALID_TOKEN",
    "message": "Invalid or expired authentication token"
  }
}

// 403 Forbidden (Budget)
{
  "error": {
    "code": "BUDGET_EXHAUSTED",
    "message": "Agent budget exhausted",
    "details": {
      "agent_id": "agent_abc",
      "budget_allocated": 100.00,
      "budget_remaining": 0.00
    }
  }
}

// 404 Not Found
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "IC Token not found",
    "details": {"token_id": "tok-nonexistent"}
  }
}

// 409 Conflict
{
  "error": {
    "code": "RESOURCE_CONFLICT",
    "message": "IC Token already exists for agent",
    "details": {"agent_id": "agent_abc", "existing_token_id": "tok-123"}
  }
}

// 429 Too Many Requests
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded (max 100 req/min)",
    "details": {
      "limit": 100,
      "window": "60s",
      "retry_after": 30
    }
  }
}
```

---

### API Versioning

**Current Version:** v1

**Versioning Strategy:**
- URL-based versioning: `/api/v1/`, `/api/v2/`
- Version in URL path (not header)
- Each version independently documented

**Breaking Changes (Require New Version):**
- Removing endpoints or fields
- Changing field types
- Changing endpoint semantics
- Removing query parameters

**Non-Breaking Changes (Same Version):**
- Adding new endpoints
- Adding optional fields
- Adding optional query parameters
- Expanding enum values (with graceful degradation)

**Version Lifecycle:**
- Latest version: Always available
- Previous version: Supported for 6 months after new version release
- Deprecated version: 3-month sunset notice before removal

**Version Discovery:**
```http
GET /api/version

Response:
{
  "current_version": "v1",
  "supported_versions": ["v1"],
  "deprecated_versions": [],
  "latest_endpoint": "/api/v1"
}
```

---

### CLI-API Parity

**Principle:** Every user-facing API resource has corresponding CLI command.

**Exceptions:**
- Agent-facing resources (budget protocol) - No CLI (used by iron_runtime)
- System resources (health, version) - No CLI (operational endpoints)

**Mapping Pattern:**

| API Pattern | HTTP Method | CLI Pattern | Example |
|-------------|-------------|-------------|---------|
| `GET /api/{resource}` | GET (list) | `iron .{resource}.list` | `GET /api/tokens` → `iron .token.list` |
| `GET /api/{resource}/{id}` | GET (get) | `iron .{resource}.get id::<id>` | `GET /api/tokens/tok-123` → `iron .token.get id::tok-123` |
| `POST /api/{resource}` | POST | `iron .{resource}.create` | `POST /api/tokens` → `iron .token.create` |
| `PUT /api/{resource}/{id}` | PUT | `iron .{resource}.update id::<id>` | `PUT /api/tokens/tok-123` → `iron .token.update id::tok-123` |
| `DELETE /api/{resource}/{id}` | DELETE | `iron .{resource}.delete id::<id>` | `DELETE /api/tokens/tok-123` → `iron .token.delete id::tok-123` |
| `POST /api/{resource}/{id}/{action}` | POST | `iron .{resource}.{action} id::<id>` | `POST /api/tokens/tok-123/rotate` → `iron .token.rotate id::tok-123` |
| `POST /api/{operation}` | POST | `iron .{operation}` | `POST /api/auth/login` → `iron .auth.login` |

**Entity Resources:**

| API Endpoint | CLI Command | Auth | Status |
|--------------|-------------|------|--------|
| `GET /api/tokens` | `iron .token.list` | User Token | ✅ Certain |
| `POST /api/tokens` | `iron .token.create` | User Token | ✅ Certain |
| `DELETE /api/tokens/{id}` | `iron .token.delete id::<id>` | User Token | ✅ Certain |
| `PUT /api/tokens/{id}/rotate` | `iron .token.rotate id::<id>` | User Token | ✅ Certain |
| `GET /api/api-tokens` | `iron .api_token.list` | User Token | ⚠️ Uncertain |
| `POST /api/api-tokens` | `iron .api_token.create` | User Token | ⚠️ Uncertain |
| `DELETE /api/api-tokens/{id}` | `iron .api_token.revoke id::<id>` | User Token | ⚠️ Uncertain |
| `GET /api/projects` | `iron .project.list` | User Token | ⚠️ Uncertain |
| `GET /api/providers` | `iron .provider.list` | User Token | ⚠️ Uncertain |

**Operation Resources:**

| API Endpoint | CLI Command | Auth | Status |
|--------------|-------------|------|--------|
| `POST /api/auth/login` | `iron .auth.login` | None → User Token | ✅ Certain |
| `POST /api/auth/logout` | `iron .auth.logout` | User Token | ✅ Certain |
| `POST /api/budget/handshake` | (no CLI) | IC Token | ✅ Certain |
| `POST /api/budget/report` | (no CLI) | IC Token | ✅ Certain |
| `POST /api/budget/refresh` | (no CLI) | IC Token | ✅ Certain |

**Analytics Resources:**

| API Endpoint | CLI Command | Auth | Status |
|--------------|-------------|------|--------|
| `GET /api/analytics/usage` | `iron .analytics.usage` | User Token | ⚠️ Uncertain |
| `GET /api/analytics/spending` | `iron .analytics.spending` | User Token | ⚠️ Uncertain |
| `GET /api/analytics/metrics` | `iron .analytics.metrics` | User Token | ⚠️ Uncertain |

**Parity Details:** See [features/004_token_management_cli_api_parity.md](../features/004_token_management_cli_api_parity.md) for complete 24-operation mapping.

---

### Version Terminology

**API Version (`/api/v1/`):**
- URL path segment indicating API iteration
- Breaking changes require new API version (e.g., `/api/v2/`)
- Current API version: **v1**
- Appears in all endpoint URLs

**Document Version (1.0, 1.1, 2.0):**
- Protocol documentation iteration
- Major version: Breaking documentation changes or restructuring
- Minor version: Clarifications, additions, non-breaking updates
- Current document version: **1.0**

**Independence:** API version and document version evolve independently. API v1 docs may be at document version 1.2 after clarifications and additions.

---

### Cross-References

**Resource Organization:**
- [architecture/009: Resource Catalog](../architecture/009_resource_catalog.md) - Complete resource inventory and entity mapping

**Resource-Specific Protocols:**
- [006: Token Management API](006_token_management_api.md) - IC Token CRUD endpoints (✅ Certain, Pilot)
- [007: Authentication API](007_authentication_api.md) - User login/logout endpoints (✅ Certain, Pilot)
- [008: User Management API](008_user_management_api.md) - User management endpoints (✅ Certain, Pilot)
- [010: Agents API](010_agents_api.md) - Agent management endpoints (✅ MUST-HAVE)
- [011: Providers API](011_providers_api.md) - Provider management endpoints (✅ MUST-HAVE)
- [012: Analytics API](012_analytics_api.md) - Usage and spending analytics endpoints (✅ MUST-HAVE)
- [013: Budget Limits API](013_budget_limits_api.md) - Budget modification endpoints, admin-only (✅ MUST-HAVE)
- [014: API Tokens API](014_api_tokens_api.md) - API token management endpoints (✅ MUST-HAVE)
- [015: Projects API](015_projects_api.md) - Project access endpoints (✅ NICE-TO-HAVE)
- [016: Settings API](016_settings_api.md) - Settings management endpoints (📋 POST-PILOT)
- [017: Budget Requests API](017_budget_requests_api.md) - Budget change request/approval workflow endpoints (✅ MUST-HAVE)

**Dependencies:**
- [protocol/005: Budget Control Protocol](005_budget_control_protocol.md) - Budget handshake/report/refresh protocol
- [architecture/007: Entity Model](../architecture/007_entity_model.md) - Domain entities (Agent, IC Token, Project, IP)
- [architecture/006: Roles and Permissions](../architecture/006_roles_and_permissions.md) - Admin, Super User, Developer roles

**Used By:**
- [capabilities/002: LLM Access Control](../capabilities/002_llm_access_control.md) - Uses budget API for enforcement
- [architecture/004: Data Flow](../architecture/004_data_flow.md) - REST API in runtime initialization
- `iron_runtime` - Calls budget protocol endpoints (handshake/report/refresh)
- `iron_cli` - Calls user-facing endpoints (tokens, auth, usage)
- `iron_dashboard` - Calls user-facing endpoints (tokens, analytics, config)

**Related:**
- [003: WebSocket Protocol](003_websocket_protocol.md) - Real-time dashboard protocol
- [features/004: Token Management CLI-API Parity](../features/004_token_management_cli_api_parity.md) - Complete CLI ↔ API mapping

**Implementation:**
- Module: `module/iron_control_api/` - REST API server (Rust/axum)
- Source: `module/iron_control_api/src/routes/` - Endpoint handlers
- Tests: `module/iron_control_api/tests/` - Endpoint integration tests
- Specification: `module/iron_control_api/spec.md` - Implementation spec

---
