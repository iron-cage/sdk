# Architecture Decision Records

**Purpose:** Document significant architectural decisions and their rationale.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| ADR-002 | **adr_002_rust_python_boundary.md** | Document decision on Rust-Python interface approach (PyO3 chosen over HTTP/gRPC for performance) |
| ADR-003 | **adr_003_client_side_primary.md** | Document decision to prioritize client-side execution (local 95%, server optional 5%, SDK governance) |
| ADR-005 | **adr_005_cli_architecture.md** | Document decision on CLI wrapper pattern (iron_cli_py wraps iron_cli binary, native vs delegated commands) |
| ADR-006 | **adr_006_package_consolidation.md** | Document decision to consolidate deployment packages (6→5 packages, iron_examples merged into iron_sdk) |
| ADR-007 | **adr_007_testing_philosophy.md** | Document decision to reject mocking (no mocking principle, test-per-module, real implementations, iron_testing removed) |
| ADR-008 | **adr_008_traces_endpoint_removal.md** | Document decision to remove traces debugging endpoint (security risk, not production feature, cleanup for Pilot) |

---

## Decision Log

| ID | Decision | Status | Date |
|----|----------|--------|------|
| ADR-002 | [Rust-Python Boundary](adr_002_rust_python_boundary.md) | Accepted | 2025-01 |
| ADR-003 | [Client-Side Primary](adr_003_client_side_primary.md) | Accepted | 2025-01 |
| ADR-005 | [CLI Architecture](adr_005_cli_architecture.md) | Accepted | 2025-12 |
| ADR-006 | [Package Consolidation](adr_006_package_consolidation.md) | Accepted | 2025-12 |
| ADR-007 | [Testing Philosophy](adr_007_testing_philosophy.md) | Accepted | 2025-12 |
| ADR-008 | [Traces Endpoint Removal](adr_008_traces_endpoint_removal.md) | Accepted | 2025-12-14 |

## ADR Format

Each ADR follows this structure:
- **Context:** Why we needed to make this decision
- **Decision:** What we decided
- **Consequences:** Trade-offs and implications

## Status Legend

| Status | Meaning |
|--------|---------|
| Proposed | Under discussion |
| Accepted | Implemented |
| Deprecated | Superseded by newer ADR |
| Rejected | Considered but not adopted |

*Source: Consolidated from pilot/decisions/ and new architectural decisions*
