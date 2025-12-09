# Identity Providers

**Purpose:** SSO and authentication integration.

---

## User Need

Use corporate identity for Iron Cage access, not separate credentials.

## Core Idea

**Delegate authentication to enterprise IdP:**

```
User --login--> Iron Cage --redirect--> Corporate IdP
                    |                        |
                    |<--------token----------+
                    |
                    +-- Extracts: user, groups, roles
```

## Supported Providers

| Provider | Protocol | Features |
|----------|----------|----------|
| Okta | OIDC | Full SSO, groups |
| Auth0 | OIDC | Full SSO, roles |
| Azure AD | OIDC/SAML | Enterprise SSO |
| Google Workspace | OIDC | Basic SSO |
| Generic | OIDC | Any compliant IdP |

## Role Mapping

| IdP Group | Iron Cage Role | Permissions |
|-----------|---------------|-------------|
| `eng-team` | Developer | Run agents, view costs |
| `ops-team` | Operations | View all, manage tokens |
| `security` | Security | Full access, audit |

## Configuration

```yaml
auth:
  provider: oidc
  oidc:
    issuer: https://company.okta.com
    client_id: iron-cage
    scopes: [openid, profile, groups]
```

---

*Related: [secret_backends.md](secret_backends.md) | [observability_backends.md](observability_backends.md)*
