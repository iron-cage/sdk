# Principles: Design Philosophy

### Scope

This document defines the seven core design principles that guide all architectural decisions and implementation choices across the Iron Cage platform. These principles serve as the foundational value system applied consistently throughout specifications, architecture documents, and code implementations.

**In scope**:
- Seven core design principles (Simplicity First, Fail-Safe Defaults, Observable Behavior, Minimal Dependencies, Composition Over Inheritance, Data Privacy, Agent-Centric Control)
- Design values and their manifestations across the platform
- Forbidden anti-patterns with rationale
- Encouraged design patterns with application examples
- Philosophical foundation for all Iron Cage architectural decisions

**Out of scope**:
- Implementation-specific details of individual components (see Architecture collection)
- Concrete quality metrics and SLAs (see Principles 002: Quality Attributes)
- Error handling implementation patterns (see Principles 003: Error Handling Principles)
- Testing methodology and test organization (see Principles 004: Testing Strategy)
- Development workflow and process steps (see Principles 005: Development Workflow)
- Technology selection rationale (see Technology collection)

### Purpose

**User Need:** Understand the fundamental principles that shape all design decisions and implementation choices across the Iron Cage platform.

**Solution:** Seven guiding principles drive every architectural choice:

1. **Simplicity First** - Prefer simple solutions over complex ones
2. **Fail-Safe Defaults** - When uncertain, block (never allow unsafe)
3. **Observable Behavior** - Everything logged, traced, visible
4. **Minimal Dependencies** - Fewer dependencies = smaller attack surface
5. **Composition Over Inheritance** - Build from simple components
6. **Data Privacy** - No data leaves developer platform (runs locally)
7. **Agent-Centric Control** - Agents are ONLY way to control budget (all other budgets informative)

These principles manifest in concrete design values (simplicity, fail-safe, observable, minimal deps, composition, data privacy, agent-centric), forbidden anti-patterns (mocking, silent errors, code duplication, backup files, premature optimization), and encouraged design patterns (entity-based architecture, wrapper pattern, two-token system, budget borrowing, defense in depth, local execution).

**Key Insight:** Design principles are not aspirational - they are enforceable constraints. Every architectural decision and implementation choice must demonstrate alignment with these seven principles. When principles conflict, Fail-Safe Defaults and Data Privacy take precedence over other considerations.

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

---

### Design Values

| Value | Manifestation |
|-------|---------------|
| **Simplicity** | SQLite before PostgreSQL, regex before ML, single-process pilot |
| **Fail-Safe** | Safety layer down = block all requests (never bypass) |
| **Observable** | Every LLM call logged, every error traced, dashboard shows all state |
| **Minimal Deps** | Carefully vetted crates, avoid kitchen-sink libraries |
| **Composition** | Small modules, clear boundaries, combined via interfaces |
| **Data Privacy** | Runtime runs on developer platform, no data sent to third-party servers (competitive advantage) |
| **Agent-Centric** | Agent budget blocks requests. Project/IP/Master budgets informative only (statistics). Keeps control simple |

### Anti-Patterns (Forbidden)

| Pattern | Why Forbidden |
|---------|---------------|
| **Mocking** | Tests must validate real behavior (ADR-007) |
| **Silent Errors** | All failures must be loud and traceable |
| **Code Duplication** | DRY principle, consolidate or reference |
| **Backup Files** | Trust git history, delete old code completely |
| **Premature Optimization** | Solve current problem, not hypothetical future |

### Design Patterns (Encouraged)

| Pattern | Application |
|---------|-------------|
| **Entity-Based Architecture** | User owns Agents, Agent has IC Token (1:1), Agent has budget (1:1). Clear ownership chain. See [architecture/007: Entity Model](../architecture/007_entity_model.md) |
| **Wrapper Pattern** | iron_cli_py wraps iron_cli (ADR-005). See [decisions/adr_005](../decisions/adr_005_cli_architecture.md) |
| **Two-Token System** | IC Token (visible) + IP Token (hidden) for security. See [features/002: Token Management](../features/002_token_management.md) |
| **Budget Borrowing** | Runtime leases portions from Control Panel (always present). See [protocol/005: Budget Control Protocol](../protocol/005_budget_control_protocol.md) |
| **Agent-Only Enforcement** | Only agent budget blocks. Project/IP/Master budgets informative (shows spending, can't block). See [protocol/005: Budget Control Protocol](../protocol/005_budget_control_protocol.md#enforcement-model) |
| **Control Panel Required** | Always present admin service. No "self-managed" mode. Admin manages all developers. See [architecture/003: Service Boundaries](../architecture/003_service_boundaries.md) |
| **Cost Reporting** | Pilot: per-request (simple). Production: batched (scale). See [constraints/004: Trade-offs](../constraints/004_trade_offs.md#cost-vs-reliability) |
| **Defense in Depth** | Multiple security layers (process, syscall, filesystem, network). See [security/002: Isolation Layers](../security/002_isolation_layers.md) |
| **Local Execution** | Runtime on developer platform (router or library mode), data stays local. See [deployment/002: Actor Model](../deployment/002_actor_model.md) |

---

### Cross-References

#### Related Principles Documents

- [002_quality_attributes.md](002_quality_attributes.md) - System-wide quality targets applying design principles to measurable attributes
- [003_error_handling_principles.md](003_error_handling_principles.md) - Error philosophy derived from Fail-Safe Defaults and Observable Behavior principles
- [004_testing_strategy.md](004_testing_strategy.md) - Testing approach enforcing "no mocking" anti-pattern and real implementations
- [005_development_workflow.md](005_development_workflow.md) - Process principles applying Simplicity First and Composition patterns

#### Used By

- Architecture: All architecture documents apply these seven principles to component design
- Protocol: API design follows Observable Behavior and Fail-Safe Defaults principles
- Security: Threat model and isolation layers apply Defense in Depth pattern and Data Privacy principle
- Constraints: Trade-off decisions reference Simplicity First principle for pilot vs production choices
- Capabilities: All capability specifications demonstrate alignment with design principles

#### Dependencies

- None - This document defines foundational principles with no dependencies on other documents

#### Implementation

- All architecture documents must demonstrate principle alignment
- Code reviews validate adherence to anti-patterns (forbidden) and design patterns (encouraged)
- ADRs (Architecture Decision Records) must cite relevant principles as decision rationale
- Testing infrastructure enforces "no mocking" anti-pattern via test organization standards
