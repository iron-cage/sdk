# Data Format Standards

**Purpose:** Define canonical representation of common data types across all APIs

**Responsibility:** Specify timestamp, currency, boolean, null, and array formats

**Status:** Normative (all APIs MUST follow these standards)

**Version:** 1.0.0

---

## TL;DR

**Timestamps:** ISO 8601 with Z (`2025-12-10T10:30:45Z`)
**Currency:** Decimal with 2 places (`100.50`)
**Booleans:** JSON standard (`true`, `false`)
**Nulls:** Omit optional fields when empty
**Arrays:** Empty array `[]` not `null`

---

## Timestamp Format

### Standard Format

**Required:** ISO 8601 with timezone (UTC with `Z` suffix)

```json
{
  "created_at": "2025-12-10T10:30:45Z",
  "updated_at": "2025-12-10T11:45:30.123Z"
}
```

**Format specification:**
```
YYYY-MM-DDTHH:MM:SS.sssZ

Where:
- YYYY: 4-digit year
- MM: 2-digit month (01-12)
- DD: 2-digit day (01-31)
- T: Separator (literal)
- HH: 2-digit hour (00-23, 24-hour format)
- MM: 2-digit minute (00-59)
- SS: 2-digit second (00-59)
- .sss: Optional milliseconds (000-999)
- Z: UTC timezone indicator (literal)
```

**Regex pattern:**
```regex
^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{3})?Z$
```

### Examples

**Valid timestamps:**
```json
"2025-12-10T10:30:45Z"           // Without milliseconds
"2025-12-10T10:30:45.123Z"       // With milliseconds
"2025-01-01T00:00:00Z"           // Start of year
"2025-12-31T23:59:59.999Z"       // End of year
```

**Invalid timestamps:**
```json
"2025-12-10T10:30:45+00:00"      // ❌ Use Z not +00:00
"2025-12-10T10:30:45"            // ❌ Missing timezone
"2025-12-10 10:30:45"            // ❌ Space instead of T
"1733830245"                     // ❌ Unix timestamp (use ISO 8601)
```

### Precision

**Recommended:** Include milliseconds for high-precision events

```json
{
  "request_received_at": "2025-12-10T10:30:45.123Z",
  "request_completed_at": "2025-12-10T10:30:45.678Z",
  "duration_ms": 555
}
```

**Acceptable:** Omit milliseconds for low-precision events

```json
{
  "created_at": "2025-12-10T10:30:45Z",
  "expires_at": "2026-12-10T10:30:45Z"
}
```

### Timezone Handling

**Always UTC:** All timestamps MUST be in UTC (Z suffix)

```json
✅ "created_at": "2025-12-10T10:30:45Z"
❌ "created_at": "2025-12-10T05:30:45-05:00"  // EST offset
```

**Client-side conversion:** Clients convert to local timezone for display

```typescript
const timestamp = "2025-12-10T10:30:45Z";
const date = new Date(timestamp);

// Display in user's local timezone
console.log(date.toLocaleString());  // "12/10/2025, 5:30:45 AM" (EST)
```

### Database Storage

**PostgreSQL:**
```sql
CREATE TABLE agents (
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Query returns ISO 8601 with Z
SELECT created_at FROM agents;
-- 2025-12-10T10:30:45Z
```

**Application layer:** Serialize to ISO 8601 with Z suffix

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Agent {
    #[serde(with = "chrono::serde::ts_seconds")]
    created_at: DateTime<Utc>,
}

// Serializes to: "created_at": "2025-12-10T10:30:45Z"
```

### Sorting and Comparison

**Lexicographic sorting:** ISO 8601 strings sort correctly alphabetically

```json
[
  "2025-12-09T10:30:45Z",
  "2025-12-10T10:30:45Z",
  "2025-12-10T11:30:45Z"
]
// ✅ Already in chronological order
```

**Comparison:** String comparison works for date/time comparison

```javascript
const timestamp1 = "2025-12-09T10:30:45Z";
const timestamp2 = "2025-12-10T10:30:45Z";

timestamp1 < timestamp2  // true (9th < 10th)
```

---

## Currency Format

### Standard Format

**Format:** Decimal number with exactly 2 decimal places

```json
{
  "budget": 100.50,
  "spent": 45.75,
  "remaining": 54.75
}
```

**Decimal precision:** Always 2 decimal places (cents precision)

```json
✅ "budget": 100.00  // OK (even if zero cents)
✅ "budget": 100.50  // OK
❌ "budget": 100     // ❌ Missing decimal places
❌ "budget": 100.5   // ❌ Only 1 decimal place
❌ "budget": 100.505 // ❌ Too many decimal places
```

### Currency Code

**Pilot:** USD only (currency code omitted)

```json
{
  "budget": 100.50  // Implicitly USD
}
```

**Post-Pilot:** Multi-currency support requires currency field

```json
{
  "budget": {
    "amount": 100.50,
    "currency": "USD"
  }
}
```

### Range and Validation

**Valid range:** 0.01 to 999,999.99 (budget constraints)

```json
✅ "budget": 0.01        // Minimum budget
✅ "budget": 100.00      // Typical budget
✅ "budget": 999999.99   // Maximum budget
❌ "budget": 0.00        // ❌ Too low (minimum 0.01)
❌ "budget": 1000000.00  // ❌ Too high (maximum 999,999.99)
❌ "budget": -10.00      // ❌ Negative not allowed
```

### Floating Point Considerations

**Storage:** Use `DECIMAL(10, 2)` in database (avoid floating point errors)

```sql
CREATE TABLE agents (
    budget_usd DECIMAL(10, 2) NOT NULL CHECK (budget_usd >= 0.01)
);
```

**Application:** Use decimal types (not f64/float)

```rust
// ✅ Use rust_decimal for currency
use rust_decimal::Decimal;

#[derive(Serialize, Deserialize)]
struct Agent {
    #[serde(with = "rust_decimal::serde::float")]
    budget: Decimal,  // Exact decimal arithmetic
}

// ❌ Don't use f64 for currency
struct BadAgent {
    budget: f64,  // Floating point rounding errors!
}
```

### Formatting

**JSON serialization:** Always 2 decimal places

```rust
use serde::Serialize;

#[derive(Serialize)]
struct Agent {
    #[serde(serialize_with = "serialize_currency")]
    budget: f64,
}

fn serialize_currency<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64((*value * 100.0).round() / 100.0)
}
```

### Display Formatting

**API responses:** Decimal number only (no currency symbol)

```json
{
  "budget": 100.50
}
```

**Client-side display:** Add currency symbol and formatting

```typescript
const budget = 100.50;

// US formatting
console.log(`$${budget.toFixed(2)}`);  // "$100.50"

// International formatting
console.log(new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD'
}).format(budget));  // "$100.50"
```

---

## Boolean Format

### Standard Format

**Format:** JSON boolean (`true` or `false`)

```json
{
  "enabled": true,
  "archived": false
}
```

**Case sensitivity:** Lowercase only

```json
✅ "enabled": true
✅ "enabled": false
❌ "enabled": True     // ❌ Uppercase
❌ "enabled": TRUE     // ❌ All caps
❌ "enabled": 1        // ❌ Integer
❌ "enabled": "true"   // ❌ String
```

### Common Boolean Fields

| Field | Meaning | Default |
|-------|---------|---------|
| `enabled` | Feature/resource is active | `true` |
| `archived` | Resource is archived | `false` |
| `deleted` | Resource is soft-deleted | `false` |
| `expired` | Token/resource has expired | `false` |

### Truthy/Falsy Conversion

**Don't accept truthy/falsy values:** Strict boolean validation

```json
❌ POST /api/v1/agents {"enabled": 1}        // Reject
❌ POST /api/v1/agents {"enabled": "true"}   // Reject
❌ POST /api/v1/agents {"enabled": null}     // Reject (unless optional)
✅ POST /api/v1/agents {"enabled": true}     // Accept
✅ POST /api/v1/agents {"enabled": false}    // Accept
```

### Optional Boolean Fields

**Omit field when false (for optional booleans):**

```json
{
  "name": "Agent",
  "archived": true  // Only include if true
}

// Not:
{
  "name": "Agent",
  "archived": false  // Omit if false
}
```

**Rationale:** Cleaner response, smaller payload.

**Exception:** Required boolean fields always included

```json
{
  "name": "Agent",
  "enabled": false  // Always include (required field)
}
```

---

## Null Handling

### Optional Fields

**Strategy:** Omit optional fields when empty (don't use `null`)

```json
✅ Good (omit empty optional fields):
{
  "name": "Production Agent"
  // description omitted (empty)
}

❌ Bad (include null):
{
  "name": "Production Agent",
  "description": null
}
```

**Rationale:**
1. **Cleaner responses:** Smaller payload
2. **Standard REST practice:** Omit means "not provided"
3. **Type safety:** Easier client-side handling (field presence = value exists)

### Required Fields

**Strategy:** Required fields always present (never null)

```json
✅ Good (required fields present):
{
  "name": "Production Agent",
  "budget": 100.00,
  "created_at": "2025-12-10T10:30:45Z"
}

❌ Bad (required field null):
{
  "name": "Production Agent",
  "budget": null,  // ❌ Budget is required
  "created_at": "2025-12-10T10:30:45Z"
}
```

### Nullable Fields (Explicit Null)

**Use null ONLY when field can be explicitly cleared:**

```json
// Set description
PUT /api/v1/agents/{id}
{"description": "Production agent"}

// Clear description (explicit null)
PUT /api/v1/agents/{id}
{"description": null}
```

**Semantics:**
- **Field omitted:** Don't modify (leave unchanged)
- **Field null:** Clear value (set to empty/null)
- **Field present:** Update value

---

## Array Handling

### Empty Arrays

**Strategy:** Use empty array `[]` not `null`

```json
✅ Good (empty array):
{
  "providers": [],
  "tags": []
}

❌ Bad (null instead of array):
{
  "providers": null,
  "tags": null
}
```

**Rationale:**
1. **Type consistency:** Always array type
2. **No null checks:** `array.length === 0` vs `array === null || array.length === 0`
3. **Iteration safety:** Can iterate empty array without checks

### Optional Arrays

**Strategy:** Omit optional array fields when empty

```json
✅ Good (omit empty optional array):
{
  "name": "Production Agent"
  // tags omitted (empty)
}

❌ Bad (include empty array):
{
  "name": "Production Agent",
  "tags": []
}
```

**Exception:** Required array fields always included (even if empty)

```json
{
  "name": "Production Agent",
  "providers": []  // Required field (even if empty)
}
```

### Array vs Single Value

**Don't use array for single values:**

```json
❌ Bad (array for single value):
{
  "project_id": ["proj_abc123"]
}

✅ Good (single value):
{
  "project_id": "proj_abc123"
}
```

**Use array only for multiple values:**

```json
✅ Good (array for multiple values):
{
  "provider_ids": [
    "ip_openai-001",
    "ip_anthropic-001"
  ]
}
```

---

## Complete Example

### Agent Resource

**GET /api/v1/agents/{id}**

```json
{
  "id": "agent_550e8400-e29b-41d4-a716-446655440000",
  "name": "Production Agent",
  "budget": 100.00,
  "spent": 45.75,
  "remaining": 54.25,
  "providers": [
    "ip_660f9511-f3ab-52e5-b827-557766551111",
    "ip_770g0622-g4bc-63f6-c938-668877662222"
  ],
  "enabled": true,
  "created_at": "2025-12-10T10:30:45Z",
  "updated_at": "2025-12-10T11:45:30.123Z"
}
```

**Field-by-field breakdown:**

| Field | Type | Format | Notes |
|-------|------|--------|-------|
| `id` | string | `prefix_uuid` | See ID Format Standards |
| `name` | string | UTF-8 | Required |
| `budget` | number | `100.00` | Decimal with 2 places |
| `spent` | number | `45.75` | Decimal with 2 places |
| `remaining` | number | `54.25` | Decimal with 2 places |
| `providers` | array | `[string, ...]` | Empty array if none |
| `enabled` | boolean | `true` or `false` | Required |
| `created_at` | string | ISO 8601 with Z | Required |
| `updated_at` | string | ISO 8601 with Z | Required |

**Optional fields (omitted when empty):**

```json
{
  "id": "agent_550e8400-e29b-41d4-a716-446655440000",
  "name": "Production Agent",
  "budget": 100.00,
  "providers": [],
  "enabled": true,
  "created_at": "2025-12-10T10:30:45Z"
  // description omitted (not provided)
  // tags omitted (empty)
  // updated_at omitted (same as created_at)
}
```

---

## Validation Rules

### Timestamp Validation

```rust
use chrono::{DateTime, Utc};

fn validate_timestamp(s: &str) -> Result<DateTime<Utc>, Error> {
    s.parse::<DateTime<Utc>>()
        .map_err(|_| Error::InvalidTimestamp)
}

// Valid:
validate_timestamp("2025-12-10T10:30:45Z");          // OK
validate_timestamp("2025-12-10T10:30:45.123Z");      // OK

// Invalid:
validate_timestamp("2025-12-10T10:30:45+00:00");     // Error
validate_timestamp("2025-12-10T10:30:45");           // Error (missing Z)
```

### Currency Validation

```rust
use rust_decimal::Decimal;

fn validate_budget(value: Decimal) -> Result<(), Error> {
    if value < Decimal::new(1, 2) {  // 0.01
        return Err(Error::BudgetTooLow);
    }
    if value > Decimal::new(99999999, 2) {  // 999,999.99
        return Err(Error::BudgetTooHigh);
    }
    if value.scale() > 2 {
        return Err(Error::InvalidDecimalPlaces);
    }
    Ok(())
}
```

### Boolean Validation

```rust
// Serde handles boolean validation automatically
#[derive(Deserialize)]
struct Agent {
    enabled: bool,  // Only accepts true/false
}

// Rejects: "enabled": "true"
// Rejects: "enabled": 1
// Accepts: "enabled": true
```

---

## Migration and Compatibility

### Timestamp Migration

**Legacy format:** Unix timestamps (seconds since epoch)

```json
// Legacy (before 2025-12-10)
{
  "created_at": 1733830245
}
```

**Migration strategy:**

```rust
use chrono::{DateTime, Utc};

// Accept both formats during migration
fn parse_timestamp_flexible(value: &serde_json::Value) -> Result<DateTime<Utc>, Error> {
    match value {
        serde_json::Value::String(s) => {
            // Current format: ISO 8601
            s.parse::<DateTime<Utc>>()
                .map_err(|_| Error::InvalidTimestamp)
        }
        serde_json::Value::Number(n) => {
            // Legacy format: Unix timestamp
            let seconds = n.as_i64().ok_or(Error::InvalidTimestamp)?;
            DateTime::from_timestamp(seconds, 0)
                .ok_or(Error::InvalidTimestamp)
        }
        _ => Err(Error::InvalidTimestamp),
    }
}
```

**Always return current format:**

```json
// Always return ISO 8601 (even if database has Unix timestamps)
{
  "created_at": "2025-12-10T10:30:45Z"
}
```

---

## References

### Related Documentation

- [ID Format Standards](./id_format_standards.md) - Entity identifier formats
- [Error Format Standards](./error_format_standards.md) - Error response formats
- [REST API Protocol](../protocol/002_rest_api_protocol.md) - API implementation

### External Standards

- [ISO 8601 - Date and Time Format](https://www.iso.org/iso-8601-date-and-time-format.html) - Timestamp format
- [JSON Specification](https://www.json.org/) - Boolean and null handling
- [IEEE 754 - Floating Point](https://en.wikipedia.org/wiki/IEEE_754) - Avoid for currency

---

**Document Version:** 1.0.0
**Last Updated:** 2025-12-10
**Status:** Normative (must follow)
