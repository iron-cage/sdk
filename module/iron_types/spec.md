# iron_types - Specification

**Module:** iron_types
**Layer:** 2 (Foundation)
**Status:** Active

---

## Responsibility

Foundation types, errors, and Result types shared across all Iron Cage modules. Provides common error hierarchy using error_tools, standard Result aliases, core domain types.

---

## Scope

**In Scope:**
- Error type hierarchy (using error_tools per rulebook)
- Result type aliases for convenience
- Core domain types (AgentId, TokenId, LeaseId, etc.)
- Common traits and interfaces

**Out of Scope:**
- Module-specific types (each module defines its own)
- Implementation logic (see respective modules)

---

## Dependencies

**Required External:**
- error_tools - Error handling per rulebook
- serde - Serialization

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **Error Hierarchy:** Unified error types using error_tools
- **Result Aliases:** Convenient Result<T> wrappers
- **Domain Types:** Core identifiers (AgentId, TokenId, etc.)

---

## Integration Points

**Used by:**
- All modules - Foundation types

**Foundation module:** Published to crates.io for shared use

---

*For detailed type definitions, see spec/-archived_detailed_spec.md*
*For error handling principles, see docs/principles/003_error_handling_principles.md*
