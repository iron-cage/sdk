# CLI Architecture

**Purpose:** Overview of Iron Cage CLI tools and their responsibilities.

**Package Distribution:** iron_cli + iron_cli_py (Package 5: CLI Tools)

**See:** [ADR-005](../decisions/adr_005_cli_architecture.md) for architecture decision.

---

## Overview

Iron Cage provides two CLI tools serving different audiences with complementary responsibilities:

| Tool | Language | Audience | Distribution | Status |
|------|----------|----------|--------------|--------|
| **iron_cli** | Rust | Operations teams, automation | Binary | Production (288 tests) |
| **iron_cli_py** | Python | Python developers | pip | Architecture defined |

---

## Architecture Pattern

iron_cli_py uses a **wrapper pattern** for operations commands, delegating to iron_cli binary while providing Python-native developer experience features.

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

---

## Responsibility Matrix

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

---

## Command Categories

### Native Commands (Python Implementation)

These commands are implemented directly in Python, providing Python-specific developer experience:

| Command | Description | Example |
|---------|-------------|---------|
| `iron-py init` | Bootstrap project from templates | `iron-py init --template langchain` |
| `iron-py config init` | Create iron.toml configuration | `iron-py config init` |
| `iron-py config validate` | Validate configuration file | `iron-py config validate --file iron.toml` |
| `iron-py agent start` | Start protected agent | `iron-py agent start --config iron.toml` |
| `iron-py agent stop` | Stop running agent | `iron-py agent stop --agent-id abc123` |
| `iron-py secrets add` | Add secret to secure storage | `iron-py secrets add OPENAI_API_KEY` |

**Why Native:**
- Project templates (LangChain, CrewAI, AutoGPT) are Python-ecosystem specific
- Configuration management benefits from Python's TOML libraries
- Agent control needs to interact with Python agent processes
- Secrets management integrates with Python keyring libraries

### Wrapper Commands (Delegate to iron_cli)

These commands delegate to iron_cli binary, ensuring single source of truth:

| Command | Description | Example |
|---------|-------------|---------|
| `iron-py token generate` | Generate API token | `iron-py token generate --project my-app` |
| `iron-py token validate` | Validate token | `iron-py token validate --file token.json` |
| `iron-py usage show` | Show usage statistics | `iron-py usage show --start-date 2025-01-01` |
| `iron-py limits list` | List budget limits | `iron-py limits list` |
| `iron-py traces list` | List request traces | `iron-py traces list --limit 20` |
| `iron-py auth login` | Authenticate with API | `iron-py auth login` |

**Why Wrapper:**
- Token algorithms exist in iron_cli (288 tests)
- Eliminates code duplication between Rust and Python
- Automatic feature parity (wrapper inherits all iron_cli features)
- Single source of truth for operations logic

---

## Syntax Translation

iron_cli_py translates Click-style arguments to iron_cli's unilang syntax:

| Operation | iron_cli (unilang) | iron_cli_py (Click) |
|-----------|-------------------|---------------------|
| Generate token | `.tokens.generate name::api scope::read` | `token generate --name api --scope read` |
| List tokens | `.tokens.list filter::api` | `token list --filter api` |
| Show usage | `.usage.show start_date::2025-01-01` | `usage show --start-date 2025-01-01` |

---

## Binary Discovery

iron_cli_py locates the iron_cli binary using this search order:

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

---

## Design Rationale

### Why Two CLIs?

| Concern | Single CLI | Two CLIs (Current) |
|---------|------------|-------------------|
| Target audience | One-size-fits-all | Optimized per audience |
| Distribution | Complex (Rust + Python) | Simple per language |
| Syntax | Compromise | Idiomatic per language |
| Dependencies | Heavy | Minimal per tool |
| Learning curve | Higher | Lower per audience |

### Why Wrapper Pattern?

| Concern | Reimplementation | Wrapper (Current) |
|---------|------------------|-------------------|
| Code duplication | HIGH (Rust + Python) | NONE |
| Maintenance | 2x codebases | Single source |
| Feature parity | Manual sync | Automatic |
| Bug propagation | Both impacted | Fix once |
| Testing burden | 2x tests | Single test suite |

---

## Related Documentation

- **iron_cli Specification:** [module/iron_cli/spec.md](../../module/iron_cli/spec.md)
- **iron_cli_py Specification:** [module/iron_cli_py/spec.md](../../module/iron_cli_py/spec.md)
- **Architecture Decision:** [ADR-002](../../pilot/decisions/002-cli-architecture.md)
- **iron_sdk Overview:** [module/iron_sdk/readme.md](../../module/iron_sdk/readme.md)

---

**Last Updated:** 2025-12-08
