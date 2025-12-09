# iron_cost - Specification

**Module:** iron_cost
**Layer:** 3 (Feature)
**Status:** Active

---

## Responsibility

Budget tracking and enforcement for LLM usage. Counts tokens using tiktoken, calculates costs based on provider pricing, enforces spending limits with automatic cutoffs and threshold warnings.

---

## Scope

**In Scope:**
- Token counting for OpenAI models (tiktoken integration)
- Cost calculation (tokens → USD)
- Budget enforcement with hard limits
- Threshold warnings (90% alert for demo)
- Per-agent cost attribution

**Out of Scope:**
- Multi-currency support (pilot: USD only)
- Multi-provider pricing (pilot: OpenAI only)
- Cost forecasting (see analytics features)
- Budget allocation per-agent (pilot: global budget)

---

## Dependencies

**Required Modules:**
- iron_types - Foundation types
- iron_state - Budget state persistence
- iron_telemetry - Warning alerts

**Required External:**
- tiktoken - Token counting for OpenAI models

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **Token Counter:** Counts tokens using tiktoken library
- **Cost Calculator:** Converts token counts to USD amounts
- **Budget Enforcer:** Blocks requests when spending limit exceeded
- **Threshold Detector:** Triggers warnings at configurable percentage

---

## Integration Points

**Used by:**
- iron_runtime - Budget checks before LLM calls
- iron_api - Cost reporting endpoints

**Uses:**
- iron_state - Persists budget and spending data
- iron_telemetry - Emits budget warning alerts

---

*For detailed pricing and algorithms, see spec/-archived_detailed_spec.md*
*For budget protocol, see docs/architecture/006_budget_control_protocol.md*
