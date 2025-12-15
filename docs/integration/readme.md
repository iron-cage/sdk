# Integration

**Purpose:** Conceptual overview of external system integration patterns.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 001 | **001_llm_providers.md** | Document LLM provider integration patterns (OpenAI, Anthropic, Azure, Google, local models, unified API, fallback chains) |
| 002 | **002_secret_backends.md** | Explain secret storage backend options (Vault, AWS Secrets Manager, GCP Secret Manager, Azure Key Vault, adapter pattern) |
| 003 | **003_identity_providers.md** | Document authentication provider integrations (SSO via Okta, Auth0, Azure AD, OIDC/SAML protocols) |
| 004 | **004_observability_backends.md** | Define observability platform integrations (OpenTelemetry, Datadog, New Relic, Grafana, Prometheus, CloudWatch) |

---

## Integration Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [LLM Providers](001_llm_providers.md) | Multi-provider LLM access |
| 002 | [Secret Backends](002_secret_backends.md) | Enterprise secrets management |
| 003 | [Identity Providers](003_identity_providers.md) | SSO and authentication |
| 004 | [Observability Backends](004_observability_backends.md) | Metrics and tracing export |

## Integration Principles

1. **Don't rebuild:** Integrate with existing enterprise systems
2. **Abstraction layer:** Unified API over multiple backends
3. **Bring your own:** Support customer's existing tools
4. **Standards-based:** OpenTelemetry, OIDC, etc.

## Provider Landscape

```
+-------------------------------------------------+
|                 Iron Cage                        |
+----------+----------+----------+----------------+
| LLM      | Secrets  | Identity | Observability  |
+----------+----------+----------+----------------+
| OpenAI   | Vault    | Okta     | Datadog        |
| Anthropic| AWS SM   | Auth0    | New Relic      |
| Azure    | GCP SM   | Azure AD | Grafana        |
| Google   |          |          | Custom         |
+----------+----------+----------+----------------+
```

**Note:** This directory follows Design Collections format with NNN_ numbered instances (001-004) per documentation.rulebook.md § integration/ standards.
