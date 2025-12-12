# usage/ - Usage Analytics API Tests

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `aggregate.rs` | Cross-token usage aggregation tests |
| `by_project.rs` | Project-specific usage aggregation tests |
| `by_provider.rs` | Provider-specific usage filtering tests |
| `mod.rs` | Module declarations |
| `path_validation.rs` | Path parameter validation tests |
| `persistence.rs` | Usage data persistence across restarts |

## Directory Purpose

Tests for usage analytics API endpoints (FR-8). Covers aggregation logic, project/provider filtering, path parameter validation, and data persistence across UsageTracker restarts.
