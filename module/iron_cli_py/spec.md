# iron_cli_py - Specification

**Module:** iron_cli_py
**Layer:** 6 (Application)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

Python CLI wrapper providing developer experience layer over iron_cli binary. Delegates token/usage/limits operations to iron_cli, provides native Python implementation for init/config/agent/secrets commands (ADR-005).

---

## Scope

**In Scope:**
- Wrapper commands (token, usage, limits) delegating to iron_cli
- Native commands (init, config, agent, secrets) in Python
- Click CLI framework integration
- Binary discovery and invocation
- Rich output formatting

**Out of Scope:**
- Token logic implementation (see iron_cli)
- REST API (see iron_api)
- Agent execution (see iron_runtime)

---

## Dependencies

**Required Modules:**
- iron_cli - Binary for delegated operations

**Required External:**
- click - CLI framework
- rich - Terminal formatting

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **Command Router:** Delegates to iron_cli or handles natively based on command type
- **Binary Invoker:** Discovers and executes iron_cli binary
- **Native Handler:** Implements Python-only commands (init, config, agent, secrets)
- **Output Formatter:** Rich terminal output

---

## Integration Points

**Used by:**
- Developers - Python CLI interface

**Uses:**
- iron_cli - Delegated token/usage/limits operations

---

*For detailed wrapper pattern, see spec/-archived_detailed_spec.md*
*For CLI architecture, see docs/decisions/adr_005_cli_architecture.md*
