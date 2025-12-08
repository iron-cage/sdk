# iron_lang - Test Suite

Integration tests for iron_lang protocol message serialization and NDJSON transport.

## Test Organization

This directory contains integration tests for the iron_lang crate. All tests verify protocol correctness and serialization behavior.

### Test Files

| File | Purpose | Tests | Status |
|------|---------|-------|--------|
| `message_serialization.rs` | Protocol message serialization/deserialization | 6 | ✅ Passing |

### Test Coverage

**message_serialization.rs** - Protocol Message Serialization (6 tests):

1. `test_read_sql_message_roundtrip` - READ message with SQL query serialization
   - Validates ReadMessage → JSON → ReadMessage round-trip
   - Tests SqlParameter serialization
   - Verifies ReadOptions (timeout, retries)
   - Confirms message_type() and is_request() methods

2. `test_write_file_message_roundtrip` - WRITE message with file operation
   - Tests WriteMessage → JSON → WriteMessage round-trip
   - Validates FileWrite operation serialization
   - Confirms format and create_dirs options

3. `test_ack_message_roundtrip` - ACK response message
   - Tests AckMessage → JSON → AckMessage round-trip
   - Validates Status::Success serialization
   - Tests JSON data payload embedding
   - Confirms is_response() method

4. `test_error_message_roundtrip` - ERROR message with severity
   - Tests ErrorMessage → JSON → ErrorMessage round-trip
   - Validates builder pattern (with_severity, with_details)
   - Confirms ErrorSeverity enum serialization

5. `test_log_message_roundtrip` - LOG diagnostic message
   - Tests LogMessage → JSON → LogMessage round-trip
   - Validates LogLevel::Info serialization
   - Tests context JSON embedding with builder pattern

6. `test_ndjson_stream` - NDJSON streaming format
   - Tests multiple messages in NDJSON format (newline-delimited)
   - Validates stream serialization and parsing
   - Confirms message order preservation

## Running Tests

```bash
# Run all tests
cargo nextest run -p iron_lang

# Run specific test file
cargo nextest run -p iron_lang message_serialization

# Run single test
cargo test -p iron_lang test_read_sql_message_roundtrip

# Run with output
cargo test -p iron_lang -- --nocapture
```

## Test Strategy

**Focus**: Protocol correctness and JSON serialization
- All 9 message types must serialize/deserialize correctly
- NDJSON format must preserve message boundaries
- All optional fields must handle None/Some correctly
- Builder patterns must work with method chaining

**Coverage Status**:
- ✅ READ messages (SQL, File, HTTP, Cache, Object) - SQL covered
- ✅ WRITE messages (File, Database, API) - File covered
- ✅ ACK messages with data payloads - covered
- ✅ ERROR messages with severity levels - covered
- ✅ LOG messages with context - covered
- ✅ NDJSON streaming - covered
- ⏳ QUERY messages - not yet covered
- ⏳ SCHEMA messages - not yet covered
- ⏳ AUTH messages - not yet covered
- ⏳ METRICS messages - not yet covered

## Known Gaps

**Missing Test Coverage**:
1. QUERY messages (metadata queries for tables, files, keys)
2. SCHEMA messages (schema information requests)
3. AUTH messages (agent authentication)
4. METRICS messages (performance metrics)

**Future Tests Needed**:
- READ operation variants (File, HTTP, Cache, Object)
- WRITE operation variants (Database, API)
- Error scenarios (malformed JSON, missing required fields)
- Large payload handling (multi-MB data in ACK messages)
- Unicode handling in string fields
- Edge cases (empty strings, null fields, max lengths)

## Design Rationale

**Why Integration Tests?**

The protocol types are designed for serde serialization, so unit tests would just duplicate what serde does. Integration tests verify the ACTUAL behavior agents and connectors will see:
- Real JSON serialization (not mocked)
- Real NDJSON parsing (newline handling)
- Real type conversions (String → enum → String)

**Test File Organization**:

Single file (message_serialization.rs) because:
- All tests verify serialization behavior (single concern)
- Message types are tightly coupled (part of same protocol)
- Tests are small (6-15 lines each)
- Easy to see complete coverage in one place

If tests grow beyond 200 lines or new concerns emerge (e.g., validation, routing), split into:
- `message_serialization.rs` - JSON round-trips
- `message_validation.rs` - Field constraints
- `ndjson_streaming.rs` - Stream parsing

## Contributing

When adding new message types or fields:

1. Add round-trip test in `message_serialization.rs`
2. Test with realistic data (not just "test" strings)
3. Cover builder pattern if message has optional fields
4. Add NDJSON streaming test if message can be batched
5. Update this readme with coverage status

**Test Naming Convention**:
- `test_{message_type}_message_roundtrip` - Basic serialization
- `test_{operation}_with_{feature}` - Specific feature tests
- `test_ndjson_{scenario}` - Streaming tests

## References

- Protocol specification: `../spec.md`
- Message types: `../src/protocol.rs`
- NDJSON format: <https://github.com/ndjson/ndjson-spec>
