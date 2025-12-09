# Principles

**Purpose:** System-wide design principles and quality attributes guiding Iron Cage development.

---

## Directory Responsibilities

| ID | Entity | Responsibility | Input → Output | Scope | Out of Scope |
|----|--------|----------------|----------------|-------|--------------|
| 001 | **001_design_philosophy.md** | Document core design values and patterns | Design question → Philosophy | Simplicity, fail-safe defaults, observable behavior, minimal dependencies, composition patterns | NOT implementation (→ module/*/spec.md), NOT architecture (→ docs/architecture/), NOT other principles (→ 002-005) |
| 002 | **002_quality_attributes.md** | Define system-wide quality targets | NFR question → Quality targets | Performance (<10ms overhead), reliability (99.9%), scalability (10K agents), security (defense in depth), usability (Pythonic) | NOT module NFRs (→ module/*/spec.md), NOT architecture (→ docs/architecture/), NOT other principles (→ 001, 003-005) |
| 003 | **003_error_handling_principles.md** | Explain error handling philosophy | Error handling question → Philosophy | Fail-fast, loud failures, no silent errors, proper fixes only, error_tools usage | NOT implementation (→ module/*/spec.md), NOT testing (→ 004), NOT other principles (→ 001-002, 004-005) |
| 004 | **004_testing_strategy.md** | Document testing approach and philosophy | Testing question → Strategy | No mocking (ADR-007), integration-focused, real implementations, module-owned tests, loud failures | NOT testing implementation (→ module/*/tests/), NOT ADR details (→ docs/decisions/adr_007), NOT other principles (→ 001-003, 005) |
| 005 | **005_development_workflow.md** | Define development process principles | Workflow question → Process principles | Spec-first, ADR-driven, documentation before code, file creation protocol, knowledge preservation | NOT implementation (→ module/*/), NOT ADR format (→ docs/decisions/), NOT other principles (→ 001-004) |

---

## Principles Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [Design Philosophy](001_design_philosophy.md) | Core design values |
| 002 | [Quality Attributes](002_quality_attributes.md) | System-wide quality targets |
| 003 | [Error Handling Principles](003_error_handling_principles.md) | Error philosophy |
| 004 | [Testing Strategy](004_testing_strategy.md) | Testing approach |
| 005 | [Development Workflow](005_development_workflow.md) | Process principles |

---

*For architectural concepts, see [architecture/](../architecture/)*
*For constraints and trade-offs, see [constraints/](../constraints/)*
