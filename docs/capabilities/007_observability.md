# Observability

**Concept:** AI-native monitoring capturing agent traces, LLM metrics, and cost attribution across the platform.

---

## User Need

Traditional APM tools don't understand AI workloads:
- Can't trace multi-step agent reasoning
- Don't capture token usage or LLM latency
- No cost attribution per request/user/team
- Miss AI-specific failures (hallucinations, context overflow)

## Core Idea

Provide **AI-native observability** that captures:
1. Agent traces (full reasoning chain, not just HTTP calls)
2. LLM metrics (tokens, latency, model used, cost)
3. Safety events (guardrail triggers, policy violations)
4. Business attribution (cost per team, per use case)

The insight: AI observability requires **semantic understanding** of what agents are doing, not just infrastructure metrics.

## Key Components

- **Agent Tracer** - Captures reasoning steps, tool calls, decisions
- **LLM Metrics** - Token counts, latencies, costs per request
- **Safety Audit** - Log of all guardrail decisions
- **Attribution Engine** - Cost allocation by dimension

## Related Capabilities

- [LLM Access Control](002_llm_access_control.md) - Source of cost data
- [AI Safety Guardrails](004_ai_safety_guardrails.md) - Source of safety events
