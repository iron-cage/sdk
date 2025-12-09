# Iron CLI (Python) Specification

**Version:** 0.3.0
**Status:** Architecture defined
**Layer:** 6 (Application)
**Date:** 2025-12-08

---

## Scope

**Responsibility:**
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
- Control Panel implementation (separate package)
- Agent runtime (use iron-cage package)
- GUI interface (CLI only)

## Deployment Context

Iron Cage supports two deployment modes. This module's behavior differs in API endpoint configuration.

**See:** [docs/deployment_packages.md](../../docs/deployment_packages.md) § Deployment Modes for deployment architecture.

**This Module (iron_cli_py):**

**Pilot Mode:**
- Runs on developer machine (demo/testing environment)
- API client connects to localhost Control Panel (http://localhost:8080/api)
- Token generation for local demo agents
- No authentication required (single-user demo environment)

**Production Mode:**
- Runs on developer machines (Agent Runtime package)
- API client connects to cloud Control Panel (https://control.company.com/api)
- Token generation for distributed production agents
- Authentication required (API keys, OAuth tokens)
- HTTPS/TLS enforced for all API communication

---

## Dependencies

**Required:**
- click >=8.0.0 (CLI framework)
- rich >=13.0.0 (terminal formatting)
- httpx >=0.24.0 (HTTP client for native commands)
- pydantic >=2.0.0 (configuration validation)
- toml >=0.10.0 (TOML file parsing)
- Python 3.8+
- **iron_cli binary** (for wrapper commands - see Binary Dependency section)

**Optional:**
- keyring >=23.0.0 (secure credential storage)
- python-dotenv >=1.0.0 (.env file support)
- iron-cli-binary >=0.1.0 (bundled iron_cli binary)

---

## Binary Dependency

iron_cli_py requires the iron_cli binary for operations commands (token, usage, limits, traces, auth, health). Native commands (init, config, agent, secrets) work without binary.

### Installation Options

```bash
# Option 1: Bundled binary (recommended)
pip install iron-cli-py[binary]

# Option 2: System binary (requires iron_cli in PATH)
pip install iron-cli-py
cargo install iron-cli  # or download from releases

# Option 3: Custom path
export IRON_CLI_PATH=/path/to/iron-token
pip install iron-cli-py
```

### Discovery Order

1. `IRON_CLI_PATH` environment variable
2. Bundled binary (`iron_cli_py/bin/iron-token`)
3. System PATH (`which iron-token`)
4. `~/.cargo/bin/iron-token`
5. `/usr/local/bin/iron-token`
6. `/usr/bin/iron-token`

### Error Handling

If binary not found for wrapper commands:
1. Displays searched locations
2. Provides installation instructions
3. Exits with clear error message

---

## API Contract

### CLI Commands

```bash
# Token management
iron-py token generate --project my-app --output token.json
iron-py token validate --file token.json
iron-py token inspect --file token.json

# Configuration
iron-py config init --template langchain
iron-py config validate --file iron.toml
iron-py config edit --file iron.toml

# Project initialization
iron-py init --template langchain --name my-agent
iron-py init --template crewai --name my-crew

# Agent control (connects to Control Panel)
iron-py agent start --config iron.toml
iron-py agent stop --agent-id abc123
iron-py agent status --agent-id abc123

# Secrets management
iron-py secrets add OPENAI_API_KEY --value sk-...
iron-py secrets rotate OPENAI_API_KEY
iron-py secrets list
```

### Programmatic API

```python
from iron_cli_py import TokenGenerator, ConfigManager, ProjectInitializer

# Token generation
generator = TokenGenerator(api_url="https://control-panel.company.com")
token = generator.create_token(project="my-app")

# Configuration management
config = ConfigManager(file_path="iron.toml")
config.validate()
config.set("budget.max_usd", 50.0)
config.save()

# Project initialization
initializer = ProjectInitializer(template="langchain")
initializer.create_project(name="my-agent", path="./my-agent")
```

---

## Architecture

### Wrapper Pattern

iron_cli_py uses a **wrapper pattern** for operations commands, delegating to iron_cli binary while providing Python-native developer experience features.

```
┌─────────────────────────────────────────────────────┐
│                  iron_cli_py                         │
├─────────────────────────────────────────────────────┤
│  NATIVE (Python):          WRAPPER (via iron_cli):  │
│  - init (templates)        - token.*                │
│  - config.*                - usage.*                │
│  - agent.*                 - limits.*               │
│  - secrets.*               - traces.*               │
│  - interactive mode        - auth.*                 │
│  - programmatic API        - health                 │
└─────────────────────────────┬───────────────────────┘
                              │ subprocess
                              ▼
                   ┌─────────────────────┐
                   │  iron_cli (binary)  │
                   └─────────────────────┘
```

### Command Routing

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

### Module Structure

```
iron_cli_py/
├── __init__.py              # Library exports + high-level API
├── cli.py                   # Main CLI entry point (Click app)
├── discovery.py             # Binary discovery logic
├── wrapper.py               # IronCliWrapper class
├── errors.py                # Custom exceptions
├── models.py                # Data models (Token, Usage, etc.)
│
├── commands/                # CLI command groups
│   ├── __init__.py
│   ├── token.py             # Token commands (WRAPPER)
│   ├── usage.py             # Usage commands (WRAPPER)
│   ├── limits.py            # Limits commands (WRAPPER)
│   ├── traces.py            # Traces commands (WRAPPER)
│   ├── auth.py              # Auth commands (WRAPPER)
│   ├── health.py            # Health command (WRAPPER)
│   ├── init.py              # Init command (NATIVE)
│   ├── config.py            # Config commands (NATIVE)
│   ├── agent.py             # Agent commands (NATIVE)
│   └── secrets.py           # Secrets commands (NATIVE)
│
├── api/                     # High-level API classes
│   ├── __init__.py
│   ├── tokens.py            # TokenGenerator class
│   ├── usage.py             # UsageClient class
│   ├── limits.py            # LimitsClient class
│   └── traces.py            # TracesClient class
│
├── config/                  # Configuration management (NATIVE)
│   ├── __init__.py
│   ├── manager.py           # ConfigManager class
│   ├── schema.py            # Pydantic schema
│   └── templates/           # Project templates
│       ├── langchain.toml
│       ├── crewai.toml
│       └── autogpt.toml
│
├── init/                    # Project initialization (NATIVE)
│   ├── __init__.py
│   ├── initializer.py       # ProjectInitializer class
│   └── templates/           # Project scaffolding
│
├── agent/                   # Agent control (NATIVE)
│   ├── __init__.py
│   └── controller.py        # AgentController class
│
├── secrets/                 # Secrets management (NATIVE)
│   ├── __init__.py
│   └── manager.py           # SecretsManager class
│
├── output/                  # Rich output formatting
│   ├── __init__.py
│   └── formatters.py        # Table, JSON formatters
│
└── bin/                     # Bundled binary (optional extra)
    └── .gitkeep             # Populated during pip install [binary]
```

---

## Development Status

**Current Phase:** Architecture defined (v0.3.0)

**Completed:**
- ✅ Project structure created
- ✅ pyproject.toml configured with CLI dependencies
- ✅ Entry point configured (iron-py command)
- ✅ Wrapper architecture defined (ADR-002)
- ✅ Command routing strategy defined

**Pending (Wrapper Commands):**
- 📋 Binary discovery (discovery.py)
- 📋 Wrapper implementation (wrapper.py)
- 📋 Token commands via wrapper
- 📋 Usage commands via wrapper
- 📋 Limits commands via wrapper
- 📋 Traces commands via wrapper
- 📋 Auth commands via wrapper
- 📋 Health command via wrapper

**Pending (Native Commands):**
- 📋 Project initialization with templates
- 📋 Configuration commands (init, validate, edit)
- 📋 Agent control commands (start, stop, status)
- 📋 Secrets management commands
- 📋 Interactive mode for guided setup

**Pending (Infrastructure):**
- 📋 Rich terminal output (colors, progress, tables)
- 📋 Error handling with clear messages
- 📋 Unit tests with pytest
- 📋 Integration tests

---

## Non-Functional Requirements

### NFR1: Usability
- **NFR1.1:** All commands must have --help with clear descriptions
- **NFR1.2:** Error messages must be actionable (tell user how to fix)
- **NFR1.3:** Rich output (colors, progress bars) for better UX

### NFR2: Compatibility
- **NFR2.1:** Feature parity with Rust iron-cli for core commands
- **NFR2.2:** Control Panel API compatibility (same endpoints as Rust CLI)
- **NFR2.3:** Configuration file compatibility (same iron.toml format)

### NFR3: Performance
- **NFR3.1:** Command startup time <500ms for simple commands
- **NFR3.2:** Token generation <2s (network dependent)
- **NFR3.3:** Configuration validation <100ms

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 0.3.0 | 2025-12-08 | Wrapper architecture: operations commands delegate to iron_cli binary, native Python for dev experience (ADR-002) |
| 0.2.0 | 2025-12-07 | Added Deployment Context - clarify API endpoint differences between Pilot and Production modes |
| 0.1.0 | 2025-12-07 | Initial scaffolding specification |

**Next Milestone:** Implement binary discovery and wrapper infrastructure
