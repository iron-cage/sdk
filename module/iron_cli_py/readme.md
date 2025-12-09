# Iron CLI (Python)

Python command-line tool for Iron Cage with wrapper architecture for operations and native Python for developer experience.

### Scope

**Responsibilities:**
Provides Python-based command-line tool for Iron Cage with two implementation patterns:
1. **Developer Experience** (native Python): Project init, config, agent control, secrets
2. **Operations** (wrapper to iron_cli): Token, usage, limits, traces via iron_cli binary

**See:** [ADR-002](../../pilot/decisions/002-cli-architecture.md) for architecture decision.

**In Scope:**

*Native Commands (Python implementation):*
- Project initialization from templates (LangChain, CrewAI, AutoGPT)
- Configuration management (create, validate, edit iron.toml)
- Agent control commands (start, stop, status)
- Secrets management interface (add, rotate, list secrets)
- Interactive mode for guided setup
- Rich terminal output (colors, progress bars, tables)
- Programmatic library usage (import iron_cli_py)

*Wrapper Commands (delegate to iron_cli binary):*
- Token operations (generate, list, rotate, revoke, validate, inspect)
- Usage reporting (show, by_project, by_provider, export)
- Limits management (list, get, create, update, delete)
- Traces inspection (list, get, export)
- Authentication (login, refresh, logout)
- Health check and version

**Out of Scope:**
- Token generation algorithm (delegated to iron_cli)
- Usage calculation logic (delegated to iron_cli)
- Limits enforcement logic (delegated to iron_cli)
- REST API server (see iron_control_api)
- Python SDK decorators (see iron_sdk)
- Framework integrations (see iron_sdk)

---

## Architecture

iron_cli_py uses a **wrapper pattern** for operations commands, delegating to iron_cli binary while providing Python-native developer experience features.

```
                    ┌──────────────────────┐
                    │   Python Developer   │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │    iron_cli_py      │
                    │   (CLI + Library)   │
                    ├─────────────────────┤
                    │ NATIVE COMMANDS:    │
                    │ - init (templates)  │
                    │ - config.*          │
                    │ - agent.*           │
                    │ - secrets.*         │
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
                                                     ▼
                                        ┌─────────────────────┐
                                        │   iron_cli (Rust)   │
                                        │      Binary         │
                                        └─────────────────────┘
```

### Command Categories

| Command Pattern | Implementation | Description |
|-----------------|----------------|-------------|
| `init`, `template *` | Native Python | Project scaffolding |
| `config *` | Native Python | Configuration management |
| `agent *` | Native Python | Agent lifecycle |
| `secrets *` | Native Python | Credential management |
| `token *` | Wrapper | Delegates to iron_cli |
| `usage *` | Wrapper | Delegates to iron_cli |
| `limits *` | Wrapper | Delegates to iron_cli |
| `traces *` | Wrapper | Delegates to iron_cli |
| `auth *` | Wrapper | Delegates to iron_cli |
| `health`, `version` | Wrapper | Delegates to iron_cli |

---

## Quick Start

```bash
# Install CLI with bundled binary (recommended)
pip install iron-cli-py[binary]

# Or install CLI only (requires iron_cli in PATH)
pip install iron-cli-py

# Initialize new project from template (NATIVE)
iron-py init --template langchain --name my-agent

# Validate configuration (NATIVE)
iron-py config validate --file iron.toml

# Generate token (WRAPPER → iron_cli)
iron-py token generate --project my-app --output token.json

# Start agent (NATIVE)
iron-py agent start --config iron.toml
```

---

## Installation

```bash
# Option 1: Bundled binary (recommended)
pip install iron-cli-py[binary]

# Option 2: System binary (requires iron_cli in PATH)
pip install iron-cli-py
cargo install iron-cli  # or download from releases

# Option 3: Custom path
export IRON_CLI_PATH=/path/to/iron-token
pip install iron-cli-py

# With secure credential storage
pip install iron-cli-py[keyring]

# With .env file support
pip install iron-cli-py[dotenv]
```

**Requirements:**
- Python 3.8+
- iron_cli binary (for wrapper commands)

**Binary Discovery Order:**
1. `IRON_CLI_PATH` environment variable
2. Bundled binary (`iron_cli_py/bin/iron-token`)
3. System PATH (`which iron-token`)
4. `~/.cargo/bin/iron-token`
5. `/usr/local/bin/iron-token`
6. `/usr/bin/iron-token`

---

## Native Commands

### Project Initialization

```bash
iron-py init --template langchain --name my-agent
iron-py init --template crewai --name my-crew
iron-py init --template autogpt --name my-autogpt
```

### Configuration Management

```bash
iron-py config init --template langchain
iron-py config validate --file iron.toml
iron-py config edit --file iron.toml
```

### Agent Control

```bash
iron-py agent start --config iron.toml
iron-py agent stop --agent-id abc123
iron-py agent status --agent-id abc123
```

### Secrets Management

```bash
iron-py secrets add OPENAI_API_KEY --value sk-...
iron-py secrets rotate OPENAI_API_KEY
iron-py secrets list
```

---

## Wrapper Commands

These commands delegate to iron_cli binary with Click-to-unilang syntax translation:

### Token Operations

```bash
iron-py token generate --project my-app --output token.json
iron-py token validate --file token.json
iron-py token inspect --file token.json
iron-py token list --filter api
iron-py token rotate --name my-token
iron-py token revoke --name my-token
```

### Usage Reporting

```bash
iron-py usage show --start-date 2025-01-01
iron-py usage by-project --project-id my-project
iron-py usage by-provider --provider openai
iron-py usage export --output usage.csv --format csv
```

### Limits Management

```bash
iron-py limits list
iron-py limits get --limit-id lim_tokens
iron-py limits create --resource-type tokens --limit-value 100000
iron-py limits update --limit-id lim_tokens --limit-value 200000
iron-py limits delete --limit-id lim_tokens
```

### Traces Inspection

```bash
iron-py traces list --filter error --limit 50
iron-py traces get --trace-id trace-123
iron-py traces export --output traces.json --format json
```

---

## Programmatic Usage

```python
from iron_cli_py import TokenGenerator, ConfigManager, ProjectInitializer

# Token generation (uses wrapper internally)
generator = TokenGenerator(api_url="https://control-panel.company.com")
token = generator.create_token(project="my-app")

# Configuration management (native)
config = ConfigManager(file_path="iron.toml")
config.validate()
config.set("budget.max_usd", 50.0)
config.save()

# Project initialization (native)
initializer = ProjectInitializer(template="langchain")
initializer.create_project(name="my-agent", path="./my-agent")
```

---

## Documentation

- **Specification:** See `spec.md` for complete requirements
- **Architecture Decision:** See [ADR-002](../../pilot/decisions/002-cli-architecture.md)
- **CLI Architecture Guide:** See [docs/features/001_cli_architecture.md](../../docs/features/001_cli_architecture.md)
- **iron_cli (authoritative):** See [module/iron_cli/readme.md](../iron_cli/readme.md)

---

## Development Status

**Current Phase:** Architecture defined (v0.3.0)

**Completed:**
- Project structure created
- pyproject.toml configured with CLI dependencies
- Entry point configured (iron-py command)
- Wrapper architecture defined (ADR-002)
- Command routing strategy defined

**Pending (Wrapper Commands):**
- Binary discovery (discovery.py)
- Wrapper implementation (wrapper.py)
- Token commands via wrapper
- Usage commands via wrapper
- Limits commands via wrapper
- Traces commands via wrapper
- Auth commands via wrapper
- Health command via wrapper

**Pending (Native Commands):**
- Project initialization with templates
- Configuration commands (init, validate, edit)
- Agent control commands (start, stop, status)
- Secrets management commands
- Interactive mode for guided setup

---

## License

Apache-2.0 - See `license` file for details
