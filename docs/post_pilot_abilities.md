# POST-PILOT Abilities

**Purpose:** What users can do after Pilot (capabilities only, no timelines)

**Last Updated:** 2025-12-10

---

## Admin Abilities

### Policy Management (Firewall-like Control)
- Set system-wide policies that Developers cannot override
- Define allowed models per provider (whitelist: gpt-4, claude-3-5-sonnet)
- Set max budget caps per agent (hard limit: e.g., $500)
- Block domains/IP ranges (security restrictions)
- Require safety guardrails (content filtering, prompt injection prevention)
- View audit trail of all policy changes and violations

### System Configuration
- Modify settings without restarting Control Panel backend (hot-reload)
- Create, update, delete projects
- Manage users across all projects (add/remove, change roles)
- Configure provider failover priority for all agents
- View all projects and cross-project analytics

---

## Developer Abilities

### Agent Management
- Assign multiple fallback providers to agents (primary, fallback1, fallback2)
- Create and manage multiple projects
- Delete agents safely (ARCHIVE strategy preserves audit trail)
- Temporarily disable agents without deletion (activate/deactivate)

### Policy Configuration (Must Not Contradict Admin)
- Ad-hoc model replacement for testing (gpt-4 → gpt-4-turbo)
- Override temperature/max_tokens for development
- Enable debug logging (verbose mode)
- Set provider preference ordering (prefer Anthropic over OpenAI)

### Project Management
- Create multiple projects
- Switch between projects
- Add/remove team members from owned projects
- Configure project-level budgets and settings

---

## Advanced User Abilities

### Cross-Project Visibility
- View all projects they belong to (multi-project dashboard)
- Access advanced analytics (provider performance, cost trends, model comparison)
- See system health metrics (not just own agents)

### Reporting
- Export audit logs and spending reports
- Generate cross-project analytics
- Track usage patterns across teams

---

## System Abilities (Runtime & Infrastructure)

### Concurrent Execution
- Run 100-1000+ concurrent agents per runtime instance
- Handle Python GIL efficiently for concurrent agents
- <5% CPU overhead per agent at 1000 concurrent agents

### Provider Resilience
- Automatic failover when primary provider fails (rate limits, outages, timeouts)
- Configurable retry logic with backoff strategy
- Provider health monitoring and circuit breaker
- Track which provider was used for each request (fallback statistics)

### Multi-Tenancy
- Project-level resource isolation (agents, providers, budgets)
- Cross-project resource sharing (shared providers)
- Users belong to multiple projects
- Project-scoped API tokens

### Configuration
- Hot-reload settings without service restart (operational, display, security settings)
- Configuration change notifications to connected clients (WebSocket/SSE)

---

## Role Hierarchy

```
Developer
  - Single project view
  - Basic analytics
  - Own agents only
  ↓
Advanced User
  - Multi-project view
  - Advanced analytics
  - Team visibility
  ↓
Admin
  - All projects
  - Full control
  - Policy management
```

---

**Related Documents:**
- Current Pilot scope: [spec/requirements.md](../spec/requirements.md)
- API specifications: [docs/protocol/](protocol/)
