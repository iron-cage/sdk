# Protocol 018: Keys API

Define RESTful API for AI provider key retrieval enabling users to fetch decrypted provider credentials for direct LLM access.

## Scope

This protocol documents the `/api/keys` endpoint which provides authenticated users with decrypted AI provider API keys. This is a **user-only endpoint** - agent tokens are explicitly blocked to enforce Protocol 005 (Budget Control).

### In Scope

- GET /api/keys endpoint specification
- API token authentication (user tokens only)
- Provider key decryption
- Rate limiting (10 requests/minute per user)
- Security enforcement (agent token blocking)
- Error responses

### Out of Scope

- Agent token access (use Protocol 005 budget handshake instead)
- Provider key storage/management (separate admin API)
- Key rotation workflows
- Multi-project key access (one project per token)

## Purpose

Enable users to retrieve decrypted AI provider keys for direct API access without budget control overhead. This supports use cases where users need direct provider access (testing, debugging, non-agent workflows) while maintaining security through authentication and rate limiting.

## Endpoints

### GET /api/v1/keys

Fetch the decrypted AI provider key assigned to the token's project.

#### Authentication

Requires API token authentication via `Authorization: Bearer <token>` header.

**Critical:** Agent tokens are **explicitly blocked** with 403 Forbidden. Only user tokens (no `agent_id` association) can use this endpoint.

#### Request

```http
GET /api/v1/keys HTTP/1.1
Host: localhost:8084
Authorization: Bearer apitok_...
```

#### Response (Success)

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "provider": "openai",
  "api_key": "sk-...",
  "base_url": "https://api.openai.com/v1"
}
```

**Fields:**
- `provider` (string, required): Provider type ("openai" or "anthropic")
- `api_key` (string, required): Decrypted full API key (not masked)
- `base_url` (string, optional): Custom base URL if configured

#### Error Responses

| Status | Condition | Response Body |
|--------|-----------|---------------|
| 400 Bad Request | Token not assigned to a project | `{"error": "Token not assigned to project"}` |
| 401 Unauthorized | Invalid or missing token | `{"error": "Invalid or missing token"}` |
| 403 Forbidden | Agent token attempted access | `{"error": "Agent tokens cannot use this endpoint", "details": "Agent credentials must be obtained through Protocol 005 (Budget Control). Use POST /api/budget/handshake with your IC Token.", "protocol": "005"}` |
| 404 Not Found | No provider key assigned to project | `{"error": "No provider key assigned to project"}` |
| 429 Too Many Requests | Rate limit exceeded (10 req/min) | `{"error": "Rate limit exceeded"}` |
| 500 Internal Server Error | Decryption failed | `{"error": "Decryption failed"}` |

## Security

### Agent Token Blocking

**Enforcement:** All agent tokens (tokens with `agent_id` association) receive 403 Forbidden.

**Rationale:** Agent LLM access MUST use Protocol 005 (Budget Control) to enforce:
- Budget limits
- Usage tracking
- Cost monitoring
- Safety controls

The `/api/keys` endpoint provides direct access to decrypted credentials without budget control, creating a bypass path if agent tokens were permitted.

**Implementation:** Database query checks `api_tokens.agent_id IS NOT NULL` and rejects before key retrieval.

### Rate Limiting

- **Limit:** 10 requests per minute per user/project combination
- **Enforcement:** In-memory sliding window rate limiter
- **Response:** 429 Too Many Requests when exceeded

### Token Authentication

- **Header Format:** `Authorization: Bearer <token>`
- **Token Verification:** SHA-256 hash lookup in `api_tokens` table
- **Validation:** Token must be active (`is_active = 1`), not expired, not revoked

### Key Decryption

- **Storage:** Provider keys stored encrypted (AES-GCM) in `provider_keys` table
- **Decryption:** CryptoService with master key (environment variable)
- **Transmission:** Decrypted key returned in HTTPS response (plaintext in JSON)
- **Warning:** Decrypted keys must be protected by clients (no logging, memory clearing)

## Use Cases

### Use Case 1: Developer Testing

**Scenario:** Developer needs OpenAI API key for local testing

**Flow:**
1. Developer creates user token: `POST /api/v1/api-tokens` (with project assignment)
2. Admin assigns OpenAI key to project
3. Developer fetches key: `GET /api/v1/keys` → receives `sk-...`
4. Developer uses key in local test script

**Security:** Rate limiting prevents key harvesting, token authentication ensures authorization

### Use Case 2: Dashboard Direct Access

**Scenario:** Dashboard needs to display model availability for user's project

**Flow:**
1. Dashboard authenticates with user token
2. Dashboard fetches provider key: `GET /api/v1/keys`
3. Dashboard tests provider availability: `curl https://api.openai.com/v1/models -H "Authorization: Bearer $KEY"`
4. Dashboard displays available models to user

**Security:** User token (not agent token) ensures no budget bypass

## Implementation Status

**Status:** ✅ Implemented

**Files:**
- `module/iron_control_api/src/routes/keys.rs` - Endpoint implementation
- `module/iron_control_api/tests/keys/endpoints.rs` - Integration tests
- `module/iron_control_api/tests/keys/security.rs` - Security tests
- `module/iron_control_api/tests/keys/full_flow.rs` - End-to-end tests

**Route Registration:** `/api/v1/keys` registered in `iron_control_api_server.rs:644`

## Related Protocols

- **Protocol 005:** [Budget Control Protocol](005_budget_control_protocol.md) - Agent credential access (IC Token flow)
- **Protocol 006:** [Token Management API](006_token_management_api.md) - API token creation
- **Protocol 002:** [REST API Protocol](002_rest_api_protocol.md) - General REST standards

## Differences from Protocol 005

| Aspect | Protocol 018 (Keys API) | Protocol 005 (Budget Control) |
|--------|------------------------|-------------------------------|
| **Authentication** | User token (Bearer header) | IC Token (two-token handshake) |
| **Audience** | Users, developers, dashboard | Agents only |
| **Budget Enforcement** | None (direct key access) | Mandatory (pre-flight check) |
| **Rate Limiting** | 10 req/min per user | Per-request budget deduction |
| **Usage Tracking** | No cost tracking | Full usage logging |
| **Agent Access** | Blocked (403 Forbidden) | Required path |

---

**Last Updated:** 2025-12-14
**Status:** Certain (Required for user workflows)
**Implementation:** ✅ Complete
