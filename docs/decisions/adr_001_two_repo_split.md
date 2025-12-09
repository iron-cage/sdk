# ADR-001: Two-Repository Split

**Status:** Accepted
**Date:** 2025-01

---

## Context

Iron Cage has components with different stability and release cadences:
- Dashboard/API: Changes weekly, product-driven
- Sandbox/CLI: Changes monthly, security-driven

Monorepo meant all components released together, causing unnecessary churn.

## Decision

Split into two repositories:
- **iron_runtime:** Control Panel, Agent Runtime (frequent changes)
- **iron_cage:** Sandbox, CLI, foundation modules (stable)

Share foundation modules via crates.io, not path dependencies.

## Consequences

**Positive:**
- Independent release cycles
- Clearer ownership (product vs security teams)
- Smaller CI/CD scope per repo

**Negative:**
- Cross-repo changes require coordination
- Version compatibility matrix needed
- Duplicate docs/scripts in some cases

**Mitigations:**
- Semantic versioning for crates.io modules
- Compatibility matrix in docs
- Shared CI templates via GitHub Actions

---

*Related: [architecture/two_repo_model.md](../architecture/two_repo_model.md)*
