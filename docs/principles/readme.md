# Principles

**Purpose:** System-wide design principles and quality attributes guiding Iron Cage development.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 001 | **001_design_philosophy.md** | Document seven core design principles and patterns (simplicity first, fail-safe defaults, observable behavior, minimal dependencies, composition, data privacy, agent-centric control) |
| 002 | **002_quality_attributes.md** | Define five system-wide quality attributes and targets (performance, reliability, scalability, security, usability) |
| 003 | **003_error_handling_principles.md** | Define three fundamental error handling principles (fail-fast, loud failures, proper fixes only) implementing Fail-Safe Defaults and Observable Behavior from Principles 001 |
| 004 | **004_testing_strategy.md** | Document testing approach and philosophy (no mocking, integration-focused, real implementations, module-owned tests) |
| 005 | **005_development_workflow.md** | Define development process principles (TDD cycle, spec-first, ADR-driven, documentation throughout, file creation protocol) |

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

*For architectural concepts, see [architecture/](../architecture/readme.md)*
*For constraints and trade-offs, see [constraints/](../constraints/readme.md)*
