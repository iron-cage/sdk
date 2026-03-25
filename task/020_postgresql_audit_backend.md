# Task 020: PostgreSQL audit backend

## Goal
Implement a PostgreSQL-backed audit storage backend that satisfies the `AuditBackend` trait established in Task 014, enabling production multi-user deployments with durable, queryable audit logs. The backend must support configurable retention policies and compliance-ready export formats so that operators can meet SOC2 and similar audit requirements without additional tooling.

## Dependencies
- Task 014

## In Scope
- `PostgresAuditBackend` struct implementing the `AuditBackend` trait
- Postgres-specific schema migrations using embedded SQL
- Configurable retention policies (max age, max row count)
- SOC2-friendly compliance export in structured JSON format
- Feature-gated compilation behind the `postgres` feature flag

## Out of Scope
- MySQL or other relational database backends
- Real-time streaming of audit events (pub/sub)
- Dashboard UI for browsing audit logs

## Description
The SQLite audit backend from Task 014 works well for single-user and local development scenarios, but production deployments with multiple users and high write throughput require PostgreSQL. This task adds a `PostgresAuditBackend` that implements the same `AuditBackend` trait, ensuring full compatibility with the existing test suite.

Beyond basic event storage, the PostgreSQL backend adds operational features that production deployments need: automatic retention policies that prune old events based on configurable age thresholds, and compliance export functionality that produces structured, timestamped records suitable for SOC2 audits. All PostgreSQL-specific code is gated behind the `postgres` Cargo feature to keep the default dependency footprint small.

## Context
SQLite audit backend (Task 014) is sufficient for local-first use. Production multi-user deployments need PostgreSQL. The `AuditBackend` trait from Task 014 enables this as a pluggable extension.

Critical areas:
- `module/iron_runtime_analytics/src/event_storage.rs`
- `module/iron_runtime_analytics/Cargo.toml`

## Work Procedure
1. Add `tokio-postgres` and `deadpool-postgres` as optional dependencies in `module/iron_runtime_analytics/Cargo.toml` behind the `postgres` feature.
2. Create the PostgreSQL schema migration file with tables mirroring the SQLite schema but using PostgreSQL-native types (TIMESTAMPTZ, JSONB).
3. Implement `PostgresAuditBackend` struct with connection pool configuration.
4. Implement the `AuditBackend` trait for `PostgresAuditBackend`, including `store_event`, `query_events`, and `export` methods.
5. Add retention policy logic with a `prune_older_than(duration)` method that deletes rows by timestamp.
6. Implement compliance export that produces structured JSON records with ISO 8601 timestamps.
7. Run the existing `AuditBackend` test suite against the PostgreSQL implementation using testcontainers.
8. Verify feature gating compiles cleanly with and without the `postgres` feature enabled.

## Implementation plan
1. Implement `PostgresAuditBackend` behind `postgres` feature gate.
2. Add Postgres-specific migrations.
3. Add audit log retention policies (configurable max age).
4. Add compliance export formats (SOC2-friendly).

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Store single audit event | Event persisted to PostgreSQL | Row exists with correct fields |
| Query events by time range | Returns matching events only | Result count matches inserted count in range |
| Retention policy with max age 30 days | Events older than 30 days pruned | No rows with timestamp older than threshold |
| Compliance export | Structured JSON with ISO 8601 timestamps | Output validates against SOC2 schema |
| Compile without `postgres` feature | No PostgreSQL dependencies pulled | `cargo check` succeeds without libpq |
| High-volume insert (1000 events) | All events persisted without error | Row count matches insert count |

## Validation Checklist
- [ ] `PostgresAuditBackend` implements `AuditBackend` trait fully
- [ ] All existing audit backend tests pass against PostgreSQL
- [ ] Retention policy correctly deletes old events
- [ ] Compliance export output contains ISO 8601 timestamps and structured fields
- [ ] Feature gate compiles cleanly in both enabled and disabled states
- [ ] Connection pool handles concurrent writes without deadlock
- [ ] Migration runs idempotently on an already-migrated database

## Validation Procedure
1. Run `cargo test -p iron_runtime_analytics --features postgres` and confirm all tests pass.
2. Start a PostgreSQL instance (via Docker or testcontainers) and run the integration test suite.
3. Insert 100 events, set retention to 0 seconds, run prune, and verify the table is empty.
4. Run compliance export and validate the JSON output contains required fields (timestamp, event_type, actor, details).
5. Run `cargo check -p iron_runtime_analytics` without the `postgres` feature and confirm no compilation errors.
6. Run `cargo doc -p iron_runtime_analytics --features postgres --no-deps` and verify zero warnings.

## Acceptance Criteria
- PostgreSQL backend passes same test suite as SQLite backend.
- Retention policies delete events older than configured threshold.
- Compliance export produces structured, timestamped records.
