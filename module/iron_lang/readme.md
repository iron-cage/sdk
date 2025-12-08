# iron_lang

Protocol for AI agents to safely access and manipulate data sources.

## Overview

`iron_lang` provides a type-safe, performant communication protocol that enables AI agents (like Claude, GPT-4, or custom models) to read, write, and query data from various sources including databases, filesystems, APIs, caches, and object storage.

Inspired by the [Airbyte Protocol](https://github.com/airbytehq/airbyte-protocol), `iron_lang` focuses on:
- **AI-first design**: Optimized for agent interaction patterns
- **Type safety**: Full Rust implementation with compile-time guarantees
- **Simplicity**: NDJSON over STDIN/STDOUT
- **Extensibility**: Easy to add new data sources

## Features

- **Type-safe protocol**: Rust types with compile-time guarantees
- **NDJSON transport**: Newline-Delimited JSON over STDIN/STDOUT
- **Multiple data sources**: SQL, files, HTTP, cache, object storage
- **Authentication**: Built-in auth with multiple credential types
- **Observability**: Logging and metrics built into protocol

### Scope

**Responsibilities:**
Defines NDJSON-based communication protocol for AI agent data exchange with streaming support and event-based message handling. Provides structured format (9 message types: READ, WRITE, QUERY, SCHEMA, AUTH, ACK, ERROR, LOG, METRICS) for agent inputs, outputs, errors, and telemetry events inspired by Airbyte Protocol. Requires Rust 1.75+, all platforms supported, uses serde for serialization.

**In Scope:**
- Protocol message definitions (IronMessage enum with 9 variants)
- Type-safe Rust types with compile-time guarantees
- NDJSON serialization/deserialization
- Multiple data source operations (SQL, files, HTTP, cache, object storage)
- Authentication message types
- Observability messages (logging, metrics)
- Request/response patterns for agent-runtime communication
- Module stubs (runtime, connectors, auth, router)

**Out of Scope:**
- ❌ Runtime implementation → stub only (future work)
- ❌ Connector implementations → stub only (future work)
- ❌ Authentication logic → stub only (future work)
- ❌ Request routing → stub only (future work)
- ❌ Database drivers → delegated to connector implementations
- ❌ HTTP client implementations → delegated to connector implementations
- ❌ Agent implementations (Claude, GPT-4) → separate projects

## Message Types

The protocol defines 9 message types:

- **READ**: Request data from source (SQL, files, HTTP, cache, objects)
- **WRITE**: Write data to destination
- **QUERY**: Query metadata (tables, files, keys)
- **SCHEMA**: Request schema information
- **AUTH**: Authenticate agent
- **ACK**: Acknowledge successful operation
- **ERROR**: Report operation failure
- **LOG**: Diagnostic logging
- **METRICS**: Performance metrics

## Usage

```rust
use iron_lang::protocol::{ IronMessage, ReadMessage, ReadOperation, SqlQuery };
use iron_lang::protocol::new_request_id;

// Create a READ message for SQL query
let msg = IronMessage::Read( ReadMessage
{
  request_id : new_request_id(),
  source : "production_db".to_string(),
  operation : ReadOperation::Sql( SqlQuery
  {
    query : "SELECT * FROM users WHERE created_at > $1".to_string(),
    parameters : Some( vec!
    [
      SqlParameter::String( "2024-01-01".to_string() ),
    ]),
  }),
  options : None,
});

// Serialize to NDJSON
let json = serde_json::to_string( &msg )?;
println!( "{}", json );
```

## Protocol Communication

Messages are exchanged as **NDJSON** (Newline-Delimited JSON) over **STDIN/STDOUT**:

```
Agent                     Runtime                    Connector
  |                          |                          |
  |---READ (SQL query)------>|                          |
  |                          |---execute_read()-------->|
  |                          |                   [query DB]
  |                          |<--rows-------------------|
  |<--ACK (result data)------|                          |
```

## Modules

- **protocol** - Message type definitions and serialization
- **runtime** - Message processing engine (stub)
- **connectors** - Connector trait and utilities (stub)
- **auth** - Authentication and authorization (stub)
- **router** - Request routing and dispatch (stub)

## Testing

```bash
cargo test -p iron_lang
```

## License

MIT
