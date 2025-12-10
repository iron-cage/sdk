# iron_cost - Specification

**Module:** iron_cost
**Layer:** 3 (Feature)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

Budget tracking and enforcement for LLM usage. Provides pricing data for multiple LLM providers, calculates costs based on token usage, and enforces spending limits with pre-reservation to prevent budget overshoot.

---

## Scope

**In Scope:**
- Multi-provider pricing data (LiteLLM pricing source)
- Cost calculation (tokens → USD)
- Pre-reservation for budget enforcement
- Per-model pricing with input/output token rates

**Out of Scope:**
- Token counting (handled by LLM response)
- Multi-currency support (USD only)
- Cost forecasting (see analytics features)
- Budget persistence (runtime uses in-memory tracking)

---

## Dependencies

**Required Modules:**
- iron_types - Foundation types

**Required External:**
- arc_swap - Lock-free concurrent access to pricing data
- serde/serde_json - Pricing data serialization

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **PricingManager:** Thread-safe pricing data store with lock-free reads (ArcSwap)
- **Model:** Pricing info for single LLM model with cost calculation methods
- **Embedded Pricing:** LiteLLM pricing JSON embedded at compile time as fallback

**Cost Calculation:**
- Actual cost: `input_tokens * input_rate + output_tokens * output_rate`
- Max cost (pre-reservation): `input_tokens * input_rate + max_output * output_rate`
- Default max_output_tokens: 128000 (when model has no limit info)

---

## Integration Points

**Used by:**
- iron_runtime - Cost calculation for LLM requests
- iron_control_api - Cost reporting endpoints

**Uses:**
- Embedded LiteLLM pricing data (asset/pricing.json)

---

*For detailed pricing and algorithms, see spec/-archived_detailed_spec.md*
*For budget protocol, see docs/protocol/005_budget_control_protocol.md*
