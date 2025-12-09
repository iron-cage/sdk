# MCP Integration

**Concept:** Zero-configuration tool access enabling AI agents to use filesystem, databases, and external services via Model Context Protocol.

---

## User Need

AI agents need access to tools (file systems, databases, APIs) but:
- Each tool requires custom integration code
- No standard protocol for tool discovery and invocation
- Security policies vary per tool
- Difficult to add new tools to existing agents

## Core Idea

Leverage **Model Context Protocol (MCP)** - Anthropic's open standard for AI-tool communication:
1. Agents discover available tools automatically
2. Tools expose capabilities via standard schema
3. Platform controls which tools each agent can access
4. New tools added without agent code changes

The insight: MCP provides the **protocol** - Iron Cage adds the **governance** (which agents can use which tools, with what parameters).

## Key Components

- **MCP Server Registry** - Catalog of available tool servers
- **Auto-Discovery** - Agents find tools without configuration
- **Policy Layer** - Per-agent tool access control
- **Security Wrapper** - Validate tool invocations against policies

## Related Capabilities

- [Safe Execution](safe_execution.md) - Sandboxes tool execution
- [AI Safety Guardrails](ai_safety_guardrails.md) - Validates tool parameters
