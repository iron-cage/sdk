# iron_cli

Command-line interface for LLM token management using unilang framework.

### Scope

**Responsibilities:**
Provides command-line access to all Iron Cage token management features using the unilang keyword::value syntax. Implements hexagonal architecture with pure handlers for business logic, async adapters for I/O, and multiple output formatters for terminal display.

**In Scope:**
- Token CRUD commands (generate, list, get, rotate, revoke)
- Authentication commands (login, refresh, logout)
- Usage reporting commands (show, by_project, by_provider, export)
- Limits management commands (list, get, create, update, delete)
- Traces inspection commands (list, get, export)
- Health check and version commands
- Unilang keyword::value command syntax
- Multiple output formats (table, expanded, json, yaml)
- Hierarchical configuration (CLI, env, local, global, defaults)

**Out of Scope:**
- REST API server implementation (see iron_control_api)
- Token generation logic (see iron_token_manager)
- Budget enforcement logic (see iron_cost)
- GUI/dashboard interface (see iron_dashboard)
- Python CLI wrapper (see iron_cli_py)
- Framework integrations (see iron_sdk)

---

## Authoritative Role

iron_cli is the **authoritative source** for operations commands. The Python CLI (iron_cli_py) wraps this binary rather than reimplementing operations logic.

**See:** [ADR-002](../../pilot/decisions/002-cli-architecture.md) for architecture decision.

**Domains owned by iron_cli:**

| Domain | Commands | Description |
|--------|----------|-------------|
| **Authentication** | login, refresh, logout | JWT token management |
| **Tokens** | generate, list, get, rotate, revoke, validate, inspect | API token CRUD |
| **Usage** | show, by_project, by_provider, export | Usage analytics |
| **Limits** | list, get, create, update, delete | Budget limits CRUD |
| **Traces** | list, get, export | Request trace inspection |
| **Health** | health, version | System status |

**Wrapper relationship:**
```
iron_cli_py (Python)         iron_cli (Rust)
     │                            │
     │ WRAPPER COMMANDS:          │ NATIVE IMPLEMENTATION:
     │ token.*  ─────────────────▶│ Token CRUD
     │ usage.*  ─────────────────▶│ Usage reporting
     │ limits.* ─────────────────▶│ Limits management
     │ traces.* ─────────────────▶│ Traces inspection
     │ auth.*   ─────────────────▶│ Authentication
     │ health   ─────────────────▶│ Health/version
     │                            │
     │ NATIVE COMMANDS:           │
     │ init, config, agent,       │
     │ secrets (NOT delegated)    │
     └────────────────────────────┘
```

---

## Architecture

**Current Status:** Unilang migration in progress (Phases 1-6 complete)

### Layers (Hexagonal Architecture)

```
CLI (unilang) → Adapter (async I/O) → Handler (pure logic) → Formatter (output)
     ↓               ↓                      ↓                      ↓
VerifiedCommand   Services          HashMap validation      Table/JSON/YAML
```

**Components:**
- **Handlers** (`src/handlers/`) - Pure business logic, no I/O, fully testable
- **Adapters** (`src/adapters/`) - Async I/O bridge, calls handlers + services
  - **HttpAdapter** - Production implementation using reqwest HTTP client
  - **InMemoryAdapter** - Test-only implementation (compile_error! guard enforced)
- **Services** - Service traits (AuthService, TokenService, UsageService, LimitsService, TracesService, HealthService, StorageService)
- **Formatters** (`src/formatting.rs`) - Universal output (table/expanded/json/yaml)
- **Config** (`src/config.rs`) - Hierarchical configuration system

## Quick Start

```bash
# Authentication
iron-token .auth.login username::alice password::secret123

# List tokens
iron-token .tokens.list

# Generate token
iron-token .tokens.generate name::my-token scope::read:tokens ttl::3600

# Revoke token
iron-token .tokens.revoke name::my-token

# Check health
iron-token .health

# Get version
iron-token .version
```

## Command Syntax

**Unilang keyword::value format:**
```bash
iron-token .command.subcommand param1::value1 param2::value2
```

**Examples:**
```bash
# Login
.auth.login username::user@example.com password::secret

# Refresh tokens
.auth.refresh

# Logout
.auth.logout

# Generate token
.tokens.generate name::api-token scope::read:write:tokens ttl::7200

# List tokens
.tokens.list filter::api

# Get token
.tokens.get name::api-token

# Rotate token
.tokens.rotate name::api-token

# Revoke token
.tokens.revoke name::api-token

# Show usage
.usage.show start_date::2025-01-01 end_date::2025-12-31

# Usage by project
.usage.by_project project_id::my-project

# Usage by provider
.usage.by_provider provider::openai

# Export usage
.usage.export output::usage.csv format::csv

# List limits
.limits.list

# Get limit
.limits.get limit_id::lim_tokens

# Create limit
.limits.create resource_type::tokens limit_value::100000

# Update limit
.limits.update limit_id::lim_tokens limit_value::200000

# Delete limit
.limits.delete limit_id::lim_tokens

# List traces
.traces.list filter::error limit::50

# Get trace
.traces.get trace_id::trace-123

# Export traces
.traces.export output::traces.json format::json

# Health check
.health

# Version
.version
```

## Configuration

**Configuration Hierarchy** (highest to lowest priority):
1. CLI arguments (keyword::value)
2. Environment variables (IRON_*)
3. Local temp config (.iron.local.tmp.toml)
4. Local project config (.iron.local.toml)
5. Global config (~/.config/iron-token/config.toml)
6. Built-in defaults

**Environment Variables:**
```bash
IRON_CLI_API_URL=https://api.example.com
IRON_CLI_FORMAT=json
IRON_CLI_USER=alice
IRON_CLI_TOKEN=your-token-here
```

**Example Config:**
```rust
use iron_cli::config::Config;

// Simple usage
let config = Config::new();

// With CLI args
let mut cli_args = HashMap::new();
cli_args.insert("format".to_string(), "json".to_string());
let config = Config::with_cli_args(cli_args);

// Builder pattern
let config = Config::builder()
    .with_cli_args(cli_args)
    .with_env()
    .with_defaults()
    .validate()
    .build();
```

## Output Formats

All commands support multiple output formats:

```bash
# Table format (default)
iron-token .tokens.list

# Expanded format
iron-token .tokens.list format::expanded

# JSON format
iron-token .tokens.list format::json

# YAML format
iron-token .tokens.list format::yaml
```

## Testing

**Test Commands:**
```bash
# Run all tests
cargo test --all-features

# Run specific test suites
cargo test --test handlers
cargo test --test adapters
cargo test --test integration_test
cargo test --test config_test

# Run with strict warnings
RUSTFLAGS="-D warnings" cargo nextest run --all-features
```

**Test Coverage:**
- Handler tests: 100 tests (pure function validation)
- Adapter tests: 110 tests (integration with services)
- Integration tests: 12 tests (end-to-end workflows)
- Config tests: 13 tests (configuration system)
- Format tests: 19 tests (output formatting)

## Development

**Architecture Principles:**
- **No mocking** - Use real alternative implementations (InMemoryAdapter)
- **TDD workflow** - PLAN → RED → GREEN → REFACTOR → DOCUMENT → VERIFY
- **Pure handlers** - No async, no I/O, fully testable
- **Hexagonal architecture** - Clear separation of concerns

**Module Structure:**
```
src/
├── handlers/           # Pure business logic (sync, no I/O)
│   ├── auth_handlers.rs
│   ├── token_handlers.rs
│   ├── usage_handlers.rs
│   ├── limits_handlers.rs
│   ├── traces_handlers.rs
│   └── health_handlers.rs
├── adapters/           # Async I/O bridge
│   ├── auth.rs
│   ├── tokens.rs
│   ├── usage.rs
│   ├── limits.rs
│   ├── traces.rs
│   ├── health.rs
│   ├── services.rs     # Service trait definitions
│   └── implementations/
│       └── in_memory.rs
├── formatting.rs       # Universal formatter
├── config.rs           # Configuration system
└── lib.rs             # Module exports

tests/
├── handlers.rs         # Handler unit tests
├── adapters.rs         # Adapter integration tests
├── integration_test.rs # End-to-end tests
├── config_test.rs      # Config tests
└── formatting.rs       # Formatter tests
```

## Migration Status

**Completed Phases (6/10):**
- ✅ Phase 1: Project Structure (YAML commands, feature flags)
- ✅ Phase 2: Handlers (22 pure functions)
- ✅ Phase 3: Formatters (4 output formats)
- ✅ Phase 4: Adapters (22 async functions, 110 tests)
- ✅ Phase 5: Configuration (hierarchical config)
- ✅ Phase 6: Integration Testing (12 tests)

**Deferred:**
- ⏸️ Phase 7: Performance Benchmarks

**Remaining:**
- Phase 8: Documentation Update (in progress)
- Phase 9: Final Cutover
- Phase 10: Cleanup

**Current Metrics:**
- Total tests: 288 (283 passing)
- Pattern ratio: 88% new, 11% old
- Architecture purity: Hexagonal ✓
- No mocking: ✓

## License

MIT

## Directory Structure

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | Command-line interface for token management |
| config.rs | Configuration system with hierarchical precedence |
| adapters/ | Adapter layer for unilang CLI |
| bin/ | CLI binary entry points for iron-token and iron commands |
| formatting/ | Universal formatter supporting 4 output formats |
| handlers/ | Pure business logic handlers for CLI commands |

**Notes:**
- Entries marked 'TBD' require manual documentation
- Entries marked '⚠️ ANTI-PATTERN' should be renamed to specific responsibilities

