# Iron CLI (Python)

Python command-line tool for Iron Cage token management and agent control.

### Scope

**Responsibilities:**
Provides a Python-based command-line alternative to the Rust iron_cli binary. Enables token generation, configuration management, project initialization, and agent control for Python developers and environments without Rust toolchain.

**In Scope:**
- Token commands (generate, validate, inspect)
- Configuration commands (init, validate, edit)
- Project initialization from templates (LangChain, CrewAI, AutoGPT)
- Agent control commands (start, stop, status)
- Secrets management commands (add, rotate, list)
- Rich terminal output (colors, progress bars, tables)
- Programmatic Python API for library usage

**Out of Scope:**
- REST API server (see iron_api)
- Token generation backend logic (see iron_token_manager)
- Budget enforcement (see iron_cost)
- Rust-based CLI (see iron_cli)
- Python SDK decorators (see iron_sdk)
- Framework integrations (see iron_sdk)

---

## Overview

Iron CLI (Python) provides a Python-based alternative to the Rust iron-cli binary. Designed for Python developers, teams without Rust toolchain, or environments where Python is preferred. Implements token generation, configuration management, project initialization, and agent control.

**Key Features:**
- **Token Management** - Generate, validate, inspect JWT tokens
- **Configuration** - Create and validate iron.toml files
- **Project Init** - Bootstrap projects from templates (LangChain, CrewAI, AutoGPT)
- **Agent Control** - Start, stop, monitor agents via Control Panel API
- **Secrets Management** - Add, rotate, list API keys and secrets
- **Rich Output** - Colors, progress bars, tables for better UX
- **Programmatic API** - Use as library in Python code

---

## Quick Start

```bash
# Install CLI
pip install iron-cli-py

# Generate token for project
iron-py token generate --project my-app --output token.json

# Initialize new project from template
iron-py init --template langchain --name my-agent

# Validate configuration
iron-py config validate --file iron.toml

# Start agent
iron-py agent start --config iron.toml
```

---

## Installation

```bash
# Install CLI
pip install iron-cli-py

# With secure credential storage
pip install iron-cli-py[keyring]

# With .env file support
pip install iron-cli-py[dotenv]
```

**Requirements:**
- Python 3.8+
- Control Panel API access for token generation

---

## Available Commands

### Token Commands

```bash
iron-py token generate --project my-app --output token.json
iron-py token validate --file token.json
iron-py token inspect --file token.json
```

### Configuration Commands

```bash
iron-py config init --template langchain
iron-py config validate --file iron.toml
iron-py config edit --file iron.toml
```

### Project Initialization

```bash
iron-py init --template langchain --name my-agent
iron-py init --template crewai --name my-crew
iron-py init --template autogpt --name my-autogpt
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

## Programmatic Usage

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

## Documentation

- **Specification:** See `spec.md` for complete requirements
- **Configuration Guide:** See iron.toml templates in `config/templates/`
- **Control Panel API:** See Control Panel documentation

---

## Development Status

**Current Phase:** Initial scaffolding

**Pending Implementation:**
- Click CLI framework setup
- Token commands implementation
- Configuration commands
- Project initialization with templates
- Agent control via Control Panel API
- Secrets management
- Rich terminal output
- Interactive mode

---

## License

Apache-2.0 - See `license` file for details
