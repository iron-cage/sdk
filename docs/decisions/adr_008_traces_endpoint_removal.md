# ADR-008: Traces Endpoint Removal

**Status:** Accepted
**Date:** 2025-12-14
**Context:** Iron Cage Pilot Launch - Phase 3 Documentation Cleanup

---

## Context

During Iron Cage Pilot preparation (FR-10), the `/api/traces` debugging endpoint was identified in Architecture 009 as inappropriate for production deployment. The traces endpoint provided request tracing functionality that was:

1. **Security Risk:** Exposed internal request details including authentication tokens, headers, and request bodies
2. **Performance Overhead:** Stored every request in database without retention limits
3. **Debugging-Only:** Served no production business value; was development tooling leaked into API
4. **Scope Creep:** Not part of Protocol 010 (Agents), 012 (Analytics), or 014 (API Tokens) - was legacy FR-10 feature

**Pre-Removal State:**
- Endpoints: `GET /api/traces`, `GET /api/traces/:id`
- Implementation: `src/routes/traces.rs` (TracesState, handler functions)
- Tests: `tests/traces/` directory (15 tests across 4 files)
- Test Infrastructure: TestTracesAppState in `tests/common/test_state.rs`
- Router Integration: Mixed into analytics_integration_tests.rs router

## Decision

**Remove traces endpoint completely** from iron_control_api crate before Pilot launch.

### Removal Scope
- ✅ Source code: `src/routes/traces.rs`
- ✅ Module declaration: `pub mod traces;` in `src/routes/mod.rs`
- ✅ Test files: `tests/traces/` directory (all files)
- ✅ Test infrastructure: TestTracesAppState struct and implementations
- ✅ Test integration: Traces routes removed from analytics test router
- ✅ Git history: All files removed via `git rm -f` (proper version control cleanup)

### Alternatives Considered
1. **Keep with authentication:** Rejected - security risk too high
2. **Add retention limits:** Rejected - still serves no production purpose
3. **Move to separate debug API:** Rejected - adds complexity without business value
4. **Feature flag:** Rejected - maintaining unused code violates YAGNI principle

## Consequences

### Positive
- **Security:** Eliminated request data exposure risk
- **Clarity:** API surface now precisely matches Protocol 010/012/014 scope
- **Maintenance:** Reduced codebase by ~500 lines (source + tests)
- **Performance:** Removed database write overhead on every request
- **Focus:** Clearer separation between production API and development tooling

### Negative
- **Debugging:** Lost built-in request tracing capability
  - **Mitigation:** Use external tools (OpenTelemetry, structured logging, APM)
- **Test Coverage:** Reduced from 994 to 960 tests (34 traces tests removed)
  - **Impact:** No loss of production functionality coverage (traces was debug-only)

### Technical Implementation Notes

**Module Removal Pattern Applied:**
1. `git rm -f` all source and test files (not plain `rm` - ensures git tracking cleanup)
2. Remove module declaration from parent mod.rs
3. Remove test infrastructure types (TestXxxAppState)
4. Remove route registrations from test routers
5. Remove test functions referencing deleted module
6. `cargo clean` to resolve linker errors (stale build artifacts)
7. Rebuild and verify compilation + test execution

**Build Impact:**
- Pre-removal: 994 tests passed, 16 skipped
- Post-removal: 960 tests passed, 15 skipped
- Build time: Unchanged (~1.3s)
- Binary size: Not measured (negligible impact)

**Git Cleanup:**
```bash
git rm -f src/routes/traces.rs \
  tests/traces.rs \
  tests/traces/*.rs \
  tests/traces/readme.md
rmdir tests/traces
```

### Future Considerations

**If Request Tracing Needed:**
- Use OpenTelemetry integration (standard observability)
- Use structured logging with correlation IDs
- Use APM tools (DataDog, New Relic, etc.)
- DO NOT re-implement custom traces endpoint in production API

**Lesson Learned:**
Debug tooling should never be exposed through production API endpoints. Use proper separation:
- Production API: Business functionality only (Protocol-defined endpoints)
- Development tooling: Separate binaries, feature flags, or external observability

---

## References

- **Plan:** `./-current_plan1.md` Phase 3 Deliverable 3.2
- **Architecture:** `docs/architecture/009_resource_catalog.md`
- **Protocols:** 010 (Agents), 012 (Analytics), 014 (API Tokens)
- **Test Impact:** 34 tests removed (tests/traces/list.rs: 7, tests/traces/get_by_id.rs: 8, analytics_integration_tests.rs: 2)

## Related ADRs

- ADR-007 (Testing Philosophy): No mocking principle - traces removal eliminated test infrastructure complexity
