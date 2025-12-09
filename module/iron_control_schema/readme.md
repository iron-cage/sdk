# iron_control_schema

PostgreSQL schema for Iron Cage Control Panel (production mode).

## Scope

**Responsibilities:**
Defines PostgreSQL database schema for production Control Panel deployment. Manages user accounts, tokens, secrets, and telemetry aggregation in centralized cloud database.

**In Scope:**
- PostgreSQL schema definitions (tables, indexes, constraints)
- Migration scripts (up/down migrations)
- Schema documentation and rationale
- Database types and enums
- Query patterns and indexes

**Out of Scope:**
- SQLite schema (see iron_runtime_state - local agent state)
- Implementation code (spec-only module until production phase)
- API endpoints (see iron_control_api)
- Business logic (see iron_token_manager, iron_secrets)

---

## Status

**Module Type:** Spec-only (no implementation yet)

**Phase:** Planning - Production deployment preparation

**Implementation:** Planned for Q1 2026

---

## Schema Overview

The Control Panel database stores:

1. **Users** - Account information, authentication
2. **Tokens** - JWT tokens, rotation schedule
3. **Secrets** - Encrypted credentials (LLM API keys, etc.)
4. **Telemetry** - Aggregated metrics from distributed agents

See `spec.md` for complete schema definitions.

---

## License

Apache-2.0 - See `license` file for details
