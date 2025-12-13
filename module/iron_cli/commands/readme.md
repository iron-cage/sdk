# Directory: commands

## Responsibility Table

| File | Responsibility |
|------|----------------|
| auth.yaml | Defines authentication commands (.auth.*) |
| health.yaml | Defines health check commands (.health.*) |
| limits.yaml | Defines token limit commands (.limits.*) |
| tokens.yaml | Defines token management commands (.tokens.*) |
| traces.yaml | Defines trace query commands (.traces.*) |
| usage.yaml | Defines usage tracking commands (.usage.*) |
| control/ | Control API command definitions subdirectory |

## Subdirectories

### control/

Contains command definitions for Iron Control API operations (iron binary):

| File | Responsibility |
|------|----------------|
| 000_test_minimal.yaml | Minimal test command for development |
| agents.yaml | Defines agent management commands (.agent.*) |
| analytics.yaml | Defines analytics commands (.analytics.*) |
| api_tokens.yaml | Defines API token commands (.api_token.*) |
| budget_limits.yaml | Defines budget limit commands (.budget.limits.*) |
| budget_requests.yaml | Defines budget request commands (.budget_request.*) |
| projects.yaml | Defines project management commands (.project.*) |
| providers.yaml | Defines provider management commands (.provider.*) |
| users.yaml | Defines user management commands (.user.*) |

## Organization

**Root Level (6 files):** Token management CLI (iron-token binary)
- Authentication, token CRUD, usage, limits, traces, health

**control/ Subdirectory (9 files):** Control API CLI (iron binary)
- Agents, providers, analytics, budgets, API tokens, projects, budget requests, users

## Validation

One-Second Test: Scan the Responsibility column - if any two entries sound similar, they overlap and must be reconsolidated.

**Result:** All entries have distinct, non-overlapping responsibilities.
