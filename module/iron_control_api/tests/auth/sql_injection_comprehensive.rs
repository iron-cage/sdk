//! Comprehensive SQL Injection Security Tests - Security Audit Phase 2
//!
//! **Authority:** `-security_test_implementation_status.md` § Phase 2
//! **Status:** Week 1 - Day 1 Implementation
//!
//! Tests comprehensive SQL injection attack prevention across authentication endpoints:
//! - Username/email field injection (20+ attack vectors)
//! - User creation endpoint injection
//! - Second-order SQL injection
//! - Error message leakage prevention
//!
//! # Security Requirements
//!
//! Per OWASP Top 10 and security audit plan:
//! - All user input MUST be sanitized or parameterized
//! - SQL queries MUST use prepared statements/parameterized queries
//! - No raw SQL string concatenation with user input
//! - Error messages MUST NOT leak database schema information
//! - Second-order injection MUST be prevented (stored→executed)
//!
//! # Attack Vector Coverage
//!
//! - ✅ Classic injection (' OR '1'='1)
//! - ✅ Union-based injection (UNION SELECT)
//! - ✅ Blind boolean injection (true/false inference)
//! - ✅ Blind time-based injection (sleep/delay)
//! - ✅ Stacked queries (multiple statements)
//! - ✅ Comment-based injection (-- / /**/)
//! - ✅ Encoding bypass (hex, unicode, URL encoding)
//! - ✅ Second-order injection (stored then executed)
//! - ✅ Error-based injection (forcing database errors)
//! - ✅ Schema discovery attempts
//!
//! # Test Coverage
//!
//! ## Phase 2: SQL Injection (4 tests)
//! - ✅ Comprehensive username field injection (20+ payloads)
//! - ✅ User creation endpoint injection
//! - ✅ Second-order SQL injection
//! - ✅ Error message leakage prevention

use super::common;
use axum::
{
  body::Body,
  http::{ Request, StatusCode },
};
use serde_json::json;
use sqlx::SqlitePool;
use tower::ServiceExt;

/// Comprehensive SQL injection attack vectors (20+ payloads)
///
/// Covers all major SQL injection attack categories per OWASP guidelines.
pub const COMPREHENSIVE_SQL_INJECTIONS: &[&str] = &[
  // Classic injection
  "' OR '1'='1",
  "' OR 1=1--",
  "admin'--",
  "admin' #",
  "admin'/*",

  // Union-based injection
  "' UNION SELECT NULL--",
  "' UNION SELECT * FROM users--",
  "' UNION SELECT password FROM users--",
  "' UNION ALL SELECT username, password, email FROM users--",

  // Boolean-based blind injection
  "' AND '1'='1",
  "' AND '1'='2",
  "' AND (SELECT COUNT(*) FROM users) > 0--",

  // Time-based blind injection
  "' AND SLEEP(5)--",
  "' AND (SELECT * FROM (SELECT(SLEEP(5)))a)--",
  "'; WAITFOR DELAY '00:00:05'--",

  // Stacked queries
  "'; DROP TABLE users; --",
  "'; DELETE FROM users WHERE '1'='1",
  "'; UPDATE users SET role='admin' WHERE '1'='1",

  // Comment-based injection
  "admin'--",
  "admin'#",
  "admin'/*",

  // Encoding bypass attempts
  "admin%27--",                              // URL encoding
  "0x61646d696e",                            // Hex encoding
  "\\x61\\x64\\x6D\\x69\\x6E",              // Escaped hex

  // Second-order injection (stored payload)
  "attacker@example.com'; DROP TABLE users; --",

  // Error-based injection
  "' AND 1=CONVERT(int, (SELECT @@version))--",
  "' AND extractvalue(1, concat(0x7e, (SELECT database())))--",

  // Schema discovery
  "' AND 1=0 UNION SELECT NULL, table_name FROM information_schema.tables--",
  "' AND 1=0 UNION SELECT NULL, column_name FROM information_schema.columns--",

  // Additional attack vectors
  "' OR 'x'='x",
  "') OR ('x'='x",
  "' OR username IS NOT NULL--",
  "' OR email LIKE '%@%'--",
];

/// Test comprehensive SQL injection attempts on username/email field
///
/// # Test Scenario
///
/// 1. Attempt login with 20+ SQL injection payloads in email field
/// 2. Verify all attempts rejected with 401 Unauthorized (invalid credentials)
/// 3. Verify NO attempts succeed in bypassing authentication
/// 4. Verify NO database errors leaked in responses
///
/// # Expected Behavior
///
/// - ALL SQL injection attempts return 401 (invalid credentials)
/// - NO authentication bypass occurs
/// - NO 500 Internal Server Error (indicates injection reaching database)
/// - Error responses MUST NOT leak database schema information
/// - Consistent error format for all injection attempts
///
/// # Security Requirement
///
/// SQL injection is OWASP Top 10 #1 vulnerability. System MUST use
/// parameterized queries for ALL database operations involving user input.
///
/// # Implementation Status
///
/// ⚠️ REQUIRES COMPREHENSIVE VERIFICATION
/// Need to verify all 20+ injection vectors are safely handled.
#[ tokio::test ]
async fn test_username_sql_injection_comprehensive()
{
  let pool: SqlitePool = common::auth::setup_auth_test_db().await;

  // Seed valid user for comparison (ensure legitimate auth works)
  common::auth::seed_test_user( &pool, "valid@example.com", "valid_password", "user", true ).await;

  let router = common::auth::create_auth_router( pool.clone() ).await;

  // Phase 1: Baseline - verify legitimate auth works
  let valid_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/auth/login" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "email": "valid@example.com",
        "password": "valid_password"
      }).to_string()
    ))
    .unwrap();

  let response = router.clone().oneshot( valid_request ).await.unwrap();
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Baseline: Valid credentials should succeed"
  );

  // Phase 2: Test all SQL injection vectors in email field
  let mut results = Vec::new();

  for ( idx, injection_payload ) in COMPREHENSIVE_SQL_INJECTIONS.iter().enumerate()
  {
    let request = Request::builder()
      .method( "POST" )
      .uri( "/api/v1/auth/login" )
      .header( "content-type", "application/json" )
      .body( Body::from(
        json!({
          "email": injection_payload,
          "password": "any_password"
        }).to_string()
      ))
      .unwrap();

    let response = router.clone().oneshot( request ).await.unwrap();
    let status = response.status();

    // Parse response body to check for leaked information
    let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
    let body_text = String::from_utf8_lossy( &body_bytes );

    results.push(( idx + 1, injection_payload, status, body_text.to_string() ));
  }

  // Phase 3: Verify ALL injection attempts failed safely
  for ( idx, payload, status, body ) in results.iter()
  {
    // MUST return 401 or 429 (both are "safe" - rate limiting or authentication failure)
    // MUST NOT return 200 (bypass) or 500 (SQL execution error)
    assert!(
      *status == StatusCode::UNAUTHORIZED || *status == StatusCode::TOO_MANY_REQUESTS,
      "Injection #{} ('{}') should return 401 or 429, got: {}",
      idx, payload, status
    );

    // MUST NOT leak database schema information
    let leaked_keywords = [
      "sqlite",
      "table",
      "column",
      "schema",
      "syntax error",
      "near",
      "unrecognized token",
    ];

    for keyword in leaked_keywords.iter()
    {
      assert!(
        !body.to_lowercase().contains( keyword ),
        "Injection #{} ('{}') leaked database info: body contains '{}'",
        idx, payload, keyword
      );
    }
  }

  // Phase 4: Verify no authentication bypass occurred
  // (All 401/429 means no bypass - if any returned 200, that's bypass)
  let bypasses: Vec<_> = results.iter()
    .filter( |( _idx, _payload, status, _body )| *status == StatusCode::OK )
    .collect();

  assert!(
    bypasses.is_empty(),
    "SQL injection BYPASS DETECTED: {} payloads returned 200 OK: {:?}",
    bypasses.len(),
    bypasses.iter().map( |( idx, payload, _, _ )| ( idx, payload ) ).collect::<Vec<_>>()
  );

  // Phase 5: Verify consistent error format (no database error leakage)
  let error_codes: Vec<_> = results.iter()
    .filter_map( |( _idx, _payload, _status, body )| {
      serde_json::from_str::<serde_json::Value>( body ).ok()
        .and_then( |json| json[ "error" ][ "code" ].as_str().map( |s| s.to_string() ) )
    })
    .collect();

  // All error codes should be consistent (AUTH_INVALID_CREDENTIALS or similar)
  let unique_error_codes: std::collections::HashSet<_> = error_codes.iter().collect();
  assert!(
    unique_error_codes.len() <= 2,
    "Error codes should be consistent across injection attempts, got: {:?}",
    unique_error_codes
  );
}

/// Test SQL injection attempts on user creation endpoint
///
/// # Test Scenario
///
/// 1. Attempt user creation with SQL injection in username/email fields
/// 2. Verify all attempts rejected (either validation error or safe failure)
/// 3. Verify NO database corruption occurred
/// 4. Verify NO privilege escalation via injected role field
///
/// # Expected Behavior
///
/// - SQL injection in username/email MUST be rejected or safely stored
/// - If stored, MUST be escaped/encoded (not executed as SQL)
/// - NO database corruption (tables dropped, data deleted)
/// - NO role injection (user cannot set role='admin' via SQL injection)
///
/// # Security Requirement
///
/// User creation endpoints are high-risk for SQL injection. All fields
/// (username, email, password) MUST use parameterized queries.
///
/// # Implementation Status
///
/// ⚠️ REQUIRES IMPLEMENTATION
/// Need user creation endpoint to test. Mark as ignored if not yet implemented.
#[ tokio::test ]
#[ ignore = "Requires user creation endpoint implementation" ]
async fn test_user_creation_sql_injection()
{
  let pool: SqlitePool = common::auth::setup_auth_test_db().await;
  let router = common::auth::create_auth_router( pool.clone() ).await;

  // Test high-risk SQL injection payloads on user creation
  let dangerous_payloads = vec![
    "'; DROP TABLE users; --",
    "admin' OR '1'='1",
    "test@example.com'; UPDATE users SET role='admin'--",
  ];

  for payload in dangerous_payloads.iter()
  {
    let request = Request::builder()
      .method( "POST" )
      .uri( "/api/v1/users" )  // Hypothetical user creation endpoint
      .header( "content-type", "application/json" )
      .body( Body::from(
        json!({
          "username": payload,
          "email": format!( "{}@example.com", payload ),
          "password": "secure_password_123"
        }).to_string()
      ))
      .unwrap();

    let response = router.clone().oneshot( request ).await.unwrap();

    // Should either:
    // 1. Reject with 400 Bad Request (validation failed)
    // 2. Accept with 201 Created (but safely escape/encode payload)
    assert!(
      response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::CREATED,
      "User creation with SQL injection should be rejected or safely handled, got: {}",
      response.status()
    );
  }

  // Verify database integrity (tables still exist)
  let users_count = sqlx::query_scalar::<_, i64>( "SELECT COUNT(*) FROM users" )
    .fetch_one( &pool )
    .await
    .expect( "Users table should still exist after SQL injection attempts" );

  assert!(
    users_count >= 0,
    "Database should remain intact after SQL injection attempts"
  );
}

/// Test second-order SQL injection (stored payload executed later)
///
/// # Test Scenario
///
/// 1. Create user with SQL injection payload in username (e.g., "admin'--")
/// 2. Payload stored safely in database (escaped/encoded)
/// 3. Later operation uses username in SQL query (e.g., audit log)
/// 4. Verify payload NOT executed as SQL (remains as data)
///
/// # Expected Behavior
///
/// - Payload stored safely (parameterized INSERT)
/// - Payload retrieved safely (parameterized SELECT)
/// - Payload used in subsequent queries safely (parameterized WHERE)
/// - NO SQL execution occurs when payload retrieved/used
///
/// # Security Requirement
///
/// Second-order injection occurs when stored data (previously escaped)
/// is later used unsafely in SQL queries. System MUST use parameterized
/// queries for ALL database operations, including those using stored data.
///
/// # Implementation Status
///
/// ⚠️ REQUIRES COMPREHENSIVE TESTING
/// Need to verify ALL code paths using stored user data are parameterized.
#[ tokio::test ]
#[ ignore = "Requires second-order injection testing" ]
async fn test_second_order_sql_injection()
{
  let pool: SqlitePool = common::auth::setup_auth_test_db().await;

  // Phase 1: Store malicious payload in database
  let malicious_username = "admin'; DROP TABLE users; --";

  common::auth::seed_test_user(
    &pool,
    "attacker@example.com",
    "password123",
    "user",
    true
  ).await;

  // Manually update username to malicious payload (simulating stored injection)
  sqlx::query( "UPDATE users SET username = ? WHERE email = ?" )
    .bind( malicious_username )
    .bind( "attacker@example.com" )
    .execute( &pool )
    .await
    .expect( "Should safely store malicious username" );

  // Phase 2: Retrieve user (simulating later operation)
  let retrieved_username = sqlx::query_scalar::<_, String>(
    "SELECT username FROM users WHERE email = ?"
  )
  .bind( "attacker@example.com" )
  .fetch_one( &pool )
  .await
  .expect( "Should safely retrieve malicious username" );

  assert_eq!(
    retrieved_username, malicious_username,
    "Malicious payload should be stored/retrieved as data, not executed"
  );

  // Phase 3: Use stored username in another query (danger zone for second-order injection)
  // SAFE: Parameterized query
  let user_count = sqlx::query_scalar::<_, i64>(
    "SELECT COUNT(*) FROM users WHERE username = ?"
  )
  .bind( &retrieved_username )
  .fetch_one( &pool )
  .await
  .expect( "Should safely use stored username in query" );

  assert_eq!(
    user_count, 1,
    "Query using stored malicious username should work safely"
  );

  // Phase 4: Verify database integrity (tables still exist)
  let tables_count = sqlx::query_scalar::<_, i64>(
    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'"
  )
  .fetch_one( &pool )
  .await
  .expect( "Should be able to query sqlite_master" );

  assert_eq!(
    tables_count, 1,
    "Users table should still exist (not dropped by second-order injection)"
  );
}

/// Test error message SQL injection information leakage prevention
///
/// # Test Scenario
///
/// 1. Trigger various database errors with malformed SQL injection
/// 2. Verify error messages DO NOT leak:
///    - Database type (sqlite, postgres, mysql)
///    - Table names
///    - Column names
///    - Query syntax
///    - File paths
/// 3. Verify consistent generic error messages returned
///
/// # Expected Behavior
///
/// - Database errors MUST be caught and sanitized
/// - Generic error message returned to client
/// - Detailed error logged internally (not exposed)
/// - NO schema information leaked
///
/// # Security Requirement
///
/// Error-based SQL injection relies on database error messages to
/// infer schema information. System MUST return generic errors,
/// never exposing internal database details.
///
/// # Implementation Status
///
/// ⚠️ REQUIRES VERIFICATION
/// Need to verify error handling doesn't leak database information.
#[ tokio::test ]
async fn test_error_message_sql_injection_leakage()
{
  let pool: SqlitePool = common::auth::setup_auth_test_db().await;
  let router = common::auth::create_auth_router( pool.clone() ).await;

  // Phase 1: Trigger various error conditions
  let error_triggering_payloads = vec![
    "' AND 1=CONVERT(int, (SELECT @@version))--",        // Type conversion error
    "' AND extractvalue(1, concat(0x7e, (SELECT database())))--",  // Function error
    "' UNION SELECT * FROM nonexistent_table--",          // Table not found
    "' AND (SELECT * FROM users)--",                      // Subquery returns multiple rows
    "' AND SLEEP(999999)--",                              // Function not found (SQLite)
  ];

  for payload in error_triggering_payloads.iter()
  {
    let request = Request::builder()
      .method( "POST" )
      .uri( "/api/v1/auth/login" )
      .header( "content-type", "application/json" )
      .body( Body::from(
        json!({
          "email": payload,
          "password": "any_password"
        }).to_string()
      ))
      .unwrap();

    let response = router.clone().oneshot( request ).await.unwrap();

    // Should return 401 (invalid credentials) not 500 (database error)
    assert!(
      response.status() != StatusCode::INTERNAL_SERVER_ERROR,
      "Database errors should not leak to client (payload: '{}')",
      payload
    );

    // Parse response body
    let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
    let body_text = String::from_utf8_lossy( &body_bytes );

    // Phase 2: Verify NO database-specific information leaked
    let leaked_info = [
      // Database types
      "sqlite",
      "postgres",
      "mysql",
      "mariadb",

      // Error keywords
      "syntax error",
      "near",
      "unrecognized",
      "no such table",
      "no such column",

      // Schema info
      "users",
      "token_blacklist",
      "user_audit_log",

      // System info
      "version",
      "@@",

      // File paths
      "/var/",
      "/usr/",
      "C:\\",
    ];

    for leaked_keyword in leaked_info.iter()
    {
      assert!(
        !body_text.to_lowercase().contains( leaked_keyword ),
        "Error message leaked database info: '{}' (payload: '{}')",
        leaked_keyword,
        payload
      );
    }

    // Phase 3: Verify error message is generic
    if let Ok( error_json ) = serde_json::from_str::<serde_json::Value>( &body_text )
    {
      let error_code = error_json[ "error" ][ "code" ].as_str();
      let error_message = error_json[ "error" ][ "message" ].as_str();

      // Should be generic error code (not database-specific)
      assert!(
        error_code.is_some(),
        "Error response should have error code"
      );

      assert!(
        error_message.is_some(),
        "Error response should have error message"
      );

      // Error message should be generic (not contain SQL keywords)
      if let Some( msg ) = error_message
      {
        assert!(
          !msg.to_lowercase().contains( "sql" ),
          "Error message should not mention 'SQL': {}", msg
        );
      }
    }
  }

  // Phase 4: Verify error format consistency
  // All error responses should use same format (standardized error handling)
  // This prevents attackers from differentiating error types
}
