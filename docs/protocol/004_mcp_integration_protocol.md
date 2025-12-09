# MCP Integration Protocol

**Purpose:** Model Context Protocol implementation for tool discovery and invocation.

---

### User Need

Understand how Iron Cage integrates with MCP for standardized tool access.

### Core Idea

**MCP provides standard interface for LLM tools with Iron Cage governance:**

```
Agent → Iron Cage → MCP Tools
        |
        - tools/list: Discover tools
        - tools/call: Invoke with governance
        - Budget tracking
        - Safety validation
```

### Tool Discovery

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "name": "read_file",
        "description": "Read file contents",
        "inputSchema": {
          "type": "object",
          "properties": {"path": {"type": "string"}},
          "required": ["path"]
        }
      }
    ]
  },
  "id": 1
}
```

### Tool Invocation

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "read_file",
    "arguments": {"path": "/etc/hosts"}
  },
  "id": 2
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {"type": "text", "text": "127.0.0.1 localhost\n..."}
    ]
  },
  "id": 2
}
```

### Iron Cage Governance

**Wraps MCP with budget and safety:**

1. Agent calls MCP tool via Iron Cage
2. Iron Cage validates tool authorization
3. Iron Cage checks budget before invocation
4. Iron Cage invokes actual MCP tool
5. Iron Cage scans result for PII
6. Iron Cage tracks cost
7. Iron Cage returns result

### Error Mapping

**Iron Cage errors → MCP error codes:**

| Iron Cage Error | MCP Code | Message |
|----------------|----------|---------|
| BudgetExceeded | -32001 | Budget limit exceeded |
| PiiDetected | -32002 | PII in tool params |
| Unauthorized | -32003 | Tool not authorized |
| RateLimited | -32004 | Rate limit exceeded |

---

*Related: [../capabilities/006_mcp_integration.md](../capabilities/006_mcp_integration.md) | [001_ironlang_data_protocol.md](001_ironlang_data_protocol.md)*
