# Why PyO3

**Purpose:** Rationale for PyO3 FFI over HTTP/gRPC for Rust-Python integration.

---

## User Need

Understand why the SDK uses in-process FFI instead of network calls to Rust services.

## Core Idea

**In-process calls eliminate network overhead for latency-critical operations:**

```
Option A (HTTP): Python --HTTP--> Rust   ~5-10ms per call
Option B (PyO3): Python --FFI--> Rust    ~0.1ms per call
                             +-- 50-100x faster
```

## Options Considered

| Option | Latency | Complexity | Deployment |
|--------|---------|------------|------------|
| HTTP API | 5-10ms | Simple | Separate services |
| gRPC | 2-5ms | Medium | Separate services |
| **PyO3 FFI** | 0.1ms | Complex | Single package |

## Why PyO3 Won

1. **Every LLM call goes through SDK** - 50x latency matters
2. **Single uv pip install** - No sidecar services to manage
3. **Type safety** - Rust types exposed to Python
4. **Zero-copy** - Large data (prompts) not serialized

## Build Complexity (Mitigated)

| Challenge | Mitigation |
|-----------|------------|
| Platform-specific wheels | Pre-built for linux/mac/windows |
| Cross-compilation | GitHub Actions matrix |
| Debug across boundary | Rich error types, detailed logs |

## PyO3 in Practice

```python
# Python code calls Rust seamlessly
from iron_cage import protect_agent

@protect_agent(budget_usd=1.0)  # Rust validation
def my_agent(prompt: str) -> str:
    return llm.chat(prompt)  # Rust cost tracking
```

## Router Mode vs Library Mode

**Question:** If PyO3 is chosen, why does Router mode use HTTP?

**Answer:** PyO3 chosen for SDK default implementation - Router mode exists for different use cases.

### Library Mode (PyO3 - Default)

**For:** iron_sdk users (default deployment)

**Architecture:**
```
from iron_cage import protect_agent → PyO3 FFI → Runtime (in-process)
```

**Characteristics:**
- PyO3 FFI (0.1-0.5ms overhead)
- Single process (runtime embedded)
- Default SDK behavior
- Best performance

**This is the PyO3 choice described above.**

### Router Mode (HTTP - Optional)

**For:** Two scenarios where PyO3 isn't suitable

**Scenario 1: Existing Frameworks (LangChain, CrewAI)**
- Framework doesn't use iron_sdk
- Can't use PyO3 (no SDK in their code)
- Runtime exposes OpenAI-compatible HTTP API
- Framework points to localhost:8080
- HTTP overhead (5ms) acceptable for compatibility

**Scenario 2: iron_sdk HTTP Deployment (Optional)**
- iron_sdk users who want HTTP instead of PyO3
- Same code: `from iron_cage import protect_agent`
- SDK configured to use HTTP internally: `export IRON_RUNTIME_URL=http://localhost:8080`
- Useful for debugging (inspect HTTP) or process isolation
- HTTP overhead (5ms) acceptable for these use cases

**Summary:**
- PyO3 is chosen and used by default (Library mode)
- HTTP exists for frameworks without SDK (Router mode) or optional SDK HTTP deployment
- iron_sdk users get PyO3 by default, can opt into HTTP if needed

**See:** [architecture/008: Runtime Modes](../architecture/008_runtime_modes.md) for complete comparison.

---

*Related: [001_why_rust.md](001_why_rust.md)*
