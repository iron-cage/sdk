# IronLang Data Protocol

**Purpose:** NDJSON-based message protocol for agent-data source communication.

---

### User Need

Understand how agents communicate with data sources through the IronLang protocol.

### Core Idea

**Streaming NDJSON messages for stateless agent-data interactions:**

```
Agent → Runtime → Data Source
        |
        IronLang Messages (NDJSON)
        - READ: Fetch data
        - WRITE: Store data
        - QUERY: Query metadata
        - AUTH: Authenticate
        - ACK: Successful response
        - ERROR: Error response
        - LOG: Diagnostic logging
```

### Message Types

**Request messages (Agent → Runtime → Connector):**

| Type | Purpose | Example |
|------|---------|---------|
| **READ** | Read data from source | SQL query, file read, cache get |
| **WRITE** | Write data to destination | SQL insert, file write, cache set |
| **QUERY** | Query metadata | List tables, list files |
| **AUTH** | Authenticate credentials | Validate connection, API key |

**Response messages (Connector → Runtime → Agent):**

| Type | Purpose | When Sent |
|------|---------|-----------|
| **ACK** | Successful operation | After READ (with data), WRITE (confirmation) |
| **ERROR** | Operation failure | Auth failed, permission denied |
| **LOG** | Structured logging | Debug, info, warn messages |

### Message Format

**NDJSON (Newline-Delimited JSON):**

```json
{"type":"READ","request_id":"req-001","source":"db","query":"SELECT * FROM users"}
{"type":"ACK","request_id":"req-001","data":[{"id":1,"name":"Alice"}]}
{"type":"WRITE","request_id":"req-002","source":"db","data":{"id":3,"name":"Bob"}}
{"type":"ACK","request_id":"req-002","status":"written"}
{"type":"ERROR","request_id":"req-003","code":"AUTH_FAILED","message":"Invalid token"}
```

**Characteristics:**
- One JSON object per line
- Line-buffered (streaming-friendly)
- Each message independent (stateless)
- request_id for correlation

### Actor Isolation

**Three-party architecture prevents direct data access:**

```
Agent (AI, untrusted)
  → Runtime (validates, enforces policies)
    → Connector (implements data access)
      → Data Source (database, file, API)
```

**Why:** Agent never talks to data source directly. Runtime validates all operations.

### Transport Channels

| Channel | Direction | Content |
|---------|-----------|---------|
| STDIN | Runtime → Connector | Request messages |
| STDOUT | Connector → Runtime | Response messages (ACK, ERROR, LOG) |
| STDERR | Connector → Runtime | Diagnostics (not parsed) |

---

*Related: [002_rest_api_protocol.md](002_rest_api_protocol.md) | [../capabilities/008_enterprise_data_access.md](../capabilities/008_enterprise_data_access.md)*
