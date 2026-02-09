//! Analytics endpoint IC Token rotation test (M5)
//!
//! Verifies that POST /api/v1/analytics/events rejects rotated IC tokens
//! and accepts new tokens after regeneration.
//!
//! # Test Matrix
//!
//! | Test Case | Token State | Expected |
//! |-----------|-------------|----------|
//! | `test_analytics_ingestion_with_rotated_token` | Old token after regeneration | 401 Unauthorized |
//! | (same test, step 4) | New token after regeneration | 202 Accepted |
//!
//! # Authority
//! - PR #44 review finding M5: Analytics endpoint missing explicit tests

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use iron_control_api::{
    ic_token::{sha256_hash, IcTokenClaims, IcTokenManager},
    routes::analytics::{post_event, AnalyticsState},
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower::ServiceExt;

/// Create in-memory database with agents + analytics_events tables
async fn setup_analytics_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("LOUD FAILURE: Should connect to in-memory SQLite");

    // agents table — needed by validate_ic_token_runtime
    sqlx::query(
        "CREATE TABLE agents (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            providers TEXT NOT NULL DEFAULT '[]',
            created_at INTEGER NOT NULL DEFAULT 0,
            ic_token_hash TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("LOUD FAILURE: Should create agents table");

    // analytics_events table — needed by post_event
    sqlx::query(
        "CREATE TABLE analytics_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id TEXT NOT NULL,
            timestamp_ms INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            model TEXT NOT NULL,
            provider TEXT NOT NULL,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            cost_micros INTEGER NOT NULL DEFAULT 0,
            agent_id INTEGER,
            provider_id TEXT,
            error_code TEXT,
            error_message TEXT,
            received_at INTEGER NOT NULL,
            UNIQUE(agent_id, event_id)
        )",
    )
    .execute(&pool)
    .await
    .expect("LOUD FAILURE: Should create analytics_events table");

    pool
}

/// Seed agent into database
async fn seed_agent(pool: &SqlitePool, agent_id: i64) {
    sqlx::query("INSERT INTO agents (id, name, providers, created_at) VALUES (?, ?, '[]', 0)")
        .bind(agent_id)
        .bind(format!("test_agent_{}", agent_id))
        .execute(pool)
        .await
        .expect("LOUD FAILURE: Should insert agent");
}

/// Generate IC token and store its hash in agents table
async fn create_ic_token(pool: &SqlitePool, agent_id: i64, manager: &IcTokenManager) -> String {
    let claims = IcTokenClaims::new(
        format!("agent_{}", agent_id),
        format!("budget_{}", agent_id),
        vec!["llm:call".to_string()],
        None,
    );
    let token = manager
        .generate_token(&claims)
        .expect("LOUD FAILURE: Should generate IC Token");
    let token_hash = sha256_hash(&token);

    sqlx::query("UPDATE agents SET ic_token_hash = ? WHERE id = ?")
        .bind(&token_hash)
        .bind(agent_id)
        .execute(pool)
        .await
        .expect("LOUD FAILURE: Should store ic_token_hash");

    token
}

/// Build analytics router with the given pool and manager
fn create_analytics_router(pool: SqlitePool, manager: Arc<IcTokenManager>) -> Router {
    let state = AnalyticsState {
        pool,
        ic_token_manager: manager,
    };
    Router::new()
        .route("/api/v1/analytics/events", post(post_event))
        .with_state(state)
}

/// Build valid analytics event JSON body
fn event_body(ic_token: &str, event_id: &str) -> String {
    serde_json::json!({
        "ic_token": ic_token,
        "event_id": event_id,
        "timestamp_ms": chrono::Utc::now().timestamp_millis(),
        "event_type": "llm_request_completed",
        "model": "gpt-4",
        "provider": "openai",
        "input_tokens": 100,
        "output_tokens": 50,
        "cost_micros": 500
    })
    .to_string()
}

/// M5: POST /api/v1/analytics/events rejects rotated IC token
///
/// # Scenario
///
/// 1. Agent gets Token A → hash stored → POST event succeeds (202)
/// 2. Admin regenerates → Token B generated → new hash stored
/// 3. Agent tries POST with old Token A → 401 Unauthorized
/// 4. Agent uses new Token B → POST event succeeds (202)
///
/// # Risk
/// HIGH - Rotated tokens must be immediately rejected across all endpoints
#[tokio::test]
async fn test_analytics_ingestion_with_rotated_token() {
    let pool = setup_analytics_db().await;
    let agent_id = 700i64;
    seed_agent(&pool, agent_id).await;

    let manager = Arc::new(IcTokenManager::new("analytics_test_secret".to_string()));

    // Step 1: Generate Token A, store hash, verify POST works
    let token_a = create_ic_token(&pool, agent_id, &manager).await;

    let router = create_analytics_router(pool.clone(), manager.clone());
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/analytics/events")
                .header("content-type", "application/json")
                .body(Body::from(event_body(&token_a, "evt_001")))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "LOUD FAILURE: Token A should be accepted before regeneration"
    );

    // Step 2: Regenerate — generate Token B and overwrite hash
    let claims_b = IcTokenClaims::new(
        format!("agent_{}", agent_id),
        format!("budget_{}", agent_id),
        vec!["llm:call".to_string()],
        None,
    );
    let token_b = manager
        .generate_token(&claims_b)
        .expect("LOUD FAILURE: Should generate Token B");
    let new_hash = sha256_hash(&token_b);

    sqlx::query("UPDATE agents SET ic_token_hash = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(agent_id)
        .execute(&pool)
        .await
        .expect("LOUD FAILURE: Should update ic_token_hash");

    // Step 3: Old Token A must be REJECTED
    let router = create_analytics_router(pool.clone(), manager.clone());
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/analytics/events")
                .header("content-type", "application/json")
                .body(Body::from(event_body(&token_a, "evt_002")))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "LOUD FAILURE: Old Token A must be rejected after regeneration (hash mismatch)"
    );

    // Step 4: New Token B must be ACCEPTED
    let router = create_analytics_router(pool.clone(), manager.clone());
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/analytics/events")
                .header("content-type", "application/json")
                .body(Body::from(event_body(&token_b, "evt_003")))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "LOUD FAILURE: New Token B must be accepted after regeneration"
    );

    // Step 5: Verify only 2 events were stored (evt_001 + evt_003, not evt_002)
    let event_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM analytics_events WHERE agent_id = ?")
            .bind(agent_id)
            .fetch_one(&pool)
            .await
            .expect("LOUD FAILURE: Should query event count");

    assert_eq!(
        event_count, 2,
        "LOUD FAILURE: Only 2 events should be stored (before rotation + after new token). Got {}",
        event_count
    );
}
