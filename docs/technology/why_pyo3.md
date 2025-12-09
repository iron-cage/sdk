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
2. **Single pip install** - No sidecar services to manage
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
from iron_sdk import protect_agent

@protect_agent(budget_usd=1.0)  # Rust validation
def my_agent(prompt: str) -> str:
    return llm.chat(prompt)  # Rust cost tracking
```

---

*Related: [why_rust.md](why_rust.md)*
