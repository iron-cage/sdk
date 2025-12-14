# fixtures

## Responsibility

Real test infrastructure for iron CLI integration testing (no mocking).

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Export test infrastructure modules |
| `test_data.rs` | Real database fixtures with SQL inserts |
| `test_harness.rs` | Real CLI execution via process spawn |
| `test_server.rs` | Real Axum HTTP server on random port |

## Notes

All fixtures follow the "No Mocking Policy" - they use real database operations, real HTTP servers, and real CLI process execution for end-to-end integration testing.
