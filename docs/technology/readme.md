# Technology

**Purpose:** Conceptual overview of technology choices and rationale.

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| 001 | **001_why_rust.md** | Justify Rust selection for runtime components | Language requirements → Rust rationale | Memory safety, performance needs, predictable latency, fearless concurrency, ecosystem maturity | NOT Python rationale (→ why_pyo3.md), NOT infrastructure (→ infrastructure_choices.md), NOT dependencies (→ dependency_strategy.md) |
| 002 | **002_why_pyo3.md** | Explain PyO3 choice for Rust-Python bridge | FFI requirements → PyO3 justification | In-process FFI, zero-copy performance, native Python types, latency comparison (0.1ms vs 5-10ms HTTP) | NOT Rust rationale (→ why_rust.md), NOT infrastructure (→ infrastructure_choices.md), NOT implementation (→ module/iron_runtime/spec.md) |
| 003 | **003_infrastructure_choices.md** | Document infrastructure technology selections | Infrastructure needs → Technology choices | Database (PostgreSQL), cache (Redis), messaging, storage (S3), rationale for each | NOT language choices (→ why_rust.md, why_pyo3.md), NOT dependency management (→ dependency_strategy.md), NOT deployment (→ docs/deployment/) |
| 004 | **004_dependency_strategy.md** | Define dependency management philosophy | Dependency question → Strategy principles | Selection criteria (maintenance, security, size, popularity, license), core dependencies, audit requirements | NOT specific tech choices (→ infrastructure_choices.md), NOT language rationale (→ why_rust.md), NOT implementation (→ Cargo.toml files) |

---

## Technology Concepts

| # | Concept | Core Idea |
|---|---------|-----------|
| 1 | [Why Rust](001_why_rust.md) | Performance + safety for critical path |
| 2 | [Why PyO3](002_why_pyo3.md) | Rust-Python FFI choice |
| 3 | [Dependency Strategy](004_dependency_strategy.md) | External crate philosophy |
| 4 | [Infrastructure Choices](003_infrastructure_choices.md) | Database, cache, queue |
| 5 | [Environments](005_environments.md) | Runtime modes and behaviors |

## Technology Principles

1. **Rust for critical path:** Safety, cost, reliability services
2. **Python for UX:** SDK, examples, developer experience
3. **Minimal dependencies:** Fewer crates = smaller attack surface
4. **Standard infrastructure:** PostgreSQL, Redis, no exotic databases

## Language Distribution

| Language | Use Case | LOC |
|----------|----------|-----|
| Rust | Core services, CLI | ~60% |
| Python | SDK, examples | ~25% |
| TypeScript | Dashboard | ~15% |

*For architecture concepts, see [architecture/](../architecture/)*
*For deployment concepts, see [deployment/](../deployment/)*
