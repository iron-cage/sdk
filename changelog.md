# Changelog

All notable changes to the iron_runtime project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security

#### P0 Security Remediation (2025-12-13)

- **CRITICAL:** Fixed authorization bypass in budget handshake endpoint
  - **Vulnerability:** Malformed `agent_id` values in IC Tokens defaulted to `agent_id=1`, allowing unauthorized budget access
  - **Attack Vectors:** Alphabetic IDs (`agent_INVALID`), special characters (`agent_!!!@@@###`), integer overflow, negative/zero IDs
  - **Fix:** Explicit validation requiring positive integer agent_id with explicit error responses (no silent fallback)
  - **Impact:** Prevents budget theft, billing fraud, and audit trail poisoning
  - **Test Coverage:** 6 comprehensive tests in `tests/handshake_malformed_agent_id_test.rs`
  - **Location:** `module/iron_control_api/src/routes/budget/handshake.rs:135-169`

- **CRITICAL:** Added production secret validation
  - **Vulnerability:** Server accepted startup with default development secrets in production environments
  - **Attack Vectors:** JWT forgery (admin access), IC Token forgery (budget bypass), IP Token decryption (session hijacking)
  - **Fix:** Server refuses to start if any default secret detected in production mode (`JWT_SECRET`, `IC_TOKEN_SECRET`, `IP_TOKEN_KEY`)
  - **Deployment Modes:** Validation triggered for `Production` and `ProductionUnconfirmed` (K8s, AWS, Heroku, release builds)
  - **Impact:** Prevents token forgery, unauthorized access, budget fraud, session hijacking
  - **Test Coverage:** 5 test scenarios in `tests/production_secret_validation_test.rs`
  - **Location:** `module/iron_control_api/src/bin/iron_control_api_server.rs:380-452`
  - **Error Messages:** Includes clear remediation steps (`openssl rand -hex 32`)

- **HIGH:** Updated pyo3 to 0.24.2 (fixes RUSTSEC-2025-0020)
  - **Vulnerability:** Buffer overflow in `PyString::from_object` (missing nul-termination validation)
  - **Attack Vector:** Python code could read past string boundary and leak out-of-bounds memory via exceptions
  - **Fix:** pyo3 0.24.x adds `CStr::from_ptr` validation ensuring nul-termination
  - **Impact:** Prevents memory disclosure in Python-Rust FFI boundary
  - **Test Coverage:** 7 tests in `module/iron_runtime/tests/pyo3_string_safety_test.rs`
  - **Version Constraint:** `Cargo.toml:238-245` (workspace.dependencies.pyo3 >= 0.24.1)
  - **Backward Compatibility:** Fully compatible, no breaking API changes

**Remaining Vulnerabilities:**
- `rsa` crate: RUSTSEC-2023-0071 (Marvin timing sidechannel) - NO PATCH AVAILABLE
  - Monitoring for rsa 0.10.x release with mitigation
  - Consider migration to `ring` or `rustls` (immune to Marvin attack)

### Added

#### Task 1.3: Add Authorization Checks (2025-12-12)

- **Multi-tenant authorization** for agent resources
  - Users can only access their own agents, tokens, and budgets
  - Admin users have unrestricted access to all resources
  - Authorization enforced at database, API, and test levels

- **Database Migration 014**: `add_agents_owner_id.sql`
  - Added `owner_id TEXT NOT NULL` column to agents table
  - Added foreign key constraint: `agents.owner_id` → `users.id`
  - Cascade deletion: agents deleted when owner user is deleted
  - Referential integrity enforced at schema level

- **Protected Endpoints** (7 total, 100% coverage):
  - Agent Endpoints (6):
    - `GET /api/agents` - List agents filtered by owner
    - `POST /api/agents` - Create agent with owner_id
    - `GET /api/agents/:id` - Get agent (ownership verified)
    - `DELETE /api/agents/:id` - Delete agent (ownership verified)
    - `POST /api/agents/:id/tokens` - Create token (ownership verified)
    - `GET /api/agents/:id/tokens` - Get tokens (ownership verified)
  - Budget Endpoints (1):
    - `POST /api/budget/lease/create` - Create lease (agent ownership verified)

- **Security Features**:
  - Owner-based access control via JWT authentication
  - HTTP 403 Forbidden for unauthorized access attempts
  - HTTP 404 Not Found for non-existent resources
  - Database-level enforcement via foreign key constraints
  - Rollback prevention through 18 code dependencies

- **Verification Framework**:
  - Eight-layer verification applied (Layers 0-7)
  - 5 verification scripts in `tests/manual/`
  - 100% migration completion verified
  - 45 tests passing in full test suite
  - All security tests passing

- **Documentation**:
  - Architecture documentation: `docs/architecture/authorization.md`
  - Completion report: `docs/verification/task_1.3_completion.md`
  - Verification scripts: `tests/manual/verify_layer*_task_1.3.sh`

### Changed

- **Agent Routes** (`module/iron_control_api/src/routes/agents.rs`):
  - All endpoints now require `AuthenticatedUser` parameter
  - List agents filters by `owner_id` (admin sees all)
  - Get/delete/token operations verify agent ownership
  - Create agent sets `owner_id` from JWT claims

- **Budget Routes** (`module/iron_control_api/src/routes/budget.rs`):
  - Create budget lease verifies agent ownership before operation
  - Queries agent owner from database and matches against user

### Security

- **User Isolation**: Users cannot access other users' agents, tokens, or budgets
- **Bypass Prevention**: Structurally impossible to bypass authorization:
  - No public unfiltered query functions
  - All endpoints require authentication
  - Database foreign key constraints
  - 18 code dependencies prevent accidental removal
- **Rollback Prevention**: Removing authorization would break:
  - Database integrity (FK constraints)
  - Compilation (missing imports/functions)
  - Test suite (9 security tests fail)
  - Specification requirements

### Migration Guide

**For Existing Deployments:**

1. **Backup Database**: Always backup before migrations
2. **Apply Migration 014**:
   ```bash
   # Migration will be applied automatically on next startup
   # or manually apply: sqlite3 <db> < migrations/014_add_agents_owner_id.sql
   ```
3. **Set owner_id for existing agents**:
   - Existing agents will have `owner_id = ''` (default)
   - Admin must assign ownership or delete orphaned agents
   - Recommended: Run data migration script to populate owner_id

**Breaking Changes:**
- All agent API endpoints now require authentication
- Unauthenticated requests will receive 401 Unauthorized
- Users can only see/modify their own agents (admins excepted)

**API Changes:**
- No changes to request/response formats
- Authorization logic transparent to API consumers
- Existing clients continue to work if authenticated

## [Previous Releases]

(No previous changelog entries - this is the first changelog entry for the project)

---

**Note:** This changelog tracks significant changes to the iron_runtime project. For detailed implementation notes, see documentation in `docs/` directory.
