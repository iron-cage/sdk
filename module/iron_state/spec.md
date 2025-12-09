# iron_state - Specification

**Module:** iron_state
**Layer:** 2 (Infrastructure)
**Status:** Active

---

## Responsibility

Local state management with SQLite for agent execution state and audit logs. Provides state persistence, event broadcasting for dashboard updates, query API for historical data.

---

## Scope

**In Scope:**
- Agent state persistence (SQLite)
- Event broadcasting (tokio broadcast channel)
- State queries (get, list, filter)
- Audit log storage
- Real-time state updates to dashboard

**Out of Scope:**
- PostgreSQL schema (see iron_control_store)
- REST API (see iron_api)
- Dashboard UI (see iron_dashboard)

---

## Dependencies

**Required Modules:**
- iron_types - Foundation types
- iron_telemetry - Logging

**Required External:**
- sqlx - SQLite ORM
- tokio - Async runtime, broadcast channels

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **State Manager:** Coordinates persistence and event broadcasting
- **SQLite Backend:** Local embedded database
- **Event Broadcaster:** Notifies subscribers of state changes
- **Query Engine:** Retrieves historical state data

---

## Integration Points

**Used by:**
- iron_runtime - Persists agent state
- iron_api - Queries state for REST endpoints
- iron_cost - Stores budget tracking
- iron_safety - Stores PII detections

**Uses:**
- SQLite - Local database storage

---

*For detailed schema, see spec/-archived_detailed_spec.md*
*For architecture, see docs/architecture/003_service_boundaries.md*
