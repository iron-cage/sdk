# ADR-008: Subsystem-Prefixed Crate Naming

**Status:** Accepted
**Date:** 2025-12-09

---

## Context

Current crate names lack clarity about which subsystem they belong to:

**Problems:**
- `iron_api` - too generic (which API? Control Panel or Runtime?)
- `iron_state` - ambiguous (Control Panel state or Runtime state?)
- `iron_control_store` - vague "store" (what does it store? how?)
- `iron_db` - documentation error (crate doesn't exist)
- `iron_lang` - managed in separate repository, shouldn't be referenced here

**Impact:**
- Developers can't identify crate purpose from name alone
- Subsystem boundaries unclear
- Documentation-code mismatches
- Cross-repository confusion

## Decision

Adopt **subsystem-prefixed naming convention** for subsystem-specific crates:

**Pattern:** `iron_<subsystem>_<role>`

**Naming Principles:**

1. **Subsystem-Specific Crates:** `iron_<subsystem>_<role>`
   - Control Panel: `iron_control_api`, `iron_control_schema`
   - Agent Runtime: `iron_runtime_state`

2. **Feature Crates:** `iron_<feature>` (unchanged)
   - Cross-subsystem features: `iron_safety`, `iron_cost`, `iron_reliability`

3. **Foundation Crates:** `iron_<foundation>` (unchanged)
   - Cross-cutting concerns: `iron_types`, `iron_telemetry`

## Renamed Crates

| Old Name | New Name | Rationale | Files Affected |
|----------|----------|-----------|----------------|
| iron_api | iron_control_api | Specifies Control Panel subsystem | ~92 |
| iron_control_store | iron_control_schema | Precise: database schemas | ~11 |
| iron_state | iron_runtime_state | Distinguishes Runtime state | ~34 |

**Documentation Fixes:**

| Issue | Fix | Files Affected |
|-------|-----|----------------|
| iron_db (doesn't exist) | Change to iron_control_schema | 1 |

**Repository Cleanup:**

| Action | Rationale | Files Affected |
|--------|-----------|----------------|
| Remove iron_lang references | Managed in separate repository | ~12 |

## Examples

**Control Panel Crates:**
- `iron_control_api` - REST API + WebSocket server
- `iron_control_schema` - PostgreSQL schemas
- `iron_token_manager` - Token management (already good)
- `iron_secrets` - Secrets management (already good)

**Agent Runtime Crates:**
- `iron_runtime` - Agent runtime orchestration (already good)
- `iron_runtime_state` - Runtime execution state
- `iron_sdk` - Python SDK (already good)

**Sandbox Crates:**
- `iron_sandbox` - OS isolation (already good)
- `iron_sandbox_core` - Core sandbox functionality (already good)

**CLI Crates:**
- `iron_cli` - Binary CLI (already good)
- `iron_cli_py` - Python CLI wrapper (already good)

**Feature Crates:**
- `iron_safety` - PII detection, content moderation (already good)
- `iron_cost` - Budget tracking (already good)
- `iron_reliability` - Circuit breakers (already good)

**Foundation Crates:**
- `iron_types` - Shared types (already good)
- `iron_telemetry` - Logging/tracing (already good)

## Consequences

**Positive:**
- Immediate subsystem identification from crate name
- Prevents name collisions (control vs runtime)
- Scalable as subsystems grow
- Self-documenting code organization
- Removes documentation-code mismatches
- Cleaner cross-repository boundaries

**Negative:**
- Breaking change for internal code (150 files affected)
- Longer crate names
- Documentation updates required

**Mitigations:**
- All changes internal (not published to crates.io)
- Systematic rename with validation gates
- Comprehensive documentation update
- Remove iron_lang cleanly (separate repo)

**Migration Strategy:**
1. Create ADR (this document)
2. Fix documentation error (iron_db → iron_control_schema)
3. Rename crates systematically with validation
4. Remove iron_lang references completely
5. Update vocabulary and cross-references
6. Validate with cargo build + tests

---

**Related:**
- Supersedes ADR-004 (earlier crate renaming)
- Complements ADR-006 (package consolidation)
