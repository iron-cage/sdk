# tree_fmt Migration CI/CD Integration Guide

This guide explains how to integrate tree_fmt migration verification into various CI/CD systems to ensure ongoing compliance and prevent regressions.

---

## Overview

**Purpose:** Automated verification that tree_fmt migration enforcement remains active across all code changes.

**What Gets Verified:**
1. ✅ All adapters use TreeFmtFormatter (no legacy formatter)
2. ✅ Zero `println!` violations in source files
3. ✅ Clippy configuration active (`.clippy.toml`)
4. ✅ Lint tests passing
5. ✅ TreeFmtFormatter implementation exists
6. ✅ Proper `#![allow]` usage (binaries only)

---

## Quick Start

### GitHub Actions (Recommended)

**File:** `.github/workflows/tree_fmt_verification.yml`

```yaml
name: tree_fmt Verification

on:
  push:
    paths: ['module/iron_cli/**']
  pull_request:
    paths: ['module/iron_cli/**']

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest
      - run: bash scripts/validation/verify_tree_fmt_migration.sh
```

**Status:** ✅ Full workflow provided in `.github/workflows/tree_fmt_verification.yml`

### GitLab CI

**File:** `.gitlab-ci.yml`

```yaml
tree_fmt_verification:
  stage: test
  image: rust:latest
  before_script:
    - cargo install cargo-nextest
  script:
    - bash scripts/validation/verify_tree_fmt_migration.sh
  only:
    changes:
      - module/iron_cli/**
      - scripts/validation/verify_tree_fmt_migration.sh
```

### Jenkins Pipeline

**File:** `Jenkinsfile`

```groovy
pipeline {
  agent { docker { image 'rust:latest' } }
  stages {
    stage('tree_fmt Verification') {
      when {
        changeset "module/iron_cli/**"
      }
      steps {
        sh 'cargo install cargo-nextest'
        sh 'bash scripts/validation/verify_tree_fmt_migration.sh'
      }
    }
  }
}
```

### CircleCI

**File:** `.circleci/config.yml`

```yaml
version: 2.1

jobs:
  verify-tree-fmt:
    docker:
      - image: cimg/rust:stable
    steps:
      - checkout
      - run:
          name: Install cargo-nextest
          command: cargo install cargo-nextest
      - run:
          name: Run tree_fmt verification
          command: bash scripts/validation/verify_tree_fmt_migration.sh

workflows:
  version: 2
  verify:
    jobs:
      - verify-tree-fmt
```

---

## Verification Components

### 1. Core Verification Script

**Location:** `scripts/validation/verify_tree_fmt_migration.sh`

**Checks:** 10 comprehensive validation steps

**Runtime:** ~30-60 seconds (depending on test suite size)

**Exit Codes:**
- `0` - All checks passed ✅
- `1` - Critical issues found ❌
- `2` - Warnings (non-blocking) ⚠️

**Usage:**
```bash
bash scripts/validation/verify_tree_fmt_migration.sh
```

### 2. Clippy Enforcement

**What:** Compile-time prevention of disallowed macros

**Configuration:** `module/iron_cli/.clippy.toml`

**Command:**
```bash
cargo clippy --package iron_cli --lib --all-features -- -D warnings
```

**Disallowed Macros:**
- `std::println`
- `std::print`
- `std::eprintln`
- `std::eprint`

**Why:** Prevents direct console output in library code (use TreeFmtFormatter instead)

### 3. Lint Tests

**What:** Runtime validation of zero `println!` violations

**Location:** `module/iron_cli/tests/lint/println_violations_test.rs`

**Command:**
```bash
cargo test --package iron_cli --test lint
```

**Coverage:**
- ✅ Scans all `.rs` files in `src/` (excluding `bin/`)
- ✅ Validates `#![allow]` usage (binaries only)
- ✅ Confirms TreeFmtFormatter usage in adapters

### 4. Performance Benchmarks (Optional)

**What:** Regression detection for formatting performance

**Location:** `module/iron_cli/benches/formatting_benchmarks.rs`

**Command:**
```bash
cargo bench --package iron_cli --bench formatting_benchmarks
```

**Baseline:** See `-tree_fmt_migration_verification_results.md`

**Thresholds:**
- JSON format: < 1 µs ✅
- Single item (10 keys): < 15 µs ✅
- List (100 items): < 250 µs ✅
- List (1000 items): < 3 ms ✅

---

## Integration Strategies

### Strategy 1: Pre-Merge Gate (Strict)

**Use Case:** High-confidence production code

**Implementation:** All checks must pass before merging

```yaml
# GitHub Actions
on:
  pull_request:
    types: [opened, synchronize]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - # ... verification steps
      - name: Block merge on failure
        if: failure()
        run: exit 1
```

**Protection:** Branch protection rules require passing checks

### Strategy 2: Advisory (Informational)

**Use Case:** Development branches, early prototypes

**Implementation:** Run verification but don't block merges

```yaml
# GitHub Actions
jobs:
  verify:
    runs-on: ubuntu-latest
    continue-on-error: true  # Don't block on failure
    steps:
      - # ... verification steps
```

**Output:** Results appear in PR comments for manual review

### Strategy 3: Scheduled Monitoring

**Use Case:** Detect drift over time

**Implementation:** Daily/weekly scheduled runs

```yaml
# GitHub Actions
on:
  schedule:
    - cron: '0 8 * * 1'  # Every Monday at 8 AM UTC

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - # ... verification steps
      - name: Notify on failure
        if: failure()
        uses: slackapi/slack-github-action@v1
        with:
          payload: '{"text": "tree_fmt verification drift detected!"}'
```

### Strategy 4: Path-Based Triggers

**Use Case:** Only verify when relevant files change

**Implementation:** Conditional execution

```yaml
# GitHub Actions
on:
  push:
    paths:
      - 'module/iron_cli/src/**/*.rs'
      - 'module/iron_cli/.clippy.toml'
      - 'module/iron_cli/tests/lint/**'
```

**Benefit:** Reduces CI runtime for unrelated changes

---

## Troubleshooting

### Issue 1: Verification Script Not Found

**Error:**
```
bash: scripts/validation/verify_tree_fmt_migration.sh: No such file or directory
```

**Solution:**
```bash
# Ensure working directory is project root
cd /path/to/iron_runtime/dev
bash scripts/validation/verify_tree_fmt_migration.sh
```

### Issue 2: cargo-nextest Not Installed

**Error:**
```
cargo nextest: command not found
```

**Solution:**
```bash
# GitHub Actions
- uses: taiki-e/install-action@v2
  with:
    tool: cargo-nextest

# Manual
cargo install cargo-nextest
```

### Issue 3: Clippy Warnings Fail Build

**Error:**
```
error: item in documentation is missing backticks
```

**Solution:** Fix warnings in code, don't disable checks

```bash
# Verify locally first
cargo clippy --package iron_cli --lib -- -D warnings
```

### Issue 4: Benchmark Performance Degradation

**Symptoms:** Benchmarks show >20% slowdown

**Solution:**
1. Review recent formatting changes
2. Profile hot paths: `cargo flamegraph --bench formatting_benchmarks`
3. Compare with baseline in verification results doc
4. Investigate TreeFmtFormatter usage patterns

### Issue 5: False Positive println! Detection

**Error:**
```
❌ FAIL: Direct printing detected in iron_cli source
```

**Cause:** Legitimate println! in binary entry point

**Solution:** Add `#![allow(clippy::disallowed_macros)]` at top of binary file

```rust
// src/bin/iron_control_unilang.rs
#![allow(clippy::disallowed_macros)]

fn main() {
  println!("Starting iron control...");  // ✅ Allowed
}
```

---

## Monitoring & Alerts

### Metrics to Track

| Metric | Threshold | Action |
|--------|-----------|--------|
| Verification failures | > 0 | Block merge |
| Benchmark degradation | > 20% | Investigate |
| New println! violations | > 0 | Reject PR |
| Clippy warnings | > 0 | Fix immediately |
| Test runtime | > 2 min | Optimize tests |

### Alerting Channels

**Slack Integration:**
```yaml
- name: Notify on failure
  if: failure()
  uses: slackapi/slack-github-action@v1
  with:
    webhook-url: ${{ secrets.SLACK_WEBHOOK }}
    payload: |
      {
        "text": "🚨 tree_fmt verification failed",
        "blocks": [
          {
            "type": "section",
            "text": {
              "type": "mrkdwn",
              "text": "PR: ${{ github.event.pull_request.html_url }}"
            }
          }
        ]
      }
```

**Email Notifications:**
```yaml
- name: Send email alert
  if: failure()
  uses: dawidd6/action-send-mail@v3
  with:
    server_address: smtp.gmail.com
    server_port: 465
    username: ${{ secrets.EMAIL_USERNAME }}
    password: ${{ secrets.EMAIL_PASSWORD }}
    subject: tree_fmt Verification Failed
    body: Check ${{ github.event.pull_request.html_url }}
    to: team@example.com
```

---

## Best Practices

### ✅ DO

1. **Run verification on every PR** - Catch issues early
2. **Use strict mode in CI** - `-D warnings` for Clippy
3. **Cache dependencies** - Speed up CI (use `actions-rust-lang/setup-rust-toolchain`)
4. **Monitor benchmarks** - Track performance over time
5. **Document exceptions** - If `#![allow]` used, explain why
6. **Version pin cargo-nextest** - Ensure consistent behavior
7. **Set timeouts** - Prevent hung CI jobs (15 min max)

### ❌ DON'T

1. **Don't skip verification** - Even for "trivial" changes
2. **Don't disable Clippy checks** - Fix issues instead
3. **Don't ignore warnings** - Treat as errors (`-D warnings`)
4. **Don't commit with failures** - Pre-commit hook should catch
5. **Don't modify .clippy.toml** - Without team review
6. **Don't add unnecessary #![allow]** - Only for binaries
7. **Don't skip benchmarks** - Performance matters

---

## Maintenance

### Quarterly Review

**Checklist:**
- [ ] Review verification script for improvements
- [ ] Update benchmark baselines if needed
- [ ] Check for new Clippy lints to enable
- [ ] Validate CI pipeline performance
- [ ] Review alert fatigue (false positives)

### When to Update Verification

**Triggers:**
1. New formatting requirements
2. Additional crates adopt tree_fmt
3. Performance baseline shifts
4. Rust version upgrade (new Clippy lints)
5. CI platform changes

**Process:**
1. Update verification script
2. Run locally to validate
3. Update CI workflow
4. Document changes in this guide
5. Notify team

---

## Advanced Configuration

### Custom Verification Levels

**Level 1: Quick (< 30s)**
```bash
# Only Clippy + lint tests
cargo clippy --package iron_cli --lib -- -D warnings
cargo test --package iron_cli --test lint
```

**Level 2: Standard (< 1min)**
```bash
# Full verification script
bash scripts/validation/verify_tree_fmt_migration.sh
```

**Level 3: Comprehensive (< 5min)**
```bash
# Verification + benchmarks
bash scripts/validation/verify_tree_fmt_migration.sh
cargo bench --package iron_cli --bench formatting_benchmarks
```

### Parallel Execution

**GitHub Actions:**
```yaml
jobs:
  quick-checks:
    strategy:
      matrix:
        check: [clippy, lint-tests, compile]
    runs-on: ubuntu-latest
    steps:
      - name: Run ${{ matrix.check }}
        run: # ... specific check command
```

**Benefit:** 3x faster verification (30s instead of 90s)

---

## Support & Resources

**Documentation:**
- Verification Results: `-tree_fmt_migration_verification_results.md`
- Strategic Analysis: `-tree_fmt_migration_strategic_analysis.md`
- Quick Reference: `module/iron_cli/docs/formatting_quick_reference.md`

**Contact:**
- Team Lead: [Reference team documentation]
- Slack Channel: `#tree-fmt-migration`
- Issues: GitHub Issues (tag: `tree-fmt`)

**Related Workflows:**
- Full CI: `.github/workflows/ci.yml`
- Pre-commit Hook: `scripts/automation/pre-commit`

---

**Last Updated:** 2025-12-13
**Version:** 1.0
**Status:** ✅ Production Ready
