# Iron Cage Standards

**Purpose:** Index of all technical standards for Iron Cage platform

**Responsibility:** Central reference for data formats, API conventions, and implementation standards

**Status:** Normative (all code and documentation must follow these standards)

---

## Overview

This directory contains canonical standards that define how Iron Cage components represent, process, and exchange data. These standards ensure consistency across REST API, CLI, SDK, and internal services.

**Compliance:** All new code MUST conform to these standards. Existing code SHOULD be updated during maintenance to follow these standards.

---

## Core Standards

### Data Format Standards

| Standard | File | Purpose | Status |
|----------|------|---------|--------|
| **ID Format** | [id_format_standards.md](id_format_standards.md) | Entity identifier format (prefix_uuid) | ✅ Normative |
| **Error Format** | [error_format_standards.md](error_format_standards.md) | Error response structure and codes | ✅ Normative |
| **Data Types** | [data_format_standards.md](data_format_standards.md) | Timestamps, currency, booleans, nulls | ✅ Normative |
| **URL Format** | [url_standards.md](url_standards.md) | Domain structure, endpoint conventions | ✅ Normative |

### API Design Standards

| Standard | File | Purpose | Status |
|----------|------|---------|--------|
| **API Design** | [api_design_standards.md](api_design_standards.md) | REST conventions, pagination, versioning | ✅ Normative |

### Documentation Standards

| Standard | File | Purpose | Status |
|----------|------|---------|--------|
| **Canonical Examples** | [canonical_examples.md](canonical_examples.md) | Standard example values for documentation consistency | ℹ️ Informative |

---

## Standard Categories

### 1. ID Format Standards (id_format_standards.md)

Defines entity identifier format using iron_types v0.3.0.

**Scope:**
- ID structure: `prefix_uuid` (e.g., `agent_550e8400-e29b-41d4-a716-446655440000`)
- Supported prefixes: `agent_`, `ip_`, `proj_`, `at_`, `breq_`, `lease_`, `req_`, `ic_`
- Migration from legacy `prefix-uuid` format
- Security validation requirements

**Applies to:**
- REST API request/response bodies
- Database primary keys and foreign keys
- CLI command arguments
- SDK function parameters
- Configuration files

---

### 2. Error Format Standards (error_format_standards.md)

Defines error response structure for all HTTP APIs.

**Scope:**
- HTTP status codes (200, 400, 401, 403, 404, 409, 500)
- Error response body format
- Validation error details
- Authentication/authorization errors
- Rate limiting responses

**Applies to:**
- REST API error responses
- Webhook error responses
- Internal service error propagation

---

### 3. Data Format Standards (data_format_standards.md)

Defines representation of common data types.

**Scope:**
- Timestamps: ISO 8601 with timezone (e.g., `2025-12-10T10:30:45Z`)
- Currency: Decimal numbers with 2 decimal places (e.g., `100.50`)
- Booleans: `true`/`false` (JSON standard)
- Null handling: Omit optional fields when empty
- Arrays: Empty arrays `[]` not `null`

**Applies to:**
- REST API request/response bodies
- Database column formats
- Configuration files
- Log messages

---

### 4. URL Format Standards (url_standards.md)

Defines canonical URLs for services and resources.

**Scope:**
- Production URLs: `https://api.ironcage.ai/v1/`
- Development URLs: `http://localhost:3000`
- Subdomain conventions: api, dashboard, gateway
- Error type URLs: `https://ironcage.ai/errors/`
- CORS configuration

**Applies to:**
- REST API documentation
- Frontend configuration
- Deployment scripts
- Error messages

---

### 5. API Design Standards (api_design_standards.md)

Defines REST API conventions and patterns.

**Scope:**
- Pagination: Offset-based (`?page=1&per_page=50`)
- Sorting: Query parameter (`?sort=name`, `?sort=-created_at`)
- Filtering: Query parameters (`?agent_id=agent-abc`)
- Versioning: URL path (`/v1/`, `/v2/`)
- Deprecation: Headers and 6-month notice period

**Applies to:**
- REST API endpoint design
- API documentation
- SDK client implementation

---

### 6. Canonical Examples (canonical_examples.md)

Defines standard example values for documentation consistency.

**Scope:**
- Primary agent: `agent_abc123` with budget `100.00` USD
- Standard users: `user-xyz789` (developer), `user-admin` (admin)
- Standard providers: `ip-openai-001`, `ip-anthropic-001`
- Standard budget values: 0.01, 10.00, 50.00, 100.00, 500.00, 1000.00
- Standard timestamps and token values

**Applies to:**
- Protocol documentation examples
- Specification requirement examples
- README files and tutorials
- Documentation that spans multiple files

**Does NOT apply to:**
- Unit tests (use varied test data)
- Integration tests (use dedicated fixtures)
- Implementation code (use actual data)

---

## Using These Standards

### For Developers

**When implementing new features:**
1. Read relevant standard documents before designing APIs/data structures
2. Reference standard in code comments (e.g., `// Format: See docs/standards/id_format_standards.md`)
3. Validate against standard in unit tests
4. Update standard if new requirements emerge (via PR review)

**When reviewing code:**
1. Check compliance with all applicable standards
2. Verify error responses match error_format_standards.md
3. Verify IDs match id_format_standards.md
4. Verify timestamps/currency match data_format_standards.md

### For Technical Writers

**When documenting APIs:**
1. Link to relevant standard documents
2. Use examples from standard documents
3. Ensure all examples comply with standards
4. Reference standard version in documentation

**Example:**
```markdown
Agent IDs use the format defined in [ID Format Standards](../standards/id_format_standards.md).

Example: `agent_550e8400-e29b-41d4-a716-446655440000`
```

### For QA/Testing

**When creating test cases:**
1. Positive tests: Valid data matching standards
2. Negative tests: Invalid data violating standards
3. Edge cases: Boundary conditions defined in standards
4. Migration tests: Legacy format compatibility (where applicable)

---

## Standard Evolution

### Versioning

Standards follow semantic versioning:
- **Major version** (X.0.0): Breaking changes requiring migration
- **Minor version** (1.X.0): Backward-compatible additions
- **Patch version** (1.0.X): Clarifications and corrections

**Current versions:**
- ID Format Standards: 0.3.0 (matches iron_types v0.3.0)
- Error Format Standards: 1.0.0
- Data Format Standards: 1.0.0
- URL Format Standards: 1.0.0
- API Design Standards: 1.0.0
- Canonical Examples: 1.0.0 (informative)

### Change Process

**Proposing changes:**
1. Create GitHub issue describing change and rationale
2. Label as `standard-change`
3. Discuss impact with team (breaking vs non-breaking)
4. Create PR updating standard document
5. Update affected code in same PR (for minor changes) or separate PR (for major changes)

**Breaking changes require:**
- [ ] Migration guide in standard document
- [ ] Deprecation period (minimum 6 months for public APIs)
- [ ] Version bump in standard document header
- [ ] Update to this readme.md with new version

### Backward Compatibility

**Legacy format support:**
- ID Format: `parse_flexible()` accepts both `prefix_uuid` and `prefix-uuid`
- Migration period: Until all data migrated (estimated 2026-06-10)
- Deprecation notice: In iron_types v0.2.0 release notes

---

## Validation Tools

### Automated Validation

**Location:** `dev/scripts/validate_standards.sh`

**Checks:**
- No hardcoded URLs (use environment variables)
- All IDs match `prefix_uuid` pattern
- All timestamps use ISO 8601 format
- All currency values have 2 decimal places
- Error responses include required fields

**Run before commit:**
```bash
./dev/scripts/validate_standards.sh
```

### Manual Checklist

**Before committing API changes:**
- [ ] IDs use `prefix_uuid` format (id_format_standards.md)
- [ ] Errors match standard structure (error_format_standards.md)
- [ ] Timestamps use ISO 8601 with Z (data_format_standards.md)
- [ ] Currency has 2 decimal places (data_format_standards.md)
- [ ] URLs use correct subdomains (url_standards.md)
- [ ] Pagination uses `page`/`per_page` parameters (api_design_standards.md)

---

## Cross-References

### Related Documentation

**Architecture:**
- [Entity Model](../architecture/007_entity_model.md) - Entities that have IDs
- [Data Flow](../architecture/004_data_flow.md) - How data formats are used

**Protocol:**
- [REST API Protocol](../protocol/002_rest_api_protocol.md) - API implementation using these standards
- [Error Handling Protocol](../protocol/021_error_handling_protocol.md) - Error handling implementation

**Implementation:**
- [iron_types crate](../../module/iron_types/readme.md) - ID type implementation
- [iron_control_schema crate](../../module/iron_control_schema/readme.md) - Database schema using IDs

---

## FAQ

### Q: What if I need a new ID type?

Add it to iron_types crate, update id_format_standards.md, and document in entity model.

Example: Adding `user_` prefix for user IDs.

### Q: What if an external API requires a different format?

Convert at the boundary. Internal code always uses Iron Cage standards.

```rust
// External API expects hyphenated IDs
fn send_to_external_api(id: &AgentId) {
    let external_format = id.as_str().replace('_', "-");
    // send to external API
}
```

### Q: What if standards conflict with external requirements?

1. Try to align external integration with standards
2. If impossible, convert at integration boundary
3. Document exception in standard document
4. Create wrapper type if extensive conversion needed

### Q: How do I propose a new standard?

1. Check if fits in existing standard document
2. If new category needed, create issue with proposed structure
3. Draft standard document following existing templates
4. Get team review before implementing

---

## Compliance Status

### Code Compliance

**Modules using iron_types v0.3.0 (ID standards):**
- ✅ iron_runtime_state
- ✅ iron_runtime
- ⚠️ iron_control_api (migration in progress)
- ⚠️ iron_dashboard (migration in progress)

**Modules following error standards:**
- ⚠️ iron_control_api (needs update)
- ❌ iron_dashboard (not implemented)

**Target:** 100% compliance by 2026-01-01

### Documentation Compliance

**Protocol docs following standards:**
- ✅ 002_rest_api_protocol.md
- ✅ 006_token_management_api.md
- ⚠️ 010_agents_api.md (needs review)

**Target:** 100% compliance by 2025-12-31

---

**Document Version:** 1.1.0
**Last Updated:** 2025-12-11
**Status:** Normative (must follow)
