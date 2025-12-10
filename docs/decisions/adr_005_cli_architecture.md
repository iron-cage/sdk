# ADR-005: CLI Wrapper Architecture

**Status:** Accepted
**Date:** 2025-12

*Source: Consolidated from pilot/decisions/002-cli-architecture.md*

---

## Context

Iron Cage provides two CLI tools:
- **iron_cli (Rust):** Production-ready, 288 tests, uses **unilang** for command definitions
- **iron_cli_py (Python):** Developer experience wrapper

**Unilang Framework:**
- iron_cli built with unilang (declarative command framework)
- Commands defined in YAML following unilang standards
- Enables consistent command structure, validation, help generation
- Follows unilang recommended CLI architecture patterns

**Original design:** iron_cli_py would reimplement token management in Python

**Problem:** Duplicate logic creates maintenance burden, feature drift risk, bug duplication.

## Decision

Adopt wrapper architecture following unilang recommended patterns:
- iron_cli_py **delegates** operations (token, usage, limits) to iron_cli binary
- iron_cli_py provides **native** developer experience features (init, config, agent)
- Leverages unilang's command infrastructure in iron_cli (single source of truth)
- Python wrapper provides ergonomic interface while preserving unilang command definitions

## Architecture

```
iron_cli_py (Python/Click)
+-- Native: init, config, agent, secrets
+-- Wrapper: token.*, usage.*, limits.* --> iron_cli (Rust/unilang)
                                              +-- Command definitions (YAML)
                                              +-- unilang framework
                                              +-- Single source of truth
```

**Unilang Integration:**
- iron_cli defines commands using unilang YAML format
- unilang provides command parsing, validation, help generation
- Wrapper pattern preserves unilang architecture while adding Python ergonomics
- Follows unilang recommended approach for CLI composition

## Consequences

**Positive:**
- Single source of truth for token logic (unilang command definitions in iron_cli)
- Automatic feature parity (wrapper inherits all unilang-defined commands)
- Reduced maintenance (one implementation following unilang patterns)
- Python-native experience preserved (pip, Click, Rich)
- Leverages unilang's robust command framework (validation, help, structure)

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
