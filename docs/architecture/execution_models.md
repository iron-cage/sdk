# Execution Models

**Purpose:** Where AI agents execute - the fundamental architectural decision.

---

## User Need

Run AI agents with enterprise governance WITHOUT surrendering source code or data to a third party.

## Core Idea

**Two execution models based on where agent code runs:**

| Model | Where Agent Runs | Use Case | Users |
|-------|------------------|----------|-------|
| **Model A: Client-Side** | User's laptop/server | Local files, private data | 95% |
| **Model B: Server-Side** | Iron Cage infrastructure | Fully managed, no setup | 5% |

**The key insight:** Governance doesn't require running agent code. The SDK intercepts LLM calls and routes them through Iron Cage for validation/tracking, while the agent stays local.

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

## Key Components

- **Iron Cage SDK:** Python library that intercepts LLM calls
- **PyO3 FFI:** Zero-copy Rust-Python bridge (<0.1ms)
- **API Gateway:** Receives calls, validates, forwards to providers

---

*Related: [layer_model.md](layer_model.md) | [data_flow.md](data_flow.md)*
