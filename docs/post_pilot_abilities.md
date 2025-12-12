# Iron Cage Features

**Purpose:** Complete feature catalog with unique codes across Pilot and POST-PILOT phases

**Last Updated:** 2025-12-10

**Coding Scheme:** F-XYY where X=Actor (1=Admin, 2=Developer, 3=Advanced User, 4=System, 5=All Users), YY=Sequential

---

## Feature Summary

| Code | Feature | Actor | Phase | Specification |
|------|---------|-------|-------|---------------|
| F-101 | User Management | Admin | PILOT | [Protocol 008](protocol/008_user_management_api.md) |
| F-102 | Agent Management | Admin | PILOT | [Protocol 010](protocol/010_agents_api.md) |
| F-103 | Budget Management | Admin | PILOT | [Protocol 013](protocol/013_budget_limits_api.md) |
| F-104 | Provider Management | Admin | PILOT | [Protocol 011](protocol/011_providers_api.md) |
| F-105 | Analytics | Admin | PILOT | [Protocol 012](protocol/012_analytics_api.md) |
| F-106 | Projects | Admin | PILOT | [Protocol 015](protocol/015_projects_api.md) |
| F-107 | Budget Request Approval | Admin | PILOT | [Protocol 017](protocol/017_budget_requests_api.md) |
| F-108 | Policy Management | Admin | POST-PILOT | Not yet specified |
| F-109 | Settings Management | Admin | POST-PILOT | [Protocol 016](protocol/016_settings_api.md) |
| F-110 | Project Management | Admin | POST-PILOT | [Protocol 015](protocol/015_projects_api.md) |
| F-111 | Fine-Grained Permissions | Admin | POST-PILOT | Not yet specified |
| F-112 | Provider Failover Config | Admin | POST-PILOT | Not yet specified |
| F-113 | Budget Request Enhancements | Admin | POST-PILOT | [Protocol 017](protocol/017_budget_requests_api.md) |
| F-201 | Agent Operations | Developer | PILOT | [Protocol 010](protocol/010_agents_api.md) |
| F-202 | API Tokens | Developer | PILOT | [Protocol 014](protocol/014_api_tokens_api.md) |
| F-203 | Analytics | Developer | PILOT | [Protocol 012](protocol/012_analytics_api.md) |
| F-204 | Budget Requests | Developer | PILOT | [Protocol 017](protocol/017_budget_requests_api.md) |
| F-205 | Agent Lifecycle | Developer | POST-PILOT | [Protocol 010](protocol/010_agents_api.md) |
| F-206 | Provider Failover | Developer | POST-PILOT | Not yet specified |
| F-207 | Policy Configuration | Developer | POST-PILOT | Not yet specified |
| F-208 | Multi-Project Operations | Developer | POST-PILOT | [Protocol 015](protocol/015_projects_api.md) |
| F-301 | Cross-Project Visibility | Advanced User | POST-PILOT | Not yet specified |
| F-302 | Reporting | Advanced User | POST-PILOT | Not yet specified |
| F-401 | Concurrent Execution | System | PILOT | [Requirements](../spec/requirements.md) |
| F-402 | Provider Integration | System | PILOT | [Protocol 005](protocol/005_budget_control_protocol.md) |
| F-403 | Provider Resilience | System | POST-PILOT | Not yet specified |
| F-404 | Multi-Tenancy | System | POST-PILOT | [Protocol 015](protocol/015_projects_api.md) |
| F-405 | Configuration | System | POST-PILOT | [Protocol 016](protocol/016_settings_api.md) |
| F-501 | Authentication | All Users | PILOT | [Protocol 007](protocol/007_authentication_api.md) |

**Total:** 29 features (15 PILOT, 14 POST-PILOT)

---

## F-101: User Management (PILOT, Admin)
[Protocol 008](protocol/008_user_management_api.md)

- Create/suspend/activate/delete users (soft delete preserves audit trail)
- Change roles (Admin/User/Viewer), reset passwords
- List with filters (role, status, search by name/email)
- Self-modification prevention

## F-102: Agent Management (PILOT, Admin)
[Protocol 010](protocol/010_agents_api.md)

- Create agents (name, budget, providers, description, tags)
- List agents, get details and status (budget/spending/active state)
- Update metadata (name, description, tags)

## F-103: Budget Management (PILOT, Admin)
[Protocol 013](protocol/013_budget_limits_api.md)

- Modify agent budgets (full mutability with force flag for decreases)
- Emergency budget top-ups
- View modification history

## F-104: Provider Management (PILOT, Admin)
[Protocol 011](protocol/011_providers_api.md)

- Full CRUD operations (create, list, get, update, delete)
- Configure provider credentials (IP tokens stored in vault)
- Assign providers to agents

## F-105: Analytics (PILOT, Admin)
[Protocol 012](protocol/012_analytics_api.md)

- 8 endpoints: spending/total, by-agent, by-provider, budget status, usage requests, tokens, models, avg-per-request
- Time ranges: today, yesterday, last-7-days, last-30-days, all-time

## F-106: Projects (PILOT, Admin)
[Protocol 015](protocol/015_projects_api.md)

- List and view projects (read-only, single Master Project)

## F-107: Budget Request Approval (PILOT, Admin)
[Protocol 017](protocol/017_budget_requests_api.md)

- Approve budget change requests (update agent budget automatically)
- Reject budget change requests (with required review notes)
- List all pending/approved/rejected requests (system-wide visibility)
- View request details (justification, requester, agent status)

## F-108: Policy Management (POST-PILOT, Admin)
Not yet specified

- Set system-wide policies (firewall-like control): allowed models, budget caps, blocked domains, safety guardrails
- Policy validation engine (conflict detection, admin-wins resolution)
- View policy audit trail (changes, violations)

## F-109: Settings Management (POST-PILOT, Admin)
[Protocol 016](protocol/016_settings_api.md)

- Hot-reload settings without service restart
- Configure 3-level hierarchy (system/project/user with inheritance)
- Operational: default budget, max agents, auto-pause thresholds
- Notifications: email alerts, webhooks, frequency
- Display: UI theme, dashboard layout, currency format
- Security: session timeout, API token expiration, IP whitelist, 2FA

## F-110: Project Management (POST-PILOT, Admin)
[Protocol 015](protocol/015_projects_api.md)

- Full CRUD (create, update, delete projects)
- Manage project users (add, remove, change roles)
- Configure project-level budgets
- Project-scoped API tokens

## F-111: Fine-Grained Permissions (POST-PILOT, Admin)
Not yet specified

- Create custom roles beyond Admin/User/Viewer
- Set resource-level permissions (agent read/write, provider access)
- Granular access controls (read-only users, restricted agents)

## F-112: Provider Failover Configuration (POST-PILOT, Admin)
Not yet specified

- Configure provider failover priority for all agents
- Provider health monitoring and circuit breaker

## F-113: Budget Request Enhancements (POST-PILOT, Admin)
[Protocol 017](protocol/017_budget_requests_api.md)

- Flexible approval amount (admin can modify budget during approval, not just approve/reject)
- Request expiration (auto-reject after N days of inactivity)
- Bulk operations (approve/reject multiple requests at once)
- Advanced filters (by requester, date range, budget threshold)
- Request templates (pre-fill justification for common scenarios)
- Delegation (senior developers can approve small increases)
- Rejection cooldown (prevent immediate re-requests after rejection)
- Webhook events (notify external systems on approval/rejection)

## F-201: Agent Operations (PILOT, Developer)
[Protocol 010](protocol/010_agents_api.md)

- Create/list/update agents with tags
- View agent status (budget, spending, active state)

## F-202: API Tokens (PILOT, Developer)
[Protocol 014](protocol/014_api_tokens_api.md)

- Create/revoke tokens (SAME-AS-USER scope, shown once on creation)
- List and view owned tokens

## F-203: Analytics (PILOT, Developer)
[Protocol 012](protocol/012_analytics_api.md)

- View metrics for owned agents
- Same 8 endpoints as Admin (scoped to own resources)

## F-204: Budget Requests (PILOT, Developer)
[Protocol 017](protocol/017_budget_requests_api.md)

- Create budget change requests (for owned agents)
- View request status (pending, approved, rejected)
- Cancel pending requests (before admin review)
- List own requests with filters (status, date range)

## F-205: Agent Lifecycle (POST-PILOT, Developer)
[Protocol 010](protocol/010_agents_api.md)

- Delete agents (ARCHIVE strategy preserves audit trail)
- Activate/deactivate agents (temporary disable without deletion)

## F-206: Provider Failover (POST-PILOT, Developer)
Not yet specified

- Assign multiple fallback providers (primary, fallback1, fallback2)
- Automatic failover on errors (rate limits, outages, timeouts)
- Configurable retry logic with backoff

## F-207: Policy Configuration (POST-PILOT, Developer)
Not yet specified

- Set agent-level policies (validated against admin policies, no contradictions)
- Ad-hoc model replacement for testing (gpt-4 → gpt-4-turbo)
- Override temperature/max_tokens for development
- Enable debug logging, set provider preference ordering

## F-208: Multi-Project Operations (POST-PILOT, Developer)
[Protocol 015](protocol/015_projects_api.md)

- Create and manage multiple projects
- Switch between projects
- Add/remove team members from owned projects
- Configure project-level settings

## F-301: Cross-Project Visibility (POST-PILOT, Advanced User)
Not yet specified

- View all projects they belong to (multi-project dashboard)
- Access advanced analytics (provider performance, cost trends, model comparison)
- See system health metrics (not just own agents)

## F-302: Reporting (POST-PILOT, Advanced User)
Not yet specified

- Export audit logs and spending reports
- Generate cross-project analytics
- Track usage patterns across teams

## F-401: Concurrent Execution (PILOT, System)
[Requirements](../spec/requirements.md)

- Run 100-1000+ concurrent agents per runtime
- <5% CPU overhead per agent at 1000 concurrent

## F-402: Provider Integration (PILOT, System)
[Protocol 005](protocol/005_budget_control_protocol.md)

- IP tokens (provider credentials) managed in Control Panel vault
- Token translation (IC Token → IP Token) for inference requests
- Budget handshake protocol for credential delivery

## F-403: Provider Resilience (POST-PILOT, System)
Not yet specified

- Automatic failover when primary provider fails
- Configurable retry logic with backoff strategy
- Provider health monitoring and circuit breaker
- Track which provider was used for each request (fallback statistics)

## F-404: Multi-Tenancy (POST-PILOT, System)
[Protocol 015](protocol/015_projects_api.md)

- Project-level resource isolation (agents, providers, budgets)
- Cross-project resource sharing (shared providers)
- Users belong to multiple projects
- Project-scoped API tokens

## F-405: Configuration (POST-PILOT, System)
[Protocol 016](protocol/016_settings_api.md)

- Hot-reload settings without service restart
- Configuration change notifications (WebSocket/SSE to connected clients)

## F-501: Authentication (PILOT, All Users)
[Protocol 007](protocol/007_authentication_api.md)

- Login with email/password (receive User Token, JWT, 30-day lifetime)
- Logout (invalidate User Token)
- Refresh User Token (extend expiration before it expires)
- Validate User Token (check if valid and not expired)

---

**Related Documents:**
- Current Pilot scope: [spec/requirements.md](../spec/requirements.md)
- API specifications: [docs/protocol/](protocol/readme.md)
