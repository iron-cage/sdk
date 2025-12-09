# Design Philosophy

**Purpose:** Core design values and patterns guiding Iron Cage development.

---

## User Need

Understand the fundamental principles that shape all design decisions and implementation choices.

## Core Idea

**Five guiding principles drive every architectural choice:**

1. **Simplicity First** - Prefer simple solutions over complex ones
2. **Fail-Safe Defaults** - When uncertain, block (never allow unsafe)
3. **Observable Behavior** - Everything logged, traced, visible
4. **Minimal Dependencies** - Fewer dependencies = smaller attack surface
5. **Composition Over Inheritance** - Build from simple components

## Design Values

| Value | Manifestation |
|-------|---------------|
| **Simplicity** | SQLite before PostgreSQL, regex before ML, single-process pilot |
| **Fail-Safe** | Safety layer down = block all requests (never bypass) |
| **Observable** | Every LLM call logged, every error traced, dashboard shows all state |
| **Minimal Deps** | Carefully vetted crates, avoid kitchen-sink libraries |
| **Composition** | Small modules, clear boundaries, combined via interfaces |

## Anti-Patterns (Forbidden)

| Pattern | Why Forbidden |
|---------|---------------|
| **Mocking** | Tests must validate real behavior (ADR-007) |
| **Silent Errors** | All failures must be loud and traceable |
| **Code Duplication** | DRY principle, consolidate or reference |
| **Backup Files** | Trust git history, delete old code completely |
| **Premature Optimization** | Solve current problem, not hypothetical future |

## Design Patterns (Encouraged)

| Pattern | Application |
|---------|-------------|
| **Wrapper Pattern** | iron_cli_py wraps iron_cli (ADR-005) |
| **Two-Token System** | IC Token (visible) + IP Token (hidden) for security |
| **Budget Borrowing** | Runtime leases portions from centralized allocation |
| **Real-Time Reporting** | Usage tracked immediately, not batched |
| **Defense in Depth** | Multiple security layers (process, syscall, filesystem, network) |

---

*Related: [003_error_handling_principles.md](003_error_handling_principles.md) | [004_testing_strategy.md](004_testing_strategy.md)*
