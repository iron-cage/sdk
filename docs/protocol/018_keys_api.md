# Protocol: Keys API

### Scope

This protocol documents the `/api/keys` endpoint which provides authenticated users with decrypted AI provider API keys. This is a **user-only endpoint** - agent tokens are explicitly blocked to enforce Protocol 005 (Budget Control).

#### In Scope

- GET /api/keys endpoint specification
- API token authentication (user tokens only)
- Provider key decryption
- Rate limiting (10 requests/minute per user)
- Security enforcement (agent token blocking)
- Error responses

#### Out of Scope

- Agent token access (use Protocol 005 budget handshake instead)
- Provider key storage/management (separate admin API)
- Key rotation workflows
- Multi-project key access (one project per token)

### Purpose

**User Need:** Users need ability to retrieve decrypted AI provider API keys for direct LLM access outside the budget control framework. This supports legitimate use cases including local testing, debugging, dashboard model availability checks, and non-agent workflows where budget enforcement overhead is unnecessary. Users must be able to fetch keys securely with proper authentication and rate limiting to prevent abuse.

**Solution:** Protocol 018 provides single GET /api/v1/keys endpoint that returns decrypted provider credentials (provider type, API key, base URL) for the project associated with the user's authentication token. Agent tokens are explicitly blocked with 403 Forbidden to enforce budget control path (Protocol 005), while user tokens receive rate-limited access (10 requests/minute per user/project). The endpoint performs token validation, project assignment verification, provider key decryption, and returns plaintext credentials over HTTPS.

**Key Insight:** The Keys API creates intentional separation between user workflows (direct provider access) and agent workflows (budget-controlled access). By blocking agent tokens at the authentication layer, the system enforces architectural boundary: agents MUST use Protocol 005 budget handshake (IC Token flow) which provides budget limits, usage tracking, and cost monitoring, while users CAN bypass budget control for testing and debugging. This separation prevents budget bypass attacks while enabling developer productivity. The rate limiting (10 req/min) prevents key harvesting without blocking legitimate usage.

---

**Status:** Certain (Required for user workflows)
**Version:** 1.0.0
**Last Updated:** 2025-12-14
**Priority:** MUST-HAVE
**Implementation:** ✅ Complete

### Standards Compliance

This protocol uses the following ID formats:

- `token_id`: `at_<alphanumeric>` (e.g., `at_xyz789`)
  - Pattern: `^at_[a-z0-9]{6,32}$`
  - Source: Protocol 006 (Token Management API)
  - Usage: API token identifier for authentication
  - Note: Token value format is `apitok_<base62_64chars>` (authentication credential, not entity ID)

- `agent_id`: `agent_<alphanumeric>` (e.g., `agent_abc123`)
  - Pattern: `^agent_[a-z0-9]{6,32}$`
  - Source: Protocol 010 (Agents API)
  - Usage: Agent identifier for token association check (blocked if present)

- `project_id`: `proj_<alphanumeric>` (e.g., `proj_master_001`)
  - Pattern: `^proj_[a-z0-9_]{3,32}$`
  - Source: Protocol 015 (Projects API)
  - Usage: Project identifier for key assignment scope

**Data Format Standards:**
- Timestamps: ISO 8601 with Z suffix (not used in Keys API responses)
- Provider types: Enum string values ("openai", "anthropic")
- API keys: Base64/base62 encoded strings (provider-specific format)
- URLs: Valid HTTPS endpoints with optional custom base URL

**Error Format Standards:**
- Simple error response structure with `error` field (string or object)
- HTTP status codes: 200, 400, 401, 403, 404, 429, 500

**Security Standards:**
- Rate limiting: 10 requests/minute per user/project combination
- Agent token blocking: 403 Forbidden with instructional error message
- Encryption: AES-GCM for provider key storage, decryption on request
- Transmission: Plaintext over HTTPS only (no additional encryption layer)

### Endpoints

#### GET /api/v1/keys

Fetch the decrypted AI provider key assigned to the token's project.

##### Authentication

Requires API token authentication via `Authorization: Bearer <token>` header.

**Critical:** Agent tokens are **explicitly blocked** with 403 Forbidden. Only user tokens (no `agent_id` association) can use this endpoint.

##### Request

```http
GET /api/v1/keys HTTP/1.1
Host: localhost:8084
Authorization: Bearer apitok_...
```

##### Success Response

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "provider": "openai",
  "api_key": "sk-...",
  "base_url": "https://api.openai.com/v1"
}
```

##### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `provider` | string | Provider type ("openai" or "anthropic") |
| `api_key` | string | Decrypted full API key (not masked) |
| `base_url` | string | Custom base URL if configured (optional) |

##### Error Responses

| Status | Condition | Response Body |
|--------|-----------|---------------|
| 400 Bad Request | Token not assigned to a project | `{"error": "Token not assigned to project"}` |
| 401 Unauthorized | Invalid or missing token | `{"error": "Invalid or missing token"}` |
| 403 Forbidden | Agent token attempted access | `{"error": "Agent tokens cannot use this endpoint", "details": "Agent credentials must be obtained through Protocol 005 (Budget Control). Use POST /api/budget/handshake with your IC Token.", "protocol": "005"}` |
| 404 Not Found | No provider key assigned to project | `{"error": "No provider key assigned to project"}` |
| 429 Too Many Requests | Rate limit exceeded (10 req/min) | `{"error": "Rate limit exceeded"}` |
| 500 Internal Server Error | Decryption failed | `{"error": "Decryption failed"}` |

##### Authorization

- Any authenticated user with user token (agent tokens blocked)
- Token must be assigned to a project
- Project must have provider key configured

##### Audit Log

Yes - User ID, timestamp, project ID, provider type (key value NOT logged)

### Security

#### Agent Token Blocking

**Enforcement:** All agent tokens (tokens with `agent_id` association) receive 403 Forbidden.

**Rationale:** Agent LLM access MUST use Protocol 005 (Budget Control) to enforce:
- Budget limits
- Usage tracking
- Cost monitoring
- Safety controls

The `/api/keys` endpoint provides direct access to decrypted credentials without budget control, creating a bypass path if agent tokens were permitted.

**Implementation:** Database query checks `api_tokens.agent_id IS NOT NULL` and rejects before key retrieval.

#### Rate Limiting

- **Limit:** 10 requests per minute per user/project combination
- **Enforcement:** In-memory sliding window rate limiter
- **Response:** 429 Too Many Requests when exceeded
- **Rationale:** Prevents key harvesting attacks while allowing legitimate testing and debugging

#### Token Authentication

- **Header Format:** `Authorization: Bearer <token>`
- **Token Verification:** SHA-256 hash lookup in `api_tokens` table
- **Validation:** Token must be active (`is_active = 1`), not expired, not revoked

#### Key Decryption

- **Storage:** Provider keys stored encrypted (AES-GCM) in `provider_keys` table
- **Decryption:** CryptoService with master key (environment variable)
- **Transmission:** Decrypted key returned in HTTPS response (plaintext in JSON)
- **Warning:** Decrypted keys must be protected by clients (no logging, memory clearing)

### Use Cases

#### Use Case 1: Developer Testing

**Scenario:** Developer needs OpenAI API key for local testing

**Flow:**
1. Developer creates user token: `POST /api/v1/api-tokens` (with project assignment)
2. Admin assigns OpenAI key to project
3. Developer fetches key: `GET /api/v1/keys` → receives `sk-...`
4. Developer uses key in local test script

**Security:** Rate limiting prevents key harvesting, token authentication ensures authorization

#### Use Case 2: Dashboard Direct Access

**Scenario:** Dashboard needs to display model availability for user's project

**Flow:**
1. Dashboard authenticates with user token
2. Dashboard fetches provider key: `GET /api/v1/keys`
3. Dashboard tests provider availability: `curl https://api.openai.com/v1/models -H "Authorization: Bearer $KEY"`
4. Dashboard displays available models to user

**Security:** User token (not agent token) ensures no budget bypass

### Differences from Protocol 005

| Aspect | Protocol 018 (Keys API) | Protocol 005 (Budget Control) |
|--------|------------------------|-------------------------------|
| **Authentication** | User token (Bearer header) | IC Token (two-token handshake) |
| **Audience** | Users, developers, dashboard | Agents only |
| **Budget Enforcement** | None (direct key access) | Mandatory (pre-flight check) |
| **Rate Limiting** | 10 req/min per user | Per-request budget deduction |
| **Usage Tracking** | No cost tracking | Full usage logging |
| **Agent Access** | Blocked (403 Forbidden) | Required path |

### Cross-References

#### Related Principles Documents

None

#### Related Architecture Documents

None

#### Used By

- Dashboard applications (fetch provider keys for model availability checks)
- Developer CLI tools (retrieve keys for local testing)
- Testing frameworks (access provider APIs without budget control)

#### Dependencies

- Protocol 002: REST API Protocol (General REST standards)
- Protocol 005: Budget Control Protocol (Agent credential access, IC Token flow)
- Protocol 006: Token Management API (API token creation, user tokens vs agent tokens)
- Protocol 015: Projects API (Project-level key assignment scope)

#### Implementation

**Status:** ✅ Implemented (Complete)

**Files:**
- `module/iron_control_api/src/routes/keys.rs` - Endpoint implementation (line 644 registration)
- `module/iron_control_api/tests/keys/endpoints.rs` - Integration tests
- `module/iron_control_api/tests/keys/security.rs` - Security tests (agent token blocking)
- `module/iron_control_api/tests/keys/full_flow.rs` - End-to-end tests

**Route Registration:** `/api/v1/keys` registered in `iron_control_api_server.rs:644`

**Database Tables:**
- `api_tokens` - Token validation and agent_id check
- `provider_keys` - Encrypted provider credentials storage
- `projects` - Project-token assignment verification
