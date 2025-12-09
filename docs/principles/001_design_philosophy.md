# Design Philosophy

**Purpose:** Core design values and patterns guiding Iron Cage development.

---

## User Need

Understand the fundamental principles that shape all design decisions and implementation choices.

## Core Idea

**Seven guiding principles drive every architectural choice:**

1. **Simplicity First** - Prefer simple solutions over complex ones
2. **Fail-Safe Defaults** - When uncertain, block (never allow unsafe)
3. **Observable Behavior** - Everything logged, traced, visible
4. **Minimal Dependencies** - Fewer dependencies = smaller attack surface
5. **Composition Over Inheritance** - Build from simple components
6. **Data Privacy** - No data leaves developer platform (runs locally)
7. **Agent-Centric Control** - Agents are ONLY way to control budget (all other budgets informative)

## Design Values

| Value | Manifestation |
|-------|---------------|
| **Simplicity** | SQLite before PostgreSQL, regex before ML, single-process pilot |
| **Fail-Safe** | Safety layer down = block all requests (never bypass) |
| **Observable** | Every LLM call logged, every error traced, dashboard shows all state |
| **Minimal Deps** | Carefully vetted crates, avoid kitchen-sink libraries |
| **Composition** | Small modules, clear boundaries, combined via interfaces |
| **Data Privacy** | Runtime runs on developer platform, no data sent to third-party servers (competitive advantage) |
| **Agent-Centric** | Agent budget blocks requests. Project/IP/Master budgets informative only (statistics). Keeps control simple |

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
| **Entity-Based Architecture** | User owns Agents, Agent has IC Token (1:1), Agent has budget (1:1). Clear ownership chain |
| **Wrapper Pattern** | iron_cli_py wraps iron_cli (ADR-005) |
| **Two-Token System** | IC Token (visible) + IP Token (hidden) for security |
| **Budget Borrowing** | Runtime leases portions from Control Panel (always present) |
| **Agent-Only Enforcement** | Only agent budget blocks. Project/IP/Master budgets informative (shows spending, can't block) |
| **Control Panel Required** | Always present admin service. No "self-managed" mode. Admin manages all developers |
| **Cost Reporting** | Pilot: per-request (simple). Production: batched (scale). See [constraints/004](../constraints/004_trade_offs.md) |
| **Defense in Depth** | Multiple security layers (process, syscall, filesystem, network) |
| **Local Execution** | Runtime on developer platform (router or library mode), data stays local |

---

*Related: [003_error_handling_principles.md](003_error_handling_principles.md) | [004_testing_strategy.md](004_testing_strategy.md)*
