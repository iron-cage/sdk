# iron_cost - Specification

**Module:** iron_cost
**Layer:** 3 (Feature)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

LLM pricing data and cost calculation. Provides pricing data for multiple LLM providers, calculates costs based on token usage using integer arithmetic (microdollars) to avoid floating-point precision errors.

---

## Scope

**In Scope:**
- Multi-provider pricing data (LiteLLM pricing source)
- Cost calculation (tokens → microdollars, with USD conversion)
- Pre-reservation cost estimation for budget enforcement
- Per-model pricing with input/output token rates
- Currency conversion utilities (USD ↔ microdollars)

**Out of Scope:**
- Token counting (handled by LLM response)
- Multi-currency support (USD only)
- Cost forecasting (see analytics features)
- Budget persistence (runtime uses in-memory tracking)
- Budget tracking/enforcement (application-level concern)

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
- **Converter:** USD ↔ microdollars conversion utilities
- **Embedded Pricing:** LiteLLM pricing JSON embedded at compile time

**Microdollar Precision:**
- 1 USD = 1,000,000 microdollars
- Per-token costs stored as "microdollars per million tokens" (u64)
- All cost calculations use integer arithmetic to avoid floating-point errors
- Example: $0.00000125/token = 1,250,000 micros/million tokens

**Cost Calculation:**
- Actual cost (micros): `(input_tokens * input_micros_per_M + output_tokens * output_micros_per_M) / 1,000,000`
- Max cost (pre-reservation): Same formula using max_output_tokens limit
- Default max_output_tokens: 128,000 (when model has no limit info)

**API Surface:**
- `calculate_cost_micros(input, output) -> u64` - Primary integer API
- `calculate_cost(input, output) -> f64` - Convenience USD wrapper
- `calculate_max_cost_micros(input, max_output) -> u64` - Budget pre-reservation
- `converter::usd_to_micros(f64) -> u64` - Currency conversion
- `converter::micros_to_usd(u64) -> f64` - Currency conversion

---

## Integration Points

**Used by:**
- iron_runtime/LlmRouter - Calculates per-request costs in microdollars, tracks total_spent
- iron_control_api - Cost reporting endpoints

**Uses:**
- Embedded LiteLLM pricing data (asset/pricing.json)

---

*For detailed pricing and algorithms, see spec/-archived_detailed_spec.md*
*For budget protocol, see docs/protocol/005_budget_control_protocol.md*
