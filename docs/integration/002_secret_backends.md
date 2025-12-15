# Secret Backends

**Purpose:** Integration with enterprise secrets management.

---

## User Need

Use existing enterprise secrets infrastructure, not a new system.

## Core Idea

**Adapter pattern: Iron Cage speaks your secrets backend's language**

## Supported Backends

| Backend | Type | Use Case |
|---------|------|----------|
| HashiCorp Vault | Self-hosted | Enterprise standard |
| AWS Secrets Manager | Cloud | AWS-native teams |
| GCP Secret Manager | Cloud | GCP-native teams |
| Azure Key Vault | Cloud | Azure-native teams |
| Local File | Dev | Development only |

## Integration Pattern

```
Iron Cage --adapter--> Backend API
    |
    +-- Vault: HTTP API + token auth
    +-- AWS SM: SDK + IAM role
    +-- GCP SM: SDK + service account
    +-- Azure KV: SDK + managed identity
```

## Sync vs On-Demand

| Pattern | Latency | Use Case |
|---------|---------|----------|
| **On-demand** | +50ms | Infrequent access, high security |
| **Sync** | 0ms (cached) | Frequent access, acceptable staleness |

## Configuration

```yaml
secrets:
  backend: vault
  vault:
    address: https://vault.company.com
    auth: kubernetes  # or token, approle
```

---

*Related: [001_llm_providers.md](001_llm_providers.md) | [003_identity_providers.md](003_identity_providers.md)*
