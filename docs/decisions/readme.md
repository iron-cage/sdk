# Architecture Decision Records

**Purpose:** Document significant architectural decisions and their rationale.

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **adr_001_two_repo_split.md** | Document decision to split into two repositories | Repository organization decision → ADR | iron_runtime vs iron_cage separation, release cycle rationale, crates.io sharing, consequences | NOT architecture details (→ docs/architecture/two_repo_model.md), NOT implementation (→ module/*/), NOT other decisions (→ other ADRs) |
| **adr_002_rust_python_boundary.md** | Record decision on Rust-Python interface approach | FFI design decision → ADR | PyO3 choice, alternatives considered (HTTP, gRPC), performance trade-offs, consequences | NOT technology rationale (→ docs/technology/why_pyo3.md), NOT implementation (→ module/iron_runtime/), NOT other decisions (→ other ADRs) |
| **adr_003_client_side_primary.md** | Explain decision to prioritize client-side execution | Execution model decision → ADR | Client-side primary (95%), server-side optional (5%), governance via SDK interception, rationale | NOT execution architecture (→ docs/architecture/execution_models.md), NOT deployment (→ docs/deployment/), NOT other decisions (→ other ADRs) |
| **adr_004_crate_renaming.md** | Document decision on crate naming convention | Naming convention decision → ADR | iron_* prefix adoption, migration strategy from old names, impact analysis, consequences | NOT module details (→ module/*/spec.md), NOT repository structure (→ adr_001), NOT other decisions (→ other ADRs) |
| **adr_005_cli_architecture.md** | Record decision on CLI wrapper pattern | CLI architecture decision → ADR | Wrapper pattern, native vs delegated commands, alternatives considered, consequences | NOT architecture guide (→ docs/features/cli_architecture.md), NOT implementation (→ module/iron_cli_py/spec.md), NOT other decisions (→ other ADRs) |

---

## Decision Log

| ID | Decision | Status | Date |
|----|----------|--------|------|
| ADR-001 | [Two-Repository Split](adr_001_two_repo_split.md) | Accepted | 2025-01 |
| ADR-002 | [Rust-Python Boundary](adr_002_rust_python_boundary.md) | Accepted | 2025-01 |
| ADR-003 | [Client-Side Primary](adr_003_client_side_primary.md) | Accepted | 2025-01 |
| ADR-004 | [Crate Renaming](adr_004_crate_renaming.md) | Accepted | 2025-12 |
| ADR-005 | [CLI Architecture](adr_005_cli_architecture.md) | Accepted | 2025-12 |

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
