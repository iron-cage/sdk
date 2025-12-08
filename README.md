# Iron Runtime

Control Panel and Agent Runtime for Iron Cage AI agent management.

## Overview

**iron_runtime** is one of two repositories in the Iron Cage project:
- **iron_runtime** (this repo) - Control Panel, Agent Runtime, runtime services
- **iron_cage** - Sandboxing, CLI tools, supporting infrastructure

This repository contains the core runtime modules for AI agent management including safety controls, cost tracking, reliability patterns, and the Control Panel web application.

## Repository Architecture

See [`docs/repository_architecture.md`](docs/repository_architecture.md) for complete documentation of the two-repository split.

## Modules

### Infrastructure Layer
- **iron_state** - State management for agent execution tracking

### Feature Layer
- **iron_safety** - PII detection and redaction
- **iron_reliability** - Circuit breaker patterns
- **iron_secrets** - Secrets management with encrypted storage
- **iron_token_manager** - JWT authentication and token management

### Specialized Layer
- **iron_lang** - LLM protocol integration

### Integration Layer
- **iron_api** - REST API + WebSocket server
- **iron_runtime** - Agent orchestration and PyO3 bridge

### Frontend
- **iron_dashboard** - Vue 3 Control Panel UI (in `frontend/dashboard/`)

### Python Packages
- **iron_sdk** - Pythonic SDK wrapper for Agent Runtime (in `python/iron_sdk/`)
- **iron_examples** - Example agent implementations (in `python/iron_examples/`)

### Production Database Schema (Spec-Only)
- **iron_control_store** - PostgreSQL schema for production Control Panel (specification in `docs/`, no implementation in this repo)

## External Dependencies

Foundation modules consumed from crates.io:
- **iron_types** - Shared types and data structures
- **iron_cost** - Budget tracking and cost enforcement
- **iron_telemetry** - Logging and tracing infrastructure

## Building

```bash
# Build all Rust modules
cargo build --release

# Run tests
cargo nextest run --all-features

# Build Control Panel UI
cd frontend/dashboard
npm install
npm run build
```

## Status

**Version:** 0.1.0
**Status:** Migration in progress from iron_cage monorepo

## License

Apache-2.0
