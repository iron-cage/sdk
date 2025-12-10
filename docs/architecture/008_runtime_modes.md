# Runtime Modes

**Purpose:** Two deployment configurations for Iron Cage Runtime - same SDK interface, different internal machinery.

---

## User Need

Understand trade-offs between router and library deployment configurations.

## Core Idea

**Developer code is IDENTICAL for both modes - difference is deployment configuration under the hood:**

```python
# Same code for Router AND Library mode
from iron_sdk import protect_agent

@protect_agent(ic_token=os.getenv("IC_TOKEN"))
def my_agent(prompt: str):
    return llm.chat(prompt)  # Automatically tracked
```

**Mode selection is deployment/configuration, not developer code:**

| Mode | SDK Implementation | Overhead | Deployment | Use Case |
|------|-------------------|----------|------------|----------|
| **Router** | SDK → HTTP → Runtime (separate process) | ~5ms | Separate runtime process | Debugging, existing frameworks |
| **Library** | SDK → PyO3 → Runtime (in-process) | ~0.5ms | Embedded in SDK | Default, performance-critical |

*Note: Overhead shown is for runtime integration method (HTTP vs direct calls), not cost tracking overhead. See [constraints/004: Trade-offs](../constraints/004_trade_offs.md#cost-vs-reliability) for budget tracking overhead.*

**Both modes:** Data stays on developer platform (no data leaves)

## Router Mode

**Architecture:**
```
Agent Code → HTTP → Runtime Process (localhost:8080) → Control Panel → LLM
```

**Two Use Cases:**

### Use Case 1: Existing Frameworks (No SDK)

**Scenario:** Team has LangChain/CrewAI codebase, wants governance without refactoring

**Integration:**
- Deploy runtime: `iron-runtime --mode=router --port=8080`
- Change framework config: `api.openai.com` → `localhost:8080`
- **No iron_sdk in agent code** (framework talks directly to runtime)
- Runtime presents OpenAI-compatible API

**Benefits:**
- Zero code changes (just endpoint configuration)
- Works with ANY Python framework
- Easy migration path

### Use Case 2: iron_sdk Users (Optional HTTP Deployment)

**Scenario:** Team uses iron_sdk, wants easier debugging or process isolation

**Integration:**
- Deploy runtime: `iron-runtime --mode=router --port=8080`
- Configure SDK: `export IRON_RUNTIME_URL=http://localhost:8080`
- **Same iron_sdk code** (SDK makes HTTP calls internally instead of PyO3)

**Benefits:**
- HTTP traffic inspectable (debugging)
- Runtime in separate process (isolation)
- Same developer code as Library mode

**Characteristics:**
- More overhead (~5ms) due to HTTP serialization/network
- Separate process (runtime must be started before agent)

## Library Mode

**Architecture:**
```
Agent Code → iron_sdk → PyO3 → Runtime (in-process) → Control Panel → LLM
```

**Default deployment for iron_sdk users.**

**Integration:**
- Install SDK: `uv pip install iron-sdk`
- Use in code: `from iron_sdk import protect_agent`
- **No separate runtime process needed** (embedded via PyO3)

**Characteristics:**
- Runtime embedded as library (PyO3 FFI)
- Direct function calls (no HTTP, no serialization)
- Minimal overhead (~0.5ms)
- Single process deployment (simpler)
- Best performance

**Benefits:**
- Default SDK behavior (no configuration needed)
- Single `uv pip install` (no separate runtime process)
- Lowest overhead (PyO3 FFI ~0.1-0.5ms)
- Simpler deployment (one process, not two)

**Use Case:** Default for all iron_sdk users, performance-critical applications

## Trade-Offs

### For iron_sdk Users (Choose Between Modes)

| Aspect | Router (HTTP) | Library (PyO3) |
|--------|---------------|----------------|
| Developer code | Identical | Identical |
| Overhead | ~5ms (HTTP) | ~0.5ms (FFI) |
| Deployment | Separate runtime process | Embedded in SDK |
| Configuration | `IRON_RUNTIME_URL=http://localhost:8080` | None (default) |
| Debugging | Easy (inspect HTTP traffic) | Harder (FFI calls) |
| Process isolation | Yes (runtime separate) | No (same process) |

**Default:** Library mode (PyO3) - simpler deployment, better performance

### For Non-SDK Frameworks (Router Only)

| Aspect | Integration Method |
|--------|--------------------|
| LangChain/CrewAI | Change endpoint: `api.openai.com` → `localhost:8080` |
| Developer code | Unchanged (framework's native API) |
| iron_sdk required | No (framework talks directly to runtime) |
| Overhead | ~5ms (HTTP) |

**Use Case:** Gradual migration, add governance to existing codebase without refactoring

## Competitive Advantage

**Both modes run on developer platform:**
- No data leaves developer infrastructure
- No confidentiality issues (vs competitors routing through their servers)
- No bandwidth waste
- Lower latency (local routing)

**vs Competitors:** Most route through centralized servers (data exposure, latency)

## Key Clarifications

**"Mode" is internal SDK configuration, not developer API:**
- Developer writes same code: `from iron_sdk import protect_agent`
- Mode determines how SDK communicates with runtime (HTTP vs PyO3)
- Configured via environment variable or runtime deployment choice

**Library mode is default (simpler):**
- Pilot uses Library mode (single process, no separate runtime)
- Production can use Library mode (default) or Router mode (optional for multi-framework support)

**Router mode enables two scenarios:**
1. Non-SDK frameworks (LangChain, CrewAI) - No code changes, just endpoint config
2. iron_sdk HTTP deployment - Same code, optional for debugging/isolation

**See:** [technology/002: Why PyO3](../technology/002_why_pyo3.md) for PyO3 vs HTTP trade-off analysis.

---

*Related: [001_execution_models.md](001_execution_models.md) | [../technology/001_why_rust.md](../technology/001_why_rust.md)*
