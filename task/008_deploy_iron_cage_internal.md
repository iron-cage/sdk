# Task 008: Deploy Iron Cage as Internal Centralized Token Control Platform

## Goal

Deploy Iron Cage as a centralized internal platform where a server-side proxy holds and injects provider keys without exposing them to agents, enabling secure team-wide LLM access governance. The result is observable through agents connecting to a server-side proxy using only IC tokens to make inference requests, with no provider credentials on the client side. Scoped to a shared LLM routing core library, a new server-side proxy binary, `iron_runtime` refactoring, and internal network deployment. Testable by an end-to-end smoke test where an agent sends a chat request via IC token and receives a provider response without ever seeing the API key.

## Dependencies
- None blocking (tasks 001–004 strengthen security and feature completeness; deployment can proceed in parallel)

## In Scope

- Shared LLM routing core library crate extracting common forwarding logic from `iron_runtime`
- New server-side proxy binary crate with IC-token authentication and key injection
- Refactoring `iron_runtime` to use the shared core (thin local wrapper)
- Internal network deployment with HTTPS, process management, and database initialization
- RBAC enforcement at the API layer (Admin, SuperUser, Developer)
- Provider API key encryption at rest and no secret leakage in logs or responses

## Out of Scope

- Public-facing deployment or internet-accessible endpoints
- Multi-region or high-availability deployment architecture
- Automated scaling or load balancing
- Client SDK or agent-side library changes beyond connection configuration

## Description

Iron Cage already implements the core model for LLM access governance: IC tokens for agent authentication, IP keys for provider access, budget enforcement, and role-based access control. However, the current architecture has a fundamental gap - `iron_runtime` is a local proxy that runs on the agent's machine and fetches real provider keys from the control API on demand. This means provider credentials are exposed to the client side, which undermines centralized security governance.

This task introduces a server-side proxy that sits between agents and LLM providers. Agents connect using only their IC token - no provider credentials are needed or exposed. The proxy authenticates the IC token, resolves the assigned IP key from storage, injects it into the forwarded request, and returns the provider response. The key material never leaves the server.

To avoid code duplication, the common LLM routing logic (request/response translation, provider detection, cost extraction) is extracted into a shared library crate. Both the server-side proxy and the refactored `iron_runtime` depend on this shared core. The deployment targets the internal team network with HTTPS, process management (systemd or equivalent), and complete database initialization.

The security baseline includes AES-256-GCM encryption for provider keys at rest, hash-only storage for IC tokens, no secrets in logs or API responses, and mandatory authentication on all endpoints except health checks.

## Context
The team needs a centralized, secure platform to govern LLM inference access across all internal agents and projects. Iron Cage already implements the core model — IC tokens for agent authentication, IP keys for provider access, budget enforcement, RBAC — but no centralized server-side proxy exists yet.

**The architectural gap:** `iron_runtime` is a *local* proxy — it runs embedded on the agent's machine, binds `127.0.0.1:PORT`, and fetches real provider keys from the control API on demand. `iron_control_api` is the control plane (key/token CRUD, budgets, analytics); no LLM traffic passes through it. For a centralized deployment, agents must connect to a *server-side* proxy that holds and injects IP keys without ever exposing them to the client.

**Required new architecture:**

Three components are needed:

1. **Shared LLM routing core** (new library crate) — Extract the common forwarding logic currently embedded in `iron_runtime::llm_router`: request/response format translation, provider detection, cost extraction, PII safety integration. Both local and server execution will depend on this, eliminating duplication.

2. **Server-side proxy** (new binary crate) — Accepts remote agent HTTP connections authenticated by IC token, resolves the assigned IP key from storage, and forwards to the actual LLM provider. No key material is sent to the agent. Depends on the shared core and on `iron_token_manager` for auth and key resolution.

3. **`iron_runtime` (refactored)** — Becomes a thin wrapper over the shared core for local/embedded execution. Local-specific concerns remain here: dynamic port binding, key fetching from the server, budget handshake protocol, analytics sync.

**Deployment scope:** Internal team network only (not public-facing).

Existing components used unchanged:
- `module/iron_control_api` — Administration REST API (IP key CRUD, IC token management, RBAC, analytics ingestion)
- `module/iron_token_manager` — Token lifecycle, RBAC enforcement, budget tracking, IP key storage

## Work Procedure

1. Extract shared LLM routing core from `iron_runtime::llm_router` into a new library crate
2. Implement the server-side proxy binary crate with IC-token authentication middleware
3. Implement IP key resolution and injection in the proxy - key material never leaves server
4. Refactor `iron_runtime` to depend on the shared core as a thin local wrapper
5. Configure TLS termination and HTTPS for all traffic
6. Set up process management (systemd units) for control API and server-side proxy
7. Initialize database with all migrations applied
8. Create initial Admin account and store credentials securely
9. Run end-to-end smoke test: agent request via IC token through proxy to provider and back
10. Write operational runbook covering startup, shutdown, and key/token rotation procedures

## Implementation plan

1. Create shared LLM routing core library crate with request/response translation, provider detection, and cost extraction.
2. Build server-side proxy binary crate that accepts IC-token-authenticated connections and resolves IP keys.
3. Implement transparent key injection - proxy adds provider API key to forwarded requests.
4. Refactor `iron_runtime` to use shared core, retaining local-specific concerns (port binding, key fetching, analytics sync).
5. Configure deployment infrastructure: HTTPS/TLS, systemd service units, database initialization.
6. Implement RBAC enforcement at the API layer for Admin, SuperUser, and Developer roles.
7. Ensure provider API key encryption at rest (AES-256-GCM) and no secret leakage in logs or responses.
8. Verify `iron_runtime` regression tests pass after refactoring.
9. Run end-to-end smoke test and write operational runbook.

## Test Matrix

| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Agent sends chat request via IC token to server proxy | Proxy resolves IP key and forwards to provider | Response returned to agent successfully |
| Agent uses revoked IC token | Request rejected immediately | 401 returned, no provider call made |
| Admin registers new IP key via control API | Key created and available for assignment | API returns success, key usable in handshake |
| IP key spending cap exceeded | Proxy rejects request | Spending cap error returned |
| Server restart | Services resume automatically | systemd restarts both services |
| HTTPS connection to proxy | TLS handshake succeeds | Certificate validated, encrypted traffic |
| Admin assigns role to user | Role assignment succeeds | Role boundaries enforced on subsequent requests |
| Non-admin attempts role assignment | Request rejected | Authorization error returned |
| Agent request with no provider credentials | Proxy injects key transparently | Provider receives valid API key |
| Health check endpoint without auth | Response returned | 200 OK without authentication |
| `iron_runtime` local mode after refactor | Local proxy works as before | All existing tests pass |
| Provider key not in logs or responses | No secret leakage | Log and response inspection clean |

## Validation Checklist

- [ ] Control API deployed and reachable from team machines
- [ ] Server-side proxy deployed and reachable from team machines
- [ ] Both services survive server restart via process manager
- [ ] All traffic is HTTPS with TLS termination
- [ ] Database initialized with all migrations applied
- [ ] Admin can register, list, rotate, and revoke IP keys
- [ ] IP key assignable to multiple IC tokens
- [ ] Per-IP-key spending cap enforced by server proxy
- [ ] Agents never receive raw provider API key
- [ ] Admin can generate and distribute IC tokens
- [ ] IC tokens carry independent spending limits
- [ ] IC token revocation takes effect immediately
- [ ] RBAC roles enforced at API layer (Admin, SuperUser, Developer)
- [ ] Provider keys encrypted at rest (AES-256-GCM)
- [ ] No secrets in logs, error responses, or list endpoints
- [ ] IC token stored as hash, raw surfaced only at generation
- [ ] Shared core crate used by both proxy and `iron_runtime`
- [ ] `iron_runtime` tests pass after refactoring
- [ ] End-to-end smoke test passing
- [ ] Operational runbook written

## Validation Procedure

1. Verify control API is accessible from team machines via HTTPS
2. Verify server-side proxy is accessible from team machines via HTTPS
3. Restart the server and verify both services come back automatically
4. Register an IP key for OpenAI and an IP key for Anthropic via control API
5. Generate an IC token and assign it spending limits and a provider-access allowlist
6. From a team machine, send a chat completion request to the server proxy using only the IC token
7. Verify the response is returned successfully and the agent never saw the provider API key
8. Revoke the IC token and send another request - verify immediate rejection with 401
9. Inspect server logs to confirm no raw keys or tokens appear
10. Verify RBAC: attempt role assignment as non-admin - confirm rejection
11. Run `iron_runtime` test suite to confirm no regressions in local mode
12. Review operational runbook for completeness (startup, shutdown, rotation procedures)

## Acceptance Criteria

### 1. Running Internal Service
- The control API and the server-side proxy binary are deployed on the internal server and reachable from team machines
- Both services survive server restart (process manager: systemd unit or equivalent)
- All traffic is HTTPS (TLS terminated at server)
- Database initialized with all migrations applied

### 2. IP Key Sharing — Centralized Provider Access
- Admin can register, list, rotate, and revoke IP keys for at least OpenAI and Anthropic via the control API
- An IP key can be assigned to multiple IC tokens (shared access under policy)
- Per-IP-key spending cap configurable; server proxy rejects requests once the cap is reached
- Agents never receive or handle the raw provider API key — the server proxy injects it transparently on every forwarded request

### 3. IC Token Distribution — Agent Authentication
- Admin can generate IC tokens and distribute them to agents or team members
- Each IC token carries independent spending limits and a provider-access allowlist
- Agents authenticate to the server proxy using only their IC token (no provider credentials on client)
- Revoking an IC token takes effect immediately; subsequent requests from that token are rejected

### 4. RBAC — Role-Based Access Enforcement
- Three roles active: Admin (full control), SuperUser (key/token management, no role assignment), Developer (read-only + own-token inspection)
- Role boundaries enforced at the API layer, not just presentation
- Role assignment restricted to Admin role

### 5. Security Baseline
- Provider API keys encrypted at rest (AES-256-GCM or equivalent)
- No secret values (raw keys, plaintext tokens) appear in logs, error responses, or API list endpoints
- IC token stored as hash; raw token surfaced exactly once at generation time
- All endpoints require a valid IC token or Admin session except the health-check route

### 6. Shared Core — No Duplication
- LLM request/response translation, provider detection, and cost extraction logic live in a single shared crate used by both `iron_runtime` (local) and the new server proxy
- `iron_runtime` tests continue to pass after refactoring (no regression in local mode)

### 7. Operational Readiness
- Initial Admin account created; credentials stored in `.persistent/internal_deployment.md`
- End-to-end smoke test passing: agent sends a chat request to the server proxy using an IC token → proxy resolves IP key → request forwarded to LLM provider → response returned to agent
- Operational runbook written at `docs/deployment.md` covering: startup, shutdown, IC token rotation, IP key rotation
