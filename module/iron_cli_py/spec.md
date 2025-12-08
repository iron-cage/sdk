# Iron CLI (Python) Specification

**Version:** 0.2.0
**Status:** Initial scaffolding
**Layer:** 6 (Application)
**Date:** 2025-12-07

---

## Scope

**Responsibility:**
Provides Python-based command-line tool for Iron Cage token management and agent control. Alternative to Rust iron-cli binary for Python developers, teams without Rust toolchain, or environments where Python is preferred. Implements same core features as Rust CLI (token generation, configuration management, project initialization) with additional Python-specific capabilities (programmatic library usage, easier contribution).

**In Scope:**
- Token generation (calls Control Panel API for JWT tokens)
- Token validation and inspection
- Configuration management (create, validate, edit iron.toml)
- Project initialization (create boilerplate from templates)
- Agent control commands (start, stop, status)
- Secrets management interface (add, rotate, list secrets)
- Rich terminal output (colors, progress bars, tables)
- Programmatic library usage (import iron_cli_py in Python code)
- Interactive mode for guided setup

**Out of Scope:**
- Control Panel implementation (separate package)
- Agent runtime (use iron-cage package)
- Token generation algorithm (calls Control Panel API)
- Low-level token cryptography (JWT handled by Control Panel)
- Binary distribution (Python package only)
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
- httpx >=0.24.0 (HTTP client for Control Panel API)
- pydantic >=2.0.0 (configuration validation)
- toml >=0.10.0 (TOML file parsing)
- Python 3.8+

**Optional:**
- keyring >=23.0.0 (secure credential storage)
- python-dotenv >=1.0.0 (.env file support)

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

### Module Structure

```
iron_cli_py/
├── __init__.py              # Library exports
├── cli.py                   # Main CLI entry point (Click app)
├── commands/                # Command implementations
│   ├── __init__.py
│   ├── token.py             # Token commands (generate, validate, inspect)
│   ├── config.py            # Config commands (init, validate, edit)
│   ├── init.py              # Project initialization commands
│   ├── agent.py             # Agent control commands
│   └── secrets.py           # Secrets management commands
├── api/                     # API client for Control Panel
│   ├── __init__.py
│   ├── client.py            # HTTPX client wrapper
│   ├── auth.py              # Authentication handling
│   └── models.py            # Pydantic models for API requests/responses
├── config/                  # Configuration management
│   ├── __init__.py
│   ├── manager.py           # ConfigManager class
│   ├── templates/           # Project templates
│   │   ├── langchain.toml
│   │   ├── crewai.toml
│   │   └── autogpt.toml
│   └── schema.py            # Configuration schema
└── utils/                   # Utilities
    ├── __init__.py
    ├── output.py            # Rich output formatting
    ├── validation.py        # Input validation
    └── errors.py            # Custom exceptions
```

---

## Development Status

**Current Phase:** Initial scaffolding (v0.1.0)

**Completed:**
- ✅ Project structure created
- ✅ pyproject.toml configured with CLI dependencies
- ✅ Entry point configured (iron-py command)

**Pending:**
- 📋 Click CLI framework setup
- 📋 Token commands (generate, validate, inspect)
- 📋 Configuration commands (init, validate, edit)
- 📋 Project initialization with templates
- 📋 Agent control commands (start, stop, status)
- 📋 Secrets management commands
- 📋 Control Panel API client (httpx)
- 📋 Rich terminal output (colors, progress, tables)
- 📋 Interactive mode for guided setup
- 📋 Configuration schema validation (Pydantic)
- 📋 Project templates (LangChain, CrewAI, AutoGPT)
- 📋 Error handling with clear messages
- 📋 Unit tests with pytest
- 📋 Integration tests with mock Control Panel

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
| 0.2.0 | 2025-12-07 | Added Deployment Context - clarify API endpoint differences between Pilot and Production modes |
| 0.1.0 | 2025-12-07 | Initial scaffolding specification |

**Next Milestone:** Implement token generate command with Control Panel API client
