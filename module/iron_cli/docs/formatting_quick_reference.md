# Formatting Quick Reference

Quick reference for using TreeFmtFormatter in iron_cli adapters.

## Table of Contents

1. [Basic Usage](#basic-usage)
2. [Output Formats](#output-formats)
3. [Common Patterns](#common-patterns)
4. [Rules & Enforcement](#rules--enforcement)
5. [Testing](#testing)
6. [Troubleshooting](#troubleshooting)

---

## Basic Usage

### Import

```rust
use crate::formatting::TreeFmtFormatter;
```

### Single Item (Key-Value Pairs)

```rust
pub async fn my_adapter<T, S>(
  command: &T,
  service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
{
  let mut data = HashMap::new();
  data.insert( "status".to_string(), "success".to_string() );
  data.insert( "result".to_string(), "data".to_string() );

  let output = formatter.format_single( &data );
  Ok( output )
}
```

### Multiple Items (List)

```rust
pub async fn list_adapter<T, S>(
  command: &T,
  service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
{
  let items = vec![ item1, item2, item3 ];
  let output = formatter.format_list( &items );
  Ok( output )
}
```

---

## Output Formats

| Format   | Usage                          | Output Style              |
|----------|--------------------------------|---------------------------|
| Table    | Default, lists, comparisons    | Headers + rows + borders  |
| Expanded | Detailed records, single items | key: value pairs          |
| JSON     | API responses, machine-read    | Pretty-printed JSON       |
| YAML     | Config export, human-read      | YAML format               |

### Format Examples

**Table Format** (most common):
```
Key         Value
----------  ------------------------
created_at  2025-12-12T10:30:00Z
name        MyProductionToken
scope       read:write:admin
status      success
token_id    tok_abc123def456
```

**Expanded Format** (detailed view):
```
-[ RECORD 1 ]
id:      1
name:    token-1
status:  active

-[ RECORD 2 ]
id:      2
name:    token-2
status:  revoked
```

**JSON Format** (machine-readable):
```json
{
  "status": "success",
  "token_id": "tok_abc123",
  "name": "MyToken"
}
```

**YAML Format** (human-readable config):
```yaml
status: success
token_id: tok_abc123
name: MyToken
```

---

## Common Patterns

### Success Response

```rust
let mut data = HashMap::new();
data.insert( "status".to_string(), "success".to_string() );
data.insert( "message".to_string(), "Operation completed".to_string() );
data.insert( "result".to_string(), result_value );

formatter.format_single( &data )
```

**Output**:
```
Key      Value
-------  -------------------
message  Operation completed
result   [value]
status   success
```

### Error Response

```rust
let mut data = HashMap::new();
data.insert( "status".to_string(), "error".to_string() );
data.insert( "error".to_string(), error.to_string() );
data.insert( "code".to_string(), error_code.to_string() );

formatter.format_single( &data )
```

**Output**:
```
Key     Value
------  ----------------
code    ERR_NOT_FOUND
error   Token not found
status  error
```

### List with Metadata

```rust
let items = vec![ /* ... */ ];
let output = formatter.format_list( &items );

// Add metadata header
let final_output = format!( "Found {} items\n\n{}", items.len(), output );
Ok( final_output )
```

**Output**:
```
Found 3 items

id  name     status
--  -------  -------
1   token-1  active
2   token-2  revoked
3   token-3  active
```

### Empty Results

```rust
let items: Vec<HashMap<String, String>> = vec![];

if items.is_empty()
{
  return Ok( "No items found".to_string() );
}

formatter.format_list( &items )
```

### Conditional Fields

```rust
let mut data = HashMap::new();
data.insert( "id".to_string(), token.id.to_string() );
data.insert( "name".to_string(), token.name.clone() );

// Optional field
if let Some( ref expires ) = token.expires_at
{
  data.insert( "expires_at".to_string(), expires.to_string() );
}

formatter.format_single( &data )
```

---

## Rules & Enforcement

### ✅ DO

- **Use TreeFmtFormatter** for all output formatting
- **Return formatted String** from adapters
- **Let binary entry point** handle `println!`
- **Sort keys** (formatter does this automatically)
- **Use eprintln!** for debugging in adapters (allowed)

### ❌ DON'T

- **Don't use println!/print!** in `src/` (except binaries)
- **Don't build output strings manually** (use formatter)
- **Don't hardcode column widths** (formatter handles dynamically)
- **Don't mix formatting styles** (consistent TreeFmtFormatter usage)

### Enforcement Mechanisms

**1. Clippy Configuration** (`.clippy.toml`):
- Disallows `println!`, `print!`, `eprintln!`, `eprint!` in library code
- Compile-time enforcement
- Error message: "Use TreeFmtFormatter::format_single() or format_list() instead"

**2. Lint Tests** (`tests/lint/no_direct_printing_test.rs`):
- Static analysis scans source files
- Fails build if violations found
- Provides file:line location of violations

**3. Allowed Exceptions** (with `#![allow(clippy::disallowed_macros)]`):
- `build.rs`: Cargo communication protocol
- `src/bin/*.rs`: Final output layer (binary entry points)
- `tests/**/*.rs`: Test debugging output

### Example: Legitimate println! Usage

**Binary Entry Point** (allowed):
```rust
// src/bin/iron_token_unilang.rs
#![allow(clippy::disallowed_macros)]

fn main()
{
  let output = run_adapter( &formatter ).await?;
  println!( "{}", output );  // ✅ Allowed here
}
```

**Adapter** (not allowed):
```rust
// src/adapters/tokens.rs
pub async fn token_adapter( formatter: &TreeFmtFormatter ) -> String
{
  println!( "Debug" );  // ❌ Clippy error
  eprintln!( "Debug" );  // ✅ Allowed for debugging

  let output = formatter.format_single( &data );
  Ok( output )
}
```

---

## Testing

### Basic Test Pattern

```rust
#[test]
fn test_my_adapter_output()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );
  let output = my_adapter( &formatter ).await?;

  // Check content (not exact format, as it may change)
  assert!( output.contains( "status" ) );
  assert!( output.contains( "success" ) );
  assert!( output.contains( "---" ) );  // Has separator
}
```

### Test All Formats

```rust
#[test]
fn test_output_formats()
{
  let formats = vec![
    OutputFormat::Table,
    OutputFormat::Expanded,
    OutputFormat::Json,
    OutputFormat::Yaml,
  ];

  for format in formats
  {
    let formatter = TreeFmtFormatter::new( format );
    let output = my_adapter( &formatter ).await?;

    assert!( !output.is_empty() );
    assert!( output.contains( "status" ) );
  }
}
```

### Test Empty Data

```rust
#[test]
fn test_empty_list()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );
  let empty_items: Vec<HashMap<String, String>> = vec![];

  let output = formatter.format_list( &empty_items );

  assert_eq!( output, "No items found" );
}
```

---

## Troubleshooting

### Issue: Clippy Error "use of a disallowed macro"

**Error Message**:
```
error: use of a disallowed macro `std::println`
  --> src/adapters/tokens.rs:42:5
   |
42 |     println!("Debug output");
   |     ^^^^^^^^
```

**Solution**: Remove `println!` or use `eprintln!` for debugging
```rust
// Wrong
println!( "Debug: {:?}", data );  // ❌

// Right
eprintln!( "Debug: {:?}", data );  // ✅ Allowed for debugging

// Production
let output = formatter.format_single( &data );  // ✅ Correct approach
```

### Issue: Test Expecting Old Format

**Error**: Test fails because output format changed

**Solution**: Update assertions to check content, not exact format
```rust
// Old (brittle)
assert_eq!( output, "status          success" );  // ❌ Breaks if format changes

// New (robust)
assert!( output.contains( "status" ) );  // ✅ Checks content
assert!( output.contains( "success" ) );
```

### Issue: Misaligned Output

**Problem**: Columns not aligned properly

**Cause**: Usually ANSI escape codes in data

**Solution**: tree_fmt handles ANSI automatically, but if needed:
```rust
// Strip ANSI codes manually
fn strip_ansi( s: &str ) -> String
{
  s.chars().filter( |c| !c.is_control() ).collect()
}

data.insert( "key".to_string(), strip_ansi( &value ) );
```

### Issue: Empty Output

**Problem**: `format_single()` returns empty string

**Cause**: Data HashMap is empty

**Solution**: Check for empty data first
```rust
if data.is_empty()
{
  return Ok( "No data available".to_string() );
}

formatter.format_single( &data )
```

### Issue: JSON/YAML Parse Error

**Problem**: Format error in JSON/YAML output

**Cause**: Invalid characters in data values

**Solution**: tree_fmt handles escaping, but verify data:
```rust
// Ensure values are valid strings
for ( k, v ) in &data
{
  assert!( !v.contains( '\0' ) );  // No null bytes
}

formatter.format_single( &data )
```

---

## Additional Resources

- **Migration Plan**: See `-tree_fmt_migration_plan_detailed.md` for full context
- **Strategic Analysis**: See `-tree_fmt_migration_strategic_analysis.md` for ROI, risk analysis
- **Source Code**: `src/formatting/tree_formatter.rs` for implementation details
- **Tests**: `tests/formatting/tree_formatter_test.rs` for comprehensive examples
- **Lint Tests**: `tests/lint/no_direct_printing_test.rs` for enforcement examples

---

## Quick Checklist

Before committing adapter code:

- [ ] Imported `TreeFmtFormatter`
- [ ] Function signature includes `formatter: &TreeFmtFormatter`
- [ ] Using `formatter.format_single()` or `formatter.format_list()`
- [ ] No `println!`/`print!` in adapter code
- [ ] Tests pass (`cargo test`)
- [ ] Clippy passes (`cargo clippy`)
- [ ] Output looks correct (manual verification)

---

**Last Updated**: 2025-12-12
**Version**: 1.0
**Migration Status**: Complete ✅
