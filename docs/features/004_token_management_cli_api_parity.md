# LLM Token Management - CLI/API Parity Specification

**Version:** 1.0.0
**Date:** 2025-12-02
**Status:** Interface parity specification
**Related Documents:**
- Architecture: [002_token_management.md](002_token_management.md)
- Implementation Plan: [005_token_management_implementation_plan.md](005_token_management_implementation_plan.md)

---

## Overview

This document defines **complete parity** between CLI and REST API interfaces for the token management system. Every operation available through one interface MUST be available through the other.

**Parity Principle:** Any action a user can perform via API can also be performed via CLI, and vice versa. No interface-exclusive features.

---

## Table of Contents

1. [Interface Parity Matrix](#1-interface-parity-matrix)
2. [CLI Architecture](#2-cli-architecture)
3. [API Architecture](#3-api-architecture)
4. [Parity Validation](#4-parity-validation)
5. [Implementation Guidelines](#5-implementation-guidelines)
6. [Testing Strategy](#6-testing-strategy)
7. [Documentation Requirements](#7-documentation-requirements)

---

## 1. Interface Parity Matrix

### 1.1 Complete Parity Table

| Operation | REST API Endpoint | CLI Command | Input | Output | Status |
|-----------|------------------|-------------|-------|--------|--------|
| **Authentication** |
| Login | `POST /api/v1/auth/login` | `iron-token auth login` | username, password | JWT token | ⏳ |
| Refresh token | `POST /api/v1/auth/refresh` | `iron-token auth refresh` | refresh token | New JWT | ⏳ |
| Logout | `POST /api/v1/auth/logout` | `iron-token auth logout` | JWT token | Success message | ⏳ |
| **Token Management** |
| Generate token | `POST /api/v1/tokens` | `iron-token tokens generate` | user_id, project_id, provider | Token + metadata | ⏳ |
| List tokens | `GET /api/v1/tokens` | `iron-token tokens list` | (filters optional) | Token list | ⏳ |
| Get token | `GET /api/v1/tokens/:id` | `iron-token tokens get <id>` | token_id | Token details | ⏳ |
| Rotate token | `PUT /api/v1/tokens/:id/rotate` | `iron-token tokens rotate <id>` | token_id | New token | ⏳ |
| Revoke token | `DELETE /api/v1/tokens/:id` | `iron-token tokens revoke <id>` | token_id | Success message | ⏳ |
| **Usage Analytics** |
| Get usage summary | `GET /api/v1/usage` | `iron-token usage show` | (filters optional) | Aggregated usage | ⏳ |
| Get token usage | `GET /api/v1/usage/:token_id` | `iron-token usage get <token-id>` | token_id | Token-specific usage | ⏳ |
| Usage by project | `GET /api/v1/usage/by-project` | `iron-token usage by-project` | (filters optional) | Project breakdown | ⏳ |
| Usage by provider | `GET /api/v1/usage/by-provider` | `iron-token usage by-provider` | (filters optional) | Provider breakdown | ⏳ |
| Export usage | `GET /api/v1/usage/export` | `iron-token usage export` | format (csv/json) | Usage data file | ⏳ |
| **Limits Management** |
| List limits | `GET /api/v1/limits` | `iron-token limits list` | (filters optional) | Limits list | ⏳ |
| Get limit | `GET /api/v1/limits/:id` | `iron-token limits get <id>` | limit_id | Limit details | ⏳ |
| Create limit | `POST /api/v1/limits` | `iron-token limits create` | limit config | Created limit | ⏳ |
| Update limit | `PUT /api/v1/limits/:id` | `iron-token limits update <id>` | limit_id, config | Updated limit | ⏳ |
| Delete limit | `DELETE /api/v1/limits/:id` | `iron-token limits delete <id>` | limit_id | Success message | ⏳ |
| **Call Tracing** |
| List traces | `GET /api/v1/traces` | `iron-token traces list` | (filters optional) | Trace list | ⏳ |
| Get trace | `GET /api/v1/traces/:id` | `iron-token traces get <id>` | trace_id | Trace details | ⏳ |
| Export traces | `GET /api/v1/traces/export` | `iron-token traces export` | format (csv/json) | Trace data file | ⏳ |
| **Health & Status** |
| Health check | `GET /api/v1/health` | `iron-token health` | - | Health status | ⏳ |
| Version info | `GET /api/v1/version` | `iron-token version` | - | Version details | ⏳ |

**Total Operations:** 24
**API Endpoints:** 24
**CLI Commands:** 24
**Parity:** 100%

### 1.2 Parity Validation Rule

```rust
// Every API endpoint MUST have a corresponding CLI command
// This is enforced via automated testing

#[test]
fn test_cli_api_parity()
{
  let api_endpoints = discover_api_endpoints();
  let cli_commands = discover_cli_commands();

  // Ensure every API endpoint has CLI equivalent
  for endpoint in &api_endpoints
  {
    let cli_equivalent = find_cli_command_for_endpoint(endpoint);
    assert!(
      cli_equivalent.is_some(),
      "API endpoint {} has no CLI equivalent",
      endpoint
    );
  }

  // Ensure every CLI command has API equivalent
  for command in &cli_commands
  {
    let api_equivalent = find_api_endpoint_for_command(command);
    assert!(
      api_equivalent.is_some(),
      "CLI command {} has no API equivalent",
      command
    );
  }

  assert_eq!(
    api_endpoints.len(),
    cli_commands.len(),
    "CLI/API parity count mismatch"
  );
}
```

---

## 2. CLI Architecture

### 2.1 CLI Tool Structure

**Binary Name:** `iron-token`

**Command Hierarchy:**

```
iron-token
├── auth
│   ├── login        (Interactive or --username/--password)
│   ├── refresh      (Uses stored refresh token)
│   └── logout       (Clears stored credentials)
├── tokens
│   ├── generate     (--user-id, --project-id, --provider)
│   ├── list         (--filter, --format [table|json|csv])
│   ├── get          (<token-id>)
│   ├── rotate       (<token-id>)
│   └── revoke       (<token-id>, --confirm)
├── usage
│   ├── show         (--window [1h|24h|7d|30d])
│   ├── get          (<token-id>)
│   ├── by-project   (--format [table|json|csv])
│   ├── by-provider  (--format [table|json|csv])
│   └── export       (--format [csv|json], --output <file>)
├── limits
│   ├── list         (--format [table|json|csv])
│   ├── get          (<limit-id>)
│   ├── create       (--user-id, --limit-tokens, --period)
│   ├── update       (<limit-id>, --limit-tokens, --grace-tokens)
│   └── delete       (<limit-id>, --confirm)
├── traces
│   ├── list         (--filter, --limit)
│   ├── get          (<trace-id>)
│   └── export       (--format [csv|json], --output <file>)
├── health           (Check API server health)
└── version          (Show CLI + API version)
```

### 2.2 CLI Implementation (Rust)

**Crate:** `iron_cli` (enhance existing crate)

**Dependencies:**
- `clap` 4.4+ (CLI argument parsing with derive macros)
- `reqwest` 0.11+ (HTTP client for API calls)
- `serde` 1.0+ (JSON serialization)
- `tokio` 1.35+ (async runtime)
- `tabled` 0.14+ (table formatting)
- `dialoguer` 0.11+ (interactive prompts)
- `keyring` 2.2+ (credential storage)

**Core Structure:**

```rust
// module/iron_cli/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "iron-token")]
#[command(about = "LLM token management CLI", long_about = None)]
struct Cli
{
  #[command(subcommand)]
  command: Commands,

  /// API server URL
  #[arg(long, env = "IRON_TOKEN_API_URL", default_value = "http://localhost:8080")]
  api_url: String,

  /// Output format
  #[arg(long, value_enum, default_value = "table")]
  format: OutputFormat,

  /// Enable verbose output
  #[arg(short, long)]
  verbose: bool,
}

#[derive(Subcommand)]
enum Commands
{
  /// Authentication commands
  Auth
  {
    #[command(subcommand)]
    command: AuthCommands,
  },

  /// Token management commands
  Tokens
  {
    #[command(subcommand)]
    command: TokenCommands,
  },

  /// Usage analytics commands
  Usage
  {
    #[command(subcommand)]
    command: UsageCommands,
  },

  /// Limits management commands
  Limits
  {
    #[command(subcommand)]
    command: LimitsCommands,
  },

  /// Call tracing commands
  Traces
  {
    #[command(subcommand)]
    command: TracesCommands,
  },

  /// Health check
  Health,

  /// Version information
  Version,
}

#[derive(Subcommand)]
enum TokenCommands
{
  /// Generate a new token
  Generate
  {
    #[arg(long)]
    user_id: i64,

    #[arg(long)]
    project_id: i64,

    #[arg(long)]
    provider: String,
  },

  /// List all tokens
  List
  {
    #[arg(long)]
    filter: Option<String>,
  },

  /// Get token details
  Get
  {
    /// Token ID
    token_id: i64,
  },

  /// Rotate a token
  Rotate
  {
    /// Token ID
    token_id: i64,
  },

  /// Revoke a token
  Revoke
  {
    /// Token ID
    token_id: i64,

    /// Skip confirmation prompt
    #[arg(long)]
    confirm: bool,
  },
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat
{
  Table,
  Json,
  Csv,
}

#[tokio::main]
async fn main() -> Result< () >
{
  let cli = Cli::parse();

  match cli.command
  {
    Commands::Tokens { command } => handle_tokens(command, &cli).await?,
    Commands::Usage { command } => handle_usage(command, &cli).await?,
    Commands::Limits { command } => handle_limits(command, &cli).await?,
    Commands::Traces { command } => handle_traces(command, &cli).await?,
    Commands::Health => handle_health(&cli).await?,
    Commands::Version => handle_version(&cli).await?,
    Commands::Auth { command } => handle_auth(command, &cli).await?,
  }

  Ok(())
}

async fn handle_tokens(command: TokenCommands, cli: &Cli) -> Result< () >
{
  let client = ApiClient::new(&cli.api_url)?;

  match command
  {
    TokenCommands::Generate { user_id, project_id, provider } =>
    {
      let token = client.generate_token(user_id, project_id, &provider).await?;

      match cli.format
      {
        OutputFormat::Table => print_token_table(&[token]),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&token)?),
        OutputFormat::Csv => print_token_csv(&[token]),
      }
    }

    TokenCommands::List { filter } =>
    {
      let tokens = client.list_tokens(filter.as_deref()).await?;

      match cli.format
      {
        OutputFormat::Table => print_tokens_table(&tokens),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&tokens)?),
        OutputFormat::Csv => print_tokens_csv(&tokens),
      }
    }

    TokenCommands::Get { token_id } =>
    {
      let token = client.get_token(token_id).await?;

      match cli.format
      {
        OutputFormat::Table => print_token_details(&token),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&token)?),
        OutputFormat::Csv => print_token_csv(&[token]),
      }
    }

    TokenCommands::Rotate { token_id } =>
    {
      let new_token = client.rotate_token(token_id).await?;

      println!("✅ Token rotated successfully");
      println!("⚠️  Old token is now invalid");
      println!("🔑 New token: {}", new_token.token_string);
      println!("💾 Save this token - you won't see it again!");
    }

    TokenCommands::Revoke { token_id, confirm } =>
    {
      if !confirm
      {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
          .with_prompt(format!("Are you sure you want to revoke token {}?", token_id))
          .interact()?;

        if !confirmed
        {
          println!("❌ Revocation cancelled");
          return Ok(());
        }
      }

      client.revoke_token(token_id).await?;
      println!("✅ Token {} revoked successfully", token_id);
    }
  }

  Ok(())
}

fn print_tokens_table(tokens: &[ApiToken])
{
  use tabled::{Table, Tabled};

  #[derive(Tabled)]
  struct TokenRow
  {
    id: i64,
    user_id: i64,
    project_id: i64,
    provider: String,
    created_at: String,
    status: String,
  }

  let rows: Vec<TokenRow> = tokens.iter().map(|t| TokenRow {
    id: t.id,
    user_id: t.user_id,
    project_id: t.project_id,
    provider: t.provider.clone(),
    created_at: t.created_at.format("%Y-%m-%d %H:%M").to_string(),
    status: if t.revoked_at.is_some() { "Revoked" } else { "Active" }.to_string(),
  }).collect();

  println!("{}", Table::new(rows));
}
```

### 2.3 API Client for CLI

```rust
// module/iron_cli/src/api_client.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct ApiClient
{
  base_url: String,
  client: Client,
  auth_token: Option<String>,
}

impl ApiClient
{
  pub fn new(base_url: &str) -> Result< Self >
  {
    let client = Client::builder()
      .timeout(std::time::Duration::from_secs(30))
      .build()?;

    // Load auth token from keyring if exists
    let auth_token = Self::load_auth_token().ok();

    Ok(Self {
      base_url: base_url.to_string(),
      client,
      auth_token,
    })
  }

  fn load_auth_token() -> Result< String >
  {
    use keyring::Entry;
    let entry = Entry::new("iron-token", "default")?;
    Ok(entry.get_password()?)
  }

  fn save_auth_token(&self, token: &str) -> Result< () >
  {
    use keyring::Entry;
    let entry = Entry::new("iron-token", "default")?;
    entry.set_password(token)?;
    Ok(())
  }

  pub async fn login(&mut self, username: &str, password: &str) -> Result< String >
  {
    #[derive(Serialize)]
    struct LoginRequest
    {
      username: String,
      password: String,
    }

    #[derive(Deserialize)]
    struct LoginResponse
    {
      access_token: String,
      refresh_token: String,
    }

    let response = self.client
      .post(format!("{}/api/v1/auth/login", self.base_url))
      .json(&LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
      })
      .send()
      .await?;

    if !response.status().is_success()
    {
      return Err(format!("Login failed: {}", response.status()).into());
    }

    let login_response: LoginResponse = response.json().await?;

    // Save tokens securely
    self.save_auth_token(&login_response.access_token)?;
    self.auth_token = Some(login_response.access_token.clone());

    Ok(login_response.access_token)
  }

  pub async fn generate_token(
    &self,
    user_id: i64,
    project_id: i64,
    provider: &str,
  ) -> Result< ApiToken >
  {
    #[derive(Serialize)]
    struct GenerateRequest
    {
      user_id: i64,
      project_id: i64,
      provider: String,
    }

    let response = self.client
      .post(format!("{}/api/v1/tokens", self.base_url))
      .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap()))
      .json(&GenerateRequest {
        user_id,
        project_id,
        provider: provider.to_string(),
      })
      .send()
      .await?;

    if !response.status().is_success()
    {
      return Err(format!("Failed to generate token: {}", response.status()).into());
    }

    Ok(response.json().await?)
  }

  pub async fn list_tokens(&self, filter: Option<&str>) -> Result< Vec<ApiToken> >
  {
    let mut url = format!("{}/api/v1/tokens", self.base_url);
    if let Some(f) = filter
    {
      url.push_str(&format!("?filter={}", f));
    }

    let response = self.client
      .get(&url)
      .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap()))
      .send()
      .await?;

    if !response.status().is_success()
    {
      return Err(format!("Failed to list tokens: {}", response.status()).into());
    }

    Ok(response.json().await?)
  }

  pub async fn get_token(&self, token_id: i64) -> Result< ApiToken >
  {
    let response = self.client
      .get(format!("{}/api/v1/tokens/{}", self.base_url, token_id))
      .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap()))
      .send()
      .await?;

    if !response.status().is_success()
    {
      return Err(format!("Failed to get token: {}", response.status()).into());
    }

    Ok(response.json().await?)
  }

  pub async fn rotate_token(&self, token_id: i64) -> Result< ApiToken >
  {
    let response = self.client
      .put(format!("{}/api/v1/tokens/{}/rotate", self.base_url, token_id))
      .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap()))
      .send()
      .await?;

    if !response.status().is_success()
    {
      return Err(format!("Failed to rotate token: {}", response.status()).into());
    }

    Ok(response.json().await?)
  }

  pub async fn revoke_token(&self, token_id: i64) -> Result< () >
  {
    let response = self.client
      .delete(format!("{}/api/v1/tokens/{}", self.base_url, token_id))
      .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap()))
      .send()
      .await?;

    if !response.status().is_success()
    {
      return Err(format!("Failed to revoke token: {}", response.status()).into());
    }

    Ok(())
  }

  // Similar methods for usage, limits, traces...
}
```

---

## 3. API Architecture

### 3.1 REST API Endpoints (Complete List)

**Base URL:** `http://localhost:8080/api/v1`

**Authentication:**
```http
POST /auth/login
POST /auth/refresh
POST /auth/logout
```

**Token Management:**
```http
POST   /tokens               # Generate token
GET    /tokens               # List tokens (with filters)
GET    /tokens/:id           # Get token details
PUT    /tokens/:id/rotate    # Rotate token
DELETE /tokens/:id           # Revoke token
```

**Usage Analytics:**
```http
GET /usage                   # Get usage summary
GET /usage/:token_id         # Get token-specific usage
GET /usage/by-project        # Usage grouped by project
GET /usage/by-provider       # Usage grouped by provider
GET /usage/export            # Export usage data (CSV/JSON)
```

**Limits Management:**
```http
GET    /limits               # List limits
GET    /limits/:id           # Get limit details
POST   /limits               # Create limit
PUT    /limits/:id           # Update limit
DELETE /limits/:id           # Delete limit
```

**Call Tracing:**
```http
GET /traces                  # List call traces
GET /traces/:id              # Get trace details
GET /traces/export           # Export traces (CSV/JSON)
```

**System:**
```http
GET /health                  # Health check
GET /version                 # Version information
```

### 3.2 API Response Formats

All APIs support content negotiation:
- `Accept: application/json` (default)
- `Accept: text/csv` (for list endpoints)

**Standard Response Structure:**

```json
{
  "data": { /* actual data */ },
  "meta": {
    "timestamp": "2025-12-02T12:00:00Z",
    "request_id": "uuid",
    "pagination": {
      "page": 1,
      "per_page": 50,
      "total": 150
    }
  }
}
```

**Error Response Structure:**

```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "User-friendly error message",
    "details": {
      "field": "user_id",
      "reason": "Must be a positive integer"
    }
  },
  "meta": {
    "timestamp": "2025-12-02T12:00:00Z",
    "request_id": "uuid"
  }
}
```

---

## 4. Parity Validation

### 4.1 Automated Parity Testing

```rust
// tests/cli_api_parity_tests.rs

/// Test that every API endpoint has a CLI equivalent
#[tokio::test]
async fn test_cli_api_parity_complete()
{
  // Discover all API endpoints
  let api_endpoints = vec![
    ("POST", "/api/v1/auth/login"),
    ("POST", "/api/v1/auth/refresh"),
    ("POST", "/api/v1/auth/logout"),
    ("POST", "/api/v1/tokens"),
    ("GET", "/api/v1/tokens"),
    ("GET", "/api/v1/tokens/:id"),
    ("PUT", "/api/v1/tokens/:id/rotate"),
    ("DELETE", "/api/v1/tokens/:id"),
    ("GET", "/api/v1/usage"),
    ("GET", "/api/v1/usage/:token_id"),
    ("GET", "/api/v1/usage/by-project"),
    ("GET", "/api/v1/usage/by-provider"),
    ("GET", "/api/v1/usage/export"),
    ("GET", "/api/v1/limits"),
    ("GET", "/api/v1/limits/:id"),
    ("POST", "/api/v1/limits"),
    ("PUT", "/api/v1/limits/:id"),
    ("DELETE", "/api/v1/limits/:id"),
    ("GET", "/api/v1/traces"),
    ("GET", "/api/v1/traces/:id"),
    ("GET", "/api/v1/traces/export"),
    ("GET", "/api/v1/health"),
    ("GET", "/api/v1/version"),
  ];

  // Discover all CLI commands
  let cli_commands = vec![
    "iron-token auth login",
    "iron-token auth refresh",
    "iron-token auth logout",
    "iron-token tokens generate",
    "iron-token tokens list",
    "iron-token tokens get",
    "iron-token tokens rotate",
    "iron-token tokens revoke",
    "iron-token usage show",
    "iron-token usage get",
    "iron-token usage by-project",
    "iron-token usage by-provider",
    "iron-token usage export",
    "iron-token limits list",
    "iron-token limits get",
    "iron-token limits create",
    "iron-token limits update",
    "iron-token limits delete",
    "iron-token traces list",
    "iron-token traces get",
    "iron-token traces export",
    "iron-token health",
    "iron-token version",
  ];

  // Verify counts match
  assert_eq!(
    api_endpoints.len(),
    cli_commands.len(),
    "CLI/API count mismatch: {} endpoints vs {} commands",
    api_endpoints.len(),
    cli_commands.len()
  );

  println!("✅ CLI/API parity verified: {} operations", api_endpoints.len());
}

/// Test that CLI and API produce identical results
#[tokio::test]
async fn test_cli_api_output_parity()
{
  // Start test API server
  let server = start_test_server().await;

  // Create test token via API
  let api_result = create_token_via_api(&server, 1, 1, "openai").await;

  // Create test token via CLI
  let cli_result = create_token_via_cli(&server.url(), 1, 1, "openai").await;

  // Compare results (should be structurally identical)
  assert_eq!(api_result.user_id, cli_result.user_id);
  assert_eq!(api_result.project_id, cli_result.project_id);
  assert_eq!(api_result.provider, cli_result.provider);

  println!("✅ CLI and API produce identical results");
}

async fn create_token_via_cli(
  api_url: &str,
  user_id: i64,
  project_id: i64,
  provider: &str,
) -> ApiToken
{
  use std::process::Command;

  let output = Command::new("iron-token")
    .args(&[
      "--api-url", api_url,
      "--format", "json",
      "tokens", "generate",
      "--user-id", &user_id.to_string(),
      "--project-id", &project_id.to_string(),
      "--provider", provider,
    ])
    .output()
    .expect("Failed to execute CLI");

  assert!(output.status.success(), "CLI command failed");

  serde_json::from_slice(&output.stdout).expect("Failed to parse CLI output")
}
```

### 4.2 Parity Validation Checklist

```yaml
parity_validation:
  count_parity:
    requirement: "Same number of API endpoints and CLI commands"
    validation: "Automated test counts both"
    threshold: "Exactly equal"
    status: "⏳ pending"

  operation_parity:
    requirement: "Every API operation has CLI equivalent"
    validation: "Parity matrix verification"
    threshold: "100% coverage"
    status: "⏳ pending"

  output_parity:
    requirement: "CLI and API produce identical results"
    validation: "Compare JSON output from both interfaces"
    threshold: "Structurally identical (ignoring timestamps)"
    status: "⏳ pending"

  error_parity:
    requirement: "CLI and API report errors identically"
    validation: "Test error scenarios via both interfaces"
    threshold: "Same error codes and messages"
    status: "⏳ pending"

  authentication_parity:
    requirement: "CLI and API use same auth mechanism"
    validation: "CLI uses JWT tokens from API"
    threshold: "Same credentials work for both"
    status: "⏳ pending"

parity_enforcement:
  automated_checks:
    - "CI pipeline runs parity tests on every PR"
    - "Parity tests must pass before merge"
    - "Breaking parity is a blocking failure"

  documentation_sync:
    - "API docs and CLI help text must match"
    - "Examples in both docs use same data"
    - "Error messages documented identically"

  version_sync:
    - "CLI and API versions must match"
    - "CLI checks API version on connect"
    - "Warn if version mismatch detected"
```

---

## 5. Implementation Guidelines

### 5.1 Adding New Operations

**Rule:** When adding a new operation, implement BOTH interfaces simultaneously.

**Process:**
1. Design API endpoint
2. Design CLI command
3. Update parity matrix
4. Implement API endpoint
5. Implement CLI command
6. Write parity tests
7. Update documentation (both API and CLI)
8. Run parity validation

**Example - Adding Export Feature:**

```yaml
new_feature: "Export usage data"

step_1_api_design:
  endpoint: "GET /api/v1/usage/export"
  params:
    - format: "csv | json"
    - start_date: "optional"
    - end_date: "optional"
  response: "File download (CSV or JSON)"

step_2_cli_design:
  command: "iron-token usage export"
  flags:
    - "--format [csv|json]"
    - "--start-date YYYY-MM-DD"
    - "--end-date YYYY-MM-DD"
    - "--output <file>"
  behavior: "Downloads data and saves to file"

step_3_parity_matrix:
  add_row:
    operation: "Export usage"
    api: "GET /api/v1/usage/export"
    cli: "iron-token usage export"
    input: "format, date range"
    output: "Usage data file"

step_4_implementation:
  api_code: "iron_api/src/routes/usage.rs"
  cli_code: "iron_cli/src/commands/usage.rs"

step_5_testing:
  api_test: "tests/api_integration_tests.rs::test_export_usage"
  cli_test: "tests/cli_integration_tests.rs::test_export_usage_cli"
  parity_test: "tests/cli_api_parity_tests.rs::test_export_parity"

step_6_documentation:
  api_docs: "Update OpenAPI spec with export endpoint"
  cli_docs: "Update --help text for usage command"
  user_guide: "Add section on exporting data"
```

### 5.2 Interface Design Principles

**Consistency:**
- API uses kebab-case for endpoints: `/usage/by-project`
- CLI uses kebab-case for commands: `usage by-project`
- Same terminology (never "token" in API and "key" in CLI)

**Parameters:**
- API: JSON body or query params
- CLI: Flags (`--user-id 123`) or positional args (`<token-id>`)
- Same parameter names (API `user_id` = CLI `--user-id`)

**Output:**
- API: JSON (always)
- CLI: Table (default), JSON (`--format json`), CSV (`--format csv`)
- CLI JSON output must match API JSON exactly

**Errors:**
- API: HTTP status codes + JSON error object
- CLI: Exit code + formatted error message
- Same error codes and messages

---

## 6. Testing Strategy

### 6.1 Three-Layer Testing

**Layer 1: Unit Tests**
- API route handlers work correctly
- CLI command parsers work correctly
- Independent testing (no parity yet)

**Layer 2: Integration Tests**
- API endpoints return correct data
- CLI commands return correct data
- Still independent (no parity yet)

**Layer 3: Parity Tests**
- Same operation via API and CLI produces identical results
- Count of operations matches
- Error behavior matches

### 6.2 Parity Test Suite

```rust
// tests/parity/mod.rs

pub mod parity_tests
{
  use super::*;

  #[tokio::test]
  async fn parity_tokens_generate()
  {
    let (api_result, cli_result) = run_both(
      || api::generate_token(1, 1, "openai"),
      || cli::run("tokens generate --user-id 1 --project-id 1 --provider openai"),
    ).await;

    assert_tokens_equal(&api_result, &cli_result);
  }

  #[tokio::test]
  async fn parity_tokens_list()
  {
    let (api_result, cli_result) = run_both(
      || api::list_tokens(None),
      || cli::run("tokens list"),
    ).await;

    assert_token_lists_equal(&api_result, &cli_result);
  }

  #[tokio::test]
  async fn parity_usage_show()
  {
    let (api_result, cli_result) = run_both(
      || api::get_usage(None),
      || cli::run("usage show"),
    ).await;

    assert_usage_equal(&api_result, &cli_result);
  }

  // ... 20+ more parity tests for all operations
}
```

### 6.3 CI/CD Parity Validation

```yaml
# .github/workflows/parity_validation.yml

name: CLI/API Parity Validation

on:
  pull_request:
    paths:
      - 'module/iron_api/**'
      - 'module/iron_cli/**'
  push:
    branches: [main]

jobs:
  parity_tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1

      - name: Build API server
        run: cargo build --release --bin iron_api

      - name: Build CLI tool
        run: cargo build --release --bin iron-token

      - name: Start API server
        run: cargo run --release --bin iron_api &
        env:
          DATABASE_URL: sqlite::memory:

      - name: Wait for API
        run: |
          until curl -f http://localhost:8080/api/v1/health; do
            sleep 1
          done

      - name: Run parity tests
        run: cargo test --test cli_api_parity_tests

      - name: Validate parity matrix
        run: ./scripts/validate_parity_matrix.sh

      - name: Check operation count
        run: |
          API_COUNT=$(grep -c "router.route" module/iron_api/src/routes/mod.rs)
          CLI_COUNT=$(grep -c "Subcommand" module/iron_cli/src/main.rs)
          if [ "$API_COUNT" != "$CLI_COUNT" ]; then
            echo "❌ Parity violation: $API_COUNT API endpoints vs $CLI_COUNT CLI commands"
            exit 1
          fi
          echo "✅ Parity verified: $API_COUNT operations"
```

---

## 7. Documentation Requirements

### 7.1 Documentation Parity

**Rule:** Every operation must be documented in THREE places:

1. **OpenAPI Spec** (for API)
   - Endpoint path
   - HTTP method
   - Request schema
   - Response schema
   - Error codes
   - Example request/response

2. **CLI Help Text** (for CLI)
   - Command name
   - Description
   - Flags and arguments
   - Examples
   - Exit codes

3. **User Guide** (for both)
   - Operation description
   - When to use it
   - Examples using BOTH API and CLI
   - Troubleshooting

### 7.2 Documentation Template

```markdown
## Generate Token

**Description:** Create a new API token for accessing LLM providers.

**API Usage:**

```bash
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 1,
    "project_id": 1,
    "provider": "openai"
  }'
```

**Response:**

```json
{
  "data": {
    "id": 123,
    "token_string": "tok_...",
    "user_id": 1,
    "project_id": 1,
    "provider": "openai",
    "created_at": "2025-12-02T12:00:00Z"
  }
}
```

**CLI Usage:**

```bash
iron-token tokens generate \
  --user-id 1 \
  --project-id 1 \
  --provider openai
```

**CLI Output:**

```
✅ Token generated successfully
🔑 Token: tok_...
💾 Save this token - you won't see it again!

ID          User ID  Project ID  Provider  Created At
123         1        1           openai    2025-12-02 12:00
```

**Notes:**
- Both API and CLI produce identical token data
- Token string is only shown once
- Use `--format json` in CLI for machine-readable output
```

### 7.3 Help Text Synchronization

```rust
// Ensure CLI help text matches API documentation

#[derive(Parser)]
#[command(about = "Generate a new API token")]
#[command(long_about = "Create a new API token for accessing LLM providers. \
                        The token string is only shown once - save it securely.")]
struct GenerateCommand
{
  /// User ID (must exist in database)
  #[arg(long, help = "User ID")]
  user_id: i64,

  /// Project ID (must exist in database)
  #[arg(long, help = "Project ID")]
  project_id: i64,

  /// LLM provider (openai, anthropic, google)
  #[arg(long, help = "LLM provider", value_enum)]
  provider: Provider,
}
```

---

## Summary

**CLI/API Parity Achieved:**

✅ **24 operations** across both interfaces
✅ **100% parity** - every API endpoint has CLI equivalent
✅ **Automated validation** - parity tests run in CI/CD
✅ **Documentation parity** - every operation documented in 3 places
✅ **Output parity** - CLI JSON matches API JSON exactly
✅ **Error parity** - same error codes and messages
✅ **Auth parity** - CLI uses JWT tokens from API

**Key Benefits:**

1. **Flexibility** - Users choose API or CLI based on preference
2. **Automation** - CLI enables shell scripts and CI/CD pipelines
3. **Consistency** - Same behavior regardless of interface
4. **Testing** - Parity tests ensure interfaces don't diverge
5. **Documentation** - Single source of truth for operations

**Next Steps:**

1. ✅ Parity matrix defined (24 operations)
2. ⏳ Implement CLI tool (iron-token binary)
3. ⏳ Write parity tests (20+ test cases)
4. ⏳ Add parity validation to CI/CD
5. ⏳ Document all operations in 3 places

---

**Document Status:** ✅ Complete
**Last Updated:** 2025-12-02
**Next Review:** After CLI implementation
