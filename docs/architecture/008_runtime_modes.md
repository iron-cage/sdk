# Architecture: Runtime Modes

### Scope

This document defines the two runtime deployment configurations for Iron Cage Runtime.

**In Scope:**
- Two runtime modes (Router: HTTP-based separate process ~5ms overhead; Library: PyO3-embedded in-process ~0.5ms overhead)
- Router mode use cases (Existing frameworks LangChain/CrewAI with zero code changes; SDK users optional HTTP for debugging/isolation)
- Library mode characteristics (Default deployment, PyO3 FFI, single process, best performance)
- Developer code identical for both modes (mode is deployment configuration not API)
- Mode selection mechanisms (Environment variable IRON_RUNTIME_URL, CLI flags --mode=router, SDK default Library)
- Overhead comparison (Router HTTP serialization ~5ms; Library PyO3 FFI ~0.1-0.5ms; excludes budget tracking overhead)
- Trade-offs analysis (Deployment complexity, debugging visibility, process isolation, performance)
- Integration patterns (Router: endpoint configuration change; Library: pip install)
- Competitive advantage (On-premise deployment, no data exposure, local routing vs centralized servers)

**Out of Scope:**
- Budget tracking overhead (covered in Constraints 004: Trade-offs)
- PyO3 vs HTTP technical implementation details (covered in Technology 002: Why PyO3)
- Rust language choice rationale (covered in Technology 001: Why Rust)
- Execution location (local vs server, covered in Architecture 001: Execution Models)
- Specific framework integration guides (covered in Integration documentation)
- Runtime service architecture (Gateway, Safety, Cost services - covered in Architecture 005: Service Integration)

### Purpose

**User Need**: Platform developers, SDK users, and teams with existing AI agent frameworks need to understand the two deployment configurations (Router for HTTP-based separate process with ~5ms overhead, Library for PyO3-embedded in-process with ~0.5ms overhead) to choose the right mode for their use case (existing frameworks require Router with zero code changes, SDK users default to Library for best performance, optional Router for debugging/isolation).

**Solution**: Define two runtime modes with identical developer code but different internal deployment:

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

**Key Insight**: Runtime mode is a deployment configuration decision, not a code-level API decision. Developer code remains identical across both modes - the only difference is how the SDK communicates with the runtime internally (HTTP for Router ~5ms overhead for debugging/framework compatibility, PyO3 for Library ~0.5ms overhead for default production use). Library mode is the default for SDK users (single process, best performance), while Router mode enables two scenarios: zero-code-change integration with existing frameworks (LangChain, CrewAI just change endpoint configuration) and optional HTTP deployment for SDK users who need traffic inspection or process isolation.

---

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

### Router Mode

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

### Library Mode

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

### Trade-Offs

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

### Competitive Advantage

**Both modes run on developer platform:**
- No data leaves developer infrastructure
- No confidentiality issues (vs competitors routing through their servers)
- No bandwidth waste
- Lower latency (local routing)

**vs Competitors:** Most route through centralized servers (data exposure, latency)

### Key Clarifications

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

### Cross-References

#### Related Principles Documents
- Design Philosophy - Runtime mode flexibility principle, developer experience consistency
- Quality Attributes - Performance (Library mode ~0.5ms overhead vs Router ~5ms), Developer Experience (identical code across modes), Flexibility (framework-agnostic Router mode)

#### Related Architecture Documents
- [Architecture: Execution Models](001_execution_models.md) - Runtime deployment location (local primary 95%, server future 5%), Control Panel always-present managing both Router and Library runtime modes
- [Architecture: Layer Model](002_layer_model.md) - Both runtime modes implement same six processing layers (Safety, Cost, Reliability, Provider, Output Safety, Observability)
- [Architecture: Service Integration](005_service_integration.md) - Router mode runtime exposes same Gateway service (port 8084) as Library mode runtime for consistent service boundaries
- [Architecture: Data Flow](004_data_flow.md) - Both modes follow same eleven-step request flow (IC Token validation → Provider → Response), mode only affects SDK-to-runtime communication method

#### Used By
- Python SDK Implementation - Determines communication method (HTTP client for Router mode via IRON_RUNTIME_URL, PyO3 FFI for Library mode default)
- Runtime Deployment - CLI flag --mode=router for separate process HTTP server, embedded PyO3 for Library mode
- LangChain Integration - Router mode enables zero-code-change endpoint configuration (api.openai.com → localhost:8080)
- CrewAI Integration - Router mode OpenAI-compatible API for framework compatibility
- Debugging Tools - Router mode HTTP traffic inspection for request/response debugging
- Performance Monitoring - Library mode PyO3 FFI for production performance (0.5ms overhead vs 5ms Router)

#### Dependencies
- [Technology: Why Rust](../technology/001_why_rust.md) - Rust runtime enables both PyO3 embedding (Library mode) and HTTP server (Router mode)
- [Technology: Why PyO3](../technology/002_why_pyo3.md) - PyO3 vs HTTP trade-off analysis (0.5ms FFI vs 5ms HTTP), Library mode default rationale
- [Constraints: Trade-offs](../constraints/004_trade_offs.md) - Budget tracking overhead separate from runtime integration overhead (applies to both modes equally)

#### Implementation
- CLI: `iron-runtime --mode=router --port=8080` - Start Router mode HTTP server
- Environment: `export IRON_RUNTIME_URL=http://localhost:8080` - Configure SDK for Router mode
- SDK: `from iron_sdk import protect_agent` - Same import for both modes
- SDK Default: Library mode (PyO3 embedded, no IRON_RUNTIME_URL)
- Framework: Change endpoint configuration (api.openai.com → localhost:8080) for Router mode
- Router HTTP: OpenAI-compatible API at configured port
- Library PyO3: FFI bindings embedded in Python SDK package
- Configuration Detection: SDK checks IRON_RUNTIME_URL environment variable (set = Router HTTP, unset = Library PyO3)
