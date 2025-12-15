# ADR-007: Testing Philosophy (No Mocking)

**Status:** Accepted
**Date:** 2025-12-09

---

## Context

Iron Cage project includes "No Mocking" principle in development standards:
- Avoid mocking in tests and codebase
- Use real implementations
- Tests must fail loudly and clearly

iron_testing module was planned to provide:
- MockRuntime, MockLLM, MockBudgetTracker
- pytest fixtures for mocking
- Test utilities based on mocks

This **directly contradicts** the "No Mocking" principle.

Additionally:
- iron_testing had 0 implementation (9 lines placeholder)
- 0 usage found (no imports)
- Each module already tests itself in tests/ directory

## Decision

Remove iron_testing module and adopt test-per-module approach:
- Each module tests itself in its own tests/ directory
- Use real implementations (not mocks)
- Integration tests use actual components
- No shared mocking utilities

**Testing approach:**
- Unit tests: Test module logic with real dependencies
- Integration tests: Test module interactions with real components
- Each module responsible for its own test utilities
- Test helpers live in module's own tests/ directory

## Consequences

**Positive:**
- Enforces "No Mocking" principle (fundamental rule)
- Tests validate real behavior (not mock behavior)
- Each module owns its testing strategy
- Simpler module structure (20 modules vs 21)
- No contradiction between principles and implementation
- Tests catch real bugs (mocks can hide integration issues)

**Negative:**
- Tests may be slower (using real components)
- Some tests require more setup (real databases, etc.)
- No shared test utilities across modules

**Mitigations:**
- Use test databases (SQLite in-memory for fast tests)
- Each module can have test helpers in its own tests/
- Integration tests can share fixtures via conftest.py
- Focus on fast unit tests, selective integration tests
- Real implementations are still fast enough for testing

## Implementation

**Removed:**
- module/iron_testing/ archived to module/-archived_iron_testing/
- All documentation references updated
- Module count: 21 → 20
- Package 3 (Agent Runtime): 10 modules → 9 modules

**Testing Strategy:**
- Each module: tests/ directory with module-specific tests
- Shared fixtures: conftest.py at workspace level if needed
- No mocking - use real implementations
- Test doubles: Only when external systems unavailable (LLM APIs in CI)

**Enforcement:**
- "No Mocking" rule from CLAUDE.md respected
- Proper fixes only (no workarounds)
- Loud failures (tests must fail clearly)

---

*Related: Testing Principles in CLAUDE.md*
