# Protocol 004: MCP Integration Protocol

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

### Scope

Iron Cage integration with Model Context Protocol (MCP) for standardized tool discovery and invocation with governance layer.

**In Scope:**
- MCP message format (JSON-RPC 2.0 based)
- Tool discovery protocol (tools/list)
- Tool invocation protocol (tools/call)
- Iron Cage governance layer (budget tracking, safety validation, authorization)
- Error mapping (Iron Cage errors → MCP error codes)
- Tool registry and policy enforcement

**Out of Scope:**
- MCP specification itself (see https://modelcontextprotocol.io)
- IronLang data protocol (see [001_ironlang_data_protocol.md](001_ironlang_data_protocol.md))
- REST API protocol (see [002_rest_api_protocol.md](002_rest_api_protocol.md))
- Implementation details (see module specifications)

---

### Purpose

Enable AI agents to discover and invoke tools through standard MCP protocol while adding Iron Cage budget tracking and safety validation.

**Problem:**

Tool integration for AI agents without standards:
- Custom code required per tool (filesystem ≠ database ≠ API)
- Manual configuration (agent must know which tools exist)
- No access control (all tools available to all agents)
- No budget tracking for tool invocations
- No safety validation of tool parameters

**Solution:**

MCP with Iron Cage governance provides:
- Standard protocol (MCP) for tool discovery and invocation
- Auto-discovery (agents find tools at runtime via tools/list)
- Policy enforcement (which agents can use which tools)
- Budget tracking (tool invocations count against agent budget)
- Safety validation (PII detection, parameter validation)
- Centralized governance without breaking MCP compatibility

---

### Protocol Definition

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

### Cross-References

**Dependencies:**
- External: Model Context Protocol specification (https://modelcontextprotocol.io)

**Used By:**
- [capabilities/006: MCP Integration](../capabilities/006_mcp_integration.md) - High-level capability overview using this protocol
- [architecture/004: Data Flow](../architecture/004_data_flow.md) - Tool invocation in request flow (Step 5)

**Related:**
- [001: IronLang Data Protocol](001_ironlang_data_protocol.md) - Alternative data access protocol
- [capabilities/003: Safe Execution](../capabilities/003_safe_execution.md) - Tool execution sandboxing
- [capabilities/004: AI Safety Guardrails](../capabilities/004_ai_safety_guardrails.md) - Parameter validation for tools

**Implementation:**
- Source: (TBD - MCP integration pending implementation)
- Specification: (TBD - MCP integration spec pending)

---

**Last Updated:** 2025-12-09
**Protocol Version:** MCP 1.0 + Iron Cage Governance Layer
**Status:** Design phase (not yet implemented)
