# Manual Test Plan for iron_control_api

## Purpose

Manual tests verify end-to-end functionality that cannot be easily automated or requires real database/server interaction. These tests complement automated integration tests by covering:

- Cross-endpoint workflows
- Real database persistence
- Server startup/shutdown behavior
- Performance characteristics
- Error recovery scenarios

## Prerequisites

Before running manual tests:

1. Build the server: `cargo build --release --bin iron_control_api_server`
2. Ensure no other instance is running on port 3000
3. Have access to `sqlite3` CLI for database inspection
4. Have `curl` or `httpie` for making HTTP requests

## Test Suite

### 1. Server Lifecycle Tests

#### 1.1 Server Startup with Fresh Database

**Purpose:** Verify server initializes correctly with no existing data.

**Steps:**
1. Delete any existing `iron_control_api.db` file
2. Start server: `./target/release/iron_control_api_server`
3. Verify server starts without errors
4. Verify database file created
5. Query health endpoint: `curl http://localhost:3000/api/health`
6. Verify 200 OK response: `{"status":"healthy"}`

**Expected Result:**
- Server starts successfully
- Database file created with schema
- Health endpoint responds

**Pass/Fail:** ___

#### 1.2 Server Restart with Existing Data

**Purpose:** Verify server preserves data across restarts.

**Steps:**
1. Create test data (limits, tokens, usage records)
2. Stop server (Ctrl+C)
3. Restart server
4. Query endpoints to verify data persisted

**Expected Result:**
- All data present after restart
- No data corruption

**Pass/Fail:** ___

### 2. Token Management API (FR-7)

#### 2.1 Token Creation and Retrieval

**Purpose:** Verify token creation returns plaintext token once, subsequent GETs return metadata only.

**Steps:**
1. Create token: `curl -X POST http://localhost:3000/api/v1/api-tokens -H "Content-Type: application/json" -d '{"user_id":"test-user","project_id":"proj-1","description":"Test token"}'`
2. Save response with plaintext `token` field
3. Extract `id` from response
4. Get token by ID: `curl http://localhost:3000/api/v1/api-tokens/{id}`
5. Verify response has NO `token` field (only metadata: id, user_id, project_id, description, created_at, is_active)
6. Verify `is_active` is true

**Expected Result:**
- Create returns plaintext token (tk_... format)
- GET by ID returns metadata only (no token value)
- Token stored securely (SHA-256 hash in database)

**Pass/Fail:** ___

#### 2.2 Token Rotation

**Purpose:** Verify rotation generates new token and deactivates old one atomically.

**Steps:**
1. Create token via test 2.1
2. Save old token value
3. Rotate: `curl -X POST http://localhost:3000/api/v1/api-tokens/{id}/rotate`
4. Verify response contains NEW plaintext token (different from old)
5. Verify new token ID is different
6. Try using old token (should fail authentication)
7. Verify new token works

**Expected Result:**
- Rotation returns new plaintext token
- Old token immediately deactivated
- Atomic operation (no window where both tokens invalid)

**Pass/Fail:** ___

#### 2.3 Token Revocation

**Purpose:** Verify revoke endpoint deletes token and prevents further use.

**Steps:**
1. Create token via test 2.1
2. Verify token works
3. Revoke: `curl -X DELETE http://localhost:3000/api/v1/api-tokens/{id}`
4. Verify 204 No Content response
5. Get token by ID: `curl http://localhost:3000/api/v1/api-tokens/{id}`
6. Verify 404 Not Found
7. Try using revoked token
8. Verify authentication fails

**Expected Result:**
- Revoke returns 204
- Subsequent GET returns 404
- Token cannot be used after revocation

**Pass/Fail:** ___

#### 2.4 Token Listing (Requires JWT Auth)

**Purpose:** Verify list endpoint returns all user's tokens (requires authentication).

**Steps:**
1. Create 3 tokens for same user_id
2. Obtain JWT token for user
3. List tokens: `curl http://localhost:3000/api/v1/api-tokens -H "Authorization: Bearer {jwt}"`
4. Verify all 3 tokens present in array
5. Verify NO plaintext token values (only metadata)
6. Verify each has correct user_id, project_id, description
7. Try list without auth header
8. Verify 401 Unauthorized

**Expected Result:**
- List returns all user's tokens
- No plaintext token values
- Requires valid JWT authentication

**Pass/Fail:** ___

#### 2.5 Token Security Verification

**Purpose:** Verify tokens stored as hashes, not plaintext.

**Steps:**
1. Create token via test 2.1
2. Save plaintext token from creation response
3. Inspect database: `sqlite3 iron_control_api.db "SELECT token_hash FROM tokens WHERE id={id}"`
4. Verify `token_hash` is 64-character hex string (SHA-256)
5. Verify `token_hash` does NOT match plaintext token
6. Verify NO `token` column exists (only `token_hash`)

**Expected Result:**
- Database stores SHA-256 hash only
- No plaintext tokens in database
- Hash format: 64 hex characters

**Pass/Fail:** ___

#### 2.6 Token Validation Endpoint (Deliverable 1.6)

**Purpose:** Verify /api/v1/api-tokens/validate endpoint validates tokens without authentication.

**Steps:**
1. Create token via test 2.1: `curl -X POST http://localhost:3000/api/v1/api-tokens -H "Content-Type: application/json" -d '{"user_id":"test-user","project_id":"proj-1"}'`
2. Save plaintext token from response: `token_value=$(jq -r '.token' response.json)`
3. Validate valid token: `curl -X POST http://localhost:3000/api/v1/api-tokens/validate -H "Content-Type: application/json" -d "{\"token\":\"$token_value\"}"`
4. Verify response: `{"valid":true,"user_id":"test-user","project_id":"proj-1","token_id":...}`
5. Validate invalid token: `curl -X POST http://localhost:3000/api/v1/api-tokens/validate -H "Content-Type: application/json" -d '{"token":"invalid_token_xyz"}'`
6. Verify response: `{"valid":false}`
7. Revoke token: `curl -X DELETE http://localhost:3000/api/v1/api-tokens/{id}`
8. Validate revoked token: `curl -X POST http://localhost:3000/api/v1/api-tokens/validate -H "Content-Type: application/json" -d "{\"token\":\"$token_value\"}"`
9. Verify response: `{"valid":false}`
10. Verify no auth required: Make validation request WITHOUT Authorization header
11. Verify 200 OK (not 401)

**Expected Result:**
- Valid active tokens return `{"valid":true}` with metadata (user_id, project_id, token_id)
- Invalid tokens return `{"valid":false}` with no metadata
- Revoked tokens return `{"valid":false}` with no metadata
- Endpoint is public (no Authorization header required)
- Response is always 200 OK with validation result

**Security Notes:**
- Constant-time comparison prevents timing attacks
- Public endpoint design allows external services to validate tokens
- No sensitive information leaked for invalid tokens
- Recommend rate limiting at reverse proxy (100 validates/min per IP)

**Pass/Fail:** ___

#### 2.7 Input Validation (Phase 1 Security)

**Purpose:** Verify DoS protection (issue-001) and NULL byte injection prevention (issue-002).

**Steps:**
1. **Test DoS protection - user_id length limit:**
   - Create 501-char user_id: `python3 -c "print('a' * 501)"`
   - Attempt create: `curl -X POST http://localhost:3000/api/v1/api-tokens -H "Content-Type: application/json" -d "{\"user_id\":\"$(python3 -c 'print("a" * 501)')\"}" -v`
   - Verify 400 Bad Request
   - Verify error message: `"Field 'user_id' exceeds maximum length (501 chars, max 500)"`

2. **Test DoS protection - project_id length limit:**
   - Create 501-char project_id
   - Attempt create with valid user_id but oversized project_id
   - Verify 400 Bad Request
   - Verify error message mentions project_id and length

3. **Test NULL byte injection - user_id:**
   - Create user_id with embedded NULL: `"test\x00user"`
   - Attempt create: `curl -X POST ... -d '{"user_id":"test\u0000user"}'`
   - Verify 400 Bad Request
   - Verify error message: `"Field 'user_id' contains NULL byte"`

4. **Test NULL byte injection - project_id:**
   - Create project_id with NULL byte
   - Attempt create with valid user_id but NULL-containing project_id
   - Verify 400 Bad Request
   - Verify error message mentions project_id and NULL byte

5. **Test valid boundary cases:**
   - Create token with exactly 500-char user_id: `curl ... -d "{\"user_id\":\"$(python3 -c 'print("a" * 500)')\"}" `
   - Verify 201 Created (success)
   - Create token with exactly 500-char project_id
   - Verify 201 Created (success)
   - Create token with 1-char user_id: `"a"`
   - Verify 201 Created (success)

6. **Test empty string validation:**
   - Attempt create with empty user_id: `curl ... -d '{"user_id":""}'`
   - Verify 400 Bad Request
   - Verify error message mentions empty/zero-length

7. **Test database constraint enforcement (defense-in-depth):**
   - Inspect database schema: `sqlite3 iron_control_api.db ".schema api_tokens"`
   - Verify CHECK constraints exist: `LENGTH(user_id) > 0 AND LENGTH(user_id) <= 500`
   - Verify CHECK constraints exist: `LENGTH(project_id) > 0 AND LENGTH(project_id) <= 500`

**Expected Result:**
- All oversized inputs (>500 chars) return 400 with descriptive errors
- All NULL byte inputs return 400 with descriptive errors
- Boundary values (1 char, 500 chars) succeed
- Empty strings return 400
- Database has CHECK constraints matching API validation (defense-in-depth)
- Error messages include: field name, actual length/issue, maximum allowed

**Rationale:**
- **DoS Protection (issue-001):** Unbounded strings cause memory exhaustion. 500-char limit prevents multi-MB attack payloads.
- **Injection Protection (issue-002):** NULL bytes terminate C strings in database drivers, enabling data corruption or validation bypasses.
- **Defense-in-Depth:** API validation (Layer 1) + Database constraints (Layer 2) provide redundant security.

**Pass/Fail:** ___

### 3. Usage Analytics Workflow (FR-8)

#### 3.1 Cross-Token Aggregation

**Purpose:** Verify /api/usage/aggregate correctly aggregates across ALL tokens.

**Steps:**
1. Create 3 tokens with different project_ids via token manager
2. Record usage for each token:
   - Token 1 (OpenAI): 1000 tokens, 5 requests, 100 cost_cents
   - Token 2 (Anthropic): 2000 tokens, 3 requests, 200 cost_cents
   - Token 3 (OpenAI): 500 tokens, 2 requests, 50 cost_cents
3. Query: `curl http://localhost:3000/api/v1/usage/aggregate`
4. Verify totals:
   - `total_tokens`: 3500
   - `total_requests`: 10
   - `total_cost_cents`: 350
5. Verify provider breakdown:
   - OpenAI: 1500 tokens, 7 requests, 150 cost
   - Anthropic: 2000 tokens, 3 requests, 200 cost

**Expected Result:**
- Correct aggregation across all tokens
- Provider breakdown matches individual totals

**Pass/Fail:** ___

#### 3.2 Project Filtering

**Purpose:** Verify /api/usage/by-project/:project_id filters correctly.

**Steps:**
1. Use tokens from test 2.1
2. Query: `curl http://localhost:3000/api/v1/usage/by-project/project_1`
3. Verify only Token 1 usage counted
4. Query: `curl http://localhost:3000/api/v1/usage/by-project/nonexistent`
5. Verify 200 OK with all zeros (not 404)

**Expected Result:**
- Project filtering works correctly
- Unknown project returns zeros, not error

**Pass/Fail:** ___

#### 3.3 Provider Filtering

**Purpose:** Verify /api/usage/by-provider/:provider filters correctly.

**Steps:**
1. Use tokens from test 2.1
2. Query: `curl http://localhost:3000/api/v1/usage/by-provider/openai`
3. Verify totals:
   - `tokens`: 1500
   - `requests`: 7
   - `cost_cents`: 150
4. Query: `curl http://localhost:3000/api/v1/usage/by-provider/anthropic`
5. Verify Anthropic totals
6. Query: `curl http://localhost:3000/api/v1/usage/by-provider/unknown`
7. Verify 200 OK with zeros

**Expected Result:**
- Provider filtering case-sensitive (lowercase only)
- Unknown provider returns zeros

**Pass/Fail:** ___

### 4. Limits CRUD Workflow (FR-9)

#### 4.1 Create-Read-Update-Delete Lifecycle

**Purpose:** Verify complete CRUD cycle persists correctly.

**Steps:**
1. Create limit: `curl -X POST http://localhost:3000/api/v1/limits -H "Content-Type: application/json" -d '{"tokens_per_day":1000,"requests_per_minute":10,"cost_per_month_cents":500}'`
2. Extract `id` from response
3. Verify appears in list: `curl http://localhost:3000/api/v1/limits`
4. Get by ID: `curl http://localhost:3000/api/v1/limits/{id}`
5. Update: `curl -X PUT http://localhost:3000/api/v1/limits/{id} -H "Content-Type: application/json" -d '{"tokens_per_day":2000}'`
6. Verify update persisted via GET
7. Delete: `curl -X DELETE http://localhost:3000/api/v1/limits/{id}`
8. Verify 404 on subsequent GET

**Expected Result:**
- All CRUD operations succeed
- Updates persist across queries
- Deletion removes record completely

**Pass/Fail:** ___

#### 4.2 Validation Enforcement

**Purpose:** Verify validation rules prevent invalid data.

**Steps:**
1. Try create with all None: `curl -X POST ... -d '{"tokens_per_day":null,"requests_per_minute":null,"cost_per_month_cents":null}'`
2. Verify 400 Bad Request
3. Try create with zero value: `curl -X POST ... -d '{"tokens_per_day":0}'`
4. Verify 400 Bad Request
5. Try create with negative: `curl -X POST ... -d '{"tokens_per_day":-100}'`
6. Verify 400 Bad Request
7. Try create with overflow (i64::MAX + 1)
8. Verify 400 Bad Request

**Expected Result:**
- All invalid requests return 400 with descriptive error
- No invalid data persisted

**Pass/Fail:** ___

### 5. Traces Storage and Retrieval (FR-10)

#### 5.1 Trace Recording and Listing

**Purpose:** Verify traces are stored and retrieved correctly.

**Steps:**
1. Record 5 API call traces via TraceStorage
2. Query: `curl http://localhost:3000/api/v1/traces`
3. Verify all 5 traces present
4. Verify ordered by `traced_at` DESC (most recent first)
5. Verify all fields present (id, token_id, provider, model, endpoint, status_code, latency_ms, input_tokens, output_tokens, cost_cents, timestamp)

**Expected Result:**
- All traces returned
- Correct ordering
- Complete field data

**Pass/Fail:** ___

#### 5.2 Trace Detail Retrieval

**Purpose:** Verify individual trace retrieval.

**Steps:**
1. Use traces from test 4.1
2. Extract ID of first trace
3. Query: `curl http://localhost:3000/api/v1/traces/{id}`
4. Verify all fields match list response
5. Query non-existent ID: `curl http://localhost:3000/api/v1/traces/999999`
6. Verify 404 Not Found

**Expected Result:**
- Valid ID returns complete trace
- Invalid ID returns 404

**Pass/Fail:** ___

### 6. Error Recovery Tests

#### 6.1 Database Connection Loss

**Purpose:** Verify graceful handling of database errors.

**Steps:**
1. Start server
2. While server running, corrupt database file: `echo "garbage" >> iron_control_api.db`
3. Try to query endpoints
4. Verify 500 Internal Server Error (not crash)
5. Check server logs for error messages

**Expected Result:**
- Server doesn't crash
- Returns 500 with error
- Logs show database error details

**Pass/Fail:** ___

#### 6.2 Concurrent Requests

**Purpose:** Verify server handles concurrent requests safely.

**Steps:**
1. Write script to make 100 concurrent requests to different endpoints
2. Run script
3. Verify all requests succeed or fail gracefully
4. Verify no data corruption
5. Verify server remains responsive

**Expected Result:**
- No crashes
- No data corruption
- Consistent responses

**Pass/Fail:** ___

### 7. Performance Baseline Tests

#### 7.1 Response Time Under Load

**Purpose:** Establish baseline response times.

**Steps:**
1. Create 1000 usage records
2. Measure response time for /api/usage/aggregate (10 requests)
3. Measure response time for /api/usage/by-project (10 requests)
4. Measure response time for /api/traces (10 requests)
5. Calculate average, min, max for each

**Expected Result:**
- Average response time < 100ms for all endpoints
- No timeouts

**Baseline Measurements:**
- /api/usage/aggregate: avg=___ ms, min=___ ms, max=___ ms
- /api/usage/by-project: avg=___ ms, min=___ ms, max=___ ms
- /api/traces: avg=___ ms, min=___ ms, max=___ ms

**Pass/Fail:** ___

## Corner Case Verification Checklist

This section documents exhaustive corner cases across all API functionality. Each corner case must be manually verified before production release.

### Token Management Corner Cases (FR-7)

#### String Field Edge Cases
- [ ] **Empty user_id**: POST /api/tokens with `{"user_id":""}` → Expect 400 Bad Request
- [ ] **Empty project_id**: POST /api/tokens with `{"project_id":""}` → Expect 400 Bad Request
- [ ] **Empty description**: POST /api/tokens with `{"description":""}` → Should accept (optional field)
- [ ] **Maximum length user_id**: Exactly 500 characters → Expect 201 Created (boundary)
- [ ] **Oversized user_id**: 501 characters → Expect 400 Bad Request (DoS protection issue-001)
- [ ] **Maximum length project_id**: Exactly 500 characters → Expect 201 Created (boundary)
- [ ] **Oversized project_id**: 501 characters → Expect 400 Bad Request (DoS protection issue-001)
- [ ] **Very long description**: 10,000+ characters → Should accept or have documented limit
- [ ] **NULL byte in user_id**: `"test\u0000user"` → Expect 400 Bad Request (injection issue-002)
- [ ] **NULL byte in project_id**: `"proj\u0000ect"` → Expect 400 Bad Request (injection issue-002)
- [ ] **NULL byte in description**: `"desc\u0000ription"` → Should reject or sanitize
- [ ] **Unicode user_id**: `"用户123"` (Chinese characters) → Should accept or reject consistently
- [ ] **Emoji user_id**: `"user🚀test"` → Should accept or reject consistently
- [ ] **SQL injection user_id**: `"'; DROP TABLE tokens; --"` → Should safely escape (parameterized queries)
- [ ] **XSS in description**: `"<script>alert('xss')</script>"` → Should escape on retrieval
- [ ] **Control characters**: `"\n\r\t"` in fields → Should reject or sanitize
- [ ] **Whitespace-only user_id**: `"   "` → Should reject (empty after trim)

#### Token State Transitions
- [ ] **Create → Rotate → Rotate**: Multiple rotations in sequence → All should succeed
- [ ] **Create → Revoke → Rotate**: Rotate revoked token → Expect 404 or 400 (cannot rotate revoked)
- [ ] **Create → Revoke → Revoke**: Revoke already-revoked token → Expect 404 or idempotent 204
- [ ] **Create → Rotate → Revoke (old)**: Revoke old token after rotation → Should already be inactive
- [ ] **Concurrent creation**: Two simultaneous POST /api/tokens → Both should succeed with unique IDs
- [ ] **Concurrent rotation**: Two simultaneous rotates of same token → One succeeds, one fails gracefully
- [ ] **Rotation during active use**: Rotate token while it's authenticating requests → Old token immediately invalid

#### Token Retrieval Edge Cases
- [ ] **List tokens with zero tokens**: GET /api/tokens → Expect 200 OK with empty array `[]`
- [ ] **Get token by non-existent ID**: GET /api/tokens/999999 → Expect 404 Not Found
- [ ] **Get token by invalid ID format**: GET /api/tokens/abc → Expect 400 Bad Request or 404
- [ ] **Get token by negative ID**: GET /api/tokens/-1 → Expect 400 or 404
- [ ] **List tokens without JWT**: GET /api/tokens (no Authorization header) → Expect 401 Unauthorized
- [ ] **List tokens with expired JWT**: Expired token → Expect 401 Unauthorized
- [ ] **List tokens with invalid JWT signature**: Tampered token → Expect 401 Unauthorized

#### Token Hash Collision (Theoretical)
- [ ] **Hash collision**: Two different token values produce same SHA-256 hash → Cryptographically infeasible, but verify parameterized query handles duplicates safely

### Usage Analytics Corner Cases (FR-8)

#### Zero Data Cases
- [ ] **Aggregate with zero records**: GET /api/usage/aggregate → Expect 200 OK with all zeros
- [ ] **By-project with zero records**: GET /api/usage/by-project/nonexistent → Expect 200 OK with zeros
- [ ] **By-provider with zero records**: GET /api/usage/by-provider/unknown → Expect 200 OK with zeros

#### Large Data Cases
- [ ] **Aggregate with 1M+ records**: Performance test → Verify response time acceptable (<1s target)
- [ ] **By-project with many projects**: 1000+ distinct project_ids → Verify correct aggregation
- [ ] **By-provider with all providers**: Anthropic, OpenAI, Cohere, etc. → Verify all counted

#### Invalid Input Cases
- [ ] **Negative tokens**: Record usage with `tokens=-100` → Should reject (if validation exists)
- [ ] **Negative cost**: Record usage with `cost_cents=-50` → Should reject (if validation exists)
- [ ] **NULL required fields**: Missing token_id, provider, or model → Should reject
- [ ] **Integer overflow**: cost_cents exceeding i64::MAX → Should handle gracefully
- [ ] **Float overflow**: Very large numbers in calculations → Should not crash

#### String Validation Cases
- [ ] **Special characters in project_id filter**: `GET /api/usage/by-project/<script>` → Should escape safely
- [ ] **SQL injection in project_id**: `GET /api/usage/by-project/' OR '1'='1` → Should safely escape
- [ ] **Case sensitivity in provider**: `GET /api/usage/by-provider/OpenAI` vs `/openai` → Document behavior
- [ ] **Empty project_id filter**: `GET /api/usage/by-project/` → Should return 404 or all usage
- [ ] **NULL bytes in filters**: `project_id=test\u0000` → Should reject or sanitize

#### Aggregation Correctness
- [ ] **Sum overflow**: Total cost_cents exceeds i64::MAX → Should handle gracefully (unlikely but test)
- [ ] **Sum underflow**: Negative total (shouldn't happen with valid data) → Should reject or clamp to 0
- [ ] **Provider name mismatch**: Mixed case providers ("OpenAI", "openai") → Verify case handling
- [ ] **Orphaned usage records**: Usage record with deleted token_id → Should still count in aggregate

### Limits CRUD Corner Cases (FR-9)

#### Create Edge Cases
- [ ] **All fields NULL**: POST /api/limits with all `null` values → Expect 400 Bad Request
- [ ] **All fields zero**: POST /api/limits with `{tokens_per_day:0}` → Expect 400 Bad Request
- [ ] **Negative values**: POST /api/limits with `{tokens_per_day:-100}` → Expect 400 Bad Request
- [ ] **Integer overflow**: POST /api/limits with `{tokens_per_day:i64::MAX+1}` → Expect 400 Bad Request
- [ ] **Single field limit**: Only `tokens_per_day` set, others NULL → Should accept (partial limits)
- [ ] **Duplicate limits**: Create two identical limits → Should both exist with different IDs
- [ ] **Very large valid value**: `tokens_per_day: 999999999` → Should accept (within i64 range)

#### Update Edge Cases
- [ ] **Update non-existent ID**: PUT /api/limits/999999 → Expect 404 Not Found
- [ ] **Update to invalid value**: PUT with negative number → Expect 400 Bad Request
- [ ] **Update to NULL**: PUT /api/limits/{id} with `{tokens_per_day:null}` → Should handle (clear limit or reject)
- [ ] **Update to zero**: PUT /api/limits/{id} with `{tokens_per_day:0}` → Expect 400 Bad Request
- [ ] **Partial update**: Only update one field, leave others unchanged → Should preserve other fields
- [ ] **Concurrent updates**: Two simultaneous PUTs to same limit → Last-write-wins or error
- [ ] **Update during enforcement**: Change limit while it's being checked → Should apply immediately

#### Delete Edge Cases
- [ ] **Delete non-existent ID**: DELETE /api/limits/999999 → Expect 404 Not Found
- [ ] **Delete already-deleted**: DELETE same ID twice → Second should return 404
- [ ] **Delete during enforcement**: Delete limit while checking against it → Should handle gracefully
- [ ] **Delete all limits**: Remove all limits → System should still function (no limits = unlimited)

#### Retrieval Edge Cases
- [ ] **List with zero limits**: GET /api/limits → Expect 200 OK with empty array `[]`
- [ ] **Get by non-existent ID**: GET /api/limits/999999 → Expect 404 Not Found
- [ ] **Get by invalid ID**: GET /api/limits/abc → Expect 400 or 404

### Traces Storage Corner Cases (FR-10)

#### Trace Creation Edge Cases
- [ ] **Negative latency_ms**: Record trace with `latency_ms:-50` → Should reject or clamp to 0
- [ ] **Zero latency**: Record trace with `latency_ms:0` → Should accept (valid for cached responses)
- [ ] **Very large latency**: `latency_ms: 3600000` (1 hour) → Should accept (timeout case)
- [ ] **NULL required fields**: Missing token_id, provider, model → Should reject
- [ ] **NULL optional fields**: Missing request_id, metadata → Should accept
- [ ] **Timestamp in future**: `traced_at` > current time → Should accept or reject consistently
- [ ] **Timestamp very old**: `traced_at` from years ago → Should accept (historical data)
- [ ] **Invalid provider name**: Empty string or NULL → Should reject
- [ ] **Invalid model name**: Empty string or NULL → Should reject
- [ ] **Very long request_id**: 1000+ characters → Should accept or have documented limit
- [ ] **Very large metadata**: 1MB+ JSON object → Should accept or reject with clear error

#### Trace Retrieval Edge Cases
- [ ] **List with zero traces**: GET /api/traces → Expect 200 OK with empty array `[]`
- [ ] **List with 10,000+ traces**: Performance test (no pagination) → Verify response time
- [ ] **Get by non-existent ID**: GET /api/traces/999999 → Expect 404 Not Found
- [ ] **Get by invalid ID**: GET /api/traces/abc → Expect 400 or 404
- [ ] **Ordering verification**: Verify DESC order by traced_at → Most recent first

#### Trace Filtering (If Implemented)
- [ ] **Filter by token_id**: Only traces for specific token → Verify correct filtering
- [ ] **Filter by provider**: Only OpenAI traces → Verify case sensitivity
- [ ] **Filter by date range**: Traces between two timestamps → Verify boundary inclusion

### Database Layer Corner Cases

#### Connection/Transaction Edge Cases
- [ ] **Database file locked**: Another process holding lock → Expect 500 Internal Server Error
- [ ] **Database file corrupted**: Malformed SQLite file → Expect 500, server should not crash
- [ ] **Disk full**: Cannot write to database → Expect 500 with descriptive error
- [ ] **Concurrent transactions**: Multiple writes at same time → Verify ACID properties maintained
- [ ] **Transaction rollback**: Error during multi-step operation → Verify partial changes rolled back
- [ ] **Connection pool exhausted**: All connections in use → Should queue or return 503

#### Database Migration Edge Cases
- [ ] **Fresh database**: No existing schema → Should create tables automatically
- [ ] **Schema version mismatch**: Old database schema → Should migrate or fail clearly
- [ ] **Partial migration failure**: Error during schema update → Should rollback or complete atomically

### HTTP Protocol Corner Cases

#### Invalid HTTP Methods
- [ ] **POST on GET-only endpoint**: POST /api/usage/aggregate → Expect 405 Method Not Allowed
- [ ] **GET on POST-only endpoint**: GET /api/tokens (should be POST to create) → Context-dependent
- [ ] **OPTIONS requests**: CORS preflight → Verify proper CORS headers returned
- [ ] **HEAD requests**: HEAD /api/health → Should return headers only, no body
- [ ] **PATCH requests**: PATCH /api/limits/{id} → Should support or return 405

#### Invalid Request Format
- [ ] **Missing Content-Type**: POST without `Content-Type: application/json` → Should return 400
- [ ] **Wrong Content-Type**: POST with `Content-Type: text/plain` → Should return 415 Unsupported Media Type
- [ ] **Malformed JSON**: POST with invalid JSON syntax → Expect 400 with parse error
- [ ] **Empty request body**: POST with no body → Expect 400
- [ ] **Very large request body**: 10MB+ JSON → Should reject with 413 Payload Too Large (DoS protection)
- [ ] **Extra fields in JSON**: Unknown fields in request → Should ignore or reject consistently
- [ ] **Type mismatch**: Send string where integer expected → Expect 400 with type error

#### URL Edge Cases
- [ ] **Invalid URL paths**: GET /api/invalid → Expect 404 Not Found
- [ ] **Trailing slashes**: GET /api/tokens/ vs /api/tokens → Should handle consistently
- [ ] **Multiple slashes**: GET /api//tokens → Should normalize or return 404
- [ ] **URL encoding**: GET /api/usage/by-project/test%20project → Should decode correctly
- [ ] **Special characters in path**: GET /api/usage/by-project/<script> → Should escape safely
- [ ] **Query parameters on endpoints that ignore them**: GET /api/health?foo=bar → Should ignore gracefully

### Authentication/Authorization Corner Cases

#### JWT Token Edge Cases
- [ ] **Missing Authorization header**: Request requiring auth with no header → Expect 401 Unauthorized
- [ ] **Invalid header format**: `Authorization: InvalidFormat` → Expect 401
- [ ] **Expired JWT**: Token past expiration time → Expect 401
- [ ] **Invalid signature**: Tampered JWT payload → Expect 401
- [ ] **Wrong algorithm**: JWT signed with different algorithm → Expect 401
- [ ] **Missing required claims**: JWT without user_id or other required claims → Expect 401
- [ ] **Token from different issuer**: JWT from another system → Expect 401
- [ ] **Very long JWT**: 10KB+ token → Should reject or handle

#### API Token Edge Cases (Bearer Tokens)
- [ ] **Invalid token format**: Not starting with expected prefix → Expect 401
- [ ] **Revoked token**: Using previously revoked token → Expect 401
- [ ] **Token for different user**: Using another user's token → Should enforce ownership
- [ ] **Token with special characters**: URL encoding issues → Should handle safely

### Error Handling Corner Cases

#### Server Error Scenarios
- [ ] **Panic in handler**: Code panic during request → Should catch and return 500
- [ ] **Deadlock**: Rare concurrent access deadlock → Should timeout and recover
- [ ] **Out of memory**: Server under extreme memory pressure → Should fail gracefully
- [ ] **CPU saturation**: Server at 100% CPU → Should continue serving (degraded)

#### Client Error Responses
- [ ] **Error message content**: All 400/401/403/404 errors have descriptive messages → Verify no stack traces
- [ ] **Error message format**: Consistent JSON error format → `{"error": "message"}`
- [ ] **Error logging**: All errors logged to server console → Verify log presence

### Performance Corner Cases

#### Request Load
- [ ] **100 concurrent requests**: Different endpoints → Verify all succeed
- [ ] **1000 sequential requests**: Same endpoint → Verify consistent response times
- [ ] **Request bursts**: 50 requests in 1 second, then idle → Verify no degradation

#### Resource Usage
- [ ] **Memory growth**: After 1000 requests → Verify no memory leak
- [ ] **File descriptor leaks**: After many requests → Verify FDs properly closed
- [ ] **Connection pool leaks**: After many requests → Verify connections returned

## Known Limitations

These are expected behaviors, not bugs:

1. **Case Sensitivity:** Provider names are case-sensitive (must be lowercase: "openai", not "OpenAI")
2. **No Pagination:** Endpoints return all results (acceptable for pilot with <1000 records)
3. **Partial Authentication:** Only GET /api/tokens (list_tokens) requires JWT auth; other FR-7/8/9/10 endpoints defer auth to future work
4. **COALESCE Behavior:** Non-existent project_id/provider returns 200 OK with zeros (not 404)

## Manual Test Results Template

When executing manual tests, document results in `tests/manual/-test_results.md`:

```markdown
# Manual Test Execution - [Date]

## Test 1.1: Server Startup with Fresh Database
- **Pass/Fail:** PASS
- **Notes:** Server started in 1.2s, database created successfully
- **Issues:** None

## Test 2.1: Cross-Token Aggregation
- **Pass/Fail:** FAIL
- **Notes:** Provider breakdown incorrect
- **Issues:** OpenAI count includes Anthropic tokens (bug #123)
- **Reproducing Test:** Added to tests/usage/aggregate.rs:456
```

## Adding Reproducing Tests

For every bug found during manual testing:

1. Create minimal reproducing test in appropriate test file
2. Verify test FAILS (proves it catches the bug)
3. Fix implementation
4. Verify test PASSES
5. Document fix with 5-section test comment + 3-field source comment

## Cleanup

After manual testing:

1. Stop server
2. Delete test database: `rm iron_control_api.db`
3. Clean up any temporary files in `tests/manual/-*`
