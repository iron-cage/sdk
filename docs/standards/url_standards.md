# URL Standards

**Purpose:** Canonical URL reference for Iron Cage platform

**Responsibility:** Define official domain structure, URL construction rules, and validation standards

**Status:** Normative (all documentation must follow these standards)

---

## Canonical Production URLs

### Service Endpoints

| Service | URL | Purpose |
|---------|-----|---------|
| **Marketing Site** | https://ironcage.ai | Public website, documentation, marketing |
| **Control Panel API** | https://api.ironcage.ai/v1 | REST API for agent management, analytics, budget control |
| **Control Panel Dashboard** | https://dashboard.ironcage.ai | Web UI for Control Panel |
| **Gateway Service** | https://gateway.ironcage.ai/v1 | Runtime request routing and processing |

### Error Type URLs (RFC 7807)

Base URL: `https://ironcage.ai/errors/`

**Examples:**
- `https://ironcage.ai/errors/validation-error`
- `https://ironcage.ai/errors/budget-exceeded`
- `https://ironcage.ai/errors/unauthorized`
- `https://ironcage.ai/errors/token-expired`

**Rationale:** Error type URLs should use the main domain (ironcage.ai) rather than the API subdomain. This follows RFC 7807 best practices where error types are documentation URLs that should be stable and service-agnostic.

---

## Development URLs

### Local Services

| Service | URL | Purpose |
|---------|-----|---------|
| **Control Panel API** | http://localhost:3000 | Local API development server |
| **Control Panel Dashboard** | http://localhost:5173 | Local UI development server (Vite) |
| **Gateway Service** | http://localhost:8080 | Local gateway development server |

### Port Assignments

| Port | Service | Standard |
|------|---------|----------|
| 3000 | Control Panel API | Default for backend APIs |
| 5173 | Dashboard (Vite dev) | Vite default development port |
| 4173 | Dashboard (Vite preview) | Vite preview mode port |
| 8080 | Gateway Service | Standard gateway/proxy port |

---

## URL Construction Rules

### 1. Protocol Requirements

**Production:**
- ✅ **ALWAYS** use `https://` for production URLs
- ❌ **NEVER** use `http://` for production
- Security: TLS 1.2+ required

**Development:**
- ✅ Use `http://` for localhost URLs
- No TLS required for local development

### 2. Subdomain Conventions

**Pattern:** Service-Oriented Subdomains

```
ironcage.ai                 → Marketing site
api.ironcage.ai            → API services
dashboard.ironcage.ai      → UI applications
gateway.ironcage.ai        → Gateway/proxy services
```

**Rationale:**
- Clear service boundaries
- Independent scaling
- Separate TLS certificates per service
- Fine-grained CORS policies
- Industry standard pattern

### 3. API Versioning

**Version in path, NOT subdomain:**
- ✅ `https://api.ironcage.ai/v1/agents`
- ❌ `https://v1.api.ironcage.ai/agents`

**Rationale:**
- Easier to manage multiple versions
- Standard REST API practice
- Simpler DNS configuration

### 4. Trailing Slashes

**No trailing slashes on API endpoints:**
- ✅ `https://api.ironcage.ai/v1/agents`
- ❌ `https://api.ironcage.ai/v1/agents/`

**Exception:** Base URLs in configuration may include trailing slash for clarity:
- Configuration: `API_BASE_URL="https://api.ironcage.ai/v1/"`
- Endpoint: `GET /agents` (concatenated to form full URL)

### 5. Example URLs in Documentation

**ALWAYS show both development and production examples:**

```bash
# Development
curl http://localhost:3000/api/v1/agents

# Production
curl https://api.ironcage.ai/v1/agents
```

**Rationale:**
- Prevents copy-paste errors (localhost URLs in production)
- Clear distinction between environments
- Helps developers understand deployment differences

---

## Prohibited Patterns

### Deprecated Domains

❌ **iron.dev** (legacy domain, replaced by ironcage.ai in 2025)

**Migration:** All references to `iron.dev` must be updated to `ironcage.ai`

### Deprecated Subdomains

❌ **control-panel.ironcage.ai** (use `api.ironcage.ai` instead)

**Rationale:** "Control Panel" encompasses both API and Dashboard. Using separate subdomains (api.ironcage.ai, dashboard.ironcage.ai) provides clearer separation.

### Anti-Patterns

❌ Mixed protocols in documentation:
```markdown
<!-- BAD: Inconsistent -->
curl https://api.ironcage.ai/v1/agents
curl http://api.ironcage.ai/v1/analytics
```

❌ Using IP addresses instead of localhost:
```bash
# BAD
curl http://127.0.0.1:3000/api/v1/agents

# GOOD
curl http://localhost:3000/api/v1/agents
```

❌ Hardcoded ports in production URLs:
```bash
# BAD
https://api.ironcage.ai:443/v1/agents

# GOOD
https://api.ironcage.ai/v1/agents
```

---

## CORS Configuration

### API Service (api.ironcage.ai)

**Allowed Origins:**
```javascript
// Production
allowed_origins: [
  "https://dashboard.ironcage.ai",
  "https://ironcage.ai"
]

// Development
allowed_origins: [
  "http://localhost:5173",  // Vite dev server
  "http://localhost:4173"   // Vite preview
]
```

### Dashboard (dashboard.ironcage.ai)

**Content Security Policy:**
```http
Content-Security-Policy:
  default-src 'self';
  connect-src 'self' https://api.ironcage.ai;
  script-src 'self' 'unsafe-inline';
  style-src 'self' 'unsafe-inline';
```

---

## DNS & Infrastructure

### DNS Records

**Required A/AAAA Records:**
```
ironcage.ai               → Static site CDN (CloudFlare/Netlify)
api.ironcage.ai           → Load balancer (API servers)
dashboard.ironcage.ai     → CDN (React SPA)
gateway.ironcage.ai       → Load balancer (Gateway servers)
```

### TLS Certificates

**Option 1: Wildcard Certificate**
```
*.ironcage.ai (covers all subdomains)
ironcage.ai (apex domain)
```

**Option 2: Individual Certificates**
```
ironcage.ai
api.ironcage.ai
dashboard.ironcage.ai
gateway.ironcage.ai
```

**Recommendation:** Use wildcard certificate (simpler management, Let's Encrypt free)

---

## Validation

### Automated Validation Script

Location: `dev/scripts/validate_urls.sh`

**Tests:**
1. No `iron.dev` references in codebase
2. No `control-panel.ironcage.ai` references in documentation
3. All production URLs use `https://` (not `http://`)
4. No trailing slashes on API endpoints
5. Code files use correct API URL (`api.ironcage.ai`)

### Manual Validation Checklist

**Before committing documentation:**
- [ ] All production URLs use `https://`
- [ ] All production URLs use correct subdomains (api, dashboard, gateway)
- [ ] Error type URLs use base domain (`ironcage.ai/errors/`)
- [ ] Examples show both development and production URLs
- [ ] No hardcoded ports in production URLs
- [ ] No trailing slashes on API endpoints

---

## Migration History

### 2025-12-10: iron.dev → ironcage.ai

**Change:** Migrated from legacy `iron.dev` domain to `ironcage.ai`

**Impact:**
- 1 file updated (RFC 7807 error type URL)
- No code changes required (already using ironcage.ai)

### 2025-12-10: control-panel → api subdomain

**Change:** Standardized to service-oriented subdomains

**Migration:**
- `control-panel.ironcage.ai` → `api.ironcage.ai`
- 1 file updated (API tokens protocol documentation)

**Rationale:** Clearer service boundaries, consistent with industry standards

---

## References

**Related Documentation:**
- [Vocabulary](../vocabulary.md) - Platform terminology including URL definitions
- [Deployment Strategy](../deployment/003_distribution_strategy.md) - Package distribution and URLs
- [REST API Protocol](../protocol/002_rest_api_protocol.md) - API conventions and standards

**External Standards:**
- [RFC 7807 - Problem Details](https://tools.ietf.org/html/rfc7807) - Error type URL format
- [RFC 3986 - URI Generic Syntax](https://tools.ietf.org/html/rfc3986) - URL structure
- [CORS Specification](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) - Cross-origin policy

---

**Document Version:** 1.0.0
**Last Updated:** 2025-12-10
**Status:** Normative (must follow)
