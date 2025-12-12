# Manual Testing Plan - iron_token_manager

This directory contains manual test plans for the iron_token_manager crate.

## Purpose

Manual tests verify functionality that cannot be easily automated or require human judgment:
- Visual inspection of generated tokens
- Protocol compliance verification
- User-facing format validation
- Backward compatibility assessment

## Test Plans

### Protocol 014: Token Format

**File:** `protocol_014_token_format.md`

**What it tests:**
- Token format matches specification (`apitok_{64 chars}`)
- Base62 encoding compliance
- Token uniqueness across large sample sizes
- Hash prefix stripping behavior
- Backward compatibility with old token format
- End-to-end token lifecycle

**When to run:**
- After changes to `token_generator.rs`
- Before releasing Protocol 014 changes
- During code reviews
- When debugging token format issues

**How to run:**
1. Read test plan: `protocol_014_token_format.md`
2. Execute validation commands listed in each test case
3. Verify actual results match expected results
4. Document any failures or discrepancies

## Running Manual Tests

### Quick Verification
```bash
# Run all token format tests
cd ../../module/iron_token_manager
cargo nextest run token_generator --all-features
cargo nextest run token_storage --all-features
```

### Full Manual Test Suite
```bash
# Follow each test plan in order
cat protocol_014_token_format.md

# Execute all validation commands
# Verify all tests PASS
```

## Test Results

Manual test results are documented within each test plan file.

**Last Updated:** 2024-12-12
**Last Run:** 2024-12-12
**Result:** All tests PASS ✅

## Adding New Manual Tests

When adding new manual test plans:

1. Create new markdown file: `{feature_name}.md`
2. Follow existing test plan structure:
   - Feature description
   - Test environment
   - Test cases with steps/expected/actual results
   - Test summary table
   - Verification commands
3. Update this readme with test plan reference
4. Run tests and document results
5. Update "Last Run" date when executed

## Test Plan Structure

Each test plan should include:

- **Header:** Feature name, module, protocol, date
- **Environment:** Dependencies, versions, setup
- **Test Cases:** Numbered cases with:
  - Purpose
  - Steps
  - Expected results
  - Validation command
  - Actual results
- **Summary:** Table of all test results
- **Verification Commands:** Commands to reproduce tests
- **Sign-off:** Tester name, date, result
