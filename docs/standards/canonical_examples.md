# Canonical Examples

**Purpose:** Define standard example values for use across all documentation

**Responsibility:** Provide consistent, recognizable examples that create coherent narratives across protocols

**Status:** Informative (for documentation consistency, not implementation requirement)

**Version:** 1.0.0

---

## TL;DR

Use these canonical values in documentation examples to create consistent, recognizable patterns:

**Primary Agent:** `agent_abc123` with budget `100.00` USD
**Primary User:** `user-xyz789`
**Primary Providers:** `["ip-openai-001", "ip-anthropic-001"]`

---

## Purpose and Scope

### Why Canonical Examples?

**Problem:** Random example values make documentation harder to follow:
- Users see `agent_x` in one doc, `agent_foo` in another - are these the same?
- Budget changes randomly between examples - is this intentional or inconsistent?
- No clear relationship between examples across different protocols

**Solution:** Use canonical example values that:
- Create recognizable "characters" (Production Agent 1, Development Agent 2, etc.)
- Maintain consistent relationships across documentation
- Tell coherent stories when examples appear in multiple protocols
- Make documentation easier to navigate and understand

**Scope:**
- Documentation examples only (specs, protocols, readme files)
- NOT implementation requirements (code can use any valid values)
- NOT test data (tests should use varied realistic values)

---

## Canonical Entities

### Primary: Production Agent 1

**Use Case:** Primary example for agent management, analytics, budget operations

**Canonical Values:**
- **Agent ID:** `agent_abc123`
- **Name:** `"production-agent-1"` or `"Production Agent"` (prose)
- **Budget Allocated:** `100.00` USD
- **Budget Spent:** `45.75` USD (when showing usage/analytics)
- **Budget Remaining:** `54.25` USD (calculated: 100.00 - 45.75)
- **Owner ID:** `user-xyz789`
- **Project ID:** `proj-master` (Master Project in Pilot)
- **Providers:** `["ip-openai-001", "ip-anthropic-001"]`
- **Status:** `"active"` (default state)
- **IC Token ID:** `ic_def456ghi789`

**Usage Contexts:**

**Initial State (creation):**
```json
{
  "id": "agent_abc123",
  "name": "production-agent-1",
  "budget": 100.00,
  "providers": ["ip-openai-001", "ip-anthropic-001"],
  "owner_id": "user-xyz789",
  "project_id": "proj-master",
  "status": "active"
}
```

**Active State (with usage):**
```json
{
  "id": "agent_abc123",
  "budget": 100.00,
  "spent": 45.75,
  "remaining": 54.25,
  "providers": ["ip-openai-001", "ip-anthropic-001"],
  "status": "active"
}
```

**Exhausted State (budget depleted):**
```json
{
  "id": "agent_abc123",
  "budget": 100.00,
  "spent": 100.00,
  "remaining": 0.00,
  "status": "exhausted"
}
```

**Protocols Using This Agent:**
- 002 (REST API Protocol) - Error examples
- 005 (Budget Control Protocol) - IC Token structure
- 010 (Agents API) - CRUD operations
- 011 (Providers API) - Provider dependencies
- 012 (Analytics API) - Usage analytics
- 013 (Budget Limits API) - Budget modification
- 017 (Budget Requests API) - Budget increase workflow

---

### Secondary: Development Agent 2

**Use Case:** Secondary example for lists, pagination, multi-agent scenarios

**Canonical Values:**
- **Agent ID:** `agent_xyz789`
- **Name:** `"development-agent-2"` or `"Development Agent"`
- **Budget:** `50.00` USD (smaller budget for dev/testing)
- **Owner ID:** `user-xyz789` (same owner as Production Agent 1)
- **Providers:** `["ip-openai-001"]` (single provider)
- **Status:** `"active"`

**Usage Contexts:**
- List responses (showing multiple agents)
- Pagination examples
- Filtering examples (different budget tier)

---

### Tertiary: Testing Agent 3

**Use Case:** Third example for pagination, bulk operations

**Canonical Values:**
- **Agent ID:** `agent_def456`
- **Name:** `"testing-agent-3"` or `"Testing Agent"`
- **Budget:** `10.00` USD (minimal budget)
- **Owner ID:** `user-abc456` (different owner)
- **Providers:** `["ip-anthropic-001"]` (different single provider)
- **Status:** `"active"`

**Usage Contexts:**
- Pagination examples (page 2+)
- Admin views (multiple users' agents)
- Filtering examples (different owner)

---

### Quaternary: Legacy Agent 4

**Use Case:** Fourth example for extended lists

**Canonical Values:**
- **Agent ID:** `agent_ghi789`
- **Name:** `"legacy-agent-4"` or `"Legacy Agent"`
- **Budget:** `500.00` USD (large legacy budget)
- **Status:** `"inactive"` (decommissioned)

**Usage Contexts:**
- Extended pagination
- Status filtering examples
- Deletion examples

---

## Canonical Infrastructure Providers

### Primary: OpenAI Provider

**Use Case:** Primary LLM provider example

**Canonical Values:**
- **Provider ID:** `ip-openai-001`
- **Type:** Infrastructure Provider (OpenAI)
- **Status:** `"active"`

**Usage Contexts:**
- Agent provider assignments
- Provider management examples
- Analytics provider breakdown

---

### Secondary: Anthropic Provider

**Use Case:** Secondary LLM provider example

**Canonical Values:**
- **Provider ID:** `ip-anthropic-001`
- **Type:** Infrastructure Provider (Anthropic)
- **Status:** `"active"`

**Usage Contexts:**
- Multi-provider agent examples
- Provider management examples
- Analytics provider breakdown

---

## Canonical Users

### Primary: Developer User

**Use Case:** Primary user example (agent owner)

**Canonical Values:**
- **User ID:** `user-xyz789`
- **Email:** `developer@example.com`
- **Role:** `"user"` (standard developer role)
- **Owns Agents:** Production Agent 1, Development Agent 2

**Usage Contexts:**
- Authentication examples
- User management examples
- Agent ownership

---

### Secondary: Admin User

**Use Case:** Admin user example

**Canonical Values:**
- **User ID:** `user-admin`
- **Email:** `admin@example.com`
- **Role:** `"admin"`

**Usage Contexts:**
- Authorization examples (admin-only operations)
- User management examples
- Budget approval workflow

---

### Tertiary: Different Owner User

**Use Case:** Different owner for multi-user scenarios

**Canonical Values:**
- **User ID:** `user-abc456`
- **Email:** `developer2@example.com`
- **Role:** `"user"`
- **Owns Agents:** Testing Agent 3

**Usage Contexts:**
- Admin list views (multiple users)
- Authorization examples (cross-user access)

---

## Canonical Projects

### Primary: Master Project

**Use Case:** Default project in Pilot scope

**Canonical Values:**
- **Project ID:** `proj-master`
- **Name:** `"Master Project"` (Pilot default)

**Usage Contexts:**
- All Pilot-scope examples (single-project mode)
- Project references in agent examples

---

## Canonical Budget Values

### Standard Budget Amounts

Use these standard amounts for consistency:

| Amount | Use Case | Example Context |
|--------|----------|-----------------|
| `0.01` | Minimum valid budget | Validation examples, edge cases |
| `10.00` | Small/testing budget | Testing Agent 3, minimal allocation |
| `50.00` | Medium/development budget | Development Agent 2 |
| `100.00` | Standard/production budget | Production Agent 1 (PRIMARY) |
| `500.00` | Large/legacy budget | Legacy Agent 4 |
| `1000.00` | Budget increase request | Budget request workflow examples |

### Budget Math Consistency

When showing usage analytics for Production Agent 1:
- **Allocated:** `100.00` USD
- **Spent:** `45.75` USD (45.75% utilization)
- **Remaining:** `54.25` USD
- **Math:** 100.00 - 45.75 = 54.25 ✅

**Why 45.75?**
- Non-round number shows realistic usage patterns
- Clear decimal places demonstrate format requirements
- Memorable value (< 50% utilization)
- Consistent across all analytics examples

---

## Canonical Timestamps

### Standard Timestamp Values

Use these timestamps for consistency:

| Timestamp | Use Case | Context |
|-----------|----------|---------|
| `2025-12-10T10:30:45Z` | Primary timestamp | Agent creation, most examples |
| `2025-12-10T10:30:45.123Z` | With milliseconds | When demonstrating precision support |
| `2025-12-09T14:20:30Z` | Earlier timestamp | Updated_at, time-series data |
| `2025-12-11T08:15:00Z` | Later timestamp | Recent activity, pagination |

**Date Rationale:**
- 2025-12-10: Recent date (close to doc creation)
- 10:30:45: Easy-to-read time (round numbers)
- Z suffix: Always included (UTC timezone)

---

## Canonical Token Values

### IC Token (Agent Authentication)

**Production Agent 1:**
- **Token ID:** `ic_def456ghi789`
- **Token Value:** `ic_xyz789abc123def456...` (truncated in examples)
- **Issued At:** `2025-12-10T10:30:45Z`

### User Token (User Authentication)

**Developer User:**
- **Token Value:** `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...` (JWT, truncated)
- **Expires At:** 30 days from issue

### API Token (Long-lived)

**Developer User:**
- **Token ID:** `at_abc123def456`
- **Token Prefix:** `sk-abc123def456...` (truncated)

---

## Example Narrative Patterns

### Pattern 1: Agent Creation → Usage → Analytics

**Step 1: Create Agent**
```json
POST /api/v1/agents
{
  "name": "production-agent-1",
  "budget": 100.00,
  "providers": ["ip-openai-001", "ip-anthropic-001"]
}
```

**Step 2: Agent Makes Requests**
(Time passes, agent consumes 45.75 USD)

**Step 3: View Analytics**
```json
GET /api/v1/analytics/spending
{
  "agents": [{
    "agent_id": "agent_abc123",
    "budget": 100.00,
    "spent": 45.75,
    "remaining": 54.25
  }]
}
```

**Narrative:** Consistent agent_abc123 with budget 100.00 throughout story

---

### Pattern 2: Budget Exhaustion → Request Increase

**Step 1: Agent Exhausts Budget**
```json
{
  "error": {
    "code": "BUDGET_EXCEEDED",
    "message": "Agent budget exhausted (allocated: $100, spent: $100)",
    "details": {
      "agent_id": "agent_abc123",
      "budget_spent": 100.00,
      "budget_remaining": 0.00
    }
  }
}
```

**Step 2: Request Budget Increase**
```json
POST /api/v1/budget-requests
{
  "agent_id": "agent_abc123",
  "current_budget": 100.00,
  "requested_budget": 1000.00,
  "justification": "Increased inference load for customer demo"
}
```

**Narrative:** Same agent_abc123, shows budget lifecycle

---

### Pattern 3: Multi-Agent Comparison

**Production Agent (high usage):**
- agent_abc123: Budget 100.00, Spent 45.75 (45.75% utilization)

**Development Agent (medium usage):**
- agent_xyz789: Budget 50.00, Spent 12.50 (25% utilization)

**Testing Agent (low usage):**
- agent_def456: Budget 10.00, Spent 1.25 (12.5% utilization)

**Narrative:** Different agents, different usage patterns, consistent naming

---

## Usage Guidelines

### When to Use Canonical Examples

✅ **Use canonical examples for:**
- Protocol documentation (API specs, endpoint examples)
- Specification examples (FR-1.8 requirements)
- README files and getting-started guides
- Documentation that spans multiple files (maintains narrative)

❌ **Don't use canonical examples for:**
- Unit tests (use varied realistic test data)
- Integration tests (use dedicated test fixtures)
- Implementation code (use actual user data)
- Security testing (use random/fuzzy values)

---

### Example Format Variations

**Full UUID Format (format demonstration):**
```
agent_550e8400-e29b-41d4-a716-446655440000
```
Use when: Demonstrating complete ID format specification

**Short Format (documentation readability):**
```
agent_abc123
```
Use when: Making examples readable in protocol documentation

**Both formats are valid** per ID Format Standards. The short format is preferred for documentation readability, while the full UUID format is preferred for demonstrating format compliance.

---

### Mixing Canonical and Custom Examples

**When to deviate from canonical examples:**

1. **Testing edge cases:**
   - Invalid IDs: `agent_not-a-valid-uuid`
   - Malformed budgets: `100.0` (wrong decimal places)
   - Out-of-range values: `999999.99`

2. **Demonstrating variety:**
   - Different owner scenarios: Use user-abc456 for Testing Agent 3
   - Different providers: Single-provider vs multi-provider agents
   - Different budget tiers: 10.00, 50.00, 100.00, 500.00

3. **Specific protocol needs:**
   - Provider management: Create provider-specific examples
   - User management: Create role-specific examples
   - Settings: Create setting-specific examples

**Rule:** Use canonical values as defaults, deviate only when needed for clarity

---

## Maintenance

### Updating Canonical Examples

**When to update:**
- Entity structure changes (new required fields)
- ID format changes (prefix changes, validation rules)
- Budget calculation changes (new budget types)

**How to update:**
1. Update this document first (single source of truth)
2. Update protocol examples referencing changed values
3. Update specification examples if needed
4. Verify consistency with grep/verification scripts

**Do NOT:**
- Change canonical values randomly (breaks narrative consistency)
- Add new canonical entities without clear use case
- Remove canonical entities still used in protocols

---

## Verification

### Consistency Checks

**Check canonical values in use:**
```bash
# Find all uses of primary agent
grep -r "agent_abc123" docs/protocol/*.md docs/standards/*.md spec/*.md

# Verify budget consistency
grep -r "agent_abc123" docs/ | grep -E "(100\.00|45\.75|54\.25)"

# Check provider consistency
grep -r "agent_abc123" docs/ | grep -E "ip-(openai|anthropic)-001"
```

**Expected:**
- agent_abc123 appears with budget 100.00 in all contexts
- Spent amount is 45.75 when showing analytics
- Remaining is 54.25 when showing analytics
- Providers are ["ip-openai-001", "ip-anthropic-001"]

---

## References

**Related Standards:**
- [ID Format Standards](id_format_standards.md) - Entity ID format specification
- [Data Format Standards](data_format_standards.md) - Currency, timestamp formats

**Usage Examples:**
- All protocol documents (002-017) use canonical examples
- Specification FR-1.8 section uses canonical examples
- Documentation map references canonical examples

---

**Created:** 2025-12-11
**Status:** Informative (documentation guidance)
**Maintainer:** Documentation team
