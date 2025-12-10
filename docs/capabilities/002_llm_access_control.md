# LLM Access Control

**Concept:** Centralized gateway managing which users access which models with real-time budget enforcement.

---

## User Need

Organizations using multiple LLM providers face:
- No visibility into who's using what and how much it costs
- Budget overruns discovered only at end of month
- No way to restrict access to expensive models
- Different APIs and authentication for each provider

## Core Idea

Route all LLM traffic through a **single control point** that:
1. Authenticates and authorizes every request
2. Tracks token usage and costs in real-time
3. Enforces budget limits automatically (not just alerts)
4. Provides unified API across all providers

The insight: **Real-time enforcement** (automatic cutoffs) is different from **monitoring** (alerts after the fact). Most tools only monitor.

## Key Components

- **Unified API** - Single interface for OpenAI, Anthropic, Azure, etc.
- **Token Counter** - Real-time usage tracking per request
- **Budget Engine** - Agent budget enforcement (restrictive). Project/IP/Master budgets are informative (statistics only)
- **Cost Attribution** - Chargeback reporting by cost center

---

## Budget Allocation Architecture

**Iron Cage uses incremental budget allocation for real-time enforcement.**

**Budget Control Design:**

Agents are the ONLY way to control budget:
- **Agent budget:** Restrictive (blocks requests when exceeded)
- **Project budget:** Informative (shows project spending, no blocking)
- **IP budget:** Informative (shows provider spending, no blocking)
- **Master budget:** Informative (shows all spending, no blocking)

**Why:** Keeps control simple and predictable. Want to limit? Set agent budget. Want to monitor? View informative budgets.

**Admin Workflow:**
1. Create project with project budget (informative)
2. Create agent with agent budget ($100, restrictive)
3. Bind IPs to agent
4. Assign developer to project
5. Monitor spending (agent, project, IP, master budgets)

**Developer Workflow:**
1. Login to Control Panel → receive IC Token (developer-visible JWT)
2. Start Runtime with IC Token: `iron-runtime --ic-token eyJhbGc...`
3. Runtime borrows budget portions ($10) from Control Panel
4. Developer sees local budget only (not total allocation)

**Runtime Workflow:**
1. Initialize with IC Token → receive IP Token (encrypted)
2. Per request: Translate IC → IP (<0.5ms overhead)
3. Report usage to Control Panel:
   - **Pilot:** Per-request reporting (5ms, simpler implementation)
   - **Production:** Batched every 10 requests (0ms async, optimized for scale)
4. Request more budget when low (<$1 threshold)
5. If total budget exhausted → stop accepting calls

**Pilot Behavior:**
- Per-request reporting (simpler to implement)
- Admin sees spending immediately after each call
- Higher overhead (5ms) acceptable for demonstration
- Same hard limits enforced

**Production Behavior:**
- Batched reporting (every 10 requests) optimized for scale
- Admin sees spending with slight delay (~10 requests)
- Lower overhead (async batching) for high-throughput scenarios
- Same hard limits enforced, but based on batched data

**See:** [protocol/005: Budget Control Protocol](../protocol/005_budget_control_protocol.md) for complete protocol including:
- IC Token format (JWT claims)
- Budget borrowing messages (INIT, REPORT, REFRESH)
- IP Token encryption (AES-256-GCM)
- Token translation mechanism

---

## Related Capabilities

- [Observability](007_observability.md) - Detailed usage analytics
- [Credential Management](005_credential_management.md) - API key storage for providers
