# ADR-006: Package Consolidation (6 → 5)

**Status:** Accepted
**Date:** 2025-12-09

---

## Context

Iron Cage had 6 deployment packages with structural redundancy:

1. **Package 5 (CLI Tool)** and **Package 6 (Python CLI)** were separate, but Package 6 depends on Package 5
2. **iron_examples** was a standalone module, but it only demonstrates iron_sdk functionality

This created confusion:
- "Do I need both Package 5 AND Package 6 for CLI functionality?"
- "Why is examples a separate module instead of shipping with the SDK?"

The separation didn't reflect the actual relationships:
- iron_cli_py **wraps** iron_cli (ADR-005 wrapper pattern)
- iron_examples **demonstrates** iron_sdk (not a reusable library)

## Decision

Consolidate to 5 deployment packages:

1. **Merge Package 6 into Package 5** as "CLI Tools"
   - Package 5 now contains: iron_cli (binary) + iron_cli_py (Python wrapper)
   - Distributed as: GitHub binary + PyPI wheel
   - Single installation script installs both

2. **Merge iron_examples into iron_sdk** as examples/ subdirectory
   - iron_sdk now includes: core SDK + examples/ directory
   - Examples ship with `uv pip install iron-sdk[examples]`
   - Clearer that examples demonstrate SDK usage

3. **Create iron_control_schema skeleton** for consistency
   - All 21 modules now have physical directories
   - Spec-only module for PostgreSQL schema documentation

## Consequences

**Positive:**
- Clearer package model (CLI tools ship together)
- Better user experience (examples ship with SDK)
- All modules have physical directories (no "spec-only" without skeleton)
- Simpler mental model (5 packages vs 6)
- Matches wrapper pattern from ADR-005
- Reduced module count (21 vs 22)

**Negative:**
- Breaking change for users importing iron_examples directly
- Documentation updates across 25+ files required
- Migration guide needed for existing users
- iron_examples PyPI package needs deprecation notice

**Mitigations:**
- Comprehensive migration guide created (docs/-migration_examples_to_sdk.md)
- All documentation updated atomically in single change
- Deprecation notice to be added to iron_examples PyPI package
- iron_control_schema skeleton prevents "missing module" confusion

## Implementation

**Changes made:**
- Moved 6 example directories from iron_examples to iron_sdk/examples/
- Created iron_control_schema skeleton (5 files)
- Updated 25+ documentation files
- Updated Cargo.toml workspace (14→15 members)
- Archived iron_examples to module/-archived_iron_examples/

**Validation:**
- ✅ Cargo workspace builds successfully
- ✅ All documentation references updated
- ✅ No broken cross-references
- ✅ All 21 modules have physical directories

---

<!-- TODO: Add related documentation links when available -->
