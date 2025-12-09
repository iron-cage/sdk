# LLM Token Management - Goal Achievement Validation Framework

**Version:** 1.0.0
**Date:** 2025-12-02
**Status:** Validation framework
**Related Documents:**
- Architecture: [token_management.md](token_management.md)
- Implementation Plan: [token_management_implementation_plan.md](token_management_implementation_plan.md)

---

## Overview

This document defines **how we prove** each goal from the implementation plan is achieved. Every objective has measurable validation criteria with automated verification.

**Validation Principle:** If it cannot be measured, it cannot be validated. If it cannot be validated, it's not done.

---

## Table of Contents

1. [Validation Strategy](#1-validation-strategy)
2. [Requirements Traceability Matrix](#2-requirements-traceability-matrix)
3. [Phase-by-Phase Validation](#3-phase-by-phase-validation)
4. [Automated Validation Pipeline](#4-automated-validation-pipeline)
5. [Performance Validation](#5-performance-validation)
6. [Security Validation](#6-security-validation)
7. [Quality Validation](#7-quality-validation)
8. [Acceptance Validation](#8-acceptance-validation)
9. [Continuous Validation Dashboard](#9-continuous-validation-dashboard)
10. [Validation Failure Protocols](#10-validation-failure-protocols)

---

## 1. Validation Strategy

### 1.1 Four-Layer Validation Model

```
Layer 4: Business Acceptance Validation
  ↑ Does it solve the business problem?
  │ Validation: User acceptance testing, stakeholder demos
  │
Layer 3: System Integration Validation
  ↑ Does everything work together?
  │ Validation: E2E tests, integration tests, smoke tests
  │
Layer 2: Component Validation
  ↑ Does each component work correctly?
  │ Validation: Unit tests, API tests, component tests
  │
Layer 1: Code Quality Validation
  ↑ Is the code well-written?
  │ Validation: Linting, type checking, coverage analysis
```

### 1.2 Validation Gates

**Gate Types:**
1. **Blocking Gate** - Must pass before proceeding (e.g., all tests passing)
2. **Warning Gate** - Should pass, but can proceed with justification (e.g., 75% coverage instead of 80%)
3. **Advisory Gate** - Nice to have, doesn't block (e.g., documentation completeness)

### 1.3 Validation Automation Levels

| Level | Automation | When | Example |
|-------|------------|------|---------|
| **L0** | Manual | On-demand | Stakeholder demo |
| **L1** | Semi-Auto | Pre-commit | `cargo test` before commit |
| **L2** | Auto PR | On pull request | CI runs full test suite |
| **L3** | Auto CD | On merge to main | Deploy to staging + smoke tests |
| **L4** | Continuous | Every 1 hour | Production health checks |

---

## 2. Requirements Traceability Matrix

### 2.1 Traceability Structure

Every requirement must be traceable to:
1. **Test(s)** - Automated tests that verify the requirement
2. **Implementation** - Code that implements the requirement
3. **Validation** - How we prove it works
4. **Acceptance** - Who approves it's done

### 2.2 Example Traceability Entry

```yaml
requirement:
  id: REQ-001
  description: "Generate API tokens with SHA-256 hashing"
  source: task/backlog/001_implement_llm_token_management_dashboard_and_backend.md

tests:
  - module/iron_token_manager/tests/token_generator_tests.rs::test_generate_token_uniqueness
  - module/iron_token_manager/tests/token_generator_tests.rs::test_token_hash_irreversible
  - module/iron_token_manager/tests/token_generator_tests.rs::test_token_statistical_uniqueness

implementation:
  - module/iron_token_manager/src/token_generator.rs::TokenGenerator::generate_token

validation:
  method: Automated tests + statistical analysis
  criteria:
    - All unit tests passing
    - 1M token generation with 0 collisions
    - SHA-256 hash verification (32 bytes, non-reversible)
  tools:
    - cargo nextest
    - custom statistical test

acceptance:
  approver: Technical Lead
  evidence: Test report + code review
  status: pending
```

### 2.3 Full Traceability Matrix

**Backend Requirements:**

| ID | Requirement | Test Files | Validation Method | Status |
|----|-------------|------------|-------------------|--------|
| REQ-001 | Token generation (SHA-256) | `token_generator_tests.rs` | Unit tests + statistical | ⏳ |
| REQ-002 | Token rotation | `token_generator_tests.rs` | Integration tests | ⏳ |
| REQ-003 | Token revocation | `token_generator_tests.rs` | Integration tests | ⏳ |
| REQ-004 | Usage tracking | `usage_tracker_tests.rs` | Unit + integration tests | ⏳ |
| REQ-005 | Cost calculation | `cost_calculator_tests.rs` | Unit tests + manual verification | ⏳ |
| REQ-006 | Limit enforcement | `limit_enforcer_tests.rs` | Unit + load tests | ⏳ |
| REQ-007 | Grace period support | `limit_enforcer_tests.rs` | Unit tests | ⏳ |
| REQ-008 | Rate limiting | `rate_limiter_tests.rs` | Unit + load tests | ⏳ |
| REQ-009 | JWT authentication | `api_integration_tests.rs` | Integration tests | ⏳ |
| REQ-010 | RBAC authorization | `api_integration_tests.rs` | Integration tests | ⏳ |
| REQ-011 | API endpoints (15+) | `api_integration_tests.rs` | Integration + E2E tests | ⏳ |
| REQ-012 | Database schema | `migrations/` | Migration tests | ⏳ |

**Frontend Requirements:**

| ID | Requirement | Test Files | Validation Method | Status |
|----|-------------|------------|-------------------|--------|
| REQ-101 | Token management view | `e2e/tokens.spec.ts` | E2E tests + manual | ⏳ |
| REQ-102 | Usage analytics view | `e2e/usage.spec.ts` | E2E tests + manual | ⏳ |
| REQ-103 | Limits management view | `e2e/limits.spec.ts` | E2E tests + manual | ⏳ |
| REQ-104 | Call tracing view | `e2e/traces.spec.ts` | E2E tests + manual | ⏳ |
| REQ-105 | Charts/visualizations | `components/*.spec.ts` | Component tests + visual | ⏳ |
| REQ-106 | Dashboard load time < 2s | `lighthouse-ci.yml` | Lighthouse CI | ⏳ |
| REQ-107 | Mobile responsive | `e2e/*.spec.ts` | E2E tests (mobile viewport) | ⏳ |

**Non-Functional Requirements:**

| ID | Requirement | Test Files | Validation Method | Status |
|----|-------------|------------|-------------------|--------|
| NFR-001 | API latency (p95) < 100ms | `load_tests/api_latency.js` | k6 load testing | ⏳ |
| NFR-002 | Token gen: 10K+ tokens/sec | `benches/token_gen.rs` | Criterion benchmarks | ⏳ |
| NFR-003 | Concurrent: 10K+ req/min | `load_tests/throughput.js` | k6 load testing | ⏳ |
| NFR-004 | Test coverage > 80% | `tarpaulin.toml` | cargo-tarpaulin | ⏳ |
| NFR-005 | Security: 0 critical vulns | `security-scan.yml` | OWASP ZAP + cargo-audit | ⏳ |

---

## 3. Phase-by-Phase Validation

### Phase 1: Database Schema + Token Generation

**Validation Checklist:**

```yaml
phase: 1
duration: Weeks 1-2
exit_criteria:
  blocking_gates:
    - name: "All unit tests passing"
      validation: "cargo nextest run --all-features"
      threshold: "100% passing"
      current: "0/20 passing"
      status: "⏳ pending"

    - name: "Token generation performance"
      validation: "cargo bench --bench token_gen"
      threshold: "> 10,000 tokens/sec"
      current: "N/A"
      status: "⏳ pending"

    - name: "Statistical uniqueness"
      validation: "cargo test --test statistical_uniqueness"
      threshold: "0 collisions in 1M samples"
      current: "N/A"
      status: "⏳ pending"

    - name: "Database migrations work"
      validation: "sqlx migrate run && sqlx migrate revert && sqlx migrate run"
      threshold: "Both PostgreSQL and SQLite succeed"
      current: "N/A"
      status: "⏳ pending"

  warning_gates:
    - name: "Code coverage"
      validation: "cargo tarpaulin --out Html"
      threshold: "> 80%"
      acceptable_min: "75%"
      current: "N/A"
      status: "⏳ pending"

  advisory_gates:
    - name: "Documentation completeness"
      validation: "Manual review of doc comments"
      threshold: "All public APIs documented"
      current: "N/A"
      status: "⏳ pending"

validation_commands:
  automated: |
    #!/bin/bash
    set -e

    echo "=== Phase 1 Validation ==="

    # Blocking Gate 1: Unit tests
    echo "✓ Running unit tests..."
    cargo nextest run --all-features

    # Blocking Gate 2: Performance benchmark
    echo "✓ Running token generation benchmark..."
    cargo bench --bench token_gen

    # Blocking Gate 3: Statistical uniqueness
    echo "✓ Running statistical uniqueness test..."
    cargo test --test statistical_uniqueness -- --ignored

    # Blocking Gate 4: Database migrations
    echo "✓ Testing migrations..."
    sqlx migrate run
    sqlx migrate revert
    sqlx migrate run

    # Warning Gate 1: Coverage
    echo "⚠ Running coverage analysis..."
    cargo tarpaulin --out Html --output-dir target/coverage

    echo "=== Phase 1 Validation Complete ==="

  manual: |
    1. Review code for public API documentation
    2. Verify token collision probability < 2^-128
    3. Inspect database schema matches spec
    4. Code review with technical lead

evidence_artifacts:
  - target/nextest/test-report.xml
  - target/criterion/token_gen/report/index.html
  - target/coverage/index.html
  - migrations/*.sql
  - code_review_approval.md
```

**Automated Validation Script:**

```bash
#!/bin/bash
# File: scripts/validate_phase1.sh

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║          Phase 1 Validation - Automated               ║"
echo "╚════════════════════════════════════════════════════════╝"

# Initialize counters
PASSED=0
FAILED=0
WARNINGS=0

# Function to run validation and track results
validate() {
  local name="$1"
  local cmd="$2"
  local threshold="$3"

  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "Validating: $name"
  echo "Threshold: $threshold"
  echo "Command: $cmd"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  if eval "$cmd"; then
    echo "✅ PASSED: $name"
    ((PASSED++))
    return 0
  else
    echo "❌ FAILED: $name"
    ((FAILED++))
    return 1
  fi
}

# Blocking Gate 1: Unit Tests
validate \
  "Unit Tests (100% passing)" \
  "RUSTFLAGS='-D warnings' cargo nextest run --all-features" \
  "All tests must pass"

# Blocking Gate 2: Token Generation Performance
validate \
  "Token Generation Performance (>10K tokens/sec)" \
  "cargo bench --bench token_gen | tee /tmp/bench.txt && grep -q '10,000' /tmp/bench.txt" \
  "Must generate >10,000 tokens/sec"

# Blocking Gate 3: Statistical Uniqueness
validate \
  "Statistical Uniqueness (0 collisions in 1M)" \
  "cargo test --test statistical_uniqueness -- --ignored --nocapture" \
  "Zero collisions required"

# Blocking Gate 4: Database Migrations
validate \
  "Database Migrations (PostgreSQL + SQLite)" \
  "sqlx migrate run && sqlx migrate revert && sqlx migrate run" \
  "Migrations must be reversible"

# Warning Gate 1: Code Coverage
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚠️  Warning Gate: Code Coverage"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cargo tarpaulin --out Html --output-dir target/coverage
COVERAGE=$(cargo tarpaulin --out Json | jq '.coverage')

if (( $(echo "$COVERAGE >= 80" | bc -l) )); then
  echo "✅ Coverage: $COVERAGE% (>80% threshold)"
else
  echo "⚠️  Coverage: $COVERAGE% (<80% threshold, but acceptable if >75%)"
  ((WARNINGS++))
fi

# Summary
echo ""
echo "╔════════════════════════════════════════════════════════╗"
echo "║                Validation Summary                      ║"
echo "╠════════════════════════════════════════════════════════╣"
echo "║  Passed:   $PASSED                                        ║"
echo "║  Failed:   $FAILED                                        ║"
echo "║  Warnings: $WARNINGS                                      ║"
echo "╚════════════════════════════════════════════════════════╝"

if [ $FAILED -gt 0 ]; then
  echo ""
  echo "❌ Phase 1 validation FAILED. Fix issues before proceeding."
  exit 1
else
  echo ""
  echo "✅ Phase 1 validation PASSED. Proceed to Phase 2."
  exit 0
fi
```

### Phase 2: Usage Tracking

**Validation Checklist:**

```yaml
phase: 2
duration: Weeks 3-4
exit_criteria:
  blocking_gates:
    - name: "Cost calculation accuracy"
      validation: "Compare calculated costs with actual provider pricing"
      threshold: "±1% accuracy for all providers"
      verification: |
        # OpenAI GPT-4: Input $30/1M, Output $60/1M
        assert_eq!(calculate_cost(Provider::OpenAI, 1000, 500), 0.06, tolerance: 0.01)
        # Anthropic Claude 3: Input $15/1M, Output $75/1M
        assert_eq!(calculate_cost(Provider::Anthropic, 1000, 500), 0.0525, tolerance: 0.01)
      status: "⏳ pending"

    - name: "Usage tracking latency"
      validation: "Measure p95 latency for record_usage()"
      threshold: "< 10ms"
      measurement: "cargo bench --bench usage_tracking"
      status: "⏳ pending"

    - name: "Provider adapter integration"
      validation: "Integration tests with mock servers"
      threshold: "All 3 providers (OpenAI, Anthropic, Google) working"
      status: "⏳ pending"

    - name: "Concurrent usage recording"
      validation: "Load test with 100 concurrent requests"
      threshold: "No data races, no lost records"
      measurement: "cargo test --test concurrent_usage -- --test-threads=100"
      status: "⏳ pending"

validation_commands:
  cost_accuracy_test: |
    #!/bin/bash
    # Validate cost calculation accuracy

    echo "Testing OpenAI pricing..."
    cargo test test_openai_cost_accuracy

    echo "Testing Anthropic pricing..."
    cargo test test_anthropic_cost_accuracy

    echo "Testing Google pricing..."
    cargo test test_google_cost_accuracy

  usage_latency_test: |
    #!/bin/bash
    # Benchmark usage tracking latency

    cargo bench --bench usage_tracking

    # Parse results and verify p95 < 10ms
    P95=$(cat target/criterion/usage_tracking/base/estimates.json | jq '.mean.point_estimate')
    if (( $(echo "$P95 < 10000000" | bc -l) )); then
      echo "✅ P95 latency: ${P95}ns (< 10ms)"
    else
      echo "❌ P95 latency: ${P95}ns (>= 10ms)"
      exit 1
    fi
```

### Phase 3: Limits & Rate Limiting

**Validation Checklist:**

```yaml
phase: 3
duration: Week 5
exit_criteria:
  blocking_gates:
    - name: "Budget enforcement accuracy"
      validation: "Verify no budget overruns in any scenario"
      threshold: "100% accurate (0 overruns)"
      test_scenarios:
        - "Sequential requests at budget limit"
        - "Concurrent requests at budget limit"
        - "Grace period boundary testing"
        - "Race condition testing (1000 concurrent requests)"
      status: "⏳ pending"

    - name: "Rate limiting accuracy"
      validation: "Measure actual vs configured rate limits"
      threshold: "±5% of configured limit"
      test_cases:
        - "10 req/min: actual should be 9.5-10.5 req/min"
        - "100 req/min: actual should be 95-105 req/min"
        - "1000 req/min: actual should be 950-1050 req/min"
      status: "⏳ pending"

race_condition_test: |
  #[tokio::test]
  async fn test_no_budget_overruns_under_concurrency()
  {
    let tracker = BudgetTracker::new(100.0); // $100 budget

    // Spawn 1000 concurrent tasks, each trying to spend $1
    let tasks: Vec<_> = (0..1000)
      .map(|i| {
        let tracker = tracker.clone();
        tokio::spawn(async move {
          tracker.record_cost(&format!("agent_{}", i), 1.0).await
        })
      })
      .collect();

    // Wait for all tasks
    let results = futures::future::join_all(tasks).await;

    // Count successes and failures
    let successes = results.iter().filter(|r| r.is_ok()).count();
    let failures = results.iter().filter(|r| r.is_err()).count();

    // Verify exactly 100 succeeded (budget exhausted exactly)
    assert_eq!(successes, 100, "Budget must allow exactly 100 operations");
    assert_eq!(failures, 900, "Budget must deny remaining 900 operations");

    // Verify total spent equals budget (no overruns)
    assert_eq!(tracker.total_spent(), 100.0, "Total spent must equal budget");
  }
```

### Phase 4: API Endpoints + Authentication

**Validation Checklist:**

```yaml
phase: 4
duration: Week 6
exit_criteria:
  blocking_gates:
    - name: "All API endpoints functional"
      validation: "Integration tests + Postman collection"
      threshold: "15+ endpoints, all returning correct status codes"
      endpoints:
        - "POST /api/v1/auth/login (200)"
        - "POST /api/v1/auth/refresh (200)"
        - "POST /api/v1/tokens (201)"
        - "GET /api/v1/tokens (200)"
        - "GET /api/v1/tokens/:id (200)"
        - "PUT /api/v1/tokens/:id/rotate (200)"
        - "DELETE /api/v1/tokens/:id (204)"
        - "GET /api/v1/usage (200)"
        - "GET /api/v1/usage/:token_id (200)"
        - "GET /api/v1/limits (200)"
        - "POST /api/v1/limits (201)"
        - "PUT /api/v1/limits/:id (200)"
        - "GET /api/v1/traces (200)"
        - "GET /api/v1/health (200)"
      status: "⏳ pending"

    - name: "JWT authentication working"
      validation: "Integration tests for auth flows"
      test_cases:
        - "Valid JWT allows access (200)"
        - "Invalid JWT denies access (401)"
        - "Expired JWT denies access (401)"
        - "Missing JWT denies access (401)"
        - "Refresh token works (200)"
        - "Logout invalidates token (401 after logout)"
      status: "⏳ pending"

    - name: "RBAC enforcement"
      validation: "Integration tests for all roles"
      authorization_matrix:
        - "Admin can manage all tokens (200)"
        - "User can manage own tokens (200)"
        - "User cannot manage others' tokens (403)"
        - "Agent cannot access dashboard endpoints (403)"
        - "Agent can use inference endpoint (200)"
      status: "⏳ pending"

    - name: "OpenAPI spec generated"
      validation: "utoipa generates valid OpenAPI 3.0 spec"
      threshold: "OpenAPI validator passes"
      command: "openapi-generator validate -i openapi.json"
      status: "⏳ pending"

postman_collection_test: |
  #!/bin/bash
  # Run Postman collection with Newman

  newman run postman/token_management_api.json \
    --environment postman/staging.json \
    --reporters cli,json \
    --reporter-json-export newman-report.json

  # Verify all tests passed
  PASSED=$(jq '.run.stats.tests.passed' newman-report.json)
  FAILED=$(jq '.run.stats.tests.failed' newman-report.json)

  if [ "$FAILED" -eq 0 ]; then
    echo "✅ All Postman tests passed ($PASSED/$PASSED)"
    exit 0
  else
    echo "❌ Postman tests failed ($FAILED failures)"
    exit 1
  fi
```

### Phase 5: Dashboard UI

**Validation Checklist:**

```yaml
phase: 5
duration: Weeks 7-9
exit_criteria:
  blocking_gates:
    - name: "All 4 views functional"
      validation: "E2E tests with Playwright"
      views:
        - "Tokens view: CRUD operations work"
        - "Usage view: Charts display correctly"
        - "Limits view: Forms submit successfully"
        - "Traces view: Filtering works"
      status: "⏳ pending"

    - name: "Dashboard load time < 2s"
      validation: "Lighthouse CI performance audit"
      threshold: "Performance score > 90, FCP < 1.5s, LCP < 2s"
      measurement: "lhci autorun --collect.numberOfRuns=5"
      status: "⏳ pending"

    - name: "No console errors"
      validation: "E2E tests check console logs"
      threshold: "Zero errors, < 5 warnings"
      status: "⏳ pending"

    - name: "Mobile responsive"
      validation: "E2E tests on mobile viewport"
      viewports:
        - "iPhone SE (375x667)"
        - "iPad (768x1024)"
        - "Desktop (1920x1080)"
      status: "⏳ pending"

  warning_gates:
    - name: "Lighthouse accessibility score"
      validation: "Lighthouse CI accessibility audit"
      threshold: "> 95"
      acceptable_min: "90"
      status: "⏳ pending"

lighthouse_ci_config: |
  # lighthouserc.json
  {
    "ci": {
      "collect": {
        "numberOfRuns": 5,
        "startServerCommand": "npm run preview",
        "url": ["http://localhost:4173"]
      },
      "assert": {
        "assertions": {
          "categories:performance": ["error", {"minScore": 0.9}],
          "categories:accessibility": ["warn", {"minScore": 0.95}],
          "first-contentful-paint": ["error", {"maxNumericValue": 1500}],
          "largest-contentful-paint": ["error", {"maxNumericValue": 2000}],
          "cumulative-layout-shift": ["error", {"maxNumericValue": 0.1}]
        }
      }
    }
  }

e2e_test_example: |
  // tests/e2e/tokens.spec.ts
  import { test, expect } from '@playwright/test';

  test.describe('Token Management View', () => {
    test.beforeEach(async ({ page }) => {
      // Login
      await page.goto('/login');
      await page.fill('#username', 'testuser');
      await page.fill('#password', 'testpass');
      await page.click('button[type=submit]');
      await page.waitForURL('/tokens');
    });

    test('should load within 2 seconds', async ({ page }) => {
      const startTime = Date.now();
      await page.goto('/tokens');
      await page.waitForSelector('.tokens-table');
      const loadTime = Date.now() - startTime;

      expect(loadTime).toBeLessThan(2000);
    });

    test('should generate new token', async ({ page }) => {
      await page.click('button.generate-token');
      await page.selectOption('#provider', 'openai');
      await page.click('button.confirm');

      // Wait for token to appear in table
      await page.waitForSelector('.token-row');

      // Verify token is displayed
      const tokenRows = await page.locator('.token-row').count();
      expect(tokenRows).toBeGreaterThan(0);
    });

    test('should have no console errors', async ({ page }) => {
      const errors = [];
      page.on('console', msg => {
        if (msg.type() === 'error') {
          errors.push(msg.text());
        }
      });

      await page.goto('/tokens');
      await page.waitForTimeout(2000);

      expect(errors).toHaveLength(0);
    });
  });
```

### Phase 6: Security Hardening + Documentation

**Validation Checklist:**

```yaml
phase: 6
duration: Week 10
exit_criteria:
  blocking_gates:
    - name: "Security scan: 0 critical/high vulnerabilities"
      validation: "OWASP ZAP + cargo-audit"
      tools:
        - "cargo audit --deny warnings"
        - "zap-baseline.py -t http://localhost:8080 -r zap-report.html"
      threshold: "Zero critical/high severity findings"
      status: "⏳ pending"

    - name: "Load testing: 10K+ req/min"
      validation: "k6 load testing"
      scenarios:
        - "Token validation: >10K req/min, p95 < 100ms"
        - "Usage recording: >5K req/min, p95 < 100ms"
        - "Dashboard API: >2K req/min, p95 < 200ms"
      status: "⏳ pending"

    - name: "All documentation complete"
      validation: "Documentation checklist"
      required_docs:
        - "API documentation (OpenAPI spec)"
        - "Deployment guide (Docker + K8s)"
        - "Developer guide (setup, testing)"
        - "User guide (dashboard usage)"
        - "Security documentation (auth, RBAC)"
      status: "⏳ pending"

security_scan_automation: |
  #!/bin/bash
  # scripts/security_scan.sh

  set -e

  echo "=== Security Validation ==="

  # Cargo audit
  echo "Running cargo-audit..."
  cargo audit --deny warnings

  # Start API server in background
  echo "Starting API server..."
  cargo run --release &
  API_PID=$!
  sleep 5

  # OWASP ZAP baseline scan
  echo "Running OWASP ZAP scan..."
  docker run -t owasp/zap2docker-stable zap-baseline.py \
    -t http://host.docker.internal:8080 \
    -r zap-report.html

  # Kill API server
  kill $API_PID

  # Parse ZAP report
  CRITICAL=$(grep -c "FAIL-NEW: 3" zap-report.html || true)
  HIGH=$(grep -c "FAIL-NEW: 2" zap-report.html || true)

  if [ "$CRITICAL" -gt 0 ] || [ "$HIGH" -gt 0 ]; then
    echo "❌ Security scan failed: $CRITICAL critical, $HIGH high"
    exit 1
  else
    echo "✅ Security scan passed: 0 critical/high vulnerabilities"
  fi

load_test_k6: |
  // load_tests/api_latency.js
  import http from 'k6/http';
  import { check, sleep } from 'k6';

  export let options = {
    stages: [
      { duration: '1m', target: 100 },  // Ramp up
      { duration: '3m', target: 100 },  // Steady state
      { duration: '1m', target: 0 },    // Ramp down
    ],
    thresholds: {
      'http_req_duration': ['p(95)<100'],  // p95 < 100ms
      'http_req_failed': ['rate<0.01'],     // Error rate < 1%
    },
  };

  export default function() {
    let response = http.get('http://localhost:8080/api/v1/health', {
      headers: { 'Authorization': 'Bearer test_token' },
    });

    check(response, {
      'status is 200': (r) => r.status === 200,
      'response time < 100ms': (r) => r.timings.duration < 100,
    });

    sleep(0.1);
  }
```

---

## 4. Automated Validation Pipeline

### 4.1 CI/CD Pipeline Architecture

```yaml
name: Token Management Validation Pipeline

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  validation_layer_1_code_quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run rustfmt check
        run: cargo fmt -- --check

      - name: Type checking
        run: cargo check --all-features

  validation_layer_2_component:
    needs: validation_layer_1_code_quality
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1

      - name: Run unit tests
        run: cargo nextest run --all-features

      - name: Run benchmarks
        run: cargo bench --no-run

      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: coverage/cobertura.xml

  validation_layer_3_integration:
    needs: validation_layer_2_component
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4

      - name: Run database migrations
        run: sqlx migrate run
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/test

      - name: Run integration tests
        run: cargo test --test '*_integration_tests'
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/test

  validation_layer_4_e2e:
    needs: validation_layer_3_integration
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install dependencies
        run: npm ci
        working-directory: dashboard

      - name: Build dashboard
        run: npm run build
        working-directory: dashboard

      - name: Install Playwright
        run: npx playwright install --with-deps
        working-directory: dashboard

      - name: Run E2E tests
        run: npm run test:e2e
        working-directory: dashboard

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: dashboard/playwright-report

  security_validation:
    needs: validation_layer_2_component
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run cargo-audit
        run: |
          cargo install cargo-audit
          cargo audit --deny warnings

      - name: Run OWASP ZAP baseline scan
        uses: zaproxy/action-baseline@v0.7.0
        with:
          target: 'http://localhost:8080'

  performance_validation:
    needs: validation_layer_3_integration
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install k6
        run: |
          sudo apt-get update
          sudo apt-get install -y k6

      - name: Run load tests
        run: k6 run load_tests/api_latency.js

      - name: Upload k6 results
        uses: actions/upload-artifact@v3
        with:
          name: k6-results
          path: k6-report.html
```

### 4.2 Pre-Commit Validation Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "🔍 Running pre-commit validation..."

# Layer 1: Code Quality (fast)
echo "  ✓ Checking formatting..."
cargo fmt -- --check

echo "  ✓ Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "  ✓ Type checking..."
cargo check --all-features

# Layer 2: Component Tests (medium)
echo "  ✓ Running unit tests..."
cargo nextest run --all-features

echo "✅ Pre-commit validation passed!"
```

---

## 5. Performance Validation

### 5.1 Performance Benchmarking Suite

```rust
// benches/token_gen.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use iron_token_manager::TokenGenerator;

fn benchmark_token_generation(c: &mut Criterion) {
  let generator = TokenGenerator::new();

  // Benchmark single token generation
  c.bench_function("generate_single_token", |b| {
    b.iter(|| {
      generator.generate_token(
        black_box(1),
        black_box(1),
        black_box(Provider::OpenAI),
      )
    })
  });

  // Benchmark throughput (tokens/sec)
  c.bench_function("generate_10k_tokens", |b| {
    b.iter(|| {
      for _ in 0..10_000 {
        generator.generate_token(
          black_box(1),
          black_box(1),
          black_box(Provider::OpenAI),
        );
      }
    })
  });
}

criterion_group!(benches, benchmark_token_generation);
criterion_main!(benches);
```

**Performance Validation Criteria:**

| Benchmark | Threshold | Measurement | Validation |
|-----------|-----------|-------------|------------|
| Token generation (single) | < 100µs | Criterion | Automated in CI |
| Token generation (throughput) | > 10K tokens/sec | Criterion | Automated in CI |
| Usage recording | < 10ms (p95) | Criterion | Automated in CI |
| API latency (health check) | < 50ms (p95) | k6 | Automated in CI |
| API latency (token create) | < 100ms (p95) | k6 | Automated in CI |
| Dashboard load time | < 2s | Lighthouse CI | Automated in CI |
| Database query time | < 10ms | sqlx logging | Manual review |

### 5.2 Load Testing Scenarios

```javascript
// load_tests/comprehensive.js
import http from 'k6/http';
import { check, group, sleep } from 'k6';

export let options = {
  scenarios: {
    // Scenario 1: Token validation (high throughput)
    token_validation: {
      executor: 'constant-arrival-rate',
      rate: 200,  // 200 req/sec = 12K req/min
      timeUnit: '1s',
      duration: '5m',
      preAllocatedVUs: 50,
      exec: 'validateToken',
    },

    // Scenario 2: Usage recording (medium throughput)
    usage_recording: {
      executor: 'constant-arrival-rate',
      rate: 100,  // 100 req/sec = 6K req/min
      timeUnit: '1s',
      duration: '5m',
      preAllocatedVUs: 30,
      exec: 'recordUsage',
    },

    // Scenario 3: Dashboard API (low throughput, high latency tolerance)
    dashboard_api: {
      executor: 'constant-arrival-rate',
      rate: 50,  // 50 req/sec = 3K req/min
      timeUnit: '1s',
      duration: '5m',
      preAllocatedVUs: 20,
      exec: 'dashboardQuery',
    },
  },

  thresholds: {
    'http_req_duration{scenario:token_validation}': ['p(95)<100'],
    'http_req_duration{scenario:usage_recording}': ['p(95)<100'],
    'http_req_duration{scenario:dashboard_api}': ['p(95)<200'],
    'http_req_failed': ['rate<0.01'],
  },
};

export function validateToken() {
  let response = http.get('http://localhost:8080/api/v1/health', {
    headers: { 'Authorization': 'Bearer test_token' },
  });
  check(response, { 'status is 200': (r) => r.status === 200 });
  sleep(0.1);
}

export function recordUsage() {
  let response = http.post('http://localhost:8080/api/v1/usage', JSON.stringify({
    token_id: 1,
    provider: 'openai',
    input_tokens: 100,
    output_tokens: 50,
  }), {
    headers: {
      'Authorization': 'Bearer test_token',
      'Content-Type': 'application/json',
    },
  });
  check(response, { 'status is 201': (r) => r.status === 201 });
  sleep(0.5);
}

export function dashboardQuery() {
  let response = http.get('http://localhost:8080/api/v1/usage?window=1h', {
    headers: { 'Authorization': 'Bearer test_token' },
  });
  check(response, { 'status is 200': (r) => r.status === 200 });
  sleep(1);
}
```

---

## 6. Security Validation

### 6.1 OWASP Top 10 Validation Checklist

```yaml
security_validation:
  owasp_top_10:
    A01_broken_access_control:
      requirement: "RBAC enforces role-based access"
      validation_method: "Integration tests"
      tests:
        - "Admin can access all resources"
        - "User can only access own resources"
        - "User cannot access other users' resources"
        - "Agent cannot access dashboard endpoints"
      automated: true
      status: "⏳ pending"

    A02_cryptographic_failures:
      requirement: "Tokens hashed with SHA-256, never plaintext"
      validation_method: "Code review + database inspection"
      checks:
        - "No plaintext tokens in api_tokens table"
        - "All tokens hashed before storage"
        - "TLS 1.3 enforced for all connections"
      automated: partial
      status: "⏳ pending"

    A03_injection:
      requirement: "All queries use prepared statements"
      validation_method: "Code review + SQLx compile-time verification"
      checks:
        - "SQLx query! macro used for all queries"
        - "No string concatenation in SQL"
        - "Input validation on all endpoints"
      automated: true (compile-time)
      status: "⏳ pending"

    A04_insecure_design:
      requirement: "Threat modeling completed"
      validation_method: "Security design review"
      deliverables:
        - "Threat model document"
        - "Security architecture review"
        - "Attack surface analysis"
      automated: false
      status: "⏳ pending"

    A05_security_misconfiguration:
      requirement: "Security headers enabled, default credentials changed"
      validation_method: "OWASP ZAP scan"
      headers:
        - "Strict-Transport-Security: max-age=31536000"
        - "X-Content-Type-Options: nosniff"
        - "X-Frame-Options: DENY"
        - "Content-Security-Policy: default-src 'self'"
      automated: true
      status: "⏳ pending"

    A06_vulnerable_components:
      requirement: "No known vulnerabilities in dependencies"
      validation_method: "cargo-audit"
      command: "cargo audit --deny warnings"
      frequency: "Weekly"
      automated: true
      status: "⏳ pending"

    A07_authentication_failures:
      requirement: "JWT with short expiry, secure password hashing"
      validation_method: "Integration tests + penetration testing"
      checks:
        - "JWT expires in 1 hour"
        - "Refresh token rotation"
        - "Rate limiting on login (10/min)"
        - "Password hashing with Argon2"
      automated: true
      status: "⏳ pending"

    A08_data_integrity_failures:
      requirement: "HMAC signatures, checksum validation"
      validation_method: "Code review"
      checks:
        - "JWT uses HMAC-SHA256"
        - "Token hashes verified before use"
      automated: partial
      status: "⏳ pending"

    A09_logging_failures:
      requirement: "Audit logging for all sensitive operations"
      validation_method: "Code review + log inspection"
      logged_events:
        - "Token created/rotated/revoked"
        - "Limit updated"
        - "Authentication success/failure"
        - "Authorization denied (403)"
      automated: true (test verifies logs)
      status: "⏳ pending"

    A10_ssrf:
      requirement: "No user-controlled URLs"
      validation_method: "Code review"
      checks:
        - "All provider URLs hardcoded"
        - "No URL parameters accepted from users"
      automated: false
      status: "⏳ pending"
```

### 6.2 Penetration Testing Checklist

```yaml
penetration_testing:
  authentication_attacks:
    - name: "JWT token theft"
      attack: "Attempt to steal JWT from localStorage"
      mitigation: "Use httpOnly cookies instead"
      validation: "Manual + automated"

    - name: "Brute force login"
      attack: "Attempt 1000 login attempts"
      expected: "Rate limiter blocks after 10 attempts"
      validation: "Automated test"

  authorization_attacks:
    - name: "Privilege escalation"
      attack: "User attempts to access admin endpoints"
      expected: "403 Forbidden"
      validation: "Automated test"

    - name: "IDOR (Insecure Direct Object Reference)"
      attack: "User A attempts to access User B's tokens"
      expected: "403 Forbidden"
      validation: "Automated test"

  injection_attacks:
    - name: "SQL injection"
      attack: "Submit SQL code in username field"
      expected: "Input validation rejects, no SQL execution"
      validation: "OWASP ZAP + manual"

    - name: "XSS (Cross-Site Scripting)"
      attack: "Submit <script> in form fields"
      expected: "Input sanitized, no script execution"
      validation: "OWASP ZAP + manual"
```

---

## 7. Quality Validation

### 7.1 Code Quality Metrics

```yaml
code_quality_dashboard:
  test_coverage:
    tool: "cargo-tarpaulin"
    threshold: "> 80%"
    current: "N/A"
    measurement: "Line coverage"
    report_url: "target/coverage/index.html"

  cyclomatic_complexity:
    tool: "cargo-clippy"
    threshold: "< 10 per function"
    current: "N/A"
    measurement: "Cognitive complexity"

  code_duplication:
    tool: "cargo-geiger"
    threshold: "< 5% duplicate lines"
    current: "N/A"

  documentation_coverage:
    tool: "cargo doc --no-deps"
    threshold: "100% public APIs documented"
    current: "N/A"

  dependency_freshness:
    tool: "cargo outdated"
    threshold: "No major version behind"
    frequency: "Monthly"
```

### 7.2 Code Review Checklist

```markdown
## Code Review Checklist

### Functionality
- [ ] Code implements requirements from task/backlog/001_*.md
- [ ] All acceptance criteria met
- [ ] Edge cases handled
- [ ] Error cases handled

### Testing
- [ ] Unit tests added for new code
- [ ] Integration tests added for API changes
- [ ] E2E tests added for UI changes
- [ ] All tests passing locally
- [ ] Test coverage > 80% for changed files

### Code Quality
- [ ] Follows custom code style (not cargo fmt)
- [ ] No clippy warnings
- [ ] No compiler warnings
- [ ] Functions < 50 lines (ideally)
- [ ] Descriptive variable names
- [ ] Comments explain "why", not "what"

### Security
- [ ] No hardcoded secrets
- [ ] Input validation on all user inputs
- [ ] SQL injection prevented (prepared statements)
- [ ] XSS prevented (input sanitization)
- [ ] Authentication/authorization checked

### Performance
- [ ] No N+1 queries
- [ ] Database indexes added for new queries
- [ ] Large operations are async
- [ ] Caching considered where appropriate

### Documentation
- [ ] Public APIs have doc comments
- [ ] README updated if needed
- [ ] OpenAPI spec updated if API changed
- [ ] Migration guide if breaking change

### Validation
- [ ] Traceability matrix updated
- [ ] Validation commands documented
- [ ] CI/CD pipeline passing
```

---

## 8. Acceptance Validation

### 8.1 User Acceptance Testing (UAT) Plan

```yaml
uat_plan:
  participants:
    - role: "Product Owner"
      responsibility: "Verify business requirements met"

    - role: "End User (Admin)"
      responsibility: "Test admin workflows"

    - role: "End User (Developer)"
      responsibility: "Test API integration"

  test_scenarios:
    scenario_1_token_lifecycle:
      description: "Complete token lifecycle from creation to revocation"
      steps:
        1. "Login to dashboard"
        2. "Navigate to Tokens view"
        3. "Click 'Generate Token'"
        4. "Select provider (OpenAI)"
        5. "Copy token to clipboard"
        6. "Use token in API call (via curl)"
        7. "Verify usage appears in Usage view"
        8. "Rotate token"
        9. "Verify old token no longer works"
        10. "Revoke token"
        11. "Verify token completely disabled"
      acceptance_criteria:
        - "All steps complete without errors"
        - "Usage tracked accurately"
        - "Revoked token returns 401"
      status: "⏳ pending"

    scenario_2_limit_enforcement:
      description: "Test hard limit enforcement with grace period"
      steps:
        1. "Create limit: 1000 tokens with 100 grace"
        2. "Make API calls totaling 950 tokens"
        3. "Verify usage shows 95%"
        4. "Make API call for 100 tokens (exceeds limit, within grace)"
        5. "Verify warning notification"
        6. "Make API call for 100 tokens (exceeds grace)"
        7. "Verify request rejected with 429"
      acceptance_criteria:
        - "Limit enforced exactly at threshold"
        - "Grace period allows temporary overage"
        - "Hard stop after grace period"
      status: "⏳ pending"

    scenario_3_analytics_dashboard:
      description: "Verify usage analytics are accurate and useful"
      steps:
        1. "Make 10 API calls across 3 providers"
        2. "Navigate to Usage view"
        3. "Verify total usage matches 10 calls"
        4. "Verify usage by provider chart shows 3 providers"
        5. "Change date range to 'Last 7 days'"
        6. "Verify chart updates correctly"
      acceptance_criteria:
        - "All data accurate"
        - "Charts visually clear"
        - "Date range filtering works"
      status: "⏳ pending"

uat_sign_off:
  required_approvals:
    - "Product Owner: Requirements met"
    - "Technical Lead: Code quality acceptable"
    - "Security Reviewer: No security concerns"
    - "End User: Usable and intuitive"
```

### 8.2 Production Readiness Checklist

```yaml
production_readiness:
  code:
    - [ ] All features implemented per spec
    - [ ] All tests passing (w3 .test l::5)
    - [ ] Code reviewed and approved
    - [ ] No TODO/FIXME comments in production code

  testing:
    - [ ] Test coverage > 80%
    - [ ] Load testing passed (10K req/min)
    - [ ] E2E tests passing
    - [ ] UAT sign-off received

  security:
    - [ ] Security scan: 0 critical/high vulns
    - [ ] Penetration testing completed
    - [ ] Secrets externalized (no hardcoded)
    - [ ] HTTPS enforced

  performance:
    - [ ] API latency p95 < 100ms
    - [ ] Dashboard load time < 2s
    - [ ] Database indexed properly
    - [ ] Caching implemented

  operations:
    - [ ] Deployment guide complete
    - [ ] Rollback procedure documented
    - [ ] Monitoring alerts configured
    - [ ] Backup strategy in place

  documentation:
    - [ ] API documentation (OpenAPI)
    - [ ] User guide
    - [ ] Developer guide
    - [ ] Security documentation

  compliance:
    - [ ] OWASP Top 10 addressed
    - [ ] Audit logging implemented
    - [ ] Data retention policy defined
    - [ ] Privacy policy reviewed
```

---

## 9. Continuous Validation Dashboard

### 9.1 Real-Time Validation Dashboard

```yaml
dashboard_config:
  metrics:
    build_status:
      source: "GitHub Actions API"
      query: "GET /repos/{owner}/{repo}/actions/runs?status=in_progress"
      display: "Badge (passing/failing)"
      refresh: "30 seconds"

    test_coverage:
      source: "Codecov API"
      query: "GET /api/gh/{owner}/{repo}/branch/main"
      display: "Percentage + trend graph"
      refresh: "5 minutes"

    security_vulnerabilities:
      source: "cargo-audit JSON output"
      query: "Parse JSON for critical/high"
      display: "Count + severity breakdown"
      refresh: "Daily"

    api_latency:
      source: "k6 results"
      query: "Parse k6 summary JSON"
      display: "p50/p95/p99 graph"
      refresh: "After each load test"

    deployment_status:
      source: "Kubernetes API"
      query: "GET /apis/apps/v1/namespaces/default/deployments"
      display: "Deployment health (ready replicas)"
      refresh: "1 minute"

  alerts:
    - condition: "test_coverage < 80%"
      action: "Post to Slack #dev-alerts"

    - condition: "security_vulns > 0"
      action: "Create GitHub issue + notify security team"

    - condition: "api_latency_p95 > 100ms"
      action: "Post to Slack #performance-alerts"

    - condition: "deployment_status != healthy"
      action: "Page on-call engineer"
```

### 9.2 Grafana Dashboard Example

```yaml
# grafana/dashboards/token_management_validation.json
panels:
  - title: "Test Results (Last 7 Days)"
    type: "graph"
    datasource: "GitHub Actions"
    targets:
      - expr: "github_actions_workflow_run_conclusion{workflow='validation'}"

  - title: "Code Coverage Trend"
    type: "graph"
    datasource: "Codecov"
    targets:
      - expr: "codecov_coverage_percentage{branch='main'}"

  - title: "API Performance (p95 Latency)"
    type: "graph"
    datasource: "k6"
    targets:
      - expr: "k6_http_req_duration{percentile='95'}"
    thresholds:
      - value: 100
        color: "red"
        line: true

  - title: "Security Vulnerability Count"
    type: "stat"
    datasource: "cargo-audit"
    targets:
      - expr: "sum(cargo_audit_vulnerabilities{severity=~'critical|high'})"
    thresholds:
      - value: 0
        color: "green"
      - value: 1
        color: "red"
```

---

## 10. Validation Failure Protocols

### 10.1 Failure Response Procedures

```yaml
validation_failure_protocols:
  blocking_gate_failure:
    severity: "Critical"
    response_time: "Immediate"
    procedure:
      1. "STOP all development on current phase"
      2. "Create GitHub issue with 'validation-failure' label"
      3. "Notify team in Slack #dev-urgent"
      4. "Root cause analysis (RCA) within 2 hours"
      5. "Fix identified issue"
      6. "Re-run validation"
      7. "Document lesson learned"
    escalation:
      - "If not resolved in 4 hours → notify technical lead"
      - "If not resolved in 1 day → schedule team meeting"

  warning_gate_failure:
    severity: "High"
    response_time: "Same day"
    procedure:
      1. "Create GitHub issue with 'validation-warning' label"
      2. "Assess if acceptable (e.g., 75% coverage vs 80% target)"
      3. "If acceptable → document justification + proceed"
      4. "If unacceptable → fix within current phase"
    escalation:
      - "Technical lead approval required to proceed with warning"

  advisory_gate_failure:
    severity: "Medium"
    response_time: "Next sprint"
    procedure:
      1. "Add to backlog as 'tech-debt' item"
      2. "Prioritize in next sprint planning"
      3. "No blocking, can proceed"

failure_documentation:
  template: |
    ## Validation Failure Report

    **Date:** YYYY-MM-DD
    **Phase:** [Phase number]
    **Gate:** [Blocking/Warning/Advisory]
    **Validator:** [Test/Tool name]

    ### What Failed
    [Specific failure description]

    ### Expected Behavior
    [What should have happened]

    ### Actual Behavior
    [What actually happened]

    ### Root Cause
    [Why it failed - technical details]

    ### Fix Applied
    [How it was fixed]

    ### Prevention
    [How to prevent in future]

    ### Lessons Learned
    [Insights for team]
```

### 10.2 Validation Metrics Tracking

```yaml
validation_metrics:
  tracking_fields:
    - "Validation name"
    - "Expected result"
    - "Actual result"
    - "Pass/Fail status"
    - "Timestamp"
    - "Duration (seconds)"
    - "Error message (if failed)"

  storage:
    type: "PostgreSQL table"
    schema: |
      CREATE TABLE validation_history (
        id BIGSERIAL PRIMARY KEY,
        phase INTEGER NOT NULL,
        validation_name VARCHAR(255) NOT NULL,
        expected_result TEXT,
        actual_result TEXT,
        status VARCHAR(20) NOT NULL, -- pass, fail, warning
        timestamp TIMESTAMPTZ DEFAULT NOW(),
        duration_seconds INTEGER,
        error_message TEXT,
        git_commit_sha VARCHAR(40),
        INDEX idx_phase (phase),
        INDEX idx_status (status),
        INDEX idx_timestamp (timestamp)
      );

  reporting:
    daily_summary:
      recipients: ["team@example.com"]
      template: |
        Daily Validation Summary
        ========================
        Date: {date}

        Total Validations: {total}
        Passed: {passed} ({pass_rate}%)
        Failed: {failed}
        Warnings: {warnings}

        Failed Validations:
        {failed_list}

        Trend: {trend_emoji}

    weekly_review:
      recipients: ["leads@example.com"]
      metrics:
        - "Total validation count"
        - "Pass rate trend"
        - "Most common failures"
        - "Average fix time"
```

---

## Appendix A: Validation Tools Matrix

| Validation Type | Tool | Automation Level | Frequency | Owner |
|----------------|------|------------------|-----------|-------|
| Unit Tests | cargo nextest | L2 (Auto PR) | Every commit | Dev |
| Integration Tests | cargo test | L2 (Auto PR) | Every commit | Dev |
| E2E Tests | Playwright | L2 (Auto PR) | Every commit | Dev |
| Code Coverage | cargo-tarpaulin | L2 (Auto PR) | Every commit | Dev |
| Security Scan | OWASP ZAP + cargo-audit | L3 (Auto CD) | Daily | DevOps |
| Performance | k6 + Criterion | L3 (Auto CD) | Weekly | DevOps |
| Load Testing | k6 | L1 (Semi-Auto) | Pre-release | DevOps |
| UAT | Manual testing | L0 (Manual) | End of phase | Product |

---

## Appendix B: Validation Command Reference

```bash
# Quick validation (pre-commit)
./scripts/validate_quick.sh

# Full validation (phase gate)
./scripts/validate_phase_1.sh
./scripts/validate_phase_2.sh
# ... etc

# Security validation
./scripts/security_scan.sh

# Performance validation
k6 run load_tests/api_latency.js

# Coverage validation
cargo tarpaulin --out Html

# Complete validation (all phases)
./scripts/validate_all.sh
```

---

**Document Status:** ✅ Complete
**Last Updated:** 2025-12-02
**Next Review:** After each phase completion
