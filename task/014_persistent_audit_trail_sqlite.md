# Task 014: Persistent audit trail with pluggable backend

## Dependencies
- None

## Context
Audit events are stored in-memory (`ArrayQueue<AnalyticsEvent>` bounded lock-free circular buffer from crossbeam, with `DashMap` for per-model/provider stats and `AtomicU64` for global counters) in `iron_runtime_analytics`. Events are lost on process restart. This contradicts promise #4 ("every call logged to audit trail").

Critical areas:
- `module/iron_runtime_analytics/src/event_storage.rs`
- `module/iron_runtime_analytics/Cargo.toml`

## Implementation plan
1. Define `AuditBackend` trait (append, query, export).
2. Implement `SqliteAuditBackend` using sqlx, consistent with `iron_runtime_state` patterns.
3. Add `events` table: `(id, timestamp, agent_id, event_type, payload_json, checksum)`.
4. Add schema migration following existing `iron_token_manager/migrations/` patterns.
5. Keep in-memory cache for real-time stats, write-through to SQLite.
6. Add JSON/CSV export capability.
7. Feature-gate Postgres behind `postgres` feature for future Stage 2 work.

## Acceptance criteria
- Events persist across process restart.
- Start router, record 100 events, kill process, restart, query audit log - all 100 present.
- Export produces valid JSON/CSV.
- In-memory real-time stats still work.
