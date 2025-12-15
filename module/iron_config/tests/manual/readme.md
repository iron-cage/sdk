# Manual Testing Plan: iron_config (Phase 2 Configuration Unification)

Manual testing plan for iron_config crate focusing on environment variable precedence, file config loading, and edge cases.

## Purpose

Manual tests verify Phase 2 Configuration Unification corner cases that are difficult to test in isolation:
- Environment variable override behavior in real environment
- File config precedence across different scenarios
- Type conversion edge cases
- Error handling for invalid configurations

## Prerequisites

Before running manual tests, ensure:

1. **Crate Built**: `cargo build --all-features` completed successfully
2. **Clean Environment**: No conflicting ENV vars set
3. **Test Directory**: Create temp directory for test config files

## Environment Setup

### 1. Prepare Test Directory

```bash
# Create temp test directory
mkdir -p /tmp/iron_config_manual_test
cd /tmp/iron_config_manual_test

# Create test config file
cat > config.toml <<EOF
api_url = "https://file-config.example.com"
timeout = 30
debug = false

[database]
url = "postgresql://file-db.example.com"
pool_size = 10
EOF
```

### 2. Set Up Test Env Vars

```bash
# Clear any existing IRON_* vars
unset $(env | grep '^IRON_' | cut -d= -f1)

# Export test ENV vars
export IRON_TEST_API_URL="https://env-override.example.com"
export IRON_TEST_TIMEOUT="60"
export IRON_TEST_DEBUG="true"
export IRON_TEST_DATABASE__URL="postgresql://env-db.example.com"
```

## Test Coverage Status

**Automated Test Coverage**: 36 tests covering critical corner cases
**Manual Test Requirements**: Platform-specific behavior and real environment validation

### Automated Tests (Fully Covered)

The following corner cases are now covered by automated tests in:
- `tests/env_layer_test.rs`
- `tests/file_layer_test.rs`
- `tests/precedence_test.rs`

✅ **ENV Variable Precedence** (precedence_test.rs:14)
✅ **Empty String ENV Var** (env_layer_test.rs:158)
✅ **Whitespace-Only ENV Var** (env_layer_test.rs:174)
✅ **Special Characters in ENV Var** (env_layer_test.rs:190)
✅ **Unicode in ENV Var** (env_layer_test.rs:207)
✅ **Boolean Parsing (true/false)** (env_layer_test.rs:89)
✅ **Boolean Parsing Extended (1/0/yes/no)** (env_layer_test.rs:224)
✅ **Integer Parsing Valid** (env_layer_test.rs:108)
✅ **Integer Parsing Invalid** (env_layer_test.rs:266)
✅ **String Fallback** (env_layer_test.rs:122)
✅ **Key Conversion (underscore→dot)** (env_layer_test.rs:9)
✅ **Prefix Filtering** (env_layer_test.rs:137)
✅ **Nested Configuration** (precedence_test.rs:137)
✅ **Empty Config File** (file_layer_test.rs:173)
✅ **Invalid TOML Syntax** (file_layer_test.rs:52)
✅ **Unknown Fields (Forward Compatibility)** (file_layer_test.rs:194)
✅ **Large Value Handling (1KB)** (env_layer_test.rs:283)

## Manual Test Cases (Real Environment Required)

The following tests require manual verification in real shell environment:

### TC-1: Platform-Specific Behavior

#### TC-1.6: ENV Var Case Sensitivity (Platform-Specific)

**Purpose**: Verify case sensitivity behavior varies by platform

**Test (Linux/macOS):**
```bash
export IRON_TEST_CASE_LOWERCASE="value1"
export IRON_TEST_CASE_lowercase="value2"

# Both should be distinct on Unix-like systems
env | grep IRON_TEST_CASE
```

**Expected (Linux/macOS):**
- Both variables exist separately
- Case-sensitive behavior confirmed

**Test (Windows):**
```cmd
set IRON_TEST_CASE_LOWERCASE=value1
set IRON_TEST_CASE_lowercase=value2

# Second assignment should override first on Windows
set IRON_TEST_CASE
```

**Expected (Windows):**
- Only one variable exists
- Case-insensitive behavior confirmed

### TC-2: Real Environment Integration

**Purpose**: Verify configuration loading in actual shell environment (not env::set_var)

#### TC-2.1: Real Shell ENV Override

**Test:**
```bash
cd /tmp
cat > test_config.toml <<EOF
[database]
url = "postgresql://file.example.com"
timeout = 30
EOF

export IRON_MANUAL_DATABASE__URL="postgresql://env-override.example.com"
export IRON_MANUAL_DATABASE__TIMEOUT="60"

# Would need test binary that uses ConfigLoader with "iron_manual" prefix
# and loads test_config.toml, then queries database.url and database.timeout
```

**Expected:**
- ENV vars override file config in real environment
- Values accessible via ConfigLoader::get()

**Status:** Requires custom test binary (out of scope for automated tests)

### TC-3: Performance Validation

**Purpose**: Verify acceptable performance for large configurations

#### TC-3.1: Concurrent Access (Thread Safety)

**Test:**
```bash
# Requires custom test program that:
# 1. Creates ConfigLoader
# 2. Spawns 10 threads
# 3. Each thread queries same config keys 1000 times
# 4. Verifies all threads get consistent results
```

**Expected:**
- Thread-safe read access
- No race conditions
- Consistent values across all threads
- No deadlocks

**Status:** Could be automated but requires stress testing framework

#### TC-3.2: Large Configuration Performance

**Test:**
```bash
# Create config with 1000 keys
cat > large_config.toml <<EOF
$(for i in {1..1000}; do echo "[section_$i]"; echo "key = \"value_$i\""; done)
EOF

# Benchmark ConfigLoader initialization and queries
# Should complete in < 100ms for 1000 keys
```

**Expected:**
- ConfigLoader initialization: < 50ms
- Individual get() queries: < 1ms
- get_all() query: < 10ms

**Status:** Performance benchmarking (out of scope for unit tests)

## Test Execution Summary

### Automated Tests Status

**Last Run:** 2024-12-15
**Result:** ✅ 36/36 tests PASSING
**Command:** `w3 .test l::3` in `module/iron_config`

**Coverage:**
- Environment variable precedence (all corner cases)
- File config loading (empty, invalid, unknown fields)
- Type conversion (bool, int, string fallback)
- Key conversion (underscore→dot, prefix filtering)
- Edge cases (empty string, whitespace, special chars, unicode, large values)

### Manual Tests Status

**Last Run:** Not yet executed
**Status:** ⏳ PENDING

**Remaining Manual Tests:**
- [ ] TC-1.6: ENV var case sensitivity (platform-specific)
- [ ] TC-2.1: Real shell environment integration
- [ ] TC-3.1: Concurrent access / thread safety
- [ ] TC-3.2: Large configuration performance

**Reason for Manual Testing:**
- Platform-specific behavior (Windows vs Linux/macOS)
- Real environment validation (not `env::set_var`)
- Performance benchmarking
- Stress testing (concurrency)

### Validation Criteria

For automated tests (✅ COMPLETE):
- ✅ No panics or crashes
- ✅ Error messages clear and actionable
- ✅ Type conversions work as documented
- ✅ Precedence order correct (ENV > File > Defaults)
- ✅ Edge cases handled gracefully

For manual tests (⏳ PENDING):
- Platform-specific behavior verified on Linux, macOS, Windows
- Real shell environment integration confirmed
- Thread safety validated under concurrent access
- Performance meets targets (< 100ms for 1000 keys)

## Cleanup

After completing manual tests:

```bash
# Remove test files
rm -f /tmp/test_config.toml /tmp/large_config.toml

# Unset test ENV vars
unset $(env | grep '^IRON_MANUAL_' | cut -d= -f1)
unset $(env | grep '^IRON_TEST_CASE_' | cut -d= -f1)
```

## Notes

### Automated vs Manual Testing

**Automated tests (36 tests)** cover:
- Functional correctness for all corner cases
- ENV var parsing edge cases (empty, whitespace, special chars, unicode)
- File config edge cases (empty, invalid, unknown fields)
- Type conversion behavior (bool, int, string fallback)
- Precedence rules (ENV > File > Defaults)

**Manual tests (4 tests)** focus on:
- Platform-specific behavior (case sensitivity)
- Real environment validation (actual shell, not `env::set_var`)
- Performance benchmarking
- Concurrency stress testing

### When to Run Manual Tests

- Before major releases (quarterly)
- When porting to new platforms (Windows, macOS, Linux distros)
- After architecture changes affecting ConfigLoader
- When performance regressions suspected

### Test Environment Requirements

- **Platform Testing**: Access to Linux, macOS, Windows environments
- **Performance Testing**: Dedicated test machine (no background load)
- **Real Environment**: Actual shell session (not IDE integrated terminal)

## Last Run

**Automated Tests:**
- Date: 2024-12-15
- Result: ✅ 36/36 PASSING
- Command: `w3 .test l::3`

**Manual Tests:**
- Date: Not yet run
- Result: ⏳ PENDING
- Tester: N/A
