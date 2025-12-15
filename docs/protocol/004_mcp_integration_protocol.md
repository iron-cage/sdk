# Protocol: MCP Integration Protocol



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
- IronLang data protocol (archived, not in use for Control Panel API)
- REST API protocol (see [002_rest_api_protocol.md](002_rest_api_protocol.md))
- Implementation details (see module specifications)


### Purpose

**User Need**: Backend developers (integrating tools), SDK developers (building agent frameworks), AI researchers (experimenting with tool-augmented models), and operations teams (enforcing governance policies) need standardized tool discovery and invocation protocol enabling AI agents to dynamically find and use tools (filesystem, database, API, custom) at runtime without hardcoding tool-specific integration logic for each new capability, while enforcing organizational safety guardrails (budget limits $10 agent max, PII detection preventing sensitive data leakage, authorization policies restricting dangerous tools to approved agents only, cost tracking $0.015 per tool call) that prevent runaway costs from uncontrolled tool usage, security violations from unrestricted access, and compliance failures from audit-less invocations, without requiring custom governance implementation duplicated across every tool integration or breaking compatibility with standard Model Context Protocol (MCP) ecosystem tools.

**Solution**: Model Context Protocol (MCP) JSON-RPC 2.0 based wire protocol with Iron Cage governance layer wrapping tool operations. Implement 2 core MCP methods: tools/list (discovery returning tool array with name read_file, description, inputSchema object/properties/required fields for each available tool) and tools/call (invocation accepting name, arguments params returning content array results). Wrap each tool invocation with Iron Cage 7-step governance flow: (1) agent calls MCP tool via Iron Cage proxy, (2) validate tool authorization against agent permissions, (3) check budget before invocation preventing $10 limit breach, (4) invoke actual MCP tool with original params, (5) scan result for PII before returning, (6) track cost $0.015 per call against agent budget, (7) return result or governance error. Map Iron Cage safety errors to standard MCP error codes: BudgetExceeded → -32001, PiiDetected → -32002, Unauthorized → -32003, RateLimited → -32004. Adhere to Iron Cage standards: ID Format (agent_<uuid> in governance), Data Format (ISO 8601 timestamps, decimal cost 2 decimals), Error Format (machine-readable TOOL_NOT_FOUND/UNAUTHORIZED/BUDGET_EXCEEDED codes).

**Key Insight**: MCP standard protocol eliminates custom tool integration complexity (no hardcoded tool-specific logic per capability, agents discover tools dynamically via tools/list instead of manual configuration, standard JSON-RPC 2.0 wire format works with ecosystem tools) while Iron Cage governance layer adds budget/safety/authorization enforcement without breaking MCP compatibility (transparent proxy wrapping tools/call, standard MCP error codes for governance failures, agents use standard MCP client unaware of governance). This architecture provides best of both worlds: open standard prevents vendor lock-in and enables ecosystem tool reuse (any MCP-compatible tool works immediately), while governance prevents the three critical failures plaguing unrestricted tool access (budget exhaustion from runaway loops $100+ costs, PII leakage from unvalidated outputs violating GDPR, unauthorized access to dangerous tools rm -rf enabled for all agents). Alternative approaches fail trade-offs: custom protocol requires reimplementing every tool and loses ecosystem benefits, governance-free MCP causes safety violations in production, per-tool governance duplicates validation logic across integrations.

---

**Status**: Specification
**Version**: 1.0.0
**Last Updated**: 2025-12-14
**Priority**: POST-PILOT


### Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- Tool IDs and invocation IDs use `prefix_uuid` format when applicable
- Agent IDs in governance layer: `agent_<uuid>`

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Timestamps in governance layer: ISO 8601 with Z suffix
- Cost tracking: Decimal with 2 decimal places

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Iron Cage errors mapped to MCP error codes
- Standard error response format in governance layer
- Machine-readable error codes: `TOOL_NOT_FOUND`, `UNAUTHORIZED`, `BUDGET_EXCEEDED`

**Note:** MCP protocol itself (JSON-RPC 2.0) is externally defined. Standards apply to Iron Cage governance layer.


### Protocol Definition

#### Tool Discovery

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

#### Tool Invocation

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

#### Iron Cage Governance

**Wraps MCP with budget and safety:**

1. Agent calls MCP tool via Iron Cage
2. Iron Cage validates tool authorization
3. Iron Cage checks budget before invocation
4. Iron Cage invokes actual MCP tool
5. Iron Cage scans result for PII
6. Iron Cage tracks cost
7. Iron Cage returns result

#### Error Mapping

**Iron Cage errors → MCP error codes:**

| Iron Cage Error | MCP Code | Message |
|----------------|----------|---------|
| BudgetExceeded | -32001 | Budget limit exceeded |
| PiiDetected | -32002 | PII in tool params |
| Unauthorized | -32003 | Tool not authorized |
| RateLimited | -32004 | Rate limit exceeded |


### Cross-References

#### Related Principles Documents
- Design Philosophy - Separation of Concerns principle separating MCP standard protocol from Iron Cage governance layer (transparent proxy wrapping enables ecosystem tool compatibility while enforcing organizational policies), Simplicity principle using standard JSON-RPC 2.0 wire format avoiding custom protocol complexity
- Quality Attributes - Interoperability (MCP standard protocol compatibility with ecosystem tools), Security (7-step governance flow with authorization/budget/PII validation), Maintainability (governance layer separation enables independent policy updates without MCP protocol changes), Extensibility (new tools integrate via standard tools/list registration without code changes)

#### Related Architecture Documents
- [Architecture: Data Flow](../architecture/004_data_flow.md) - Tool invocation in request flow Step 5, MCP tool proxy service integration in eleven-step data flow

#### Used By
- [Capabilities: MCP Integration](../capabilities/006_mcp_integration.md) - High-level capability overview using this protocol for standardized tool discovery and invocation (NOTE: File may not exist, forward reference for future implementation)

#### Dependencies
- [Standards: ID Format Standards](../standards/id_format_standards.md) - Agent ID format `agent_<uuid>` in governance layer, tool IDs and invocation IDs in MCP messages
- [Standards: Data Format Standards](../standards/data_format_standards.md) - ISO 8601 timestamp format in governance layer, decimal currency format (2 decimal places) for cost tracking $0.015 per tool call
- [Standards: Error Format Standards](../standards/error_format_standards.md) - Iron Cage error codes (TOOL_NOT_FOUND, UNAUTHORIZED, BUDGET_EXCEEDED) mapped to MCP error codes (-32001 to -32004), standard error response format in governance layer
- Model Context Protocol Specification (https://modelcontextprotocol.io) - External standard defining JSON-RPC 2.0 based tool discovery (tools/list) and invocation (tools/call) protocol, inputSchema format, content array response structure
- [Capabilities: Safe Execution](../capabilities/003_safe_execution.md) - Tool execution sandboxing referenced in Iron Cage governance Step 4 (invoke actual MCP tool), PII scanning in Step 5
- [Capabilities: AI Safety Guardrails](../capabilities/004_ai_safety_guardrails.md) - Parameter validation for tool arguments referenced in governance Step 2 (validate authorization), safety checks before invocation

#### Implementation
- Module: `module/iron_mcp_proxy/` (TBD - MCP integration pending implementation) - MCP proxy service wrapping tools with Iron Cage governance
- Source: `module/iron_mcp_proxy/src/protocol.rs` (TBD) - MCP JSON-RPC 2.0 message parsing, tools/list and tools/call handlers, error mapping
- Tests: `module/iron_mcp_proxy/tests/governance_test.rs` (TBD) - 7-step governance flow tests, error mapping validation (4 error codes), budget/authorization/PII scenarios
- Specification: `module/iron_mcp_proxy/spec.md` (TBD) - Functional requirements for MCP protocol implementation, governance layer integration, tool registry design
- Tool Registry: (TBD) - Tool discovery service implementing tools/list endpoint, tool metadata storage, permission policy enforcement

