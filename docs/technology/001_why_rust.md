# Why Rust

**Purpose:** Rationale for Rust as the core implementation language.

---

## User Need

Understand why Iron Cage uses Rust instead of Python/Go/Node for core services.

## Core Idea

**Rust provides memory safety + performance for latency-critical governance path:**

```
Request --> [Safety: 10ms] --> [Cost: 5ms] --> [Reliability: 5ms] --> LLM
              |                  |                |
              +---- Must be FAST (Rust) ---------+
```

## Key Benefits

| Benefit | Why It Matters |
|---------|---------------|
| **Memory safety** | No null pointers, buffer overflows in critical code |
| **Zero-cost abstractions** | High-level code compiles to efficient machine code |
| **Predictable latency** | No GC pauses in request path |
| **Fearless concurrency** | Safe async code for high throughput |

## Performance Comparison

| Operation | Rust | Python | Go |
|-----------|------|--------|-----|
| JSON parse (1KB) | 0.05ms | 0.5ms | 0.1ms |
| Regex match | 0.01ms | 0.1ms | 0.02ms |
| HTTP handler | 0.1ms | 2ms | 0.3ms |

## Where Rust Is Used

- iron_api: REST API + WebSocket server
- iron_safety: PII detection, prompt injection blocking
- iron_cost: Budget tracking, token counting
- iron_reliability: Circuit breakers, retry logic
- iron_cli: Binary CLI tool

## Where Rust Is NOT Used

- SDK: Python (developer experience priority)
- Dashboard: TypeScript (web ecosystem)
- Examples: Python (familiarity for AI developers)

---

*Related: [why_pyo3.md](why_pyo3.md) | [dependency_strategy.md](dependency_strategy.md)*
