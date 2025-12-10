# Technology

**Purpose:** Conceptual overview of technology choices and rationale.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 001 | **001_why_rust.md** | Explain Rust selection rationale for runtime components (memory safety, performance, predictable latency, no GC pauses) |
| 002 | **002_why_pyo3.md** | Explain PyO3 choice for Rust-Python integration (in-process FFI 0.1ms vs HTTP 5-10ms, zero-copy performance) |
| 003 | **003_infrastructure_choices.md** | Document infrastructure technology selections (Database: SQLite/PostgreSQL, Cache: In-memory/Redis, Storage: None/S3, pilot vs production) |
| 004 | **004_dependency_strategy.md** | Define dependency management philosophy (selection criteria, minimal dependencies, audit requirements, core dependencies) |

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
