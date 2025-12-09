# Credential Flow

**Purpose:** How secrets are accessed without exposure to agents.

---

## User Need

Give agents API keys and database passwords without storing secrets in agent memory.

## Core Idea

**Just-in-time, scoped credential injection:**

```
Agent --needs key--> Credential Service --fetches--> Vault
                            |
                    +-------+-------+
                    | Checks:       |
                    | - Agent ID    |
                    | - Scope       |
                    | - Rate limit  |
                    +---------------+
                            |
                    <-------+--- Returns scoped credential
```

## Flow Steps

1. Agent requests credential by name (e.g., "openai_key")
2. Credential service checks agent's permissions
3. If authorized, fetches from Vault
4. Returns credential with TTL (expires in N minutes)
5. Logs access to audit trail

## Key Principles

| Principle | Implementation |
|-----------|---------------|
| **Never in memory** | Credentials used inline, not stored |
| **Scoped access** | Agent X can only access its secrets |
| **Audit trail** | Every access logged with timestamp |
| **Auto-rotation** | Credentials rotate without agent restart |

## Credential Types

| Type | Storage | Rotation |
|------|---------|----------|
| LLM API keys | Vault | 90 days |
| Database passwords | Vault | 30 days |
| OAuth tokens | Vault | On expiry |

---

## Two-Token Architecture (Model C)

**For Control Panel-Managed deployments, Iron Cage uses dual-token security separating developer-visible credentials from provider credentials.**

**See:** [architecture/006: Budget Control Protocol](../architecture/006_budget_control_protocol.md) for complete specification.

### Token Separation

```
┌────────────────────────────────────────────────────┐
│          TWO-TOKEN SECURITY MODEL                   │
├────────────────────────────────────────────────────┤
│                                                     │
│  IC TOKEN (Internal Control)                       │
│  ┌───────────────────────────────────────────────┐ │
│  │ Visibility: ✅ Developer sees                 │ │
│  │ Format: JWT (eyJhbGc...)                      │ │
│  │ Contains: agent_id, budget_id, permissions    │ │
│  │ Safe to: Log, CLI args, config files          │ │
│  │ Risk: 🟢 LOW (no provider credentials)       │ │
│  └───────────────────────────────────────────────┘ │
│                        ⬇                            │
│               TRANSLATION (<1ms)                    │
│                        ⬇                            │
│  IP TOKEN (Inference Provider)                     │
│  ┌───────────────────────────────────────────────┐ │
│  │ Visibility: ❌ Developer NEVER sees           │ │
│  │ Format: Provider-specific (sk-proj-...)       │ │
│  │ Contains: Full provider API key               │ │
│  │ Storage: Memory only (AES-256 encrypted)      │ │
│  │ Risk: 🔴 CRITICAL (full provider access)     │ │
│  └───────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────┘
```

### Security Comparison

| Property | IC Token | IP Token |
|----------|----------|----------|
| **Developer Visibility** | ✅ Visible | ❌ Hidden |
| **Logging** | ✅ Safe to log | ❌ NEVER log |
| **CLI Arguments** | ✅ Safe | ❌ NEVER |
| **Config Files** | ✅ Safe | ❌ NEVER |
| **Disk Storage** | ✅ OK | ❌ Memory only |
| **Lifetime** | 24 hours | Session only |
| **If Stolen** | ⚠️ Limited (24h) | 🚨 Full provider access |

**Rationale:** IC Token identifies budget without exposing provider credentials. IP Token managed by Control Panel, delivered to Runtime encrypted, never exposed to developer.

**See:** [architecture/006: Budget Control Protocol](../architecture/006_budget_control_protocol.md) § The Two Tokens for complete specification.

---

*Related: [004_audit_model.md](004_audit_model.md) | [002_isolation_layers.md](002_isolation_layers.md) | [architecture/006: Budget Control](../architecture/006_budget_control_protocol.md)*
