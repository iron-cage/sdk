# POST-PILOT Roadmap

**Purpose:** Central tracking of features deferred from Pilot for immediate post-Pilot implementation

**Last Updated:** 2025-12-10

---

## Priority 0 (Critical - First 2 Weeks Post-Pilot)

### Settings API - Hot-Reload
- **Spec:** [016_settings_api.md](protocol/016_settings_api.md)
- **Effort:** 3-4 days
- **Value:** Admin can modify settings without service restart
- **Dependencies:** Settings API implementation
- **Features:**
  - Hot-reload for operational settings (default_agent_budget, rate_limits)
  - Hot-reload for display settings (UI theme, currency format)
  - Configuration change notifications (WebSocket/SSE to connected clients)

### Projects API - Full CRUD
- **Spec:** [015_projects_api.md](protocol/015_projects_api.md#post-pilot-expansion)
- **Effort:** 5-7 days
- **Value:** Multi-project support, team collaboration
- **Dependencies:** None
- **Features:**
  - `POST /api/v1/projects` - Create project (Admin)
  - `PUT /api/v1/projects/{id}` - Update project (Owner/Admin)
  - `DELETE /api/v1/projects/{id}` - Delete project (Admin, ARCHIVE strategy)
  - Project-scoped budgets and analytics

---

## Priority 1 (High - Weeks 3-4 Post-Pilot)

### Agent Lifecycle Management
- **Spec:** [010_agents_api.md](protocol/010_agents_api.md#future-enhancements-post-pilot)
- **Effort:** 4-5 days
- **Value:** Safe agent deletion, temporary disabling
- **Dependencies:** None
- **Features:**
  - `DELETE /api/v1/agents/{id}` - ARCHIVE strategy (preserve audit trail)
  - `POST /api/v1/agents/{id}/activate` - Re-enable archived agent
  - `POST /api/v1/agents/{id}/deactivate` - Temporarily disable without deletion
  - IC Token invalidation on delete/deactivate

### Multi-Project Support
- **Spec:** [015_projects_api.md](protocol/015_projects_api.md#multi-project-support-future)
- **Effort:** 6-8 days
- **Value:** Users work across multiple projects
- **Dependencies:** Projects API Full CRUD
- **Features:**
  - User-project relationships (`project_members` table)
  - Context switching between projects (UI + API)
  - Project-scoped API tokens
  - Project-level analytics and dashboards

---

## Priority 2 (Medium - Weeks 5-6 Post-Pilot)

### Project User Management
- **Spec:** [015_projects_api.md](protocol/015_projects_api.md#user-management-endpoints-future)
- **Effort:** 3-4 days
- **Value:** Team management within projects
- **Dependencies:** Multi-Project Support
- **Features:**
  - `POST /api/v1/projects/{id}/users` - Add user to project
  - `DELETE /api/v1/projects/{id}/users/{user_id}` - Remove user
  - `PUT /api/v1/projects/{id}/users/{user_id}/role` - Update role

### Fine-Grained Permissions
- **Spec:** [015_projects_api.md](protocol/015_projects_api.md#security-considerations)
- **Effort:** 5-6 days
- **Value:** Granular access control (read-only users, restricted agents)
- **Dependencies:** Multi-Project Support
- **Features:**
  - Project-level permission model (beyond admin/owner/user)
  - Resource-level permissions (agent read/write, provider access)
  - Role-based access control (RBAC) implementation

### Project Resource Management
- **Spec:** [015_projects_api.md](protocol/015_projects_api.md#resource-management-future)
- **Effort:** 4-5 days
- **Value:** Resource isolation, cross-project sharing
- **Dependencies:** Multi-Project Support
- **Features:**
  - Project-level resource isolation (agents, providers, budgets)
  - Cross-project resource sharing (shared providers)
  - Project-level budget allocation (total budget cap per project)
  - Project-specific provider configurations

---

## Total Effort Estimate

- **Priority 0 (Critical):** 8-11 days
- **Priority 1 (High):** 10-13 days
- **Priority 2 (Medium):** 12-15 days
- **Total:** 30-39 days (6-8 weeks)

---

## Feature Dependencies

```
Projects CRUD
  ↓
Multi-Project Support
  ↓
├─→ Project User Management
├─→ Fine-Grained Permissions
└─→ Project Resource Management

Settings API ← Hot-Reload (independent)
Agent Lifecycle (independent)
```

---

## Implementation Order (Recommended)

1. **Week 1-2:** Settings API + Hot-Reload (P0)
2. **Week 2-3:** Projects API Full CRUD (P0)
3. **Week 3-4:** Agent Lifecycle Management (P1)
4. **Week 4-5:** Multi-Project Support (P1)
5. **Week 5-6:** Project User Management (P2)
6. **Week 6-7:** Fine-Grained Permissions (P2)
7. **Week 7-8:** Project Resource Management (P2)

---

## Success Metrics

- **Settings Hot-Reload:** 0 service restarts for configuration changes
- **Projects CRUD:** Teams can create 10+ projects without admin intervention
- **Agent Lifecycle:** 0 accidental permanent deletions (ARCHIVE strategy protects)
- **Multi-Project:** Users can switch between 5+ projects seamlessly
- **Permissions:** 90%+ of access control requests handled without admin escalation

---

**Related Documents:**
- Pilot Scope: [spec/requirements.md](../spec/requirements.md)
- API Specifications: [docs/protocol/](protocol/)
- REST API Decisions: [-rest_api_questions_complete.md](-rest_api_questions_complete.md)
