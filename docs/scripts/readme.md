# Documentation Validation Scripts

Automated validation tools to ensure documentation quality and consistency across the Iron Runtime project.

## Overview

This directory contains validation scripts that enforce documentation standards:

1. **`validate_metadata.sh`** - Validates protocol file metadata format
2. **`validate_id_format.sh`** - Detects hyphenated entity IDs (enforces underscore format)
3. **`validate_cross_references.sh`** - Verifies all markdown links resolve correctly
4. **`validate_all.sh`** - Master script that runs all validators

## Quick Start

### Validate Everything

```bash
./docs/scripts/validate_all.sh
```

### Validate Specific Areas

```bash
# Validate protocol metadata
./docs/scripts/validate_metadata.sh docs/protocol

# Check for hyphenated IDs
./docs/scripts/validate_id_format.sh docs/protocol

# Verify all cross-references
./docs/scripts/validate_cross_references.sh docs
```

### Validate Single File

```bash
./docs/scripts/validate_metadata.sh docs/protocol/010_agents_api.md
./docs/scripts/validate_id_format.sh docs/protocol/010_agents_api.md
```

## Installation

### Pre-commit Hook (Recommended)

Automatically validate documentation before commits:

```bash
# Install hook
cp docs/scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Test it
git add docs/protocol/010_agents_api.md
git commit -m "test: validate pre-commit hook"
```

To bypass the hook in emergencies (not recommended):

```bash
git commit --no-verify
```

### CI/CD Integration

The project includes a GitHub Actions workflow (`.github/workflows/validate-docs.yml`) that automatically validates documentation on:
- Every pull request affecting documentation
- Every push to main/master/dev branches

## Validation Rules

### Metadata Validation

All protocol files must have standardized metadata at lines 3-8:

```markdown
# Protocol Title

**Status:** Specification | Pilot | POST-PILOT | Archived
**Version:** X.Y.Z
**Last Updated:** YYYY-MM-DD
**Priority:** MUST-HAVE | NICE-TO-HAVE | POST-PILOT | TBD

---
```

**Common Errors:**
- Missing or malformed metadata fields
- Invalid date format (must be YYYY-MM-DD)
- Incorrect status/priority values
- Duplicate metadata blocks at end of file

### ID Format Validation

Entity IDs must use underscore separators, not hyphens:

- ✅ Correct: `agent_abc123`, `ip_openai_001`, `budget_req_456`
- ❌ Incorrect: `agent-abc123`, `ip-openai-001`, `budget-req-456`

**Excluded Patterns** (legitimate hyphenated terms):
- Technical terms: `rate-limit`, `real-time`, `read-only`
- Status values: `POST-PILOT`, `MUST-HAVE`, `NICE-TO-HAVE`
- Auth headers: `user-token`, `api-token`
- 20+ other compound words (see `validate_id_format.sh`)

### Cross-Reference Validation

All markdown links must resolve correctly:

- ✅ Valid: `[Protocol 010](010_agents_api.md)` (file exists)
- ✅ Valid: `[Standards](../standards/api_design_standards.md)` (file exists)
- ❌ Invalid: `[Missing](nonexistent.md)` (file not found)
- ❌ Invalid: `[Bad Anchor](file.md#missing-section)` (anchor not found)

**Supported Link Types:**
1. Internal protocol links: `[text](010_agents_api.md)`
2. Cross-directory: `[text](../standards/file.md)`
3. Anchors: `[text](file.md#section-name)`
4. External URLs: `[text](https://...)` (not validated)

## Exit Codes

All scripts follow consistent exit code conventions:

- **0** - All validations passed
- **1** - One or more validations failed

Use in scripts:

```bash
if ./docs/scripts/validate_metadata.sh docs/protocol; then
  echo "Validation passed!"
else
  echo "Validation failed!"
fi
```

## Troubleshooting

### Script Permission Denied

```bash
chmod +x docs/scripts/validate_*.sh
```

### False Positive for Compound Word

If a legitimate hyphenated term is flagged, add it to `EXCLUDE_PATTERNS` in `validate_id_format.sh`:

```bash
EXCLUDE_PATTERNS=(
  "existing-patterns"
  "your-new-pattern"
)
```

### Pre-commit Hook Not Running

```bash
# Verify hook is executable
ls -la .git/hooks/pre-commit

# Reinstall hook
cp docs/scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Validation Failing in CI

1. Run validation locally: `./docs/scripts/validate_all.sh`
2. Fix all reported errors
3. Commit and push fixes
4. CI should pass on next run

## Output Format

### Success Output

```
====================================
Metadata Validator
====================================

Validating: protocol/010_agents_api.md
  ✅ PASS

====================================
Summary:
Total Files:  1
Passed:       1
Failed:       0
====================================
✅ ALL VALIDATIONS PASSED
```

### Failure Output

```
====================================
Metadata Validator
====================================

Validating: protocol/015_projects_api.md
  ❌ FAIL
    ▸ Line 6: Missing or malformed **Priority:** field

====================================
Summary:
Total Files:  1
Passed:       0
Failed:       1
====================================
❌ SOME VALIDATIONS FAILED
```

## Maintenance

### Adding New Validation Rules

1. Edit the relevant validator script
2. Test on all protocol files: `./validate_all.sh`
3. Update this readme with new rules
4. Commit changes

### Updating Exclusion Patterns

Edit `validate_id_format.sh` and add patterns to `EXCLUDE_PATTERNS` array:

```bash
EXCLUDE_PATTERNS=(
  "existing-pattern"
  "new-compound-word"
)
```

### Performance Optimization

- Validators are designed for fast execution (< 5 seconds for all files)
- Use single-file validation during development
- Run full validation before committing

## Integration

### Local Development Workflow

```bash
# 1. Make documentation changes
vim docs/protocol/010_agents_api.md

# 2. Validate changes
./docs/scripts/validate_metadata.sh docs/protocol/010_agents_api.md

# 3. Fix any errors

# 4. Commit (pre-commit hook validates automatically)
git add docs/protocol/010_agents_api.md
git commit -m "docs: update agents API"
```

### Pull Request Workflow

1. Create PR with documentation changes
2. GitHub Actions runs validation automatically
3. PR checks show validation status
4. Fix any errors before merging
5. Merge only when all checks pass

## Support

For issues or questions:
1. Check this readme for troubleshooting
2. Review validation error messages (they include fix suggestions)
3. Consult the automation plan: `docs/-automation_detailed_plan.md`

## Related Documentation

- **[Automation Plan](../-automation_detailed_plan.md)** - Comprehensive automation strategy
- **[Fix Summary](../-fix_execution_summary.md)** - Record of documentation fixes applied
- **[Project Summary](../-comprehensive_project_summary.md)** - Full initiative overview
