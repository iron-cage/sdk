# Contributing to Iron Cage

**Note:** This guide is for platform contributors developing Iron Cage itself. If you're a Python developer using Iron Cage to protect your agents, see [Getting Started](docs/getting_started.md) instead.

---

## Prerequisites

- **Rust:** 1.75+ (`rustup update`)
- **Python:** 3.11+ (`python --version`)
- **Node.js:** 18+ (`node --version`)
- **Git:** Latest stable
- **PostgreSQL:** 14+ (for Control Panel development)

---

## Development Setup

### 1. Clone Repository

```bash
git clone https://github.com/iron-cage/iron_runtime.git
cd iron_runtime
```

### 2. Build All Modules

```bash
# Build all Rust crates
cargo build --release --workspace

# Build Control Panel UI
cd module/iron_dashboard
npm install
npm run build
cd ../..
```

### 3. Install Python SDK in Dev Mode

```bash
cd module/iron_sdk
pip install -e .[dev,all]  # Include dev tools and all framework integrations
cd ../..
```

### 4. Run Full Test Suite

```bash
# Rust unit tests
cargo nextest run --all-features

# Rust doctests
cargo test --doc --all-features

# Clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Python SDK tests
cd module/iron_sdk
pytest
cd ../..
```

---

## Module Structure

Iron Cage is organized into modules by layer:

**Foundation Layer (crates.io published):**
- `iron_types` - Shared types, Result types, error definitions
- `iron_cost` - Budget tracking, token counting
- `iron_telemetry` - Logging, tracing, metrics

**Infrastructure Layer:**
- `iron_runtime_state` - Agent execution state (SQLite)

**Feature Layer:**
- `iron_safety` - PII detection, prompt injection blocking
- `iron_reliability` - Circuit breakers, retry logic
- `iron_secrets` - Encrypted secrets storage
- `iron_token_manager` - JWT token management, user auth

**Integration Layer:**
- `iron_control_api` - REST API + WebSocket server
- `iron_runtime` - Agent orchestrator + PyO3 bridge

**Application Layer:**
- `iron_cli` - Command-line interface (Rust, authoritative)
- `iron_cli_py` - Python CLI wrapper
- `iron_sdk` - Pythonic SDK (Python)
- `iron_testing` - Test utilities

**Frontend Layer:**
- `iron_dashboard` - Control Panel web UI (Vue.js)

---

## Packaging Flow

Understanding how code becomes user-installable packages:

```
Developer writes code:
  [Rust] iron_runtime crate (src/)
    ↓
  maturin build (PyO3 compilation)
    ↓
  [Wheel] iron-cage (PyPI)  ← Contains compiled Rust binary
    ↓
  Declared as dependency in iron-sdk's pyproject.toml
    ↓
  [Python] iron-sdk (PyPI)  ← User-facing package
    ↓
  User runs: pip install iron-sdk
    ↓
  User imports: from iron_sdk import protect_agent
```

**Key Insight:**
- **End users see:** Only iron-sdk (never know about iron-cage or iron_runtime)
- **Contributors work with:** All layers (iron_runtime crate → iron-cage wheel → iron-sdk package)

---

## Building iron-cage Wheel (PyO3 Package)

The iron-cage package is built from iron_runtime using maturin:

```bash
cd module/iron_runtime

# Install maturin
pip install maturin

# Build wheel
maturin build --release

# Built wheel appears in: target/wheels/iron_cage-*.whl
```

This wheel is then published to PyPI and automatically installed when users run `pip install iron-sdk`.

---

## Running the Control Panel Locally

```bash
# 1. Start PostgreSQL
docker run -d -p 5432:5432 \
  -e POSTGRES_PASSWORD=dev \
  -e POSTGRES_DB=iron_control \
  postgres:14

# 2. Run migrations
cd module/iron_control_api
sqlx migrate run

# 3. Start API server
cargo run --bin iron_control_api_server

# 4. Start dashboard (separate terminal)
cd module/iron_dashboard
npm run dev

# Open http://localhost:3000
```

---

## Testing Guidelines

### Unit Tests

Place tests in the same file as the code:

```rust
#[cfg(test)]
mod tests
{
  use super::*;

  #[test]
  fn test_budget_calculation()
  {
    // Test implementation
  }
}
```

### Integration Tests

Place in `tests/` directory of the relevant crate:

```
module/iron_runtime/
  tests/
    budget_integration_test.rs
```

### Python Tests

Use pytest for SDK tests:

```bash
cd module/iron_sdk
pytest tests/ -v
```

---

## Code Style

### Rust

- **DO NOT** use `cargo fmt` - the project uses custom formatting rules
- Use 2-space indentation
- Follow conventions in existing code
- Run `cargo clippy` before committing

### Python

- Follow PEP 8
- Use type hints
- Document all public APIs

### Documentation

- All public APIs must have doc comments
- Use examples in doc comments where helpful
- Keep docs up-to-date with code changes

---

## Submitting Changes

### 1. Create Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Changes

- Write code following style guidelines
- Add tests for new functionality
- Update documentation

### 3. Run Full Test Suite

```bash
# Rust tests (Level 3)
cargo nextest run --all-features
cargo test --doc --all-features
cargo clippy --all-targets --all-features -- -D warnings

# Python tests
cd module/iron_sdk && pytest
```

### 4. Commit Changes

```bash
git add .
git commit -m "feat: Add budget allocation API"
```

Use conventional commit format:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `test:` - Test changes
- `refactor:` - Code refactoring

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Create pull request on GitHub with:
- Clear description of changes
- References to related issues
- Test results

---

## Architecture Resources

- [Architecture Overview](docs/architecture/readme.md)
- [Layer Model](docs/architecture/002_layer_model.md)
- [Data Flow](docs/architecture/004_data_flow.md)
- [Budget Control Protocol](docs/architecture/006_budget_control_protocol.md)
- [Vocabulary](docs/vocabulary.md)

---

## Getting Help

- **Documentation:** [docs/readme.md](docs/readme.md)
- **Module Specs:** Each module has `spec.md` defining responsibility
- **Questions:** Open a discussion on GitHub

---

*Last Updated: 2025-12-10*
