# Task 014: Persistent audit trail with pluggable backend

## Goal
Implement a persistent audit trail backed by SQLite so that analytics events survive process restarts, fulfilling promise #4 ("every call logged to audit trail"). This is observable by recording events, killing the process, restarting, and querying the audit log to find all events intact. Scoped to the SQLite backend with a trait abstraction for future backends.

## Dependencies
- None

## In Scope
- Defining an `AuditBackend` trait with append, query, and export operations
- Implementing `SqliteAuditBackend` using sqlx
- Creating the `events` table schema with migration
- Write-through caching (in-memory for real-time stats, SQLite for persistence)
- JSON and CSV export of audit events
- Feature-gating Postgres behind a `postgres` feature flag

## Out of Scope
- Implementing the Postgres backend (deferred to Stage 2)
- Modifying the existing in-memory analytics counters beyond adding write-through
- Building a UI or dashboard for audit trail inspection
- Event compression or archival strategies

## Description
The `iron_runtime_analytics` module currently stores audit events in an in-memory `ArrayQueue` circular buffer, meaning all events are lost on process restart. This directly contradicts promise #4 that every call is logged to an audit trail, since the trail evaporates on shutdown.

This task introduces a pluggable `AuditBackend` trait and its first implementation, `SqliteAuditBackend`, following the patterns already established in `iron_runtime_state` and `iron_token_manager`. The SQLite backend stores events in an `events` table with fields for id, timestamp, agent_id, event_type, JSON payload, and checksum. The existing in-memory cache is retained for real-time stats but events are written through to SQLite for durability. A JSON/CSV export capability is added for compliance reporting.

## Context
Audit events are stored in-memory (`ArrayQueue<AnalyticsEvent>` bounded lock-free circular buffer from crossbeam, with `DashMap` for per-model/provider stats and `AtomicU64` for global counters) in `iron_runtime_analytics`. Events are lost on process restart. This contradicts promise #4 ("every call logged to audit trail").

Critical areas:
- `module/iron_runtime_analytics/src/event_storage.rs`
- `module/iron_runtime_analytics/Cargo.toml`

## Work Procedure
1. Study the existing `event_storage.rs` to understand the current in-memory storage structure.
2. Review `iron_runtime_state` and `iron_token_manager/migrations/` for SQLite and migration patterns.
3. Define the `AuditBackend` trait with `append`, `query_by_time_range`, `query_by_agent`, and `export` methods.
4. Create the SQLite migration file for the `events` table schema.
5. Implement `SqliteAuditBackend` using sqlx with connection pooling.
6. Modify `event_storage.rs` to write-through: append to both in-memory cache and the backend.
7. Implement JSON and CSV export methods on the backend.
8. Add the `postgres` feature flag in `Cargo.toml` (implementation deferred).
9. Write integration tests: record events, drop the backend, reconnect, verify events persist.
10. Run `cargo test --workspace` and confirm all tests pass.

## Implementation plan
1. Define `AuditBackend` trait (append, query, export).
2. Implement `SqliteAuditBackend` using sqlx, consistent with `iron_runtime_state` patterns.
3. Add `events` table: `(id, timestamp, agent_id, event_type, payload_json, checksum)`.
4. Add schema migration following existing `iron_token_manager/migrations/` patterns.
5. Keep in-memory cache for real-time stats, write-through to SQLite.
6. Add JSON/CSV export capability.
7. Feature-gate Postgres behind `postgres` feature for future Stage 2 work.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Append single event to SQLite backend | Event stored with correct fields | Query returns event with matching id, timestamp, payload |
| Append 100 events then reconnect | All events persist across reconnect | Query returns exactly 100 events |
| Query by time range | Only events within range returned | Result count matches events in time window |
| Query by agent_id | Only events for that agent returned | All results have matching agent_id |
| Export to JSON | Valid JSON array of events | Output parses as valid JSON, field count matches |
| Export to CSV | Valid CSV with header row | CSV has correct headers and row count |
| In-memory stats after write-through | Real-time counters still accurate | DashMap and AtomicU64 values match event count |
| Checksum validation | Stored checksum matches recomputed checksum | Integrity check passes for all events |

## Validation List
- [ ] `AuditBackend` trait is defined with append, query, and export methods
- [ ] `SqliteAuditBackend` implements the trait using sqlx
- [ ] Migration file creates `events` table with correct schema
- [ ] Events persist across backend reconnection
- [ ] JSON export produces valid, parseable JSON
- [ ] CSV export produces valid CSV with headers
- [ ] In-memory real-time stats remain functional
- [ ] `postgres` feature flag exists in Cargo.toml
- [ ] All tests pass with `cargo test --workspace`

## Validation Procedure
1. Run the integration test that records 100 events, drops the connection, reconnects, and queries all 100.
2. Verify the `events` table schema by inspecting the migration file for required columns.
3. Export events to JSON and validate the output with a JSON parser.
4. Export events to CSV and verify the header row and data row count.
5. Confirm in-memory stats (DashMap, AtomicU64) reflect the correct event counts after write-through.
6. Check `Cargo.toml` for the `postgres` feature flag definition.
7. Run `cargo test --workspace` and confirm no regressions.

## Acceptance criteria
- Events persist across process restart.
- Start router, record 100 events, kill process, restart, query audit log - all 100 present.
- Export produces valid JSON/CSV.
- In-memory real-time stats still work.
