# ID Format Standards

**Purpose:** Define canonical identifier format for all Iron Cage entities

**Responsibility:** Specify ID structure, validation rules, and migration from legacy formats

**Status:** Normative (all new IDs MUST follow this standard)

**Version:** 0.3.0 (aligns with iron_types v0.3.0)

---

## TL;DR

All entity IDs use underscore-separated format: `prefix_uuid`

```
✅ agent_550e8400-e29b-41d4-a716-446655440000
❌ agent-550e8400-e29b-41d4-a716-446655440000  (legacy)
❌ 550e8400-e29b-41d4-a716-446655440000        (no prefix)
```

**Implementation:** Use iron_types crate (`AgentId`, `ProviderId`, etc.)

---

## Canonical Format

### Structure

```
<prefix>_<uuid>
```

**Components:**
1. **Prefix**: Type-specific identifier (lowercase, ASCII letters only)
2. **Separator**: Single underscore (`_`)
3. **UUID**: Standard UUID v4 (8-4-4-4-12 hexadecimal format)

**Regex pattern:**
```regex
^[a-z]+_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$
```

**Characteristics:**
- **Fixed length**: Prefix length + 1 (underscore) + 36 (UUID) characters
- **Case sensitivity**: Lowercase only (both prefix and UUID)
- **Character set**: `[a-z_0-9-]` (alphanumeric + underscore + hyphen)
- **Collision resistance**: 2^122 possible UUIDs per prefix (practically infinite)

---

## Supported Entity Types

### Production Entity IDs

| Entity Type | Prefix | Example | Use Case | Since |
|-------------|--------|---------|----------|-------|
| **Agent** | `agent_` | `agent_550e8400-e29b-41d4-a716-446655440000` | Runtime AI agents | v0.1.0 |
| **Provider** | `ip_` | `ip_660f9511-f3ab-52e5-b827-557766551111` | LLM providers (OpenAI, Anthropic) | v0.1.0 |
| **Project** | `proj_` | `proj_770g0622-g4bc-63f6-c938-668877662222` | User projects | v0.1.0 |
| **API Token** | `at_` | `at_880h1733-h5cd-74g7-d049-779988773333` | API authentication tokens | v0.2.0 |
| **IC Token** | `ic_` | `ic_990i2844-i6de-85h8-e15a-88aa99884444` | Iron Cage runtime tokens | v0.1.0 |
| **Budget Request** | `breq_` | `breq_aa0j3955-j7ef-96i9-f26b-99bb00995555` | Budget allocation requests | v0.2.0 |
| **Lease** | `lease_` | `lease_bb0k4066-k8fg-07j0-g37c-00cc11006666` | Budget leases | v0.2.0 |
| **Request** | `req_` | `req_cc0l5177-l9gh-18k1-h48d-11dd22117777` | Generic API requests | v0.1.0 |

### Reserved Prefixes

These prefixes are reserved for future use:

| Prefix | Reserved For | Planned Version |
|--------|--------------|-----------------|
| `user_` | User accounts | POST-PILOT |
| `org_` | Organizations | POST-PILOT |
| `audit_` | Audit log entries | POST-PILOT |
| `webhook_` | Webhook subscriptions | POST-PILOT |

**Adding new prefixes:**
1. Propose in GitHub issue with rationale
2. Add to iron_types crate as new ID type
3. Update this document with new row
4. Update entity model documentation

---

## Design Rationale

### Why Underscore (`_`) Not Hyphen (`-`)?

**1. Programming Language Compatibility**

Underscores are valid identifier characters in most languages:

```python
# Python: Can copy-paste ID directly
agent_550e8400 = get_agent("agent_550e8400-e29b-41d4-a716-446655440000")
```

```rust
// Rust: Valid variable name prefix
let agent_550e8400 = Agent::load("agent_550e8400-e29b-41d4-a716-446655440000");
```

```javascript
// JavaScript: Valid identifier
const agent_550e8400 = await getAgent("agent_550e8400-e29b-41d4-a716-446655440000");
```

Hyphens require escaping or quoting in most contexts.

**2. Database Conventions**

PostgreSQL and MySQL naming standards prefer snake_case:

```sql
-- Standard naming (consistent with ID format)
CREATE TABLE agent_budgets (
    agent_id VARCHAR(48) PRIMARY KEY,  -- agent_<uuid>
    budget_usd DECIMAL(10, 2)
);
```

**3. JSON Style Guides**

Google and Airbnb JSON style guides recommend snake_case:

```json
{
  "agent_id": "agent_550e8400-e29b-41d4-a716-446655440000",
  "project_id": "proj_770g0622-g4bc-63f6-c938-668877662222"
}
```

**4. Consistency with Existing IDs**

`ic_` prefix already uses underscore, established in v0.1.0.

**5. Industry Standards**

Major platforms use underscore-separated prefixes:
- Stripe: `sk_live_`, `pk_test_`
- GitHub: `ghp_`, `gho_`
- AWS: `arn:aws:`, uses underscores in resource IDs

**6. URL Safety**

Both formats are URL-safe (RFC 3986 unreserved characters). No encoding needed:

```
✅ https://api.ironcage.ai/v1/agents/agent_550e8400-e29b-41d4-a716-446655440000
✅ https://api.ironcage.ai/v1/agents/agent-550e8400-e29b-41d4-a716-446655440000
```

---

## Implementation

### Using iron_types Crate

**Recommended approach:** Use type-safe ID wrappers from iron_types.

#### Generating New IDs

```rust
use iron_types::{AgentId, ProviderId};

// Generate new agent ID
let agent_id = AgentId::generate();
println!("{}", agent_id.as_str());
// Output: agent_550e8400-e29b-41d4-a716-446655440000

// Generate new provider ID
let provider_id = ProviderId::generate();
println!("{}", provider_id.as_str());
// Output: ip_660f9511-f3ab-52e5-b827-557766551111
```

#### Parsing Existing IDs

**Strict validation (current format only):**

```rust
use iron_types::AgentId;

// Parse current format
let id = AgentId::parse("agent_550e8400-e29b-41d4-a716-446655440000")?;

// Fails on legacy format
let bad = AgentId::parse("agent-550e8400-e29b-41d4-a716-446655440000");
assert!(bad.is_err());  // IdError::InvalidFormat
```

**Flexible validation (accepts legacy format):**

```rust
use iron_types::AgentId;

// Parse current format
let id1 = AgentId::parse_flexible("agent_550e8400-e29b-41d4-a716-446655440000")?;

// Parse legacy format (auto-normalized to current format)
let id2 = AgentId::parse_flexible("agent-550e8400-e29b-41d4-a716-446655440000")?;

// Both produce same normalized ID
assert_eq!(id1.as_str(), id2.as_str());
assert_eq!(id1.as_str(), "agent_550e8400-e29b-41d4-a716-446655440000");
```

#### Type Safety

```rust
use iron_types::{AgentId, ProviderId};

fn start_agent(id: &AgentId) {
    println!("Starting agent: {}", id.as_str());
}

fn configure_provider(id: &ProviderId) {
    println!("Configuring provider: {}", id.as_str());
}

let agent = AgentId::generate();
let provider = ProviderId::generate();

start_agent(&agent);           // ✅ OK
configure_provider(&provider); // ✅ OK

// Compile-time error: mismatched types
// start_agent(&provider);     // ❌ ERROR: expected AgentId, got ProviderId
```

#### Serialization (JSON, Database)

```rust
use iron_types::AgentId;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Agent {
    id: AgentId,  // Serializes as string
    name: String,
}

let agent = Agent {
    id: AgentId::generate(),
    name: "Production Agent".to_string(),
};

let json = serde_json::to_string(&agent)?;
// {"id":"agent_550e8400-e29b-41d4-a716-446655440000","name":"Production Agent"}

let parsed: Agent = serde_json::from_str(&json)?;
// ID validated during deserialization
```

---

## Migration from Legacy Format

### Legacy Format

Prior to v0.2.0, IDs used hyphen separator:

```
agent-550e8400-e29b-41d4-a716-446655440000  (legacy)
```

### Migration Strategy

**Phase 1: Accept Both Formats (CURRENT)**

Use `parse_flexible()` to accept both formats during migration:

```rust
// Database read (may contain legacy IDs)
fn load_agent_from_db(id_str: &str) -> Result<Agent, Error> {
    let id = AgentId::parse_flexible(id_str)?;  // Accept both formats
    // ... fetch agent ...
}
```

**Phase 2: Database Migration (IN PROGRESS)**

See `/dev/scripts/migrate_ids_to_underscore.sql` for complete migration script.

Key steps:
1. Create backup tables
2. Update primary keys (`agent-` → `agent_`)
3. Update foreign key references
4. Validate data integrity (no hyphenated IDs remain)
5. Add CHECK constraints for underscore format

**Phase 3: Strict Validation (TARGET: 2026-06-10)**

After all data migrated, use `parse()` (strict validation):

```rust
// Post-migration: Only accept current format
let id = AgentId::parse(id_str)?;  // Fails on legacy format
```

### Compatibility Period

**Duration:** Until all data migrated (estimated 2026-06-10)

**During migration:**
- ✅ `parse_flexible()` accepts both formats
- ✅ `generate()` always produces current format
- ✅ Database contains mixed formats
- ✅ API accepts both formats in requests
- ✅ API always returns current format in responses

**After migration:**
- ✅ `parse()` only accepts current format
- ✅ Database contains only current format
- ❌ API rejects legacy format (400 Bad Request)

---

## Validation Rules

### Format Validation

**Required checks:**
1. **Prefix validation**: Must be registered prefix (see table above)
2. **Separator validation**: Exactly one underscore between prefix and UUID
3. **UUID validation**: Valid UUID v4 format (8-4-4-4-12 hexadecimal)
4. **Case validation**: Lowercase only (both prefix and UUID hex digits)
5. **Length validation**: Total length = prefix length + 1 + 36

**Regex validation:**
```regex
^(agent|ip|proj|at|ic|breq|lease|req)_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$
```

**Example invalid IDs:**

```rust
// Invalid prefix
AgentId::parse("invalid_550e8400-e29b-41d4-a716-446655440000")  // Error

// Missing underscore
AgentId::parse("agent550e8400-e29b-41d4-a716-446655440000")     // Error

// Invalid UUID format
AgentId::parse("agent_not-a-valid-uuid")                         // Error

// Uppercase UUID
AgentId::parse("agent_550E8400-E29B-41D4-A716-446655440000")     // Error

// Wrong separator (hyphen)
AgentId::parse("agent-550e8400-e29b-41d4-a716-446655440000")     // Error (use parse_flexible)
```

### Security Validation

**Injection prevention:**

IDs are validated against UUID format, preventing injection attacks:

```rust
// SQL injection attempt blocked
AgentId::parse("agent_'; DROP TABLE users; --").unwrap();  // Error: InvalidFormat

// Path traversal attempt blocked
AgentId::parse("agent_../../etc/passwd").unwrap();         // Error: InvalidFormat

// XSS attempt blocked
AgentId::parse("agent_<script>alert(1)</script>").unwrap(); // Error: InvalidFormat
```

**Type safety:**

Compile-time type checking prevents mixing different ID types:

```rust
fn delete_agent(id: &AgentId) { /* ... */ }

let provider_id = ProviderId::generate();
delete_agent(&provider_id);  // ❌ Compile error: expected AgentId, found ProviderId
```

---

## Performance Characteristics

### Benchmarks (iron_types v0.3.0)

| Operation | Time | Allocations | Throughput |
|-----------|------|-------------|------------|
| `generate()` | ~487 ns | 1 (Arc) | 2.1M ops/sec |
| `parse()` | ~271 ns | 0 | 3.7M ops/sec |
| `parse_flexible()` | ~500 ns | 0-1 | 2.0M ops/sec |
| `as_str()` | ~2.36 ns | 0 | 424M ops/sec |
| `clone()` | ~23.2 ns | 0 (Arc clone) | 43M ops/sec |
| `to_string()` | ~128 ns | 1 | 7.8M ops/sec |
| `serialize()` | ~308 ns | 1 | 3.2M ops/sec |
| `deserialize()` | ~485 ns | 1 | 2.1M ops/sec |

**Production Impact:**

At 1M requests/day:
- ID generation overhead: <0.001% CPU
- ID parsing overhead: <0.0006% CPU
- Memory overhead: 67% smaller than String (Arc-based)

**REST API overhead:**

Single ID operation (parse + validate): ~271 ns = 0.00027 ms = 0.04% of typical 500ms request.

---

## Database Integration

### Schema Definition

**PostgreSQL:**

```sql
CREATE TABLE agents (
    id VARCHAR(48) PRIMARY KEY,  -- agent_<uuid> = 6 + 1 + 36 = 43 chars (use 48 for safety)
    name VARCHAR(255) NOT NULL,
    budget_usd DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT id_format CHECK (id ~ '^agent_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
);

CREATE TABLE providers (
    id VARCHAR(48) PRIMARY KEY,  -- ip_<uuid> = 3 + 1 + 36 = 40 chars
    name VARCHAR(255) NOT NULL,
    endpoint_url VARCHAR(512) NOT NULL,
    CONSTRAINT id_format CHECK (id ~ '^ip_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
);

CREATE TABLE agent_providers (
    agent_id VARCHAR(48) REFERENCES agents(id) ON DELETE CASCADE,
    provider_id VARCHAR(48) REFERENCES providers(id) ON DELETE CASCADE,
    PRIMARY KEY (agent_id, provider_id)
);
```

**Column sizing:**
- Minimum: `prefix length + 1 + 36`
- Recommended: `48` (accommodates longest prefix + UUID + future growth)

### Indexing

**Primary key index:** Automatic (B-tree)

**Foreign key index:**

```sql
-- Recommended for foreign key lookups
CREATE INDEX idx_agent_providers_agent ON agent_providers(agent_id);
CREATE INDEX idx_agent_providers_provider ON agent_providers(provider_id);
```

**Prefix search (if needed):**

```sql
-- Efficient prefix search (uses index)
SELECT * FROM agents WHERE id LIKE 'agent_%';
```

---

## REST API Usage

### Request Examples

**Create agent:**

```bash
POST /api/v1/agents
Content-Type: application/json

{
  "name": "Production Agent",
  "budget": 100.00,
  "providers": [
    "ip_660f9511-f3ab-52e5-b827-557766551111",
    "ip_770g0622-g4bc-63f6-c938-668877662222"
  ]
}

Response: 201 Created
{
  "id": "agent_550e8400-e29b-41d4-a716-446655440000",  # Generated server-side
  "name": "Production Agent",
  "budget": 100.00,
  "providers": [
    "ip_660f9511-f3ab-52e5-b827-557766551111",
    "ip_770g0622-g4bc-63f6-c938-668877662222"
  ],
  "created_at": "2025-12-10T10:30:45Z"
}
```

**Get agent:**

```bash
GET /api/v1/agents/agent_550e8400-e29b-41d4-a716-446655440000

Response: 200 OK
{
  "id": "agent_550e8400-e29b-41d4-a716-446655440000",
  "name": "Production Agent",
  "budget": 100.00,
  "spent": 45.75,
  "created_at": "2025-12-10T10:30:45Z"
}
```

**Error on invalid ID:**

```bash
GET /api/v1/agents/invalid-id

Response: 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid agent ID format",
    "field": "id",
    "details": "Expected format: agent_<uuid>"
  }
}
```

### URL Parameters

IDs are URL-safe (no encoding needed):

```
GET /api/v1/agents/agent_550e8400-e29b-41d4-a716-446655440000
                    ↑ No URL encoding needed

# Both underscore and hyphen are RFC 3986 unreserved characters
```

### Example Format Guidance

**Documentation Examples: When to Use Full UUID vs Short Format**

Iron Cage documentation uses two valid ID formats in examples:

**1. Full UUID Format (Format Demonstration)**

```
agent_550e8400-e29b-41d4-a716-446655440000
```

**Use when:**
- Demonstrating complete ID format specification
- Showing UUID structure and validation rules
- Technical documentation about ID format itself
- Examples in this standards document

**Example:**
```markdown
Agent IDs follow the format `agent_<uuid>` where UUID is RFC 4122 compliant:
`agent_550e8400-e29b-41d4-a716-446655440000`
```

**2. Short Format (Documentation Readability)**

```
agent_abc123
```

**Use when:**
- Protocol documentation and API examples
- Specification requirements and acceptance criteria
- README files and tutorials
- Examples where readability is more important than format demonstration

**Example:**
```json
{
  "id": "agent_abc123",
  "name": "Production Agent",
  "budget": 100.00
}
```

**Both formats are valid** per this standard. The short format still follows the `prefix_<identifier>` structure, just with a shorter identifier for readability.

**Consistency Guidelines:**

1. **Within a single document:** Use one format consistently (prefer short for protocols, full for standards)
2. **Cross-document:** Can vary by document type (protocols use short, standards use full)
3. **Canonical examples:** Use short format for recognizable patterns (see [Canonical Examples](canonical_examples.md))

**Implementation Note:**

These are **documentation conventions only**. Implementation code always uses real UUIDs generated by iron_types crate:

```rust
// Production code ALWAYS uses real UUIDs
let agent_id = AgentId::new();  // Generates: agent_<real-uuid-v4>

// Never use short format in implementation
let fake_id = AgentId::parse("agent_abc123");  // ❌ Invalid in production
```

---

## CLI Usage

### Command Examples

**Create agent:**

```bash
# Generate ID server-side (recommended)
iron agents create --name "Production Agent" --budget 100.00

# Output
Agent created: agent_550e8400-e29b-41d4-a716-446655440000
```

**Get agent:**

```bash
iron agents get agent_550e8400-e29b-41d4-a716-446655440000
```

**List agents:**

```bash
iron agents list

# Output (table format)
ID                                            NAME               BUDGET    SPENT
agent_550e8400-e29b-41d4-a716-446655440000   Production Agent   $100.00   $45.75
agent_660f9511-f3ab-52e5-b827-557766551111   Test Agent         $50.00    $12.30
```

### Shell Completion

IDs use valid shell identifier characters (no quoting needed):

```bash
# No quotes needed (underscore is shell-safe)
AGENT_ID=agent_550e8400-e29b-41d4-a716-446655440000
iron agents get $AGENT_ID
```

---

## Testing

### Test Fixtures

**Deterministic IDs for testing:**

```rust
#[cfg(test)]
use iron_types::AgentId;

// Sequential IDs for predictable tests
let id1 = AgentId::test_fixture(1);  // agent_00000000-0000-0000-0000-000000000001
let id2 = AgentId::test_fixture(2);  // agent_00000000-0000-0000-0000-000000000002

// From known UUID
let uuid = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
let id = AgentId::from_uuid(uuid);   // agent_550e8400-e29b-41d4-a716-446655440000
```

### Validation Tests

```rust
#[cfg(test)]
mod tests {
    use iron_types::AgentId;

    #[test]
    fn test_valid_id() {
        let id = AgentId::parse("agent_550e8400-e29b-41d4-a716-446655440000");
        assert!(id.is_ok());
    }

    #[test]
    fn test_invalid_prefix() {
        let id = AgentId::parse("invalid_550e8400-e29b-41d4-a716-446655440000");
        assert!(id.is_err());
    }

    #[test]
    fn test_legacy_format_rejected() {
        let id = AgentId::parse("agent-550e8400-e29b-41d4-a716-446655440000");
        assert!(id.is_err());
    }

    #[test]
    fn test_legacy_format_flexible() {
        let id = AgentId::parse_flexible("agent-550e8400-e29b-41d4-a716-446655440000");
        assert!(id.is_ok());
        assert_eq!(id.unwrap().as_str(), "agent_550e8400-e29b-41d4-a716-446655440000");
    }
}
```

---

## External Integration

### Legacy System Compatibility

If external system requires hyphenated format, convert at boundary:

```rust
use iron_types::AgentId;

fn send_to_legacy_api(id: &AgentId) {
    // Convert to legacy format for external API
    let legacy_format = id.as_str().replace('_', "-");

    // POST to external API with legacy format
    client.post("https://legacy-api.example.com/agents")
        .json(&json!({ "agent_id": legacy_format }))
        .send()?;
}

fn receive_from_legacy_api(legacy_id: &str) -> Result<AgentId, Error> {
    // Accept legacy format from external API
    AgentId::parse_flexible(legacy_id)
}
```

### API Gateways

Configure API gateway to accept both formats during migration:

```nginx
# Nginx rewrite rule (if needed)
location /api/v1/ {
    # Rewrite legacy format to current format
    rewrite ^/api/v1/(agents|providers)/([a-z]+)-([0-9a-f-]+)$ /api/v1/$1/$2_$3 permanent;
    proxy_pass http://backend;
}
```

---

## Troubleshooting

### Common Errors

**Error: "Invalid ID format"**

```rust
AgentId::parse("agent-550e8400-e29b-41d4-a716-446655440000")
// Error: InvalidFormat (hyphen separator)

// Fix: Use parse_flexible() during migration
AgentId::parse_flexible("agent-550e8400-e29b-41d4-a716-446655440000")  // OK
```

**Error: "Invalid prefix"**

```rust
AgentId::parse("agnt_550e8400-e29b-41d4-a716-446655440000")
// Error: InvalidPrefix (typo: "agnt" instead of "agent")

// Fix: Use correct prefix
AgentId::parse("agent_550e8400-e29b-41d4-a716-446655440000")  // OK
```

**Error: "Invalid UUID format"**

```rust
AgentId::parse("agent_not-a-valid-uuid")
// Error: InvalidUuid (invalid UUID characters)

// Fix: Use valid UUID v4
AgentId::parse("agent_550e8400-e29b-41d4-a716-446655440000")  // OK
```

### Debugging

**Enable telemetry for parse failures:**

```toml
[dependencies]
iron_types = { version = "0.3", features = ["telemetry"] }
```

```rust
// Parse failures logged with structured fields
let id = AgentId::parse(untrusted_input);
// Log: WARN parse_failed input="invalid" reason="InvalidUuid"
```

---

## Documentation Compliance

### Automated Lint Checking

The Iron Runtime development environment includes an automated lint checker to ensure all documentation follows ID format standards.

**Run the lint check:**

```bash
# From dev/ directory
make lint-docs

# Or run the script directly
./scripts/lint_id_formats.sh
```

**What it checks:**

The lint script validates that all documentation uses the correct underscore format:
- Provider IDs: `ip_<identifier>` (not `ip-<identifier>`)
- User IDs: `user_<identifier>` (not `user-<identifier>`)
- Project IDs: `proj_<identifier>` (not `proj-<identifier>`)
- Agent IDs: `agent_<identifier>` (not `agent-<identifier>`)
- Token IDs: `ic_<identifier>` or `ip_<identifier>` (not hyphen format)

**Edge cases automatically excluded:**
- `user-token` (token type descriptor, not entity ID)
- `user-facing` (adjective, not entity ID)
- `user-level` (scope descriptor, not entity ID)
- Temporary files (prefixed with `-`)

**Success output:**

```
========================================
  ID Format Lint Check
========================================

Checking: Provider ID violations...
✓ Provider ID violations: OK
Checking: User ID violations...
✓ User ID violations: OK
Checking: Project ID violations...
✓ Project ID violations: OK
Checking: Agent ID violations...
✓ Agent ID violations: OK
Checking: Token ID violations...
✓ Token ID violations: OK
Verifying underscore format compliance...
✓ Found 161 properly formatted IDs

========================================
✓ No ID format violations found
========================================
```

**Failure output:**

When violations are detected, the script shows file paths, line numbers, and the specific violations:

```
✗ Provider ID violations
    docs/protocol/010_agents_api.md:45: "providers": ["ip-openai-001"]
    docs/protocol/011_providers_api.md:78: "id": "ip-anthropic-001"

✗ Found 2 violation(s)
========================================

ID Format Standards:
  Provider IDs:  ip_openai_001   (NOT ip-openai-001)
  User IDs:      user_xyz789     (NOT user-xyz789)
  Project IDs:   proj_master     (NOT proj-master)
  Agent IDs:     agent_abc123    (NOT agent-abc123)
  Token IDs:     ic_def456       (NOT ic-def456)

See docs/standards/id_format_standards.md for details
```

### Pre-Submission Checklist

Before submitting documentation changes:

1. **Run the lint check:**
   ```bash
   make lint-docs
   ```

2. **Fix any violations** using the guidance in the output

3. **Verify compliance:**
   ```bash
   make lint-docs  # Should show 0 violations
   ```

4. **Use canonical examples** from [canonical_examples.md](canonical_examples.md)

### CI Integration

The lint check will be integrated into CI/CD pipelines to automatically validate all documentation changes:

```yaml
# Planned CI integration
- name: Lint Documentation
  run: make lint-docs
```

This ensures that ID format violations are caught before code review, maintaining consistency across all documentation.

### Contributing Documentation

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for complete contributor guidelines including:
- Documentation standards
- ID format requirements
- Canonical example usage
- Pull request process

---

## References

### Related Documentation

- [iron_types crate](../../module/iron_types/readme.md) - Implementation reference
- [Entity Model](../architecture/007_entity_model.md) - Entities that have IDs
- [REST API Protocol](../protocol/002_rest_api_protocol.md) - API usage of IDs

### External Standards

- [RFC 4122 - UUID Specification](https://tools.ietf.org/html/rfc4122) - UUID format
- [RFC 3986 - URI Generic Syntax](https://tools.ietf.org/html/rfc3986) - URL-safe characters
- [OWASP - Input Validation](https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/07-Input_Validation_Testing/README) - Security validation

### Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.3.0 | 2025-12-10 | Standardized underscore format, added type safety |
| 0.2.0 | 2024-11-01 | Added `parse_flexible()` for migration |
| 0.1.0 | 2024-06-01 | Initial release with hyphen format |

---

**Document Version:** 0.3.0
**Last Updated:** 2025-12-10
**Status:** Normative (must follow)
