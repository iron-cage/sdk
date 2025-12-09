# Decision Record: CLI Architecture - Wrapper Pattern

**Date:** 2025-12-08
**Status:** Accepted
**Decision ID:** 002

---

## Decision

Adopt wrapper architecture where iron_cli_py delegates operations commands (token, usage, limits, traces, auth, health) to iron_cli binary while providing Python-native developer experience features (init, config, agent, secrets).

---

## Context

Iron Cage provides two CLI tools with overlapping responsibilities:

| Tool | Language | Status | Tests | Distribution |
|------|----------|--------|-------|--------------|
| iron_cli | Rust | Production-ready | 288 | Binary |
| iron_cli_py | Python | Scaffolding | 0 | pip |

### Problem Statement

The original design called for iron_cli_py to reimplement token management in Python, creating:

- **Duplicate Logic**: Same token algorithms in Rust and Python
- **Maintenance Burden**: Two codebases for same functionality
- **Feature Drift**: CLIs may diverge in behavior over time
- **Bug Risk**: Same bug could appear in both implementations
- **Test Duplication**: Testing same logic twice

### Driving Question

Should iron_cli_py reimplement operations commands or delegate to iron_cli?

---

## Decision Drivers

1. **Single Source of Truth** - Token logic should exist in one place only
2. **Automatic Feature Parity** - Python CLI should inherit all Rust CLI features
3. **Reduced Maintenance** - One implementation to maintain
4. **Python Developer Experience** - pip install, Click syntax, Rich output
5. **Programmatic API** - Python library usage for scripting
6. **No Duplication** - Follows Anti-Duplication Principle from organizational_principles.rulebook.md

---

## Considered Options

### Option A: Full Reimplementation

iron_cli_py reimplements all commands in Python.

- **Pros:** No binary dependency, pure Python solution
- **Cons:** Duplicate logic (HIGH RISK), double maintenance burden, potential feature drift
- **Decision:** Rejected

### Option B: API-Only Python CLI

iron_cli_py calls Control Panel API directly (like iron_cli does).

- **Pros:** No binary dependency, consistent approach
- **Cons:** Still duplicate HTTP client code, authentication logic duplicated
- **Decision:** Rejected

### Option C: Wrapper Pattern

iron_cli_py wraps iron_cli binary for operations, native for dev experience.

- **Pros:** Single source of truth, automatic parity, reduced maintenance
- **Cons:** Requires iron_cli binary, ~50ms subprocess overhead
- **Mitigations:** Bundled binary option, clear error messages
- **Decision:** **ACCEPTED**

---

## Architecture

### Component Diagram

```
                     ┌──────────────────────┐
                     │   Python Developer   │
                     └──────────┬───────────┘
                                │
            ┌───────────────────┴───────────────────┐
            │                                       │
            ▼                                       ▼
 ┌─────────────────────┐               ┌─────────────────────┐
 │    iron_cli_py      │               │     iron_sdk        │
 │   (CLI + Library)   │               │  (Runtime Library)  │
 ├─────────────────────┤               ├─────────────────────┤
 │ NATIVE COMMANDS:    │               │ @protect_agent      │
 │ - init (templates)  │               │ Budget context      │
 │ - config.*          │               │ Safety context      │
 │ - agent.*           │               │ Framework integs    │
 │ - secrets.*         │               └─────────────────────┘
 │ - interactive mode  │
 ├─────────────────────┤
 │ WRAPPER COMMANDS:   │
 │ - token.*  ─────────┼──────────┐
 │ - usage.*  ─────────┼──────────┤
 │ - limits.* ─────────┼──────────┤
 │ - traces.* ─────────┼──────────┤
 │ - auth.*   ─────────┼──────────┤
 │ - health   ─────────┼──────────┤
 └─────────────────────┘          │
                                  │ subprocess call
                                  │ (JSON output parsing)
                                  ▼
                     ┌─────────────────────┐
                     │   iron_cli (Rust)   │
                     │      Binary         │
                     ├─────────────────────┤
                     │ AUTHORITATIVE FOR:  │
                     │ - Token CRUD        │
                     │ - Usage reporting   │
                     │ - Limits management │
                     │ - Traces inspection │
                     │ - Authentication    │
                     │ - Health/version    │
                     └─────────────────────┘
```

### Responsibility Matrix

| Domain | Owner | iron_cli | iron_cli_py |
|--------|-------|----------|-------------|
| token.* | iron_cli | Native | Wrapper |
| usage.* | iron_cli | Native | Wrapper |
| limits.* | iron_cli | Native | Wrapper |
| traces.* | iron_cli | Native | Wrapper |
| auth.* | iron_cli | Native | Wrapper |
| health | iron_cli | Native | Wrapper |
| init | iron_cli_py | - | Native |
| config.* | iron_cli_py | - | Native |
| agent.* | iron_cli_py | - | Native |
| secrets.* | iron_cli_py | - | Native |

### Wrapper Flow

```
User: iron-py token generate --name foo --scope read
                    │
                    ▼
┌─────────────────────────────────────────┐
│           iron_cli_py (Click)           │
│  Parse: --name foo --scope read         │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│           IronCliWrapper                │
│  Convert to: .tokens.generate           │
│              name::foo scope::read      │
│              format::json               │
└─────────────────┬───────────────────────┘
                  │ subprocess.run()
                  ▼
┌─────────────────────────────────────────┐
│           iron_cli (Rust)               │
│  Execute command, return JSON           │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│           iron_cli_py (Rich)            │
│  Format output with colors/tables       │
└─────────────────────────────────────────┘
```

---

## Implementation Details

### Binary Discovery Order

1. `IRON_CLI_PATH` environment variable (explicit override)
2. Bundled binary in pip package (`pip install iron-cli-py[binary]`)
3. System PATH (`which iron-token`)
4. `~/.cargo/bin/iron-token`
5. `/usr/local/bin/iron-token`
6. `/usr/bin/iron-token`

### Installation Options

| Option | Command | Binary Source |
|--------|---------|---------------|
| Bundled | `pip install iron-cli-py[binary]` | Downloaded during install |
| System | `pip install iron-cli-py` | Must be in PATH |
| Cargo | `cargo install iron-cli` | Built from source |
| Manual | Download from releases | Set IRON_CLI_PATH |

### Syntax Translation

| Operation | iron_cli (unilang) | iron_cli_py (Click) |
|-----------|-------------------|---------------------|
| Generate token | `.tokens.generate name::api scope::read` | `token generate --name api --scope read` |
| List tokens | `.tokens.list filter::api` | `token list --filter api` |
| Show usage | `.usage.show start_date::2025-01-01` | `usage show --start-date 2025-01-01` |

---

## Consequences

### Positive

- Single source of truth for operations logic (iron_cli)
- Automatic feature parity (wrapper inherits all features)
- Reduced maintenance burden (one implementation)
- Consistent behavior across CLIs
- Python-native experience preserved (pip, Click, Rich)
- Programmatic API available for scripting

### Negative

- Requires iron_cli binary installed
- Subprocess overhead (~50ms per command)
- Binary distribution complexity

### Mitigations

- Bundled binary option: `pip install iron-cli-py[binary]`
- Clear error messages for missing binary with installation instructions
- Binary discovery with multiple fallback locations
- Batch API for performance-sensitive use cases

---

## Validation Criteria

- [ ] No duplicate token logic in Python codebase
- [ ] Wrapper delegates all operations to iron_cli
- [ ] Native features work without binary (init, config, agent, secrets)
- [ ] Binary discovery works on Linux, macOS, Windows
- [ ] Error messages clear for missing binary
- [ ] Programmatic API functional

---

## References

- [CLI Architecture Guide](../../docs/features/001_cli_architecture.md)
- [iron_cli Specification](../../module/iron_cli/spec.md)
- [iron_cli_py Specification](../../module/iron_cli_py/spec.md)
- [Comprehensive Plan](../../-cli_architecture_plan.md)

---

## Sign-off

**Proposed by:** Development Team
**Date:** 2025-12-08
**Status:** Accepted

This decision eliminates code duplication while preserving Python developer experience.
