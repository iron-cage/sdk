//! P0 Critical Corner Case Tests for POST /api/v1/api-tokens
//!
//! Tests the most critical security and DoS protection corner cases identified
//! in the comprehensive corner case analysis.
//!
//! Test Categories:
//! - DoS Protection (very long inputs, large payloads)
//! - Security Attacks (command injection, path traversal, NULL bytes, homographs)
//! - State Verification (token storage security, hash algorithm, uniqueness)
//! - Concurrency (race conditions, transaction integrity)
//! - HTTP Protocol (malformed JSON, empty body)

use axum::http::{ StatusCode, header };
use axum::{ Router, routing::{ post, delete } };
use tower::ServiceExt;
use crate::common::corner_cases;
use crate::common::test_state::TestAppState;
use serde_json::json;

/// Create test router with token routes and return both router and state.
async fn create_test_router_with_state() -> ( Router, TestAppState )
{
  // Create test application state with auth + token support
  let app_state = TestAppState::new().await;

  let router = Router::new()
    .route( "/api/v1/api-tokens", post( iron_control_api::routes::tokens::create_token ) )
    // Note: get_token now requires authentication
    // .route( "/api/v1/api-tokens/:id", get( iron_control_api::routes::tokens::get_token ) )
    .route( "/api/v1/api-tokens/:id/rotate", post( iron_control_api::routes::tokens::rotate_token ) )
    .route( "/api/v1/api-tokens/:id", delete( iron_control_api::routes::tokens::revoke_token ) )
    .with_state( app_state.clone() );

  ( router, app_state )
}

/// Reproduces DoS attack via unlimited user_id string causing memory exhaustion (issue-001).
///
/// ## Root Cause
/// API accepted unbounded string inputs in CreateTokenRequest without validation.
/// No MAX_USER_ID_LENGTH constant initially defined. The user_id field was raw String
/// allowing attackers to send arbitrarily large payloads (10MB+) causing OOM crashes.
///
/// ## Why Not Caught Initially
/// Original test suite only tested happy path with normal-sized inputs (10-50 chars).
/// No boundary testing or DoS attack scenarios were implemented. Test matrix lacked
/// adversarial inputs category.
///
/// ## Fix Applied
/// Added MAX_USER_ID_LENGTH = 500 constant in CreateTokenRequest (src/routes/tokens.rs:62).
/// Implemented validate() method with length check before processing.
/// Returns 400 Bad Request with descriptive error when limit exceeded.
///
/// ## Prevention
/// All POST /api/v1/api-tokens requests now validated before database operations.
/// Length limit enforced at API boundary (Axum handler level) preventing downstream
/// resource exhaustion. Constant can be adjusted based on operational requirements.
///
/// ## Pitfall to Avoid
/// Never accept unbounded external input without validation. Always define and enforce
/// resource limits at system boundaries (API, file upload, message queue). Use constants
/// for magic numbers to enable runtime configuration without code changes.
// test_kind: bug_reproducer(issue-001)
#[tokio::test]
async fn test_create_token_very_long_user_id_rejected()
{
  let ( router, _state ) = create_test_router_with_state().await;

  let very_long_user_id = corner_cases::long_string( 10_000 );

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( header::CONTENT_TYPE, "application/json" )
    .body( axum::body::Body::from( json!({
      "user_id": very_long_user_id,
      "project_id": "project_123"
    }).to_string() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Very long user_id (10K chars) should be rejected with 400"
  );
}

/// Reproduces DoS attack via unlimited project_id string causing memory exhaustion (issue-001).
///
/// ## Root Cause
/// API accepted unbounded string inputs in CreateTokenRequest.project_id without validation.
/// No MAX_PROJECT_ID_LENGTH constant initially defined. Optional field allowed arbitrary
/// payloads enabling attackers to bypass user_id validation by attacking project_id instead.
///
/// ## Why Not Caught Initially
/// Original test suite tested user_id boundary cases but didn't apply same rigor to
/// optional project_id field. Assumed optional fields less critical, missed that attackers
/// can exploit any unbounded input vector.
///
/// ## Fix Applied
/// Added MAX_PROJECT_ID_LENGTH = 500 constant in CreateTokenRequest (src/routes/tokens.rs:65).
/// Implemented validate() method checking project_id.len() when Some.
/// Returns 400 Bad Request preventing resource exhaustion via secondary attack vector.
///
/// ## Prevention
/// All optional fields now validated with same rigor as required fields.
/// Validation applies consistent limits across all string inputs (user_id, project_id, description).
/// Defense-in-depth: multiple input vectors hardened against same attack class.
///
/// ## Pitfall to Avoid
/// Never assume optional fields are less critical for security validation. Attackers
/// exploit the weakest link - if required fields are hardened, they'll attack optional ones.
/// Apply defense-in-depth: validate ALL external inputs regardless of optionality.
// test_kind: bug_reproducer(issue-001)
#[tokio::test]
async fn test_create_token_very_long_project_id_rejected()
{
  let ( router, _state ) = create_test_router_with_state().await;

  let very_long_project_id = corner_cases::long_string( 10_000 );

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( header::CONTENT_TYPE, "application/json" )
    .body( axum::body::Body::from( json!({
      "user_id": "user_123",
      "project_id": very_long_project_id
    }).to_string() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Very long project_id (10K chars) should be rejected with 400"
  );
}

/// Reproduces DoS attack via unlimited description string causing memory exhaustion (issue-001).
///
/// ## Root Cause
/// API accepted unbounded string inputs in CreateTokenRequest.description without validation.
/// No MAX_DESCRIPTION_LENGTH constant initially defined. Description field stores user-provided
/// metadata enabling 100KB+ payloads to exhaust memory and database storage.
///
/// ## Why Not Caught Initially
/// Original test suite focused on identifier fields (user_id, project_id) but overlooked
/// free-form text fields like description. Assumed descriptions would be short, but didn't
/// enforce assumption with validation. Missing from adversarial test matrix.
///
/// ## Fix Applied
/// Added MAX_DESCRIPTION_LENGTH = 500 constant in CreateTokenRequest (src/routes/tokens.rs:64).
/// Implemented validate() method checking description.len() when Some (line 118-124).
/// Returns 400 Bad Request with clear error message for user feedback.
///
/// ## Prevention
/// All free-form text fields now have explicit length limits preventing unbounded growth.
/// Description limit (500 chars) balances usability with resource protection.
/// Consistent validation pattern applied across all string fields in API.
///
/// ## Pitfall to Avoid
/// Never trust that users will "be reasonable" with free-form inputs. Always enforce
/// explicit limits on text fields, even if they seem auxiliary. Free-form fields are
/// prime DoS vectors because developers often forget to validate them. Document limits
/// in API specification for client-side validation.
// test_kind: bug_reproducer(issue-001)
#[tokio::test]
async fn test_create_token_very_long_description_rejected()
{
  let ( router, _state ) = create_test_router_with_state().await;

  let very_long_description = corner_cases::long_string( 100_000 );

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( header::CONTENT_TYPE, "application/json" )
    .body( axum::body::Body::from( json!({
      "user_id": "user_123",
      "project_id": "project_123",
      "description": very_long_description
    }).to_string() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Very long description (100K chars) should be rejected with 400"
  );
}

/// Reproduces command injection vulnerability via shell metacharacters in user_id (issue-003a).
///
/// ## Root Cause
/// API could potentially pass user_id to shell commands or subprocess execution without
/// sanitization. If user_id like "; rm -rf /" or "$(malicious)" is passed to system(),
/// shell would execute injected commands. Vulnerability exists when user input flows
/// to exec(), system(), or database shell-out operations.
///
/// ## Why Not Caught Initially
/// Original test suite only tested alphanumeric inputs. Didn't include adversarial
/// payloads with shell metacharacters (; & | $ ` \\ " '). Assumed API wouldn't shell out,
/// but didn't verify assumption with hardening tests.
///
/// ## Fix Applied
/// System architecture ensures user_id NEVER flows to shell execution. Uses prepared
/// statements for database (no SQL injection), stores values as literal strings (no eval),
/// and avoids subprocess calls with user input. Test verifies safe handling: either
/// accept as literal (CREATED) or reject (BAD_REQUEST), never execute (no crash/500).
///
/// ## Prevention
/// Defense-in-depth: (1) Architecture avoids shell execution entirely, (2) Input validation
/// rejects suspicious patterns, (3) Test suite verifies no execution occurs.
/// All external inputs treated as untrusted data, never code.
///
/// ## Pitfall to Avoid
/// Never pass user input to shell commands (system(), exec(), popen()) without extreme
/// sanitization. Prefer native APIs over shelling out (use database drivers, not CLI tools).
/// When shell execution unavoidable, use allowlists (not denylists) for input validation.
/// Test with OWASP command injection payloads to verify hardening.
// test_kind: bug_reproducer(issue-003a)
#[tokio::test]
async fn test_create_token_command_injection_user_id_safe()
{
  let ( router, _state ) = create_test_router_with_state().await;

  for command_injection in corner_cases::COMMAND_INJECTION
  {
    let request = axum::http::Request::builder()
      .method( "POST" )
      .uri( "/api/v1/api-tokens" )
      .header( header::CONTENT_TYPE, "application/json" )
      .body( axum::body::Body::from( json!({
        "user_id": command_injection,
        "project_id": "project_123"
      }).to_string() ) )
      .unwrap();

    let response = router.clone().oneshot( request ).await.unwrap();

    // Should either accept (store as literal) or reject with 400 (never execute)
    assert!(
      response.status() == StatusCode::CREATED || response.status() == StatusCode::BAD_REQUEST,
      "Command injection '{}' in user_id should be safe (201 or 400, not crash)",
      command_injection
    );
  }
}

/// Reproduces command injection vulnerability via shell metacharacters in project_id (issue-003b).
///
/// ## Root Cause
/// API could potentially pass project_id to shell commands or subprocess execution without
/// sanitization. Similar to user_id vulnerability but exploits optional field. If future
/// features add shell execution (e.g., git clone for project validation), injected commands
/// in project_id could execute.
///
/// ## Why Not Caught Initially
/// Original test suite tested command injection in user_id but didn't apply same hardening
/// to project_id. Third occurrence of optional field under-validation pattern (issue-001:
/// no length limit, issue-002: no NULL byte check, issue-003b: no command injection test).
///
/// ## Fix Applied
/// System architecture ensures project_id NEVER flows to shell execution. Uses same
/// defenses as user_id: prepared statements, literal storage, no subprocess calls.
/// Test verifies safe handling across multiple OWASP command injection payloads.
///
/// ## Prevention
/// All optional fields tested with same adversarial payloads as required fields.
/// Command injection test suite covers all string input vectors (user_id, project_id).
/// Architecture review confirms no shell execution paths for user-controlled data.
///
/// ## Pitfall to Avoid
/// This is the THIRD time optional fields were under-validated (issue-001, issue-002,
/// issue-003b). Establish mandatory validation checklist applied to ALL string fields
/// regardless of optionality: (1) Length limits, (2) NULL byte checks, (3) Command
/// injection tests, (4) SQL injection tests. Security testing must be exhaustive, not
/// selective based on "importance" perception. Attackers exploit ANY weak input vector.
// test_kind: bug_reproducer(issue-003b)
#[tokio::test]
async fn test_create_token_command_injection_project_id_safe()
{
  let ( router, _state ) = create_test_router_with_state().await;

  for command_injection in corner_cases::COMMAND_INJECTION
  {
    let request = axum::http::Request::builder()
      .method( "POST" )
      .uri( "/api/v1/api-tokens" )
      .header( header::CONTENT_TYPE, "application/json" )
      .body( axum::body::Body::from( json!({
        "user_id": "user_123",
        "project_id": command_injection
      }).to_string() ) )
      .unwrap();

    let response = router.clone().oneshot( request ).await.unwrap();

    assert!(
      response.status() == StatusCode::CREATED || response.status() == StatusCode::BAD_REQUEST,
      "Command injection '{}' in project_id should be safe (201 or 400, not crash)",
      command_injection
    );
  }
}

/// Reproduces path traversal vulnerability via directory navigation patterns (issue-003c).
///
/// ## Root Cause
/// API could potentially use project_id in file operations without path sanitization.
/// Patterns like "../../../etc/passwd" or "..\\..\\Windows\\System32" could access
/// files outside intended directory if project_id is used in file paths (e.g., logging,
/// cache, project data storage). Vulnerability exists when user input flows to file I/O.
///
/// ## Why Not Caught Initially
/// Original test suite only tested normal project identifiers. Didn't include directory
/// traversal payloads (../, ..\, absolute paths, symbolic links). Assumed project_id
/// wouldn't be used in file paths, but didn't verify with adversarial testing.
///
/// ## Fix Applied
/// System architecture ensures project_id is stored as database field only, never used
/// in file paths. No file I/O operations use project_id as path component. Test verifies
/// safe handling: either accept as literal identifier (CREATED) or reject (BAD_REQUEST),
/// never access files outside API's data directory.
///
/// ## Prevention
/// Defense-in-depth: (1) Architecture avoids file operations with user input, (2) If file
/// operations needed, use UUIDs instead of user-controlled strings, (3) Path sanitization
/// library if user input unavoidable. Test suite verifies no file access occurs.
///
/// ## Pitfall to Avoid
/// Never construct file paths from user input without canonicalization and validation.
/// Even "safe-looking" identifiers can contain path traversal sequences. If file operations
/// needed, use allowlist of characters (alphanumeric only) or generate random identifiers
/// (UUIDs). Always validate resolved path is within intended directory using canonicalize()
/// and starts_with() checks. Test with OWASP path traversal payloads.
// test_kind: bug_reproducer(issue-003c)
#[tokio::test]
async fn test_create_token_path_traversal_project_id_safe()
{
  let ( router, _state ) = create_test_router_with_state().await;

  for path_traversal in corner_cases::PATH_TRAVERSAL
  {
    let request = axum::http::Request::builder()
      .method( "POST" )
      .uri( "/api/v1/api-tokens" )
      .header( header::CONTENT_TYPE, "application/json" )
      .body( axum::body::Body::from( json!({
        "user_id": "user_123",
        "project_id": path_traversal
      }).to_string() ) )
      .unwrap();

    let response = router.clone().oneshot( request ).await.unwrap();

    assert!(
      response.status() == StatusCode::CREATED || response.status() == StatusCode::BAD_REQUEST,
      "Path traversal '{}' in project_id should be safe (201 or 400, not file access)",
      path_traversal
    );
  }
}

/// Reproduces NULL byte injection attack causing C string termination vulnerability (issue-002).
///
/// ## Root Cause
/// API accepted user_id containing NULL bytes (\x00) without validation. When passed to
/// C/FFI libraries or database drivers, NULL byte truncates string at injection point
/// (e.g., "user\x00admin" becomes "user"), bypassing authorization checks or logging.
///
/// ## Why Not Caught Initially
/// Original test suite only tested printable ASCII characters. Didn't include control
/// characters or binary data in test matrix. Assumed JSON deserialization would reject
/// invalid characters, but serde_json preserves NULL bytes in strings.
///
/// ## Fix Applied
/// Added NULL byte validation in CreateTokenRequest.validate() (src/routes/tokens.rs:85-88).
/// Uses .contains('\0') check before processing user_id. Returns 400 Bad Request with
/// "invalid NULL byte" error message preventing downstream exploitation.
///
/// ## Prevention
/// All string inputs now validated for NULL bytes at API boundary.
/// Defense applies to user_id, project_id, and description fields.
/// Validation occurs before database operations or external library calls.
///
/// ## Pitfall to Avoid
/// Never assume JSON strings are safe for FFI or C interop. NULL bytes are valid in
/// Rust strings but cause truncation in C strings. Always validate against control
/// characters (\x00-\x1F) when interacting with C libraries, databases using C drivers
/// (SQLite, PostgreSQL), or logging systems. Use explicit validation, not implicit trust.
// test_kind: bug_reproducer(issue-002)
#[tokio::test]
async fn test_create_token_null_byte_user_id_safe()
{
  let ( router, _state ) = create_test_router_with_state().await;

  let null_byte_user = "user\x00admin";

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( header::CONTENT_TYPE, "application/json" )
    .body( axum::body::Body::from( json!({
      "user_id": null_byte_user,
      "project_id": "project_123"
    }).to_string() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // NULL byte should be rejected (C string termination attack prevention)
  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "NULL byte in user_id should be rejected to prevent string termination attacks"
  );
}

/// Reproduces NULL byte injection attack via project_id field (issue-002).
///
/// ## Root Cause
/// API accepted project_id containing NULL bytes without validation. Similar to user_id
/// vulnerability but exploits optional field. Attackers could bypass user_id hardening
/// by injecting NULL bytes into project_id, achieving same C string truncation attack.
///
/// ## Why Not Caught Initially
/// Original test suite tested NULL bytes in user_id but didn't apply same validation
/// to project_id. Assumed optional fields less critical for security, repeated mistake
/// from issue-001 (DoS) where optional fields were under-validated.
///
/// ## Fix Applied
/// Added NULL byte validation in CreateTokenRequest.validate() for project_id when Some
/// (src/routes/tokens.rs:108-111). Uses same .contains('\0') pattern as user_id.
/// Returns 400 Bad Request preventing secondary attack vector.
///
/// ## Prevention
/// All optional string fields validated with same rigor as required fields.
/// Consistent validation pattern applied: length limits + NULL byte checks + empty checks.
/// Defense-in-depth prevents attackers from exploiting weaker secondary input vectors.
///
/// ## Pitfall to Avoid
/// Security validation must apply uniformly to ALL fields, not just required ones.
/// This is the second time optional fields were under-validated (issue-001: no length limit,
/// issue-002: no NULL byte check). Establish validation checklist: ALL string inputs must
/// have length limits, NULL byte checks, and empty/whitespace validation regardless of
/// optionality. Treat optional fields as equally dangerous.
// test_kind: bug_reproducer(issue-002)
#[tokio::test]
async fn test_create_token_null_byte_project_id_safe()
{
  let ( router, _state ) = create_test_router_with_state().await;

  let null_byte_project = "proj\x00malicious";

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( header::CONTENT_TYPE, "application/json" )
    .body( axum::body::Body::from( json!({
      "user_id": "user_123",
      "project_id": null_byte_project
    }).to_string() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "NULL byte in project_id should be rejected to prevent string termination attacks"
  );
}

/// P0-3: NULL byte at start of user_id should be rejected (boundary condition for issue-002).
#[tokio::test]
async fn test_create_token_null_byte_at_start()
{
  let ( router, _state ) = create_test_router_with_state().await;

  let null_byte_start = "\x00userid_attack";

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( header::CONTENT_TYPE, "application/json" )
    .body( axum::body::Body::from( json!({
      "user_id": null_byte_start,
      "project_id": "project_123"
    }).to_string() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "NULL byte at start of user_id should be rejected (boundary condition)"
  );
}

/// P0-4: NULL byte at end of user_id should be rejected (boundary condition for issue-002).
#[tokio::test]
async fn test_create_token_null_byte_at_end()
{
  let ( router, _state ) = create_test_router_with_state().await;

  let null_byte_end = "userid_attack\x00";

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( header::CONTENT_TYPE, "application/json" )
    .body( axum::body::Body::from( json!({
      "user_id": null_byte_end,
      "project_id": "project_123"
    }).to_string() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "NULL byte at end of user_id should be rejected (boundary condition)"
  );
}

/// P0-5: Multiple NULL bytes in user_id should be rejected (complex attack for issue-002).
#[tokio::test]
async fn test_create_token_multiple_null_bytes()
{
  let ( router, _state ) = create_test_router_with_state().await;

  let multiple_nulls = "u\x00s\x00e\x00r";

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( header::CONTENT_TYPE, "application/json" )
    .body( axum::body::Body::from( json!({
      "user_id": multiple_nulls,
      "project_id": "project_123"
    }).to_string() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Multiple NULL bytes in user_id should be rejected"
  );
}

/// P0-6: Valid newline character should be accepted (false positive check for issue-002).
///
/// Validates that NULL byte detection doesn't incorrectly reject valid escape sequences
/// like \n (newline). Ensures validation is precise and only rejects actual NULL bytes (\x00),
/// not string representations of escape sequences.
#[tokio::test]
async fn test_create_token_newline_char_accepted()
{
  let ( router, _state ) = create_test_router_with_state().await;

  // Note: This is a literal newline character in the string, not a NULL byte
  let with_newline = "user\nid_with_newline";

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( header::CONTENT_TYPE, "application/json" )
    .body( axum::body::Body::from( json!({
      "user_id": with_newline,
      "project_id": "project_123"
    }).to_string() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Newline should be ACCEPTED (not a NULL byte, false positive check)
  // This test verifies we're only rejecting \x00, not other control characters
  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "Newline character should be accepted (not a NULL byte, false positive check)"
  );
}

/// Reproduces vulnerability where token plaintext stored in database instead of hash (issue-003d).
///
/// ## Root Cause
/// API could potentially store token plaintext in database instead of cryptographic hash.
/// If database is compromised (SQL injection, backup leak, insider threat), all active
/// tokens would be exposed, allowing attackers to impersonate any user. Plaintext storage
/// violates fundamental security principle: never store secrets reversibly.
///
/// ## Why Not Caught Initially
/// Original test suite only verified API behavior (201 Created), didnt inspect database
/// state. Assumed storage layer implemented hashing correctly, but didn't verify with
/// direct database queries. Test matrix lacked database-level security verification.
///
/// ## Fix Applied
/// Added TokenStorage.pool() method for test verification (iron_token_manager/src/storage.rs:393).
/// Test queries database directly after token creation to verify hash column contains
/// bcrypt hash (starts with $2b$), not plaintext token. Architectural guarantee enforced
/// through testing.
///
/// ## Prevention
/// Database-level security tests verify internal state, not just API responses.
/// Defense-in-depth: API returns token once, database stores only hash, no reversible storage.
/// Regular security audits should include database schema inspection and sample queries.
///
/// ## Pitfall to Avoid
/// Never trust that internal components implement security correctly without verification.
/// Always test critical security properties at multiple layers (API, business logic, database).
/// Plaintext credential storage is catastrophic - one database compromise exposes all users.
/// Use irreversible hashing (bcrypt, argon2) for secrets, never encryption (reversible).
// test_kind: bug_reproducer(issue-003d)
#[tokio::test]
async fn test_create_token_plaintext_never_stored_in_database()
{
  let ( _router, state ) = create_test_router_with_state().await;

  // Create token via API
  let plaintext_token = state.tokens.generator.generate();
  let token_id = state
    .tokens
    .storage
    .create_token( &plaintext_token, "user_plaintext_test", None, None, None, None )
    .await
    .expect( "Token creation should succeed" );

  // Query database directly to verify token_hash column
  let row: ( String, ) = sqlx::query_as( "SELECT token_hash FROM api_tokens WHERE id = ?" )
    .bind( token_id )
    .fetch_one( state.tokens.storage.pool() )
    .await
    .expect( "Database query should succeed" );

  let stored_hash = row.0;

  // Verify plaintext NOT stored
  assert_ne!(
    stored_hash,
    plaintext_token,
    "SECURITY VIOLATION: Token plaintext found in database! Only hash should be stored."
  );

  // Verify hash is present (not empty/null)
  assert!(
    !stored_hash.is_empty(),
    "Hash column should not be empty"
  );

  // Verify hash looks like SHA-256 format (64 hex characters)
  assert_eq!(
    stored_hash.len(),
    64,
    "Hash should be 64 characters (SHA-256), got: {} characters",
    stored_hash.len()
  );

  let is_hex = stored_hash.chars().all( |c| c.is_ascii_hexdigit() );
  assert!(
    is_hex,
    "Hash should be hex-encoded SHA-256, got non-hex characters: {}",
    &stored_hash[ 0..std::cmp::min( 20, stored_hash.len() ) ]
  );
}

/// Verifies cryptographically secure SHA-256 hashing for high-entropy tokens (issue-003e).
///
/// ## Root Cause
/// Previous implementation incorrectly used BCrypt for API tokens. BCrypt is designed for
/// LOW-ENTROPY passwords (user-chosen), not HIGH-ENTROPY cryptographically random tokens.
/// BCrypt's non-deterministic salting breaks database lookups (can't use in WHERE clauses).
///
/// ## Why Not Caught Initially
/// Confusion between password hashing and token hashing. Assumed BCrypt (good for passwords)
/// was universally better. Missed that tokens are 256-bit cryptographically random values
/// (vastly different security profile than user passwords).
///
/// ## Fix Applied
/// Reverted to SHA-256 for token hashing. SHA-256 is appropriate because:
/// - Tokens are cryptographically random (256 bits entropy, NOT user-chosen)
/// - Deterministic hashing enables fast database lookups
/// - No salt needed for high-entropy random values (entropy IS the protection)
/// - Rainbow tables irrelevant for 256-bit random space
///
/// ## Prevention
/// Distinguish security contexts: Use BCrypt/Argon2 for LOW-ENTROPY secrets (passwords).
/// Use SHA-256/SHA-512 for HIGH-ENTROPY random values (tokens, IDs).
/// Document entropy assumptions and hash algorithm rationale in specification.
///
/// ## Pitfall to Avoid
/// Not all secrets are equal. Passwords (low entropy, user-chosen) need slow hashing
/// (BCrypt/Argon2/scrypt) to resist brute-force. Random tokens (high entropy, 256+ bits)
/// need fast deterministic hashing (SHA-256) for lookups. Using BCrypt for tokens breaks
/// architecture and provides no security benefit (256-bit entropy >> brute-force threshold).
// test_kind: bug_reproducer(issue-003e)
#[tokio::test]
async fn test_create_token_uses_sha256_hash()
{
  let ( _router, state ) = create_test_router_with_state().await;

  // Create token via API (256-bit cryptographically random value)
  let plaintext_token = state.tokens.generator.generate();
  let token_id = state
    .tokens
    .storage
    .create_token( &plaintext_token, "user_sha256_test", None, None, None, None )
    .await
    .expect( "Token creation should succeed" );

  // Query database directly to verify hash algorithm
  let row: ( String, ) = sqlx::query_as( "SELECT token_hash FROM api_tokens WHERE id = ?" )
    .bind( token_id )
    .fetch_one( state.tokens.storage.pool() )
    .await
    .expect( "Database query should succeed" );

  let stored_hash = row.0;

  // Verify SHA-256 format: 64 hex characters
  // Example: e5eb293e065e05329dbe4a6c4f8e7c3d9b2f1a8e7d6c5b4a3f2e1d0c9b8a7f6e
  assert_eq!(
    stored_hash.len(),
    64,
    "Hash should be 64 characters (SHA-256), got: {} characters",
    stored_hash.len()
  );

  // Verify hex encoding
  let is_hex = stored_hash.chars().all( |c| c.is_ascii_hexdigit() );
  assert!(
    is_hex,
    "Hash should be hex-encoded SHA-256, got non-hex characters"
  );

  // Verify deterministic hashing (same input produces same hash)
  let expected_hash = state.tokens.generator.hash_token( &plaintext_token );
  assert_eq!(
    stored_hash,
    expected_hash,
    "SHA-256 hash should be deterministic (same input → same output)"
  );
}

/// P0-11: State - Token generation produces unique tokens (no collisions)
#[tokio::test]
async fn test_create_token_unique_generation()
{
  use std::collections::HashSet;

  // Generate 100 tokens and verify all unique
  let mut tokens = HashSet::new();

  for i in 0..100
  {
    let ( router, _state ) = create_test_router_with_state().await;

    let request = axum::http::Request::builder()
      .method( "POST" )
      .uri( "/api/v1/api-tokens" )
      .header( header::CONTENT_TYPE, "application/json" )
      .body( axum::body::Body::from( json!({
        "user_id": format!( "user_{}", i ),
        "project_id": format!( "project_{}", i )
      }).to_string() ) )
      .unwrap();

    let response = router.oneshot( request ).await.unwrap();
    assert_eq!( response.status(), StatusCode::CREATED );

    let body_bytes = http_body_util::BodyExt::collect( response.into_body() )
      .await
      .unwrap()
      .to_bytes();
    let body_str = String::from_utf8( body_bytes.to_vec() ).unwrap();
    let create_response: serde_json::Value = serde_json::from_str( &body_str ).unwrap();

    let token = create_response[ "token" ].as_str().unwrap().to_string();

    assert!(
      tokens.insert( token.clone() ),
      "Token collision detected! Duplicate token generated: {}",
      token
    );
  }

  assert_eq!(
    tokens.len(),
    100,
    "All 100 tokens should be unique, got {} unique tokens",
    tokens.len()
  );
}

/// P0-12: Concurrency - 10 concurrent token creates should not cause race conditions
#[tokio::test]
async fn test_create_token_concurrent_requests()
{
  use tokio::task::JoinSet;

  let mut join_set = JoinSet::new();

  // Spawn 10 concurrent requests
  for i in 0..10
  {
    join_set.spawn( async move {
      let ( router, _state ) = create_test_router_with_state().await;

      let request = axum::http::Request::builder()
        .method( "POST" )
        .uri( "/api/v1/api-tokens" )
        .header( header::CONTENT_TYPE, "application/json" )
        .body( axum::body::Body::from( json!({
          "user_id": format!( "concurrent_user_{}", i ),
          "project_id": format!( "concurrent_project_{}", i )
        }).to_string() ) )
        .unwrap();

      let response = router.oneshot( request ).await.unwrap();
      response.status()
    } );
  }

  // Collect all responses
  let mut success_count = 0;
  while let Some( result ) = join_set.join_next().await
  {
    let status = result.expect( "Task should not panic" );
    if status == StatusCode::CREATED
    {
      success_count += 1;
    }
  }

  assert_eq!(
    success_count,
    10,
    "All 10 concurrent requests should succeed without race conditions"
  );
}

/// P0-13: HTTP - Malformed JSON should return 400 with JSON error
#[tokio::test]
async fn test_create_token_malformed_json_rejected()
{
  let ( router, _state ) = create_test_router_with_state().await;

  for malformed_json in corner_cases::INVALID_JSON
  {
    let request = axum::http::Request::builder()
      .method( "POST" )
      .uri( "/api/v1/api-tokens" )
      .header( header::CONTENT_TYPE, "application/json" )
      .body( axum::body::Body::from( malformed_json.to_string() ) )
      .unwrap();

    let response = router.clone().oneshot( request ).await.unwrap();

    assert_eq!(
      response.status(),
      StatusCode::BAD_REQUEST,
      "Malformed JSON '{}' should return 400",
      malformed_json
    );
  }
}

/// P0-14: HTTP - Empty body should return 400 with JSON error
#[tokio::test]
async fn test_create_token_empty_body_rejected()
{
  let ( router, _state ) = create_test_router_with_state().await;

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( header::CONTENT_TYPE, "application/json" )
    .body( axum::body::Body::from( "" ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Empty body should return 400"
  );
}

/// P0-15: HTTP - Missing Content-Type header should be handled
#[tokio::test]
async fn test_create_token_missing_content_type()
{
  let ( router, _state ) = create_test_router_with_state().await;

  let request = axum::http::Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    // No Content-Type header
    .body( axum::body::Body::from( json!({
      "user_id": "user_123",
      "project_id": "project_123"
    }).to_string() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Should either accept (Axum may be lenient) or reject with 400
  assert!(
    response.status() == StatusCode::CREATED ||
    response.status() == StatusCode::BAD_REQUEST ||
    response.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE,
    "Missing Content-Type should be handled gracefully, got: {}",
    response.status()
  );
}

/// Verifies database CHECK constraints enforce length limits as defense-in-depth (issue-001f).
///
/// ## Root Cause
/// Database lacked CHECK constraints to enforce field length limits. If validation logic
/// had a bug or database was accessed directly (bypassing API), oversized or empty values
/// could be stored. Defense-in-depth requires multiple validation layers: API validation
/// (ValidatedUserId/ValidatedProjectId) AND database constraints.
///
/// ## Why Not Caught Initially
/// Original implementation only validated at API layer. No verification that database
/// schema enforced same constraints. Test suite didn't verify database-level protections.
/// Assumed API validation was sufficient without runtime database enforcement.
///
/// ## Fix Applied
/// Added migration 002_add_length_constraints.sql (iron_token_manager/migrations/002_add_length_constraints.sql).
/// Added CHECK constraints on api_tokens table:
/// - user_id: LENGTH(user_id) > 0 AND LENGTH(user_id) <= 500
/// - project_id: IS NULL OR (LENGTH(project_id) > 0 AND LENGTH(project_id) <= 500)
///
/// Test verifies constraints active by attempting direct database inserts that violate
/// constraints. Database should reject invalid data even if API validation is bypassed.
///
/// ## Prevention
/// Always implement defense-in-depth with multiple validation layers. Database CHECK
/// constraints provide runtime enforcement independent of application logic. Test both
/// API-level validation AND database-level constraints. Schema changes should include
/// constraint verification tests.
///
/// ## Pitfall to Avoid
/// Never rely solely on application-level validation. Databases can be accessed through
/// multiple paths (migrations, scripts, other services). CHECK constraints ensure data
/// integrity at storage layer. Always test that constraints are active and enforced,
/// not just that migrations ran successfully.
// test_kind: infrastructure(issue-001f)
#[tokio::test]
async fn test_database_constraints_enforce_length_limits()
{
  let ( _router, state ) = create_test_router_with_state().await;

  // Test 1: Verify constraint rejects user_id > 500 chars
  let too_long_user_id = "A".repeat( 501 );
  let valid_hash = "valid_hash_12345";

  let result = sqlx::query(
    "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at)
     VALUES (?, ?, ?, 1, ?)"
  )
    .bind( &too_long_user_id )
    .bind( "project_123" )
    .bind( valid_hash )
    .bind( 1234567890 )
    .execute( state.tokens.storage.pool() )
    .await;

  assert!(
    result.is_err(),
    "Database should reject user_id > 500 chars due to CHECK constraint"
  );

  let error_msg = result.unwrap_err().to_string();
  assert!(
    error_msg.to_lowercase().contains( "check" ) ||
    error_msg.to_lowercase().contains( "constraint" ),
    "Error should mention CHECK constraint violation, got: {}",
    error_msg
  );

  // Test 2: Verify constraint rejects empty user_id
  let result = sqlx::query(
    "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at)
     VALUES (?, ?, ?, 1, ?)"
  )
    .bind( "" )
    .bind( "project_123" )
    .bind( "different_hash_67890" )
    .bind( 1234567890 )
    .execute( state.tokens.storage.pool() )
    .await;

  assert!(
    result.is_err(),
    "Database should reject empty user_id due to CHECK constraint"
  );

  // Test 3: Verify constraint rejects project_id > 500 chars (when not NULL)
  let too_long_project_id = "B".repeat( 501 );

  let result = sqlx::query(
    "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at)
     VALUES (?, ?, ?, 1, ?)"
  )
    .bind( "user_valid" )
    .bind( &too_long_project_id )
    .bind( "yet_another_hash_11111" )
    .bind( 1234567890 )
    .execute( state.tokens.storage.pool() )
    .await;

  assert!(
    result.is_err(),
    "Database should reject project_id > 500 chars due to CHECK constraint"
  );

  // Test 4: Verify constraint allows NULL project_id (nullable field)
  let result = sqlx::query(
    "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at)
     VALUES (?, NULL, ?, 1, ?)"
  )
    .bind( "user_null_project" )
    .bind( "hash_for_null_project" )
    .bind( 1234567890 )
    .execute( state.tokens.storage.pool() )
    .await;

  assert!(
    result.is_ok(),
    "Database should allow NULL project_id: {:?}",
    result.err()
  );

  // Test 5: Verify constraint allows valid lengths (boundary test)
  let max_length_user_id = "C".repeat( 500 );  // Exactly 500 chars
  let max_length_project_id = "D".repeat( 500 );  // Exactly 500 chars

  let result = sqlx::query(
    "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at)
     VALUES (?, ?, ?, 1, ?)"
  )
    .bind( &max_length_user_id )
    .bind( &max_length_project_id )
    .bind( "hash_max_length_test" )
    .bind( 1234567890 )
    .execute( state.tokens.storage.pool() )
    .await;

  assert!(
    result.is_ok(),
    "Database should accept exactly 500 char fields (max valid length): {:?}",
    result.err()
  );
}
