# Entity Model

**Purpose:** Core entities and their relationships in Iron Cage platform.

> **Note:** "IP" in this document means **Inference Provider** (e.g., OpenAI, Anthropic), NOT IP address.

---

## User Need

Understand the main entities (Agent, Project, Inference Provider) and how they relate.

## Core Idea

**Six core entities with clear relationships:**

```
User (person)
  ├─ has role (Admin, Super User, Developer)
  ├─ belongs to Project(s)
  ├─ owns multiple Agents
  └─ has multiple User Tokens
       |
       └─ Master Project (admin only)
            ├─ contains ALL Projects
            └─ contains ALL Users
                 |
                 └─ Project
                      ├─ contains multiple Users
                      ├─ contains multiple Agents
                      ├─ contains multiple IPs
                      └─ has Project Budget (informative)
                           |
                           └─ Agent
                                ├─ owned by one User
                                ├─ has one IC Token (1:1)
                                ├─ has one Agent Budget (1:1, restrictive)
                                └─ can use multiple IPs (developer selects)
                                     |
                                     └─ IP (Inference Provider)
                                          ├─ has IP Budget (informative)
                                          └─ has IP Token(s)
```

## Entity 1: Agent

**Definition:** AI agent executing on developer's machine

**Relationships:**
- Has exactly one IC Token (1:1, can't share)
- Has exactly one Agent Budget (1:1, restrictive)
- Can use multiple Inference Providers/IPs (developer selects which to use)
- Belongs to exactly one Project

**Attributes:**
- agent_id (unique identifier)
- owner (developer)
- status (running, stopped, etc.)

## Entity 2: Project

**Definition:** Collection of agents, Inference Provider assignments, and related entities

**Relationships:**
- Contains multiple Agents
- Contains multiple Inference Provider (IP) assignments
- Has exactly one Project Budget (1:1, informative)
- Owned by admin or team

**Attributes:**
- project_id (unique identifier)
- name (human-readable)
- owner (admin or team)

## Entity 3: Master Project

**Definition:** Special project containing ALL resources (admin-only)

**Relationships:**
- Contains ALL agents across all projects
- Contains ALL Inference Providers (IPs)
- Has Master Budget (informative, aggregates all)
- Available only to admin

**Pilot Requirement:** Master project MUST be implemented in Pilot for admin oversight

## Entity 4: IP (Inference Provider)

**Definition:** LLM inference provider (OpenAI, Anthropic, etc.)

**Relationships:**
- Has IP Budget (informative, tracks spending per provider)
- Has IP Token(s): Pilot = 1, Future = multiple
- Can be assigned to multiple Agents

**Attributes:**
- ip_id (unique identifier)
- provider_name (openai, anthropic, etc.)
- endpoint_url

## Entity 5: User

**Definition:** Person using Iron Cage platform (admin, developer, super user)

**Relationships:**
- Has role (Admin, Super User, or Developer)
- Owns/creates multiple Agents (1:N)
- Belongs to Project(s) (N:M)
- Has multiple User Tokens (1:N)

**Attributes:**
- user_id (unique identifier)
- email (login)
- role (Admin, Super User, Developer)

**Token Permissions:**
- Can regenerate own IC Tokens (for owned agents)
- Can regenerate own User Tokens
- Admin can regenerate ANY tokens (all users, all agents)

## Entity 6: IC Token

**Definition:** Iron Cage Token for agent authentication

**Relationships:**
- Belongs to exactly one Agent (1:1)
- Owned by exactly one User (via agent ownership)
- Agent can't have multiple IC Tokens
- IC Token can't belong to multiple agents

**Lifecycle:** Created with agent, lives until agent deleted or regenerated (long-lived, no auto-expiration)

## Budget Types

**Restrictive Budget (ONE TYPE ONLY):**
- **Agent Budget:** Blocks requests when exceeded
- Hard limit enforcement
- Real-time tracking

**Informative Budgets (STATISTICS ONLY):**
- **Project Budget:** Shows project spending, no blocking
- **IP Budget:** Shows provider spending, no blocking
- **Master Budget:** Shows all spending, no blocking

**Key Point:** ONLY agent budget blocks requests. All others are for monitoring.

## Token Lifecycle Management

**IC Token (Agent Authentication):**
- Created: When agent is created by admin
- Lifetime: Until agent deleted (long-lived, no auto-expiration)
- Regeneration: Developer can regenerate own (replaces existing), Admin can regenerate any
- 1:1 with agent: Cannot be shared or transferred
- Invalidation: When agent deleted or IC Token regenerated

**User Token (Control Panel Access):**
- Created: When user account created or when user requests new token
- Lifetime: Configurable (default 30 days)
- Regeneration: User can regenerate own, Admin can regenerate any
- Multiple per user: User can have multiple active User Tokens

**IP Token (Provider Credential):**
- Created: Admin adds to Control Panel vault
- Lifetime: Provider-managed (typically long-lived)
- Regeneration: Admin only (in Control Panel vault)
- Users never see: Stored encrypted in Control Panel

## Budget Control Principle

**Critical Design Decision:** Agents are the ONLY way to control budget.

**Why:**
- Agent budget: Blocks requests when exceeded (restrictive)
- All other budgets: Statistics/monitoring only (informative)

**Want to limit spending?**
- Create agent with specific budget limit
- Agent budget will block requests when exceeded

**Want to monitor spending?**
- View project budget (all agents in project)
- View Inference Provider budget (all usage of provider)
- View master budget (everything)

**Can't control via:**
- Project budget (informative only)
- Inference Provider budget (informative only)
- Master budget (informative only)

This design keeps control simple and predictable.

---

*Related: [006_roles_and_permissions.md](006_roles_and_permissions.md) | [../protocol/005_budget_control_protocol.md](../protocol/005_budget_control_protocol.md)*
