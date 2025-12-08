# Iron Runtime

Control Panel and Agent Runtime for Iron Cage AI agent management.

## Overview

**iron_runtime** is one of two repositories in the Iron Cage project:
- **iron_runtime** (this repo) - Control Panel, Agent Runtime, runtime services
- **iron_cage** - Sandboxing only (OS-level isolation via Landlock, seccomp, rlimits)

This repository contains the core runtime modules for AI agent management including safety controls, cost tracking, reliability patterns, and the Control Panel web application.

## Repository Architecture

See [`docs/repository_architecture.md`](docs/repository_architecture.md) for complete documentation of the two-repository split.

## Modules

All modules are located in the `module/` directory.

| Module | Responsibility | Language |
|--------|----------------|----------|
| **iron_types** | Shared types, data structures, API contracts | Rust |
| **iron_cost** | Budget tracking, cost calculation, spending limits | Rust |
| **iron_telemetry** | Logging, tracing, metrics collection | Rust |
| **iron_state** | Agent execution state management | Rust |
| **iron_safety** | PII detection and redaction | Rust |
| **iron_reliability** | Circuit breaker, retry patterns, fault tolerance | Rust |
| **iron_secrets** | Encrypted secrets storage, access control | Rust |
| **iron_token_manager** | API token generation, JWT auth, rate limiting | Rust |
| **iron_lang** | LLM protocol integration, provider routing | Rust |
| **iron_api** | REST API server, WebSocket endpoints | Rust |
| **iron_runtime** | Agent orchestration, PyO3 Python bridge | Rust |
| **iron_cli** | Command-line interface for API operations | Rust |
| **iron_cli_py** | Python bindings for CLI | Python |
| **iron_sdk** | Pythonic SDK for Agent Runtime integration | Python |
| **iron_examples** | Example agent implementations | Python |
| **iron_testing** | Test utilities, fixtures, mock runtime | Python |
| **iron_dashboard** | Control Panel web UI | Vue.js |

### Layer Organization

```
Foundation:     iron_types, iron_cost, iron_telemetry
Infrastructure: iron_state
Feature:        iron_safety, iron_reliability, iron_secrets, iron_token_manager
Specialized:    iron_lang
Integration:    iron_api, iron_runtime
Application:    iron_cli, iron_cli_py, iron_sdk, iron_examples, iron_testing
Frontend:       iron_dashboard
```

## Building

```bash
# Build all Rust modules
cargo build --release

# Run tests
cargo nextest run --all-features

# Build Control Panel UI
cd module/iron_dashboard
npm install
npm run build

# Install Python SDK
cd module/iron_sdk
pip install -e .
```

## Status

**Version:** 0.1.0
**Status:** Migration in progress from iron_cage monorepo

## License

Apache-2.0
