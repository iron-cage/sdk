# ADR-005: CLI Wrapper Architecture

**Status:** Accepted
**Date:** 2025-12

*Source: Consolidated from pilot/decisions/002-cli-architecture.md*

---

## Context

Iron Cage provides two CLI tools:
- **iron_cli (Rust):** Production-ready, 288 tests
- **iron_cli_py (Python):** Developer experience wrapper

Original design called for iron_cli_py to reimplement token management in Python.

Problem: Duplicate logic creates maintenance burden, feature drift risk, bug duplication.

## Decision

Adopt wrapper architecture:
- iron_cli_py **delegates** operations (token, usage, limits) to iron_cli binary
- iron_cli_py provides **native** developer experience features (init, config, agent)

## Architecture

```
iron_cli_py (Python/Click)
+-- Native: init, config, agent, secrets
+-- Wrapper: token.*, usage.*, limits.* --> iron_cli (Rust)
                                              +-- Single source of truth
```

## Consequences

**Positive:**
- Single source of truth for token logic
- Automatic feature parity (wrapper inherits all features)
- Reduced maintenance (one implementation)
- Python-native experience preserved (pip, Click, Rich)

**Negative:**
- Requires iron_cli binary installed
- Subprocess overhead (~50ms per command)
- Binary distribution complexity

**Mitigations:**
- Bundled binary option: `pip install iron-cli-py[binary]`
- Clear error messages for missing binary
- Binary discovery with multiple fallback locations

---

*Original: pilot/decisions/002-cli-architecture.md*

**Note:** As of December 2025, iron_cli and iron_cli_py are packaged together as "Package 5: CLI Tools" to reflect their tight integration and unified distribution model.
