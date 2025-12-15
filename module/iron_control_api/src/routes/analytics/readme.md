# analytics/ - Analytics REST API Endpoints (Protocol 012)

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `budget.rs` | Budget status monitoring endpoint |
| `ingestion.rs` | Event ingestion from LlmRouter |
| `mod.rs` | Export analytics module public API |
| `shared.rs` | Common types and response structures |
| `spending.rs` | Spending analytics endpoints (4 handlers) |
| `usage.rs` | Usage statistics endpoints (3 handlers) |

## Directory Purpose

Analytics REST API implementation (Protocol 012). Provides event ingestion, spending analytics, budget monitoring, and usage statistics. All costs use microdollars (1 USD = 1,000,000 microdollars) for precision.
