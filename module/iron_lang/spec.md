# IronLang Protocol Specification

**Version**: 0.1.0
**Status**: Alpha

## Overview

IronLang is a protocol for AI agents to safely access and manipulate data sources. It provides type-safe message definitions for reading, writing, and querying databases, files, APIs, caches, and object storage.

**Design Philosophy**: IronLang is designed for one-shot AI agent operations (query → response → done), not continuous pipelines. Each operation is independent with no persistent state between requests.

## Design Principles

### Principles Overview

| # | Principle | Status | Priority | Why It Matters | Current State | Action Needed | Time |
|---|-----------|--------|----------|----------------|---------------|---------------|------|
| 1 | Simplicity Through Standards | ✅ Applied | - | Standard formats = debuggable | NDJSON + Rust types implemented | None | - |
| 2 | Actor Isolation | ✅ Applied | - | Agent never talks to connector directly | Agent→Runtime→Connector architecture | None | - |
| 3 | Uniform Message Envelope | ✅ Applied | - | Multiplexing different message types | IronMessage enum with tag discrimination | None | - |
| 4 | Rich Trace Messages | ✅ Applied | - | Structured error reporting for debugging | ERROR has severity/code/details/timestamp | None | - |
| 5 | Structured Logging | ✅ Applied | - | Machine-parseable logs | LogMessage with levels/component/context | None | - |
| 6 | Streaming Over Batching | 🎯 Should | Critical | Will OOM on large datasets | No implementation, runtime is stub | Implement `BufReader::lines()` on stdin/stdout | 3d |
| 7 | Command-Based Interface | 🎯 Should | Critical | Cannot invoke connectors without CLI | No CLI, only library | Add `iron spec/check/read/write` commands | 5d |
| 8 | Schema-Driven Validation | 🎯 Should | High | Platform can't validate configs | No SPEC message for connector capabilities | Add SpecMessage with JSONSchema config | 3d |
| 9 | Container as Universal Runtime | 🎯 Should | High | Rust-only connectors limits ecosystem | No container interface | Define container interface + IRON_ENTRYPOINT | 5d |
| 10 | File-Based Configuration | 🎯 Should | High | Credentials in args = security risk | No config loading defined | Load from `--config config.json` | 2d |
| 11 | Fail-Fast Check | 🎯 Should | Critical | Expensive ops without validation | AUTH exists, no connection test | AUTH must test connection before allowing READ/WRITE | 3d |
| 12 | Graceful Degradation | 🎯 Should | Med | Never fail on schema mismatch | No explicit rules defined | Document + test mismatch behavior | 2d |
| 13 | Semantic Versioning | 🎯 Should | High | No compatibility checking | No protocol_version field | Add version to SPEC, check compatibility | 2d |
| 14 | Additive Evolution | 🎯 Should | High | Protocol evolution breaks old connectors | Rust enums not forward-compatible | Add #[non_exhaustive] + Unknown variant | 3d |
| 15 | Schema as Contract | 🤔 Could | Low | Strict validation may be too rigid for AI | SCHEMA message exists, no enforcement | Evaluate if strict validation needed | 1w |
| 16 | Checkpointing | 🤔 Could | Med | Resume from failure in long operations | No checkpoint messages | Add if long-running ops emerge | 1w |
| 17 | Batch Operations | 🤔 Could | Low | Batch queries reduce round trips | Single request→response | Add BatchReadMessage if pattern emerges | 1w |
| 18 | Namespace Isolation | 🤔 Could | Low | Multi-tenant data partitioning | No namespace field | Add if multi-tenant use case emerges | 3d |

**Summary**:
- **Applied**: 5/18 (28%)
- **Should Apply**: 9/18 (50%) → **28 days total**
- **Could Apply**: 4/18 (22%) → **17 days if all**

---

## "Should" Principles: Detailed Rationale

### #6: Streaming Over Batching

**Why critical**: Without streaming, reading large datasets will cause out-of-memory (OOM) errors. If an agent queries a 1GB database table, loading it entirely into memory will crash the runtime.

**Current state**: Runtime is a stub with no streaming implementation.

**Implementation**:
```rust
// In runtime.rs
use std::io::{ BufRead, BufReader, Write };

pub fn run_connector( connector_path : &str ) -> Result< (), Error >
{
  let mut child = std::process::Command::new( connector_path )
    .stdin( std::process::Stdio::piped() )
    .stdout( std::process::Stdio::piped() )
    .spawn()?;

  let stdin = child.stdin.take().unwrap();
  let stdout = child.stdout.take().unwrap();

  // Stream messages line-by-line (NDJSON)
  let reader = BufReader::new( stdout );
  for line in reader.lines()
  {
    let line = line?;
    let msg : IronMessage = serde_json::from_str( &line )?;
    handle_message( msg )?;
  }

  Ok(())
}
```

**Test case**:
```rust
// tests/streaming.rs
#[ test ]
fn test_large_dataset_streaming()
{
  // Generate 100MB of data
  let large_dataset = generate_large_ndjson( 100_000_000 );

  // Should process without OOM
  let runtime = Runtime::new();
  runtime.process_stream( large_dataset.as_bytes() )?;

  // Memory usage should stay < 10MB
  assert!( get_memory_usage() < 10_000_000 );
}
```

**Time estimate**: 3 days (implementation + tests + documentation)

---

### #7: Command-Based Interface

**Why critical**: Without a CLI, connectors cannot be invoked from the command line. The protocol is currently library-only, making it unusable for containers, shell scripts, or CI/CD pipelines.

**Current state**: No CLI, only library (`iron_lang::protocol`).

**Implementation**:
```rust
// src/bin/iron.rs
use clap::{ Parser, Subcommand };
use iron_lang::{ protocol::*, runtime::* };

#[ derive( Parser ) ]
struct Cli
{
  #[ command( subcommand ) ]
  command : Commands,
}

#[ derive( Subcommand ) ]
enum Commands
{
  /// Output connector specification
  Spec
  {
    #[ arg( long ) ]
    config : Option< String >,
  },

  /// Validate connection and credentials
  Check
  {
    #[ arg( long ) ]
    config : String,
  },

  /// Read data from source
  Read
  {
    #[ arg( long ) ]
    config : String,

    #[ arg( long ) ]
    catalog : String,
  },

  /// Write data to destination
  Write
  {
    #[ arg( long ) ]
    config : String,

    #[ arg( long ) ]
    catalog : String,
  },
}

fn main()
{
  let cli = Cli::parse();

  match cli.command
  {
    Commands::Spec { config } =>
    {
      let spec = generate_spec( config )?;
      println!( "{}", serde_json::to_string( &spec )? );
    },
    Commands::Check { config } =>
    {
      let result = check_connection( &config )?;
      println!( "{}", serde_json::to_string( &result )? );
    },
    Commands::Read { config, catalog } =>
    {
      run_read( &config, &catalog )?;
    },
    Commands::Write { config, catalog } =>
    {
      run_write( &config, &catalog )?;
    },
  }
}
```

**Test case**:
```bash
# tests/manual/cli_interface.md

## Test: SPEC command
iron spec --config config.json
# Expected: JSON output with connector specification

## Test: CHECK command
iron check --config config.json
# Expected: {"status": "success"} or {"status": "failed", "error": "..."}

## Test: READ command
iron read --config config.json --catalog catalog.json < /dev/null
# Expected: NDJSON stream of READ messages to stdout
```

**Time estimate**: 5 days (CLI scaffold + 4 commands + tests + docs)

---

### #8: Schema-Driven Validation

**Why high priority**: Without a SPEC message, the runtime can't validate connector configurations. If a SQL connector requires `{host, port, database, username, password}` but the user provides `{url}`, the error happens at runtime instead of upfront.

**Current state**: SCHEMA message exists for querying data schemas, but no SPEC message for connector configuration schemas.

**Implementation**:
```rust
// Add to protocol.rs
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct SpecMessage
{
  pub protocol_version : String, // "0.1.0"
  pub connector_name : String,
  pub connector_version : String,
  pub config_schema : serde_json::Value, // JSONSchema
}

// Add to IronMessage enum
#[ serde( rename = "SPEC" ) ]
Spec( SpecMessage ),
```

**Example SPEC output**:
```json
{
  "type": "SPEC",
  "protocol_version": "0.1.0",
  "connector_name": "iron-connector-postgres",
  "connector_version": "1.0.0",
  "config_schema": {
    "type": "object",
    "properties": {
      "host": {"type": "string"},
      "port": {"type": "integer", "default": 5432},
      "database": {"type": "string"},
      "username": {"type": "string"},
      "password": {"type": "string"}
    },
    "required": ["host", "database", "username", "password"]
  }
}
```

**Test case**:
```rust
// tests/spec_validation.rs
#[ test ]
fn test_spec_validation()
{
  let spec = SpecMessage
  {
    protocol_version : "0.1.0".to_string(),
    connector_name : "test-connector".to_string(),
    connector_version : "1.0.0".to_string(),
    config_schema : serde_json::json!
    ({
      "type" : "object",
      "required" : [ "api_key" ],
    }),
  };

  let valid_config = serde_json::json!({ "api_key" : "abc123" });
  assert!( validate_config( &spec, &valid_config ).is_ok() );

  let invalid_config = serde_json::json!({ "wrong_field" : "value" });
  assert!( validate_config( &spec, &invalid_config ).is_err() );
}
```

**Time estimate**: 3 days (SPEC message + validation + tests)

---

### #9: Container as Universal Runtime

**Why high priority**: Rust-only connectors severely limit the ecosystem. Many data sources have official SDKs in Python, Node.js, or Go. Containers allow connectors in any language as long as they implement the protocol.

**Current state**: No container interface defined.

**Implementation**:

**Dockerfile template**:
```dockerfile
# Example: iron-connector-postgres/Dockerfile
FROM python:3.11-slim

WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt

COPY connector.py .

ENTRYPOINT ["python", "connector.py"]
```

**Connector contract**:
```python
# connector.py
import sys
import json

def handle_spec():
    spec = {
        "type": "SPEC",
        "protocol_version": "0.1.0",
        "connector_name": "iron-connector-postgres",
        "connector_version": "1.0.0",
        "config_schema": {...}
    }
    print(json.dumps(spec))

def handle_check(config):
    # Test connection
    try:
        conn = psycopg2.connect(**config)
        print(json.dumps({"type": "ACK", "status": "success"}))
    except Exception as e:
        print(json.dumps({"type": "ERROR", "code": "CONNECTION_FAILED", "message": str(e)}))

def main():
    if len(sys.argv) < 2:
        sys.exit(1)

    command = sys.argv[1]

    if command == "spec":
        handle_spec()
    elif command == "check":
        config = json.loads(sys.argv[2])
        handle_check(config)
    # ...

if __name__ == "__main__":
    main()
```

**Runtime integration**:
```rust
// runtime.rs
pub fn run_container_connector( image : &str, command : &str ) -> Result< (), Error >
{
  let output = std::process::Command::new( "docker" )
    .args([ "run", "--rm", image, command ])
    .stdin( std::process::Stdio::piped() )
    .stdout( std::process::Stdio::piped() )
    .spawn()?;

  // Stream NDJSON from container stdout
  stream_messages( output.stdout )?;

  Ok(())
}
```

**Test case**:
```bash
# tests/manual/container_interface.md

## Build test connector
docker build -t iron-test-connector ./test-connector

## Test SPEC
docker run --rm iron-test-connector spec
# Expected: SPEC message JSON

## Test CHECK
docker run --rm iron-test-connector check --config '{"host":"localhost"}'
# Expected: ACK or ERROR message
```

**Time estimate**: 5 days (container interface + example connector + tests + docs)

---

### #10: File-Based Configuration

**Why high priority**: Passing credentials via command-line arguments is a security risk (visible in `ps`, logged in shell history, exposed in CI logs). File-based configs with restrictive permissions (0600) are standard practice.

**Current state**: No configuration loading mechanism defined.

**Implementation**:
```rust
// config.rs
use serde::{ Deserialize, Serialize };
use std::path::Path;

#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ConnectorConfig
{
  pub source : String,
  pub credentials : serde_json::Value,
  pub options : Option< serde_json::Value >,
}

pub fn load_config( path : &Path ) -> Result< ConnectorConfig, Error >
{
  let contents = std::fs::read_to_string( path )?;
  let config : ConnectorConfig = serde_json::from_str( &contents )?;

  // Verify file permissions
  let metadata = std::fs::metadata( path )?;
  let permissions = metadata.permissions();

  #[ cfg( unix ) ]
  {
    use std::os::unix::fs::PermissionsExt;
    let mode = permissions.mode();
    if mode & 0o077 != 0
    {
      return Err( Error::InsecureConfig( format!
      (
        "Config file {} has insecure permissions {:o}, should be 0600",
        path.display(),
        mode
      )));
    }
  }

  Ok( config )
}
```

**CLI integration**:
```rust
// bin/iron.rs
Commands::Read { config, catalog } =>
{
  let cfg = load_config( Path::new( &config ) )?;
  run_read( &cfg, &catalog )?;
},
```

**Config file format**:
```json
// config.json (chmod 600)
{
  "source": "postgres",
  "credentials": {
    "host": "localhost",
    "port": 5432,
    "database": "mydb",
    "username": "user",
    "password": "secret123"
  },
  "options": {
    "timeout_ms": 30000,
    "max_retries": 3
  }
}
```

**Test case**:
```rust
// tests/config_security.rs
#[ test ]
#[ cfg( unix ) ]
fn test_rejects_insecure_config()
{
  let path = "/tmp/test_config.json";
  std::fs::write( path, r#"{"source":"test"}"# )?;
  std::fs::set_permissions( path, std::fs::Permissions::from_mode( 0o644 ) )?;

  let result = load_config( Path::new( path ) );
  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "insecure permissions" ) );
}

#[ test ]
fn test_loads_secure_config()
{
  let path = "/tmp/test_config_secure.json";
  std::fs::write( path, r#"{"source":"test"}"# )?;
  std::fs::set_permissions( path, std::fs::Permissions::from_mode( 0o600 ) )?;

  let result = load_config( Path::new( path ) );
  assert!( result.is_ok() );
}
```

**Time estimate**: 2 days (config loading + permission checks + tests)

---

### #11: Fail-Fast Check

**Why critical**: Without connection validation, expensive READ/WRITE operations fail late. If an agent tries to read from a database with wrong credentials, it shouldnt wait until the query executes to discover the error.

**Current state**: AUTH message exists, but doesn't test the connection.

**Implementation**:
```rust
// Add to AuthMessage
impl AuthMessage
{
  /// Validate credentials by testing connection.
  pub async fn check_connection( &self ) -> Result< (), Error >
  {
    match &self.credentials
    {
      Credentials::ApiKey { key, .. } =>
      {
        // Test API key by making lightweight request
        let client = reqwest::Client::new();
        let response = client
          .get( &self.endpoint )
          .header( "Authorization", format!( "Bearer {}", key ) )
          .send()
          .await?;

        if !response.status().is_success()
        {
          return Err( Error::AuthFailed( "Invalid API key".to_string() ) );
        }
      },
      Credentials::Basic { username, password } =>
      {
        // Test database connection
        let conn_string = format!
        (
          "postgres://{}:{}@{}",
          username,
          password,
          self.endpoint
        );
        let pool = sqlx::postgres::PgPool::connect( &conn_string ).await?;
        pool.close().await;
      },
      // ... other credential types
    }

    Ok(())
  }
}
```

**CLI integration**:
```rust
Commands::Check { config } =>
{
  let cfg = load_config( Path::new( &config ) )?;
  let auth_msg = AuthMessage::from_config( &cfg );

  match auth_msg.check_connection().await
  {
    Ok( _ ) =>
    {
      let ack = IronMessage::Ack( AckMessage
      {
        request_id : new_request_id(),
        status : Status::Success,
        data : None,
        metadata : None,
      });
      println!( "{}", serde_json::to_string( &ack )? );
    },
    Err( e ) =>
    {
      let err = IronMessage::Error( ErrorMessage::new
      (
        "CHECK_FAILED".to_string(),
        e.to_string(),
      ));
      println!( "{}", serde_json::to_string( &err )? );
      std::process::exit( 1 );
    },
  }
},
```

**Test case**:
```rust
// tests/fail_fast_check.rs
#[ tokio::test ]
async fn test_check_detects_invalid_credentials()
{
  let auth = AuthMessage
  {
    request_id : new_request_id(),
    endpoint : "postgres://localhost:5432/testdb".to_string(),
    credentials : Credentials::Basic
    {
      username : "invalid_user".to_string(),
      password : "wrong_password".to_string(),
    },
  };

  let result = auth.check_connection().await;
  assert!( result.is_err() );
}

#[ tokio::test ]
async fn test_check_succeeds_with_valid_credentials()
{
  let auth = AuthMessage
  {
    request_id : new_request_id(),
    endpoint : "postgres://localhost:5432/testdb".to_string(),
    credentials : Credentials::Basic
    {
      username : "test_user".to_string(),
      password : "test_password".to_string(),
    },
  };

  let result = auth.check_connection().await;
  assert!( result.is_ok() );
}
```

**Time estimate**: 3 days (connection testing + CLI check command + tests)

---

### #12: Graceful Degradation

**Why medium priority**: Schema mismatches shouldnt cause catastrophic failures. If a database adds a new column or an API adds a new field, connectors should handle it gracefully instead of failing.

**Current state**: No explicit rules for handling schema evolution.

**Implementation**:

**Principle**: Unknown fields are warnings, not errors.

```rust
// In protocol.rs
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( deny_unknown_fields ) ] // WRONG - breaks on new fields
pub struct ReadMessage { ... }

// CORRECT - ignore unknown fields
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct ReadMessage { ... }
```

**Runtime behavior**:
```rust
// runtime.rs
pub fn handle_message( json : &str ) -> Result< (), Error >
{
  match serde_json::from_str::< IronMessage >( json )
  {
    Ok( msg ) => process_message( msg ),
    Err( e ) =>
    {
      // Log warning but continue
      eprintln!( "WARNING: Failed to parse message: {}. Skipping.", e );
      Ok(()) // Don't fail the entire stream
    },
  }
}
```

**Test case**:
```rust
// tests/graceful_degradation.rs
#[ test ]
fn test_handles_unknown_fields()
{
  let json = r#"
  {
    "type": "READ",
    "request_id": "123e4567-e89b-12d3-a456-426614174000",
    "source": "test",
    "operation": {"type": "SQL", "query": "SELECT 1"},
    "new_future_field": "some_value"
  }
  "#;

  // Should parse successfully, ignoring unknown field
  let msg : IronMessage = serde_json::from_str( json ).unwrap();
  assert_eq!( msg.message_type(), "READ" );
}

#[ test ]
fn test_handles_unknown_message_type()
{
  let json = r#"{"type": "FUTURE_MESSAGE_TYPE", "data": "..."}"#;

  // Should log warning but not crash
  let result = handle_message( json );
  assert!( result.is_ok() );
}
```

**Schema Evolution Rules**:
1. **Adding fields**: Always safe (old connectors ignore them)
2. **Removing fields**: Deprecate for 2 major versions first
3. **Renaming fields**: Add new field, deprecate old field
4. **Changing types**: Never allowed (add new field instead)

**Time estimate**: 2 days (serde config + runtime behavior + tests + docs)

---

### #13: Semantic Versioning

**Why high priority**: Without version checking, the runtime can't detect incompatible connectors. If protocol changes from v0.1 to v0.2 with breaking changes, old connectors will fail with cryptic deserialization errors instead of clear version mismatches.

**Current state**: No `protocol_version` field in messages.

**Implementation**:
```rust
// protocol.rs
pub const PROTOCOL_VERSION : &str = "0.1.0";

// Add to SpecMessage (from principle #8)
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct SpecMessage
{
  pub protocol_version : String, // "0.1.0"
  pub connector_name : String,
  pub connector_version : String,
  pub config_schema : serde_json::Value,
}

// Runtime version check
pub fn check_compatibility( connector_version : &str ) -> Result< (), Error >
{
  let runtime_version = semver::Version::parse( PROTOCOL_VERSION )?;
  let connector_version = semver::Version::parse( connector_version )?;

  // Major version must match
  if runtime_version.major != connector_version.major
  {
    return Err( Error::IncompatibleVersion( format!
    (
      "Runtime protocol v{} incompatible with connector protocol v{}",
      runtime_version,
      connector_version
    )));
  }

  // Minor version: runtime >= connector
  if runtime_version.minor < connector_version.minor
  {
    return Err( Error::IncompatibleVersion( format!
    (
      "Runtime protocol v{} too old for connector protocol v{}",
      runtime_version,
      connector_version
    )));
  }

  Ok(())
}
```

**CLI integration**:
```rust
Commands::Read { config, catalog } =>
{
  // Get connector spec first
  let spec = get_connector_spec( &config )?;
  check_compatibility( &spec.protocol_version )?;

  // Proceed with read
  run_read( &config, &catalog )?;
},
```

**Test case**:
```rust
// tests/version_compatibility.rs
#[ test ]
fn test_rejects_major_version_mismatch()
{
  let result = check_compatibility( "1.0.0" ); // Runtime is 0.1.0
  assert!( result.is_err() );
}

#[ test ]
fn test_rejects_newer_minor_version()
{
  let result = check_compatibility( "0.2.0" ); // Runtime is 0.1.0
  assert!( result.is_err() );
}

#[ test ]
fn test_accepts_older_minor_version()
{
  let result = check_compatibility( "0.0.5" ); // Runtime is 0.1.0
  assert!( result.is_ok() );
}

#[ test ]
fn test_accepts_same_version()
{
  let result = check_compatibility( "0.1.0" );
  assert!( result.is_ok() );
}
```

**Time estimate**: 2 days (version field + semver checks + tests)

---

### #14: Additive Evolution

**Why high priority**: Rust enums are not forward-compatible by default. If protocol v0.2 adds a new message type `BATCH`, old connectors will fail to deserialize it. Adding `#[non_exhaustive]` and an `Unknown` variant enables graceful handling.

**Current state**: `IronMessage` enum has no forward-compatibility mechanism.

**Implementation**:
```rust
// protocol.rs
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( tag = "type" ) ]
#[ non_exhaustive ] // Prevents exhaustive matching in external crates
pub enum IronMessage
{
  #[ serde( rename = "READ" ) ]
  Read( ReadMessage ),

  #[ serde( rename = "WRITE" ) ]
  Write( WriteMessage ),

  #[ serde( rename = "QUERY" ) ]
  Query( QueryMessage ),

  #[ serde( rename = "SCHEMA" ) ]
  Schema( SchemaMessage ),

  #[ serde( rename = "AUTH" ) ]
  Auth( AuthMessage ),

  #[ serde( rename = "ACK" ) ]
  Ack( AckMessage ),

  #[ serde( rename = "ERROR" ) ]
  Error( ErrorMessage ),

  #[ serde( rename = "LOG" ) ]
  Log( LogMessage ),

  #[ serde( rename = "METRICS" ) ]
  Metrics( MetricsMessage ),

  /// Unknown message type (forward compatibility)
  #[ serde( other ) ]
  Unknown,
}

impl IronMessage
{
  pub fn message_type( &self ) -> &'static str
  {
    match self
    {
      IronMessage::Read( _ ) => "READ",
      IronMessage::Write( _ ) => "WRITE",
      IronMessage::Query( _ ) => "QUERY",
      IronMessage::Schema( _ ) => "SCHEMA",
      IronMessage::Auth( _ ) => "AUTH",
      IronMessage::Ack( _ ) => "ACK",
      IronMessage::Error( _ ) => "ERROR",
      IronMessage::Log( _ ) => "LOG",
      IronMessage::Metrics( _ ) => "METRICS",
      IronMessage::Unknown => "UNKNOWN",
    }
  }
}
```

**Runtime handling**:
```rust
// runtime.rs
pub fn process_message( msg : IronMessage ) -> Result< (), Error >
{
  match msg
  {
    IronMessage::Read( m ) => handle_read( m ),
    IronMessage::Write( m ) => handle_write( m ),
    // ...
    IronMessage::Unknown =>
    {
      // Log but don't fail
      eprintln!( "WARNING: Received unknown message type, skipping" );
      Ok(())
    },
  }
}
```

**Test case**:
```rust
// tests/forward_compatibility.rs
#[ test ]
fn test_handles_future_message_type()
{
  let json = r#"{"type": "BATCH_READ", "requests": [...]}"#;

  let msg : IronMessage = serde_json::from_str( json ).unwrap();
  assert_eq!( msg.message_type(), "UNKNOWN" );

  // Should process without error
  let result = process_message( msg );
  assert!( result.is_ok() );
}

#[ test ]
fn test_non_exhaustive_prevents_match()
{
  // This should not compile (caught at compile time)
  // match msg {
  //   IronMessage::Read(_) => {},
  //   IronMessage::Write(_) => {},
  //   // ... missing Unknown case
  // } // ERROR: non-exhaustive match

  // Must use catch-all
  match msg
  {
    IronMessage::Read( _ ) => {},
    _ => {}, // Required
  }
}
```

**Time estimate**: 3 days (enum changes + runtime handling + tests + verify no breaking changes)

---

## "Could" Principles: Detailed Rationale

### #15: Schema as Contract

**Why "could" not "should"**: Strict schema enforcement may be too rigid for AI agents. Agents often need to explore unknown data structures flexibly.

**Current state**: SCHEMA message exists for querying schemas, but no enforcement mechanism.

**When to implement**: If production use cases show agents frequently breaking due to unexpected data formats, add optional schema validation.

**Implementation sketch**:
```rust
// Add to ReadOptions
pub struct ReadOptions
{
  pub timeout_ms : Option< u64 >,
  pub max_retries : Option< u32 >,
  pub enforce_schema : Option< bool >, // Default: false
  pub schema : Option< serde_json::Value >, // JSONSchema
}

// In connector
pub fn read_with_validation( msg : ReadMessage ) -> Result< Vec< serde_json::Value >, Error >
{
  let rows = execute_query( &msg )?;

  if let Some( schema ) = msg.options.and_then( |o| o.schema )
  {
    if msg.options.and_then( |o| o.enforce_schema ).unwrap_or( false )
    {
      for row in &rows
      {
        validate_against_schema( row, &schema )?;
      }
    }
  }

  Ok( rows )
}
```

**Time estimate**: 1 week (optional validation + tests + docs)

---

### #16: Checkpointing

**Why "could" not "should"**: Most agent operations are short-lived (< 1 minute). State management adds complexity without clear benefit for typical use cases.

**Current state**: No checkpoint messages.

**When to implement**: If long-running operations emerge (e.g., processing 1TB dataset over 6 hours), add checkpoint support for resumability.

**Implementation sketch**:
```rust
// Add to protocol.rs
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct CheckpointMessage
{
  pub request_id : Uuid,
  pub operation_id : Uuid,
  pub state : serde_json::Value, // Opaque state blob
  pub progress : Option< f64 >, // 0.0 to 1.0
}

#[ serde( rename = "CHECKPOINT" ) ]
Checkpoint( CheckpointMessage ),

// In connector
pub fn read_resumable( msg : ReadMessage, checkpoint : Option< CheckpointMessage > ) -> Result< (), Error >
{
  let start_offset = checkpoint
    .and_then( |c| c.state.get( "offset" ).and_then( |v| v.as_u64() ) )
    .unwrap_or( 0 );

  let rows = fetch_rows( &msg, start_offset )?;

  for ( i, row ) in rows.iter().enumerate()
  {
    output_row( row )?;

    // Emit checkpoint every 10k rows
    if i % 10_000 == 0
    {
      let checkpoint = CheckpointMessage
      {
        request_id : msg.request_id,
        operation_id : Uuid::new_v4(),
        state : serde_json::json!({ "offset" : start_offset + i as u64 }),
        progress : Some( i as f64 / rows.len() as f64 ),
      };
      output_checkpoint( checkpoint )?;
    }
  }

  Ok(())
}
```

**Time estimate**: 1 week (checkpoint message + resume logic + tests)

---

### #17: Batch Operations

**Why "could" not "should"**: Single request/response pattern is simpler and sufficient for most use cases. Batching adds complexity.

**Current state**: One READ message → one ACK with data.

**When to implement**: If agents frequently make 100+ similar queries (e.g., "get user profile for IDs 1-1000"), batch support reduces latency.

**Implementation sketch**:
```rust
// Add to protocol.rs
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct BatchReadMessage
{
  pub request_id : Uuid,
  pub requests : Vec< ReadMessage >,
}

#[ serde( rename = "BATCH_READ" ) ]
BatchRead( BatchReadMessage ),

// Response includes request_id mapping
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct BatchAckMessage
{
  pub request_id : Uuid,
  pub results : Vec< BatchResult >,
}

#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct BatchResult
{
  pub request_id : Uuid, // From original ReadMessage
  pub status : Status,
  pub data : Option< serde_json::Value >,
}
```

**Test case**:
```rust
// tests/batch_operations.rs
#[ test ]
fn test_batch_read()
{
  let batch = BatchReadMessage
  {
    request_id : new_request_id(),
    requests : vec!
    [
      ReadMessage { /* query user 1 */ },
      ReadMessage { /* query user 2 */ },
      ReadMessage { /* query user 3 */ },
    ],
  };

  let response = execute_batch( batch )?;
  assert_eq!( response.results.len(), 3 );
}
```

**Time estimate**: 1 week (batch messages + connector support + tests)

---

### #18: Namespace Isolation

**Why "could" not "should"**: Single-tenant use case is primary. Multi-tenancy adds significant complexity.

**Current state**: No namespace concept.

**When to implement**: If SaaS use case emerges (one runtime serving multiple customers), add namespace isolation to prevent data leakage.

**Implementation sketch**:
```rust
// Add to all messages
pub struct ReadMessage
{
  pub request_id : Uuid,
  pub namespace : Option< String >, // e.g., "tenant-123"
  pub source : String,
  pub operation : ReadOperation,
  pub options : Option< ReadOptions >,
}

// Runtime enforces isolation
pub fn execute_read( msg : ReadMessage, runtime_namespace : &str ) -> Result< (), Error >
{
  if let Some( ns ) = msg.namespace
  {
    if ns != runtime_namespace
    {
      return Err( Error::NamespaceMismatch( format!
      (
        "Message namespace '{}' doesn't match runtime namespace '{}'",
        ns,
        runtime_namespace
      )));
    }
  }

  // Proceed with isolated read
  read_data( &msg )?;
  Ok(())
}
```

**Test case**:
```rust
// tests/namespace_isolation.rs
#[ test ]
fn test_rejects_wrong_namespace()
{
  let msg = ReadMessage
  {
    namespace : Some( "tenant-123".to_string() ),
    // ...
  };

  let result = execute_read( msg, "tenant-456" );
  assert!( result.is_err() );
}
```

**Time estimate**: 3 days (namespace field + runtime checks + tests)

---

## Protocol Messages

### Core Envelope

All messages use the `IronMessage` enum with tagged union serialization:

```rust
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( tag = "type" ) ]
pub enum IronMessage
{
  #[ serde( rename = "READ" ) ]
  Read( ReadMessage ),

  #[ serde( rename = "WRITE" ) ]
  Write( WriteMessage ),

  #[ serde( rename = "QUERY" ) ]
  Query( QueryMessage ),

  #[ serde( rename = "SCHEMA" ) ]
  Schema( SchemaMessage ),

  #[ serde( rename = "AUTH" ) ]
  Auth( AuthMessage ),

  #[ serde( rename = "ACK" ) ]
  Ack( AckMessage ),

  #[ serde( rename = "ERROR" ) ]
  Error( ErrorMessage ),

  #[ serde( rename = "LOG" ) ]
  Log( LogMessage ),

  #[ serde( rename = "METRICS" ) ]
  Metrics( MetricsMessage ),
}
```

### Request Messages

- **READ**: Read data from a source (SQL query, file read, HTTP request, cache get, object get)
- **WRITE**: Write data to a destination (SQL insert, file write, HTTP post, cache set, object put)
- **QUERY**: Query metadata or filter data
- **SCHEMA**: Request schema information about a source
- **AUTH**: Authenticate credentials before operations

### Response Messages

- **ACK**: Successful operation with optional data payload
- **ERROR**: Operation failure with error code, severity, details

### Diagnostic Messages

- **LOG**: Structured logging (debug, info, warn, error levels)
- **METRICS**: Performance metrics (latency, throughput, resource usage)

### Transport

- **Format**: NDJSON (newline-delimited JSON)
- **Channels**: STDIN (requests from agent), STDOUT (responses to agent)
- **Streaming**: Line-by-line processing to avoid OOM

---

## Architecture

### Three-Party System

```
┌─────────┐         ┌─────────┐         ┌───────────┐
│         │         │         │         │           │
│  Agent  │────────▶│ Runtime │────────▶│ Connector │
│         │         │         │         │           │
└─────────┘         └─────────┘         └───────────┘
    │                    │                     │
    │                    │                     │
    ▼                    ▼                     ▼
 Business           Orchestration         Data Source
  Logic              + Security            Interface
```

### Isolation Guarantees

1. **Agent ↔ Runtime**: Agent never accesses data sources directly
2. **Runtime ↔ Connector**: Runtime validates auth before forwarding requests
3. **Connector ↔ Source**: Connector implements source-specific logic

### Message Flow

```
Agent                Runtime              Connector
  │                     │                     │
  │──AUTH──────────────▶│                     │
  │                     │──AUTH──────────────▶│
  │                     │                     │──validate credentials
  │                     │◀──ACK───────────────│
  │◀──ACK───────────────│                     │
  │                     │                     │
  │──READ──────────────▶│                     │
  │                     │──READ──────────────▶│
  │                     │                     │──execute query
  │                     │◀──ACK(data)─────────│
  │◀──ACK(data)─────────│                     │
```

---

## Implementation Roadmap

### Critical Path (11 days)

Must-have for basic functionality:

1. **Streaming Over Batching** (#6) - 3 days
   - Implement `BufReader::lines()` on stdin/stdout
   - Test with 100MB+ datasets

2. **Command-Based Interface** (#7) - 5 days
   - CLI with `spec`, `check`, `read`, `write` commands
   - Integration with runtime

3. **Fail-Fast Check** (#11) - 3 days
   - Connection testing in AUTH
   - CLI `check` command

### High Priority (15 days)

Production-ready features:

1. **Schema-Driven Validation** (#8) - 3 days
   - SPEC message with JSONSchema
   - Config validation

2. **Container as Universal Runtime** (#9) - 5 days
   - Container interface definition
   - Example Python connector

3. **File-Based Configuration** (#10) - 2 days
   - Config loading with permission checks
   - Secure credential handling

4. **Semantic Versioning** (#13) - 2 days
   - Protocol version field
   - Compatibility checking

5. **Additive Evolution** (#14) - 3 days
   - `#[non_exhaustive]` + Unknown variant
   - Forward compatibility tests

### Medium Priority (2 days)

Nice-to-have improvements:

1. **Graceful Degradation** (#12) - 2 days
   - Schema mismatch handling
   - Unknown field tolerance

### Total to Production-Ready: 28 days

---

## Testing Strategy

### Unit Tests

- Message serialization/deserialization roundtrips
- Type safety guarantees
- Error handling edge cases

### Integration Tests

- NDJSON stream processing
- CLI command invocations
- Container connector lifecycle
- Configuration loading

### Manual Tests

- Large dataset streaming (no OOM)
- Connection validation (fail-fast)
- Schema evolution (backward compatibility)
- Multi-connector workflows

### Test Organization

All tests in `tests/` directory:
- `tests/message_serialization.rs` - Message type tests
- `tests/streaming.rs` - Streaming behavior tests
- `tests/cli_interface.rs` - CLI integration tests
- `tests/manual/` - Manual test plans and procedures

---

## References

- [NDJSON Specification](http://ndjson.org/)
- [JSON Schema](https://json-schema.org/)
- [Semantic Versioning](https://semver.org/)
