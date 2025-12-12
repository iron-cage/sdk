# common

## Responsibility

Shared test utilities for token manager integration tests.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Test helpers for temporary databases with production schema |

## Notes

This module provides utilities for creating test databases that match production schema exactly via unified migrations. Currently migrating from legacy `( pool, TempDir )` pattern to new `iron_test_db::TestDatabase` crate. New functions use `_v2` suffix. Prefer `*_v2()` functions for new tests; old functions maintained for backward compatibility.
