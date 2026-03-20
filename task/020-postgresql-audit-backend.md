# Task 020: PostgreSQL audit backend

## Dependencies
- Task 014

## Context
SQLite audit backend (Task 014) is sufficient for local-first use. Production multi-user deployments need PostgreSQL. The `AuditBackend` trait from Task 014 enables this as a pluggable extension.

Critical areas:
- `module/iron_runtime_analytics/src/event_storage.rs`
- `module/iron_runtime_analytics/Cargo.toml`

## Implementation plan
1. Implement `PostgresAuditBackend` behind `postgres` feature gate.
2. Add Postgres-specific migrations.
3. Add audit log retention policies (configurable max age).
4. Add compliance export formats (SOC2-friendly).

## Acceptance criteria
- PostgreSQL backend passes same test suite as SQLite backend.
- Retention policies delete events older than configured threshold.
- Compliance export produces structured, timestamped records.
