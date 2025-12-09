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
- **Budget Engine** - Per-user, per-team, per-project limits with automatic cutoffs
- **Cost Attribution** - Chargeback reporting by cost center

---

## Budget Allocation Architecture (Model C)

**Control Panel-Managed deployments use incremental budget allocation for real-time enforcement.**

**Admin Workflow:**
1. Allocate budget per agent ($100) in Control Panel
2. Assign developer access to budget
3. Monitor real-time spending in dashboard
4. Increase/decrease budgets as needed

**Developer Workflow:**
1. Login to Control Panel → receive IC Token (developer-visible JWT)
2. Start Runtime with IC Token: `iron-runtime --ic-token eyJhbGc...`
3. Runtime borrows budget portions ($10) from Control Panel
4. Developer sees local budget only (not total allocation)

**Runtime Workflow:**
1. Initialize with IC Token → receive IP Token (encrypted)
2. Per request: Translate IC → IP (<1ms overhead)
3. Report usage to Control Panel (async, 0ms perceived)
4. Request more budget when low (<$1 threshold)
5. If total budget exhausted → stop accepting calls

**Real-Time Enforcement:**
- Every LLM call reported to Control Panel
- Admin sees spending within seconds
- Hard limits enforced (request denied when exhausted)
- Incremental borrowing prevents full budget exposure

**See:** [architecture/006: Budget Control Protocol](../architecture/006_budget_control_protocol.md) for complete protocol including:
- IC Token format (JWT claims)
- Budget borrowing messages (INIT, REPORT, REFRESH)
- IP Token encryption (AES-256-GCM)
- Token translation mechanism

---

## Related Capabilities

- [Observability](007_observability.md) - Detailed usage analytics
- [Credential Management](005_credential_management.md) - API key storage for providers
