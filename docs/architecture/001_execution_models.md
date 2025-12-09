# Execution Models

**Purpose:** Where AI agents execute - the fundamental architectural decision.

---

## User Need

Run AI agents with enterprise governance WITHOUT surrendering source code or data to a third party.

## Core Idea

**Three execution models based on where agent runs and where control happens:**

| Model | Where Agent Runs | Where Control Happens | Use Case | Users |
|-------|------------------|----------------------|----------|-------|
| **Model A: Client-Side** | User's machine | User's machine (self-managed) | Local files, private data, self-managed | Small teams (1-5 devs) |
| **Model B: Server-Side** | Iron Cage servers | Iron Cage servers (managed) | Fully managed, no setup | Managed hosting (5%) |
| **Model C: Control Panel-Managed** | User's machine | Control Panel (centralized admin) | Enterprise budgets, central control | Enterprise (10+ devs) |

**The key insight:** Governance doesn't require running agent code. Model C keeps agent local while centralizing budget control via two-token architecture.

## Model A: Client-Side (Primary)

```
User's Machine                    Iron Cage Gateway
+-----------------+              +-----------------+
| Python Agent    |--- LLM ---->| Validate/Track  |----> OpenAI
| + Iron Cage SDK |    calls    | Forward         |
| (code stays)    |<-- response-|                 |<---- Response
+-----------------+              +-----------------+
```

- Agent has full access to local files, APIs, databases
- Code never leaves user's infrastructure
- SDK adds <0.1ms overhead via PyO3 FFI

## Model B: Server-Side (Optional)

- Iron Cage runs agent code in managed environment
- User uploads agent definition, we execute it
- Best for: teams without infrastructure, quick prototypes

## Model C: Control Panel-Managed (Enterprise)

**Where Agent Runs:** Developer's machine (local execution, data stays local)
**Where Control Happens:** Centralized Control Panel (admin oversight, budget enforcement)

```
Developer Machine              Control Panel (Centralized)
+-----------------+            +-------------------------+
| Python Agent    |            | Admin Dashboard         |
| + Runtime       |            | - Budget allocations    |
| Uses: IC Token  |<─ HTTPS ─>| - IP Tokens (vault)     |
| (visible)       |   Budget   | - Real-time tracking    |
+-----------------+   Protocol +-------------------------+
                                       │
                                       │ Admin sees ALL agents
                                       ▼
                               Aggregate spending,
                               enforce limits,
                               real-time control
```

**Key Characteristics:**
- ✅ Agent runs locally (code/data stay on developer machine)
- ✅ Budget controlled centrally (admin approves spending)
- ✅ IC Token → IP Token translation (developer never sees provider credentials)
- ✅ Incremental budgeting (borrow $10 portions, not full $100)
- ✅ Real-time enforcement (admin can stop spending mid-session)

**See:** [architecture/006: Budget Control Protocol](006_budget_control_protocol.md) for complete two-token architecture and budget borrowing protocol.

## Key Components

- **Iron Cage SDK:** Python library that intercepts LLM calls
- **PyO3 FFI:** Zero-copy Rust-Python bridge (<0.1ms)
- **API Gateway:** Receives calls, validates, forwards to providers

---

*Related: [002_layer_model.md](002_layer_model.md) | [004_data_flow.md](004_data_flow.md)*
