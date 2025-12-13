//! Command-line interface for Iron Runtime token management and operations.
//!
//! Provides a comprehensive CLI tool for managing LLM tokens, tracking usage,
//! configuring limits, and viewing request traces. Built on Unilang architecture
//! for clean separation between pure business logic and I/O operations.
//!
//! # Purpose
//!
//! This crate implements the authoritative CLI for Iron Runtime:
//! - Token lifecycle management (create, list, revoke, rotate, show)
//! - Usage tracking and analytics (by project, provider, time period)
//! - Rate limit configuration (create, update, delete, show)
//! - Request trace inspection (list, show, export)
//! - Authentication management (login, logout, whoami)
//! - System health monitoring (health check, status)
//!
//! # Architecture
//!
//! The CLI follows a three-layer Unilang architecture:
//!
//! ## Layer 1: Routing (Binary)
//!
//! Entry point in `src/bin/iron_token_unilang.rs`:
//! - Parses CLI arguments using clap
//! - Routes commands to appropriate adapters
//! - Handles global flags and configuration
//! - Returns exit codes
//!
//! ## Layer 2: Adapters (Async I/O Bridge)
//!
//! Located in `src/adapters/`:
//! - Bridge pure handlers to async HTTP client
//! - Call iron_token_manager API endpoints
//! - Handle network errors and retries
//! - Serialize/deserialize JSON
//!
//! **Adapter Modules:**
//! - `token/` - Token operations (create, list, revoke, show, rotate)
//! - `control/` - Control API client and configuration
//! - `keyring` - Secure token storage using system keyring
//!
//! ## Layer 3: Handlers (Pure Logic)
//!
//! Located in `src/handlers/`:
//! - Pure functions: `HashMap<String, String> → Result<String>`
//! - Parameter validation (required fields, formats, ranges)
//! - Output formatting (tables, JSON, text)
//! - Zero I/O operations (no async, no network calls)
//!
//! **Handler Categories:**
//! - Authentication (login, logout, whoami)
//! - Token management (create, list, revoke, show, rotate)
//! - Usage tracking (show, by_project, by_provider, export)
//! - Limits configuration (list, show, create, update, delete)
//! - Trace inspection (list, show, export)
//! - Health monitoring (health, status)
//!
//! # Key Types
//!
//! - [`handlers`] - Pure business logic functions (30 handlers)
//! - [`adapters`] - HTTP client adapters (22 adapters)
//! - [`formatting`] - Output formatting utilities
//! - [`config`] - CLI configuration management
//!
//! # Public API
//!
//! ## Binary Usage
//!
//! ```bash
//! # Authentication
//! iron-token login --email user@example.com --password secret
//! iron-token whoami
//! iron-token logout
//!
//! # Token management
//! iron-token token create --name "Production API" --project proj_123
//! iron-token token list --project proj_123
//! iron-token token revoke --token-id at_550e8400-...
//! iron-token token show --token-id at_550e8400-...
//! iron-token token rotate --token-id at_550e8400-...
//!
//! # Usage tracking
//! iron-token usage show --token-id at_550e8400-...
//! iron-token usage by-project --project-id proj_123
//! iron-token usage by-provider --provider openai
//! iron-token usage export --format json --output usage.json
//!
//! # Rate limits
//! iron-token limits list --project proj_123
//! iron-token limits show --limit-id lim_123
//! iron-token limits create --project proj_123 --rpm 100
//! iron-token limits update --limit-id lim_123 --rpm 200
//! iron-token limits delete --limit-id lim_123
//!
//! # Request traces
//! iron-token traces list --token-id at_550e8400-...
//! iron-token traces show --trace-id trace_123
//! iron-token traces export --format json --output traces.json
//!
//! # System health
//! iron-token health
//! iron-token status
//! ```
//!
//! ## Library Usage
//!
//! While primarily a binary crate, handlers can be used as library functions:
//!
//! ```rust
//! use iron_cli::handlers::token_handlers;
//! use std::collections::HashMap;
//!
//! let mut params = HashMap::new();
//! params.insert("name".to_string(), "Production API".to_string());
//! params.insert("scope".to_string(), "read:tokens".to_string());
//! params.insert("project_id".to_string(), "proj_123".to_string());
//!
//! // Pure handler function (no I/O)
//! let formatted_request = token_handlers::generate_token_handler(&params)?;
//! # Ok::<(), iron_cli::handlers::CliError>(())
//! ```
//!
//! # Configuration
//!
//! CLI configuration stored in `~/.config/iron/config.toml`:
//!
//! ```toml
//! [api]
//! base_url = "http://localhost:8080"
//! timeout_secs = 30
//!
//! [output]
//! format = "table"  # table, json, text
//! color = true
//!
//! [auth]
//! # Token stored securely in system keyring
//! # Use 'iron-token login' to authenticate
//! ```
//!
//! # Testing Strategy
//!
//! ## No Mocking Policy
//!
//! All tests use real implementations:
//! - **Real HTTP server**: Axum server on random port
//! - **Real database**: PostgreSQL with fixtures
//! - **Real CLI binary**: Process spawn with actual binary
//!
//! ## Test Coverage
//!
//! - **Handler tests** (137 tests): Pure function validation
//! - **Integration tests** (207+ tests): Full command execution
//! - **Parameter tests** (2,251+ tests): Comprehensive parameter validation
//! - **Manual tests** (15+ cases): Real API integration testing
//!
//! ## Quality Gates
//!
//! - Command coverage: 100% (69/69 commands)
//! - Parameter coverage: 100% (250/250 parameters)
//! - Zero orphaned adapters (NC-A.1)
//! - Zero orphaned routes (NC-R.1)
//!
//! # Design Principles
//!
//! ## Pure Handlers
//!
//! All business logic is pure:
//! - No I/O operations (no async, no .await)
//! - No external service calls
//! - Deterministic output for given input
//! - Easy to test without mocks
//!
//! ## Adapter Verification
//!
//! Every adapter must:
//! - Call valid API endpoint (no orphaned adapters)
//! - Handle all error cases
//! - Implement proper retry logic
//! - Use typed errors
//!
//! ## Migration Hardening
//!
//! CLI undergoes continuous hardening:
//! - Eliminated 6 orphaned adapters (28→22)
//! - 100% adapter-to-endpoint mapping
//! - Comprehensive test coverage
//! - Zero-tolerance for drift
//!
//! # Error Handling
//!
//! All errors use structured `CliError` type:
//!
//! ```text
//! Error: Missing required parameter
//! Parameter: token_id
//! Command: token revoke
//!
//! Error: Invalid parameter format
//! Parameter: date
//! Expected: YYYY-MM-DD
//! Got: 2023/01/01
//! ```
//!
//! # Performance
//!
//! CLI operations are optimized for developer experience:
//! - Startup time: <50ms cold start
//! - Network overhead: 1 HTTP request per command
//! - Response time: <200ms for typical operations
//! - Memory usage: <10MB resident set size

pub mod handlers;
pub mod formatting;
pub mod adapters;
pub mod config;
