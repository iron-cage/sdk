//! Tests for analytics sync to Control API
//!
//! These tests verify the sync module functionality including:
//! - SyncConfig builder pattern
//! - SyncClient lifecycle (start/stop)
//! - Event filtering (only LLM events synced)
//! - HTTP sync with mock server
//! - Concurrent sync operations

#![cfg(feature = "sync")]

use iron_runtime_analytics::event::*;
use iron_runtime_analytics::event_storage::EventStore;
use iron_runtime_analytics::sync::{SyncClient, SyncConfig};
use std::sync::Arc;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ============================================================================
// SyncConfig Tests
// ============================================================================

#[test]
fn test_sync_config_new() {
    let config = SyncConfig::new("http://localhost:3001", "test_token");

    assert_eq!(config.server_url, "http://localhost:3001");
    assert_eq!(config.ic_token, "test_token");
    assert_eq!(config.sync_interval, Duration::from_secs(30));
    assert_eq!(config.batch_threshold, 10);
    assert_eq!(config.timeout, Duration::from_secs(30));
}

#[test]
fn test_sync_config_with_interval() {
    let config = SyncConfig::new("http://localhost:3001", "token")
        .with_interval(Duration::from_secs(5));

    assert_eq!(config.sync_interval, Duration::from_secs(5));
}

#[test]
fn test_sync_config_with_batch_threshold() {
    let config = SyncConfig::new("http://localhost:3001", "token")
        .with_batch_threshold(50);

    assert_eq!(config.batch_threshold, 50);
}

#[test]
fn test_sync_config_builder_chain() {
    let config = SyncConfig::new("http://localhost:3001", "token")
        .with_interval(Duration::from_secs(10))
        .with_batch_threshold(25);

    assert_eq!(config.sync_interval, Duration::from_secs(10));
    assert_eq!(config.batch_threshold, 25);
}

// ============================================================================
// SyncClient Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_sync_client_start_stop() {
    let store = Arc::new(EventStore::new());
    let config = SyncConfig::new("", "token"); // Empty URL = sync disabled

    let client = SyncClient::new(store.clone(), config);
    let handle = client.start(&tokio::runtime::Handle::current());

    // Should be able to stop cleanly
    handle.stop();
}

#[tokio::test]
async fn test_sync_handle_drop_sends_shutdown() {
    let store = Arc::new(EventStore::new());
    let config = SyncConfig::new("", "token");

    let client = SyncClient::new(store.clone(), config);
    let handle = client.start(&tokio::runtime::Handle::current());

    // Drop should trigger shutdown
    drop(handle);

    // Give time for shutdown
    tokio::time::sleep(Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_sync_client_with_empty_url_is_noop() {
    let store = Arc::new(EventStore::new());

    // Record some events
    store.record(create_completed_event("gpt-4", 100, 50, 1000));
    store.record(create_completed_event("gpt-4", 200, 100, 2000));

    let config = SyncConfig::new("", "token")
        .with_interval(Duration::from_millis(10));

    let client = SyncClient::new(store.clone(), config);
    let handle = client.start(&tokio::runtime::Handle::current());

    tokio::time::sleep(Duration::from_millis(50)).await;

    handle.stop();

    // Events should still be unsynced (no server to sync to)
    assert_eq!(store.unsynced_count(), 2);
}

// ============================================================================
// Event Sync with Mock Server
// ============================================================================

#[tokio::test]
async fn test_sync_events_to_mock_server() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/analytics/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let store = Arc::new(EventStore::new());

    // Record LLM events (these should be synced)
    store.record(create_completed_event("gpt-4", 100, 50, 1000));
    store.record(create_failed_event("gpt-4", "rate_limit", "Rate limit exceeded"));

    assert_eq!(store.unsynced_count(), 2);

    let config = SyncConfig::new(mock_server.uri(), "test_token")
        .with_interval(Duration::from_secs(60)); // Long interval, we'll use manual sync

    let client = SyncClient::new(store.clone(), config);

    // Use manual sync instead of background task for predictable behavior
    let synced = client.sync_events().await;

    assert_eq!(synced, 2);
    assert_eq!(store.unsynced_count(), 0);
}

#[tokio::test]
async fn test_sync_filters_non_llm_events() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/analytics/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let store = Arc::new(EventStore::new());

    // Record mixed events
    store.record(create_completed_event("gpt-4", 100, 50, 1000)); // Should sync
    store.record(AnalyticsEvent::new(EventPayload::RouterStarted { port: 8080 })); // Should NOT sync
    store.record(AnalyticsEvent::new(EventPayload::RouterStopped {
        total_requests: 10,
        total_cost_micros: 5000,
    })); // Should NOT sync

    // All 3 events are unsynced initially
    assert_eq!(store.unsynced_count(), 3);

    let config = SyncConfig::new(mock_server.uri(), "test_token");
    let client = SyncClient::new(store.clone(), config);

    // Use manual sync for predictable behavior
    let synced = client.sync_events().await;

    // Only 1 LLM event should be synced (router events are filtered out)
    assert_eq!(synced, 1);

    // RouterStarted and RouterStopped remain unsynced (they're filtered, not synced)
    // The LlmRequestCompleted is now synced
    // Note: non-LLM events aren't marked as synced since they're filtered
    assert_eq!(store.unsynced_count(), 2); // RouterStarted + RouterStopped
}

#[tokio::test]
async fn test_sync_retries_on_server_error() {
    let mock_server = MockServer::start().await;

    // First request fails with 500, second succeeds
    Mock::given(method("POST"))
        .and(path("/api/v1/analytics/events"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/analytics/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let store = Arc::new(EventStore::new());
    store.record(create_completed_event("gpt-4", 100, 50, 1000));

    let config = SyncConfig::new(mock_server.uri(), "test_token")
        .with_interval(Duration::from_millis(20));

    let client = SyncClient::new(store.clone(), config);
    let handle = client.start(&tokio::runtime::Handle::current());

    // Wait for retry
    tokio::time::sleep(Duration::from_millis(100)).await;

    handle.stop();

    // Event should eventually be synced
    assert_eq!(store.unsynced_count(), 0);
}

#[tokio::test]
async fn test_sync_marks_synced_on_client_error() {
    let mock_server = MockServer::start().await;

    // 400 errors should mark as synced (no retry)
    Mock::given(method("POST"))
        .and(path("/api/v1/analytics/events"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": "Bad request"
        })))
        .mount(&mock_server)
        .await;

    let store = Arc::new(EventStore::new());
    store.record(create_completed_event("gpt-4", 100, 50, 1000));

    let config = SyncConfig::new(mock_server.uri(), "test_token")
        .with_interval(Duration::from_millis(10));

    let client = SyncClient::new(store.clone(), config);
    let handle = client.start(&tokio::runtime::Handle::current());

    tokio::time::sleep(Duration::from_millis(50)).await;

    handle.stop();

    // Event should be marked as synced even on 4xx (no retry for client errors)
    assert_eq!(store.unsynced_count(), 0);
}

// ============================================================================
// Flush on Shutdown Tests
// ============================================================================

#[tokio::test]
async fn test_sync_flushes_on_shutdown() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/analytics/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .expect(3)
        .mount(&mock_server)
        .await;

    let store = Arc::new(EventStore::new());

    // Record events
    store.record(create_completed_event("gpt-4", 100, 50, 1000));
    store.record(create_completed_event("gpt-4o", 200, 100, 2000));
    store.record(create_failed_event("claude-3", "timeout", "Request timeout"));

    // Use long interval so sync won't happen during test
    let config = SyncConfig::new(mock_server.uri(), "test_token")
        .with_interval(Duration::from_secs(60));

    let client = SyncClient::new(store.clone(), config);
    let handle = client.start(&tokio::runtime::Handle::current());

    // Don't wait for interval, just stop immediately
    // Shutdown should flush all events
    handle.stop();

    // Give time for flush
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(store.unsynced_count(), 0);
}

// ============================================================================
// Concurrent Sync Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_recording_during_sync() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/analytics/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let store = Arc::new(EventStore::new());

    let config = SyncConfig::new(mock_server.uri(), "test_token")
        .with_interval(Duration::from_millis(10))
        .with_batch_threshold(5);

    let client = SyncClient::new(store.clone(), config);
    let handle = client.start(&tokio::runtime::Handle::current());

    // Spawn multiple tasks to record events concurrently
    let mut tasks = vec![];
    for i in 0..5 {
        let store = store.clone();
        tasks.push(tokio::spawn(async move {
            for j in 0..10 {
                store.record(create_completed_event(
                    "gpt-4",
                    (i * 10 + j) as u64,
                    5,
                    100,
                ));
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        }));
    }

    // Wait for all recording tasks
    for task in tasks {
        task.await.unwrap();
    }

    // Wait for sync to catch up
    tokio::time::sleep(Duration::from_millis(200)).await;

    handle.stop();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // All 50 events should be recorded
    let stats = store.stats();
    assert_eq!(stats.total_requests, 50);

    // All should be synced
    assert_eq!(store.unsynced_count(), 0);
}

#[tokio::test]
async fn test_multiple_sync_clients_same_store() {
    // This tests that multiple clients can share the same store
    // (though in practice you'd only have one sync client)

    let store = Arc::new(EventStore::new());

    store.record(create_completed_event("gpt-4", 100, 50, 1000));

    let config1 = SyncConfig::new("", "token1");
    let config2 = SyncConfig::new("", "token2");

    let client1 = SyncClient::new(store.clone(), config1);
    let client2 = SyncClient::new(store.clone(), config2);

    let handle1 = client1.start(&tokio::runtime::Handle::current());
    let handle2 = client2.start(&tokio::runtime::Handle::current());

    tokio::time::sleep(Duration::from_millis(50)).await;

    handle1.stop();
    handle2.stop();

    // Store should still be consistent
    let stats = store.stats();
    assert_eq!(stats.total_requests, 1);
}

// ============================================================================
// Manual Sync Tests
// ============================================================================

#[tokio::test]
async fn test_manual_sync_events() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/analytics/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .expect(2)
        .mount(&mock_server)
        .await;

    let store = Arc::new(EventStore::new());

    store.record(create_completed_event("gpt-4", 100, 50, 1000));
    store.record(create_completed_event("claude-3", 200, 100, 2000));

    let config = SyncConfig::new(mock_server.uri(), "test_token");
    let client = SyncClient::new(store.clone(), config);

    // Manual sync (not using background task)
    let synced = client.sync_events().await;

    assert_eq!(synced, 2);
    assert_eq!(store.unsynced_count(), 0);
}

#[tokio::test]
async fn test_manual_sync_empty_store() {
    let store = Arc::new(EventStore::new());
    let config = SyncConfig::new("http://localhost:3001", "test_token");
    let client = SyncClient::new(store.clone(), config);

    // Manual sync with no events
    let synced = client.sync_events().await;

    assert_eq!(synced, 0);
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_completed_event(model: &str, input: u64, output: u64, cost: u64) -> AnalyticsEvent {
    AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: if model.starts_with("gpt") || model.starts_with("o1") {
                "openai".into()
            } else {
                "anthropic".into()
            },
            model: model.into(),
        },
        input_tokens: input,
        output_tokens: output,
        cost_micros: cost,
    }))
}

fn create_failed_event(model: &str, code: &str, message: &str) -> AnalyticsEvent {
    AnalyticsEvent::new(EventPayload::LlmRequestFailed(LlmFailureData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: if model.starts_with("gpt") || model.starts_with("o1") {
                "openai".into()
            } else {
                "anthropic".into()
            },
            model: model.into(),
        },
        error_code: Some(code.to_string()),
        error_message: Some(message.to_string()),
    }))
}
