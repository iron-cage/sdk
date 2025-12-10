# ADR-002: Rust-Python Boundary via PyO3

**Status:** Accepted
**Date:** 2025-01

---

## Context

Iron Cage provides governance (safety, cost control, reliability) for Python AI agents while keeping agent code and data on developer's platform. The governance layer must:
- Run on developer's machine (data privacy)
- Add minimal overhead (<1ms per LLM call)
- Enable both router and library runtime modes
- Support Control Panel communication (budget protocol)

**Architecture requirement:** Agent runs locally (Python), governance runs locally (Rust), Control Panel manages budget (admin service).

**Integration options considered:**
1. **HTTP API:** Python SDK calls separate Rust service via HTTP
2. **gRPC:** Python SDK calls Rust service via gRPC
3. **PyO3 FFI:** Rust compiled as Python extension, in-process calls

## Decision

Use PyO3 FFI for Rust-Python boundary:
- Rust code compiled to Python extension (.so, .pyd, .dylib)
- Zero-copy data transfer where possible
- Enables two runtime modes:
  - **Router mode:** Runtime exposes OpenAI-compatible API (more overhead, flexibility)
  - **Library mode:** Direct SDK integration (minimal overhead)
- Both modes run on developer platform (no data leaves)

## Consequences

**Positive:**
- <0.1ms overhead for library mode (<1ms for router mode)
- No network dependency (in-process calls)
- Type safety across boundary (Rust types in Python)
- Single `pip install iron-sdk` deployment
- Enables both router and library modes
- Data stays on developer platform (competitive advantage)
- Control Panel communication via HTTPS (budget protocol)

**Negative:**
- Complex build system (maturin, cross-compilation)
- Platform-specific wheels needed (Linux, macOS, Windows)
- Debugging across Rust-Python boundary harder
- GIL (Global Interpreter Lock) affects threading

**Mitigations:**
- Pre-built wheels for x86_64 Linux, macOS, Windows (ARM support future)
- Extensive integration tests for Rust ↔ Python boundary
- Rich error messages with Python tracebacks from Rust
- Error types mapped across boundary (Rust errors → Python exceptions)
- Comprehensive logging for debugging

## Implementation

**What PyO3 Enables:**
- Python SDK (`iron_sdk`) wraps Rust runtime (`iron_runtime`)
- Agent code stays pure Python (developer experience)
- Rust enforces governance transparently
- Runtime modes (router vs library) both via PyO3
- Control Panel budget protocol handled in Rust

**Key Components:**
- `iron_runtime` (Rust) - Agent orchestrator, policy enforcement
- `iron_sdk` (Python) - Developer-facing API
- PyO3 bridge - Type conversion, error mapping

---

*Related: [technology/002_why_pyo3.md](../technology/002_why_pyo3.md) | [architecture/008_runtime_modes.md](../architecture/008_runtime_modes.md) | [architecture/001_execution_models.md](../architecture/001_execution_models.md)*
