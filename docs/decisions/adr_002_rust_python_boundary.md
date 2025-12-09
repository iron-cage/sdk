# ADR-002: Rust-Python Boundary via PyO3

**Status:** Accepted
**Date:** 2025-01

---

## Context

Need to protect Python AI agents with Rust safety/cost services.
Options considered:
1. HTTP API (Python calls Rust service)
2. gRPC (binary protocol)
3. PyO3 FFI (in-process calls)

## Decision

Use PyO3 FFI for Rust-Python integration:
- Rust code compiled to Python extension (.so)
- Zero-copy data transfer where possible
- Python decorators wrap Rust functions

## Consequences

**Positive:**
- <0.1ms overhead (vs 5-10ms for HTTP)
- No network dependency
- Type safety across boundary
- Single `pip install` deployment

**Negative:**
- More complex build (maturin, cross-compilation)
- Platform-specific wheels needed
- Debugging across boundary harder

**Mitigations:**
- Pre-built wheels for Linux/macOS/Windows
- Extensive integration tests
- Rich error messages from Rust to Python

---

*Related: [technology/002_why_pyo3.md](../technology/002_why_pyo3.md)*
