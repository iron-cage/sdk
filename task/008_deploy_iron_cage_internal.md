# Task 008: Deploy Iron Cage as Internal Centralized Token Control Platform

## Dependencies
- None blocking (tasks 001–004 strengthen security and feature completeness; deployment can proceed in parallel)

## Context
The team needs a centralized, secure platform to govern LLM inference access across all internal agents and projects. Iron Cage already implements the core model — IC tokens for agent authentication, IP keys for provider access, budget enforcement, RBAC — but no centralized server-side proxy exists yet.

**The architectural gap:** `iron_runtime` is a *local* proxy — it runs embedded on the agent's machine, binds `127.0.0.1:PORT`, and fetches real provider keys from the control API on demand. `iron_control_api` is the control plane (key/token CRUD, budgets, analytics); no LLM traffic passes through it. For a centralized deployment, agents must connect to a *server-side* proxy that holds and injects IP keys without ever exposing them to the client.

**Required new architecture:**

Three components are needed:

1. **Shared LLM routing core** (new library crate) — Extract the common forwarding logic currently embedded in `iron_runtime::llm_router`: request/response format translation, provider detection, cost extraction, PII safety integration. Both local and server execution will depend on this, eliminating duplication.

2. **Server-side proxy** (new binary crate) — Accepts remote agent HTTP connections authenticated by IC token, resolves the assigned IP key from storage, and forwards to the actual LLM provider. No key material is sent to the agent. Depends on the shared core and on `iron_token_manager` for auth and key resolution.

3. **`iron_runtime` (refactored)** — Becomes a thin wrapper over the shared core for local/embedded execution. Local-specific concerns remain here: dynamic port binding, key fetching from the server, budget handshake protocol, analytics sync.

**Deployment scope:** Internal team network only (not public-facing). Deadline: **2026-02-28**.

Existing components used unchanged:
- `module/iron_control_api` — Administration REST API (IP key CRUD, IC token management, RBAC, analytics ingestion)
- `module/iron_token_manager` — Token lifecycle, RBAC enforcement, budget tracking, IP key storage

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
