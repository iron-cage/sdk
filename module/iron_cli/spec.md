# iron_cli - Specification

**Module:** iron_cli
**Layer:** 6 (Application)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

Binary CLI tool for Iron Cage token management, usage tracking, and limits. Provides authoritative implementation of token operations with iron_cli_py wrapper delegating to this binary (ADR-005).

---

## Scope

**In Scope:**
- Token management commands (create, list, revoke, show)
- Usage tracking commands (usage, limits)
- Trace viewing commands
- Configuration management
- Integration with iron_token_manager for operations

**Out of Scope:**
- Python CLI wrapper (see iron_cli_py)
- REST API (see iron_control_api)
- Dashboard UI (see iron_dashboard)
- Agent execution (see iron_runtime)

---

## Dependencies

**Required Modules:**
- iron_token_manager - Token operations
- iron_runtime_state - State persistence
- iron_telemetry - Logging

**Required External:**
- clap - CLI argument parsing
- unilang - Command definitions

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **Command Router:** Parses CLI args, routes to handlers
- **Token Operations:** Delegates to iron_token_manager
- **Output Formatter:** Tables and JSON output
- **Config Manager:** Handles CLI configuration

---

## Integration Points

**Used by:**
- iron_cli_py - Python wrapper delegates operations here (ADR-005)
- Developers - Direct CLI usage

**Uses:**
- iron_token_manager - Token CRUD operations
- iron_runtime_state - Data retrieval

---

*For detailed command specifications, see spec/-archived_detailed_spec.md*
*For wrapper pattern, see docs/decisions/adr_005_cli_architecture.md*
