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

*Related: [audit_model.md](audit_model.md) | [isolation_layers.md](isolation_layers.md)*
