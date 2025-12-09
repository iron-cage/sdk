# iron_lang - Specification

**Module:** iron_lang
**Layer:** 4 (Infrastructure)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

AI agent data protocol enabling safe data source access through type-safe NDJSON messages. Defines IronMessage types (READ, WRITE, AUTH, LOG, ERROR) for agent-data source communication with validation layer.

---

## Scope

**In Scope:**
- IronMessage enum (READ, WRITE, QUERY, AUTH, ACK, ERROR, LOG, METRICS)
- NDJSON transport format and parsing
- Message validation and type checking
- Protocol version compatibility
- Actor isolation (Agent → Runtime → Connector)

**Out of Scope:**
- Data source implementations (see data connectors)
- REST API protocol (see docs/protocol/002)
- WebSocket protocol (see docs/protocol/003)
- Budget control protocol (see docs/protocol/005)

---

## Dependencies

**Required Modules:**
- iron_types - Message type definitions
- iron_telemetry - Logging

**Required External:**
- serde - Serialization framework
- serde_json - JSON parsing

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **IronMessage Enum:** Tagged union for all message types
- **NDJSON Parser:** Streaming line-by-line JSON parsing
- **Message Validator:** Type checking and validation
- **Actor Model:** Three-party isolation (Agent/Runtime/Connector)

---

## Integration Points

**Used by:**
- iron_runtime - Routes messages between agent and data sources
- Data connectors - Implement data source interfaces

**Uses:**
- iron_types - Common type definitions

---

*For detailed protocol specification, see spec/-archived_detailed_spec.md*
*For protocol design, see docs/protocol/001_ironlang_data_protocol.md*
