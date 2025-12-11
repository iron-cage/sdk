# API Design Standards

**Purpose:** Define REST API conventions for pagination, sorting, filtering, versioning, and deprecation

**Scope:** All REST API endpoints across Iron Cage platform

**Status:** Normative (all APIs must follow these standards)

**Version:** 1.0.0

**Last Updated:** 2025-12-10

---

## TL;DR

```
Pagination:   GET /api/v1/agents?page=1&per_page=50
Sorting:      GET /api/v1/agents?sort=-created_at
Filtering:    GET /api/v1/agents?name=production (partial match)
Versioning:   /api/v1/ (URL-based)
Deprecation:  X-API-Deprecation: true, 6-month notice
```

**Key Principles:**
- Consistent pagination across all list endpoints
- Optional sorting with clear ascending/descending syntax
- Partial match filtering for text fields
- URL-based versioning for clarity
- Respectful deprecation with headers and documentation

---

## Pagination

### Standard

All list endpoints **MUST** support offset-based pagination with query parameters.

**Query Parameters:**
- `page` - Page number (1-indexed)
- `per_page` - Results per page

**Format:**
```
GET /api/v1/agents?page=1&per_page=50
GET /api/v1/providers?page=2&per_page=25
GET /api/v1/projects?page=1&per_page=100
```

### Response Format

All paginated responses **MUST** include a `pagination` object with metadata.

**Structure:**
```json
{
  "data": [
    {
      "id": "agent_550e8400-e29b-41d4-a716-446655440000",
      "name": "Production Agent"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 250,
    "total_pages": 5
  }
}
```

**Fields:**
- `page` (integer) - Current page number (1-indexed)
- `per_page` (integer) - Results per page
- `total` (integer) - Total number of results across all pages
- `total_pages` (integer) - Total number of pages

### Default Values

| Parameter | Default | Minimum | Maximum |
|-----------|---------|---------|---------|
| `page` | 1 | 1 | unlimited |
| `per_page` | 50 | 1 | 100 |

**Rationale:**
- Default `per_page=50` balances response size and round trips
- Maximum `per_page=100` prevents abuse and timeout issues
- Page numbers start at 1 (not 0) for human readability

### Applies To

All list endpoints:
- `GET /api/v1/agents`
- `GET /api/v1/providers`
- `GET /api/v1/projects`
- `GET /api/v1/api-tokens`
- `GET /api/v1/analytics/*` (all analytics endpoints)

### Examples

**First page with default size:**
```bash
GET /api/v1/agents
# Equivalent to: GET /api/v1/agents?page=1&per_page=50
```

**Specific page:**
```bash
GET /api/v1/agents?page=3&per_page=25
```

**Maximum page size:**
```bash
GET /api/v1/agents?page=1&per_page=100
```

**Response:**
```json
{
  "data": [
    {
      "id": "agent_550e8400-e29b-41d4-a716-446655440000",
      "name": "Production Agent 1",
      "budget": 100.00,
      "created_at": "2025-12-10T10:30:45Z"
    },
    {
      "id": "agent_660f9511-f3ab-52e5-b827-557766551111",
      "name": "Production Agent 2",
      "budget": 150.50,
      "created_at": "2025-12-10T09:15:22Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 123,
    "total_pages": 3
  }
}
```

### Error Cases

**Invalid page number:**
```json
GET /api/v1/agents?page=0

400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Page must be >= 1",
    "fields": {
      "page": "Must be >= 1"
    }
  }
}
```

**Invalid per_page:**
```json
GET /api/v1/agents?per_page=150

400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Per page must be between 1 and 100",
    "fields": {
      "per_page": "Must be between 1 and 100"
    }
  }
}
```

**Empty result set:**
```json
GET /api/v1/agents?page=10

200 OK
{
  "data": [],
  "pagination": {
    "page": 10,
    "per_page": 50,
    "total": 123,
    "total_pages": 3
  }
}
```

### Implementation Notes

**Database Query:**
```rust
// Rust example
let offset = (page - 1) * per_page;
sqlx::query("SELECT * FROM agents ORDER BY created_at DESC LIMIT ? OFFSET ?")
  .bind(per_page)
  .bind(offset)
  .fetch_all(&pool)
  .await?;
```

**Count Query:**
```rust
// Separate query for total count
let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents")
  .fetch_one(&pool)
  .await?;

let total_pages = (total as f64 / per_page as f64).ceil() as i64;
```

**Performance Consideration:**
- Count queries can be expensive for large tables
- Consider caching `total` for frequently accessed resources
- For large datasets (>10,000 records), consider cursor-based pagination in future versions

---

## Sorting

### Standard

List endpoints **SHOULD** support optional sorting with a `sort` query parameter.

**Format:**
```
GET /api/v1/agents?sort=name           # Ascending by name
GET /api/v1/agents?sort=-created_at    # Descending by created_at (newest first)
GET /api/v1/agents?sort=-budget        # Descending by budget (highest first)
```

**Syntax:**
- **Ascending:** `?sort=field_name`
- **Descending:** `?sort=-field_name` (prefix with `-`)

### Supported Sort Fields

| Endpoint | Sortable Fields | Default |
|----------|----------------|---------|
| `GET /api/v1/agents` | `name`, `budget`, `created_at`, `updated_at` | `-created_at` |
| `GET /api/v1/providers` | `name`, `created_at`, `updated_at` | `-created_at` |
| `GET /api/v1/projects` | `name`, `created_at`, `updated_at` | `-created_at` |
| `GET /api/v1/api-tokens` | `created_at`, `expires_at` | `-created_at` |

### Default Sort Order

When `sort` parameter is **not provided**, endpoints **MUST** use default sort order:

**Default:** `-created_at` (newest first)

**Rationale:** Most recent items are typically most relevant to users.

### Examples

**Sort by name (ascending):**
```bash
GET /api/v1/agents?sort=name

# Response: Agents sorted alphabetically A-Z
{
  "data": [
    {"name": "Agent Alpha", "created_at": "2025-12-10T10:00:00Z"},
    {"name": "Agent Beta", "created_at": "2025-12-09T14:30:00Z"},
    {"name": "Agent Gamma", "created_at": "2025-12-08T09:15:00Z"}
  ]
}
```

**Sort by budget (descending - highest first):**
```bash
GET /api/v1/agents?sort=-budget

# Response: Agents sorted by budget, highest to lowest
{
  "data": [
    {"name": "Production Agent", "budget": 500.00},
    {"name": "Test Agent", "budget": 100.00},
    {"name": "Dev Agent", "budget": 10.00}
  ]
}
```

**Sort by created_at (descending - newest first):**
```bash
GET /api/v1/agents?sort=-created_at

# Response: Newest agents first (default behavior)
{
  "data": [
    {"name": "Agent 3", "created_at": "2025-12-10T10:00:00Z"},
    {"name": "Agent 2", "created_at": "2025-12-09T14:30:00Z"},
    {"name": "Agent 1", "created_at": "2025-12-08T09:15:00Z"}
  ]
}
```

**Combine sorting with pagination:**
```bash
GET /api/v1/agents?sort=name&page=2&per_page=25

# Response: Page 2 of agents sorted alphabetically
```

### Error Cases

**Invalid sort field:**
```json
GET /api/v1/agents?sort=invalid_field

400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid sort field: invalid_field",
    "fields": {
      "sort": "Must be one of: name, budget, created_at, updated_at"
    }
  }
}
```

### Implementation Notes

**SQL Generation:**
```rust
// Rust example
let order_clause = match sort_param {
  Some("name") => "ORDER BY name ASC",
  Some("-name") => "ORDER BY name DESC",
  Some("budget") => "ORDER BY budget ASC",
  Some("-budget") => "ORDER BY budget DESC",
  Some("-created_at") | None => "ORDER BY created_at DESC", // Default
  _ => return Err(ValidationError::InvalidSortField),
};

let query = format!("SELECT * FROM agents {} LIMIT ? OFFSET ?", order_clause);
```

**Security:**
- **Never** interpolate sort field directly into SQL (SQL injection risk)
- Use allowlist of valid sort fields
- Validate sort parameter against allowlist before query

### Future Considerations

**Multiple sort fields (not in v1.0.0):**
```
# Hypothetical future syntax
GET /api/v1/agents?sort=budget,-created_at
# Sort by budget ascending, then created_at descending for ties
```

**Current limitation:** v1.0.0 supports single sort field only. Defer multi-field sorting to future version if needed.

---

## Filtering

### Standard

List endpoints **SHOULD** support filtering with resource-specific query parameters.

**Format:**
```
GET /api/v1/agents?name=production
GET /api/v1/agents?project_id=proj_550e8400-e29b-41d4-a716-446655440000
```

### Filter Types

| Filter Type | Behavior | Case Sensitive | Example |
|-------------|----------|----------------|---------|
| **Text (partial match)** | Contains substring | No | `?name=prod` matches "Production Agent" |
| **ID (exact match)** | Exact string match | Yes | `?project_id=proj_abc` |
| **Boolean** | Exact match | N/A | `?enabled=true` |

### Text Field Filtering

Text fields use **partial match** (contains) with **case-insensitive** comparison.

**Examples:**
```bash
# Match agents with "production" anywhere in name
GET /api/v1/agents?name=production

# Matches:
# - "Production Agent"
# - "My Production Agent"
# - "PRODUCTION-001"
# - "Test production environment"

# Does NOT match:
# - "Prod Agent" (partial word doesn't match)
```

**Implementation:**
```sql
SELECT * FROM agents WHERE LOWER(name) LIKE LOWER('%production%')
```

### ID Field Filtering

ID fields use **exact match** with **case-sensitive** comparison.

**Examples:**
```bash
# Filter agents by project ID
GET /api/v1/agents?project_id=proj_550e8400-e29b-41d4-a716-446655440000

# Filter by specific provider (if endpoint supports it)
GET /api/v1/analytics/requests?provider_id=ip_660f9511-f3ab-52e5-b827-557766551111
```

### Boolean Field Filtering

Boolean fields use **exact match** with JSON boolean values.

**Examples:**
```bash
# Filter enabled agents only (hypothetical)
GET /api/v1/agents?enabled=true

# Valid values: true, false
# Invalid: 1, 0, "true", "false"
```

### Supported Filters by Endpoint

| Endpoint | Supported Filters |
|----------|-------------------|
| `GET /api/v1/agents` | `name` (text), `project_id` (ID) |
| `GET /api/v1/providers` | `name` (text) |
| `GET /api/v1/projects` | `name` (text) |
| `GET /api/v1/analytics/requests` | `agent_id` (ID), `provider_id` (ID) |

### Combining Filters

Multiple filters use **AND** logic.

**Example:**
```bash
GET /api/v1/agents?name=production&project_id=proj_550e8400-e29b-41d4-a716-446655440000

# Returns: Agents matching BOTH:
# - Name contains "production"
# - AND project_id equals "proj_550e8400-e29b-41d4-a716-446655440000"
```

### Combining with Pagination and Sorting

Filters work seamlessly with pagination and sorting.

**Example:**
```bash
GET /api/v1/agents?name=production&sort=-budget&page=1&per_page=25

# Returns: Page 1 of agents:
# - Filtered: name contains "production"
# - Sorted: by budget descending (highest first)
# - Paginated: 25 results per page
```

### Empty Results

When no resources match filters, return empty `data` array with pagination metadata.

**Example:**
```json
GET /api/v1/agents?name=nonexistent

200 OK
{
  "data": [],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 0,
    "total_pages": 0
  }
}
```

### Error Cases

**Invalid filter value:**
```json
GET /api/v1/agents?project_id=invalid

400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid project ID format",
    "fields": {
      "project_id": "Must match pattern: proj_<uuid>"
    }
  }
}
```

### Implementation Notes

**SQL Generation:**
```rust
// Rust example - build WHERE clauses dynamically
let mut conditions = vec![];
let mut params = vec![];

if let Some(name) = filters.name {
  conditions.push("LOWER(name) LIKE LOWER(?)");
  params.push(format!("%{}%", name));
}

if let Some(project_id) = filters.project_id {
  conditions.push("project_id = ?");
  params.push(project_id);
}

let where_clause = if conditions.is_empty() {
  String::new()
} else {
  format!("WHERE {}", conditions.join(" AND "))
};

let query = format!("SELECT * FROM agents {} ORDER BY created_at DESC", where_clause);
```

**Security:**
- Validate all filter values before query
- Use parameterized queries (prevent SQL injection)
- Validate ID format against standard (see ID Format Standards)

### Future Considerations

**Advanced filtering (not in v1.0.0):**
- Range filters: `?budget_min=100&budget_max=500`
- OR logic: `?name=prod|test`
- Negation: `?name!=production`

**Current scope:** v1.0.0 supports simple equality and partial match only. Defer advanced filtering to future versions.

---

## API Versioning

### Standard

All API endpoints **MUST** use URL-based versioning with `/v{N}/` path segment.

**Format:**
```
https://api.ironcage.ai/v1/agents
https://api.ironcage.ai/v1/providers
https://api.ironcage.ai/v2/agents  # Hypothetical future version
```

**Version Format:** `/v{N}/` where `N` is an integer (1, 2, 3, ...)

### Current Version

**Production API:** `/v1/`

All current endpoints use v1:
- `GET /api/v1/agents`
- `POST /api/v1/agents`
- `GET /api/v1/providers`
- `GET /api/v1/analytics/spending`

### Rationale: Why URL Versioning?

**Advantages:**
1. **Clarity** - Version visible in URL
2. **Simplicity** - No custom headers needed
3. **Caching** - HTTP caches handle versions correctly
4. **Debugging** - Easy to test different versions in browser/curl
5. **Consistency** - Already using `/api/v1/` in current implementation

**Alternative Considered:**
- Header versioning (`Accept: application/vnd.iron.v1+json`) - More complex, harder to debug
- No versioning - Forces backward compatibility for all changes (too restrictive)

### Version Lifecycle

**v1 (Pilot - Current):**
- Status: Stable
- Maintained: Yes
- Breaking changes: No
- Deprecation: No plans

**v2+ (Post-Pilot - Future):**
- Created when: Breaking changes required (see Breaking Changes section)
- Backward compatibility: Not required (v1 remains available)
- Migration period: Minimum 6 months (see Deprecation Policy)

### Breaking vs Non-Breaking Changes

**Breaking changes (require new version):**
- ❌ Removing endpoint
- ❌ Removing request field
- ❌ Removing response field
- ❌ Changing field type (e.g., string → number)
- ❌ Changing URL structure
- ❌ Changing authentication method
- ❌ Changing HTTP status codes for existing scenarios

**Non-breaking changes (same version):**
- ✅ Adding new endpoint
- ✅ Adding optional request field
- ✅ Adding response field
- ✅ Adding new error code
- ✅ Changing internal implementation (same external behavior)
- ✅ Performance improvements

### Examples

**Non-breaking change (v1 continues):**
```
# Before
GET /api/v1/agents
Response: { "id": "agent_abc", "name": "Test", "budget": 100.00 }

# After (new field added)
GET /api/v1/agents
Response: { "id": "agent_abc", "name": "Test", "budget": 100.00, "tags": [] }

# No version increment needed - new field is additive
```

**Breaking change (requires v2):**
```
# v1 (old - still available)
GET /api/v1/agents
Response: { "id": "agent-abc-123", "name": "Test", "budget": 100.00 }

# v2 (new - breaking ID format change)
GET /api/v2/agents
Response: { "id": "agent_550e8400-e29b-41d4-a716-446655440000", "name": "Test", "budget": 100.00 }

# Both versions coexist during migration period
```

### Version Selection

**Default version:** v1 (current)

**Client responsibilities:**
- Explicitly include version in all requests
- Do not hardcode version in client libraries (make configurable)
- Monitor deprecation headers (see Deprecation Policy)

**Server responsibilities:**
- Support all non-deprecated versions
- Return deprecation headers for sunset versions
- Maintain compatibility within version (no breaking changes)

### Implementation Notes

**Routing (Rust example):**
```rust
// axum router
let v1_routes = Router::new()
  .route("/agents", get(list_agents_v1).post(create_agent_v1))
  .route("/agents/:id", get(get_agent_v1).put(update_agent_v1).delete(delete_agent_v1))
  .route("/providers", get(list_providers_v1).post(create_provider_v1));

// Future v2 routes
let v2_routes = Router::new()
  .route("/agents", get(list_agents_v2).post(create_agent_v2));

let app = Router::new()
  .nest("/api/v1", v1_routes)
  .nest("/api/v2", v2_routes); // Future
```

### Documentation

All API documentation **MUST** specify version:

**Example:**
```markdown
## List Agents

**Endpoint:** `GET /api/v1/agents`
**Version:** v1
**Status:** Stable
```

---

## Deprecation Policy

### Standard

When API changes require deprecating endpoints or versions, follow deprecation policy:

1. **Announce deprecation** - 6 months notice minimum
2. **Add deprecation headers** - All responses from deprecated endpoints
3. **Update documentation** - Mark endpoint as deprecated with sunset date
4. **Maintain functionality** - Deprecated endpoints remain fully functional until sunset
5. **Sunset date** - After 6 months, endpoint returns 410 Gone

### Deprecation Headers

Deprecated endpoints **MUST** include headers in all responses.

**Headers:**
```
X-API-Deprecation: true
X-API-Sunset: 2026-06-10T00:00:00Z
```

**Example Response:**
```
HTTP/1.1 200 OK
Content-Type: application/json
X-API-Deprecation: true
X-API-Sunset: 2026-06-10T00:00:00Z

{
  "id": "agent_550e8400-e29b-41d4-a716-446655440000",
  "name": "Production Agent"
}
```

### Timeline

**Month 0 (Announcement):**
- Add deprecation headers to deprecated endpoint
- Update API documentation with deprecation notice
- Send email notification to API users (if contact info available)
- Post announcement in changelog/release notes

**Month 1-5 (Grace Period):**
- Continue serving deprecated endpoint normally
- Monitor usage metrics
- Provide migration guide and support

**Month 6 (Sunset):**
- Endpoint returns `410 Gone` status
- Response includes migration information

### Sunset Response

After sunset date, endpoint returns `410 Gone` with migration information.

**Example:**
```
HTTP/1.1 410 Gone
Content-Type: application/json

{
  "error": {
    "code": "ENDPOINT_DEPRECATED",
    "message": "This endpoint was deprecated on 2025-12-10 and sunset on 2026-06-10",
    "migration": {
      "new_endpoint": "/api/v2/agents",
      "documentation": "https://docs.ironcage.ai/api/v2/agents",
      "sunset_date": "2026-06-10T00:00:00Z"
    }
  }
}
```

### Migration Guide Requirements

All deprecations **MUST** include migration guide in documentation:

**Template:**
```markdown
## Migration from v1 to v2

**Deprecated:** GET /api/v1/agents
**Replacement:** GET /api/v2/agents
**Sunset Date:** 2026-06-10

**Changes:**
- ID format changed from `agent-abc` to `agent_550e8400-e29b-41d4-a716-446655440000`
- Response includes new `tags` field

**Migration Steps:**
1. Update client to use `/api/v2/agents` endpoint
2. Update ID parsing to handle new format
3. Test integration with v2 endpoint
4. Deploy before sunset date: 2026-06-10
```

### Examples

**Endpoint deprecation (hypothetical):**
```
# Scenario: Deprecating old agent creation endpoint

# 2025-12-10: Announcement
POST /api/v1/agents/create
Response headers:
  X-API-Deprecation: true
  X-API-Sunset: 2026-06-10T00:00:00Z

Documentation:
  ⚠️ DEPRECATED: Use POST /api/v1/agents instead
  This endpoint will be removed on 2026-06-10

# 2026-06-10: Sunset
POST /api/v1/agents/create
Response: 410 Gone
{
  "error": {
    "code": "ENDPOINT_DEPRECATED",
    "message": "Use POST /api/v1/agents instead",
    "migration": {
      "new_endpoint": "/api/v1/agents",
      "documentation": "https://docs.ironcage.ai/api/v1/agents"
    }
  }
}
```

**Version deprecation (hypothetical):**
```
# Scenario: Deprecating v1 API entirely

# 2025-12-10: Announcement
All v1 endpoints receive deprecation headers:
  X-API-Deprecation: true
  X-API-Sunset: 2026-06-10T00:00:00Z

Documentation:
  ⚠️ API v1 DEPRECATED
  - Sunset date: 2026-06-10
  - Migration guide: https://docs.ironcage.ai/migration/v1-to-v2

# 2026-06-10: Sunset
All v1 endpoints return 410 Gone with migration information
```

### Client Responsibilities

API clients **SHOULD**:
1. Check for `X-API-Deprecation` header in responses
2. Log deprecation warnings
3. Monitor `X-API-Sunset` date
4. Plan migration before sunset date

### Implementation Notes

**Middleware (Rust example):**
```rust
// axum middleware
async fn deprecation_middleware(
  req: Request,
  next: Next,
) -> Response {
  let mut response = next.run(req).await;

  // Check if endpoint is deprecated
  if is_deprecated_endpoint(&req.uri()) {
    response.headers_mut().insert(
      "X-API-Deprecation",
      "true".parse().unwrap()
    );
    response.headers_mut().insert(
      "X-API-Sunset",
      "2026-06-10T00:00:00Z".parse().unwrap()
    );
  }

  response
}
```

---

## Cross-References

### Related Standards

- [ID Format Standards](id_format_standards.md) - Entity ID format used in filtering
- [Error Format Standards](error_format_standards.md) - Error responses for validation failures
- [Data Format Standards](data_format_standards.md) - Timestamp format in pagination responses
- [URL Standards](url_standards.md) - API base URLs and domain structure

### Related Protocol Documents

- `002_rest_api_protocol.md` - REST API implementation using these standards
- `010_agents_api.md` - Agent endpoints using pagination/sorting/filtering
- `011_providers_api.md` - Provider endpoints using these standards

### Related Code

- `iron_control_api` - REST API implementation
- `iron_control_schema` - Database queries for pagination/filtering/sorting

---

## Compliance Checklist

**Before deploying new list endpoint:**
- [ ] Implements pagination with `page` and `per_page` parameters
- [ ] Returns `pagination` object in response
- [ ] Default `per_page=50`, max `per_page=100`
- [ ] Supports `sort` parameter with ascending/descending syntax
- [ ] Default sort order: `-created_at` (newest first)
- [ ] Implements appropriate filters for resource type
- [ ] Text filters use partial match, case-insensitive
- [ ] ID filters use exact match
- [ ] Combines pagination, sorting, filtering correctly
- [ ] Validates all query parameters
- [ ] Returns standard error format for invalid parameters

**Before making breaking change:**
- [ ] Determined if change is truly breaking (see Breaking vs Non-Breaking)
- [ ] Created new API version (e.g., v2) if breaking
- [ ] Added deprecation headers to old version
- [ ] Set sunset date (minimum 6 months from deprecation)
- [ ] Created migration guide in documentation
- [ ] Notified API users via email/changelog

---

## FAQ

### Q: Why offset pagination instead of cursor pagination?

**A:** Offset pagination (`?page=1&per_page=50`) is simpler to implement and sufficient for Pilot scale. Cursor pagination is more performant for very large datasets but adds complexity. Can migrate to cursor pagination post-Pilot if needed.

### Q: Can I paginate without total count?

**A:** No. All paginated responses must include `total` and `total_pages` in pagination object. This is required for UI pagination controls. If count query is expensive, consider caching the total.

### Q: What if I need to sort by multiple fields?

**A:** v1.0.0 supports single sort field only. For tied records, database default order (typically primary key) is used. Multi-field sorting can be added in future version if needed.

### Q: Should filters use exact match or partial match?

**A:** Depends on field type:
- **Text fields** (name, description): Partial match (contains), case-insensitive
- **ID fields** (agent_id, project_id): Exact match, case-sensitive
- **Boolean fields**: Exact match

### Q: How do I know if a change is breaking?

**A:** Use the checklist in Breaking vs Non-Breaking Changes section. General rule:
- **Breaking:** Removes or changes existing behavior that clients depend on
- **Non-breaking:** Adds new behavior without affecting existing clients

### Q: What if I need to deprecate an endpoint before 6 months?

**A:** 6 months is the **minimum** notice period. Shorter periods are only acceptable for security vulnerabilities or critical bugs. Document exception in deprecation notice and communicate urgency to users.

### Q: Do I need deprecation headers for internal APIs?

**A:** Yes. All APIs (internal and external) should follow deprecation policy. Internal teams benefit from structured deprecation just as external users do.

### Q: Can I use header versioning instead of URL versioning?

**A:** No. URL versioning (`/v1/`) is the standard for Iron Cage APIs. Header versioning adds complexity and is harder to debug. Stick with URL versioning for consistency.

---

**Document Version:** 1.0.0

**Last Updated:** 2025-12-10

**Status:** Normative (must follow)

**Next Review:** 2026-06-10 (or when first breaking change is needed)
