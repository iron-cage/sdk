# Execution Models

**Purpose:** Where AI agents execute - Control Panel always manages budget.
**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

## User Need

Run AI agents with admin-controlled budgets and governance while keeping code/data local.

## Core Idea

**Control Panel is ALWAYS present (admin service managing developers):**

```
All Deployments:
+-------------------+
| Control Panel     | <-- ALWAYS exists (admin manages developers)
| - Admin dashboard |
| - Budget control  |
| - IP Token vault  |
+-------------------+
         |
         | IC Token ↔ Budget Protocol
         |
         v
    Agent Execution
    (local or server)
```

**The fundamental choice: WHERE does the agent execute?**

| Execution Location | Where Agent Runs | Who Manages Budget | Use Case |
|--------------------|------------------|-------------------|----------|
| **Local Execution** | Developer's machine | Admin via Control Panel | Primary (current, pilot) |
| **Server Execution** | Iron Cage servers | Admin via Control Panel | Future (post-pilot) |

**Key insight:** Control Panel ALWAYS manages budget (admin oversight). The only choice is WHERE the agent code executes.

## Local Execution (Primary - 95% of deployments)

**Where Agent Runs:** Developer's machine
**Who Manages Budget:** Admin via Control Panel (standalone service)
**Budget Control:** IC Token → IP Token protocol

```
Developer Machine              Control Panel (Admin Service)
+-----------------+            +-------------------------+
| Python Agent    |            | Admin Dashboard         |
| + Runtime       |            | - Allocate budgets      |
| Uses: IC Token  |<─ HTTPS ─>| - Monitor spending      |
+-----------------+   Budget   | - Store IP Tokens       |
                      Protocol +-------------------------+
                                       |
                                       | Admin manages ALL developers
```

**Characteristics:**
- Agent code stays on developer machine (privacy)
- Data stays local (files, databases, APIs)
- Control Panel is separate service (localhost in pilot, cloud in production)
- Developer receives IC Token from admin
- Runtime borrows budget portions ($10 from $100 total)
- Admin sees spending across all developers in dashboard

**Developer Roles:**
- **Developer:** IC Token, runs agents, views usage via CLI + Dashboard (read-only, own usage), selects model/IP
- **Super User:** Also has read-only Control Panel dashboard access (own budgets only, CLI + Dashboard)

**Developer Control (High Level for Efficient Development):**
- Select LLM model among allowed list
- Select IP/provider among allowed list (Pilot: admin pre-binds)
- Regenerate own IC Token and User Token
- View own usage in real-time

## Server Execution (Future - 5% of deployments, post-pilot)

**Where Agent Runs:** Iron Cage servers
**Who Manages Budget:** Admin via Control Panel (identical to local)
**Budget Control:** Same IC Token → IP Token protocol

- Developer uploads agent code to Iron Cage
- Iron Cage executes agent on managed infrastructure
- Control Panel manages budget identically to local execution
- Same two-token protocol, same admin oversight
- Developer still uses IC Token, never sees IP Token

**Status:** Deferred to post-pilot (not in current scope)

## Control Panel (Always Present)

**Role:** Standalone admin service for managing developers and budgets

**Admin Functions:**
- Create developer accounts
- Allocate budgets per developer/team
- Monitor spending real-time across all developers
- Manage IP Tokens (provider credentials in vault)
- Revoke access, adjust limits real-time
- Protect developers from overspending

**Developer Functions (with elevated access):**
- View own spending in dashboard (read-only)
- See budget allocation and remaining
- Cannot allocate budgets or see IP Tokens
- Cannot view other developers' data

**Deployment:**
- Pilot: localhost (admin-managed service)
- Production: cloud (admin-managed service)
- Future: Local emulation service using same protocol (development only)

**Always Required:** Cannot run agents without Control Panel - it's fundamental infrastructure.

## Runtime Modes (On Developer Platform)

**Both modes run on developer's platform - no data leaves developer infrastructure:**

**Developer code IDENTICAL for both modes:**
```python
from iron_sdk import protect_agent

@protect_agent(ic_token=os.getenv("IC_TOKEN"))
def my_agent(prompt: str):
    return llm.chat(prompt)
```

**Mode is deployment configuration (SDK internal machinery):**

| Mode | SDK Implementation | Overhead | Primary Use Case |
|------|-------------------|----------|------------------|
| **Library** | PyO3 FFI (in-process) | ~0.5ms | Default for iron_sdk users |
| **Router** | HTTP API (separate process) | ~5ms | Non-SDK frameworks (LangChain, CrewAI) |

**Router Mode:**
- Runtime exposes OpenAI-compatible REST API (localhost:8080)
- Two use cases: (1) Non-SDK frameworks change endpoint, (2) Optional SDK HTTP deployment
- Works with ANY framework (LangChain, CrewAI, custom)
- Easy migration (just change endpoint URL for non-SDK frameworks)

**Library Mode:**
- Runtime embedded via PyO3 (in-process)
- Default for iron_sdk users
- Better performance (direct FFI calls, no HTTP)
- Single process deployment (simpler)

**Competitive Advantage:**
- Both modes run locally on developer platform
- No data leaves developer infrastructure
- No confidentiality issues (vs competitors routing through centralized servers)
- Lower latency, no bandwidth waste

**See:** [008_runtime_modes.md](008_runtime_modes.md) for detailed comparison.

## Key Components

- **Control Panel:** Admin service (ALWAYS present, standalone)
- **Runtime:** Agent execution environment (local or server)
- **IC Token:** Developer-visible budget credential (from Control Panel)
- **IP Token:** Admin-managed provider credential (Control Panel vault only)
- **Budget Protocol:** Communication between Runtime and Control Panel

---

*Related: [../protocol/005_budget_control_protocol.md](../protocol/005_budget_control_protocol.md) | [003_service_boundaries.md](003_service_boundaries.md) | [006_roles_and_permissions.md](006_roles_and_permissions.md)*
