# iron_types - Specification

**Module:** iron_types
**Layer:** 2 (Foundation)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

Foundation types, errors, and Result types shared across all Iron Cage modules. Provides common error hierarchy using thiserror, standard Result aliases, core domain types.

---

## Scope

**In Scope:**
- Error type hierarchy (using thiserror)
- Result type aliases for convenience
- Core domain types (AgentId, TokenId, LeaseId, etc.)
- Common traits and interfaces

**Out of Scope:**
- Module-specific types (each module defines its own)
- Implementation logic (see respective modules)

---

## Dependencies

**Required External:**
- thiserror - Error handling
- serde - Serialization

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **Error Hierarchy:** Unified error types using thiserror
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
