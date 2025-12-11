//! Tests for EventStore basic operations

use iron_runtime_analytics::event::*;
use iron_runtime_analytics::event_storage::EventStore;

// ============================================================================
// EventStore Creation
// ============================================================================

#[test]
fn test_event_store_new_creates_empty_store() {
    let store = EventStore::new();

    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
    assert_eq!(store.unsynced_count(), 0);
}

#[test]
fn test_event_store_with_capacity_respects_size() {
    let store = EventStore::with_capacity(100);

    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn test_event_store_default_capacity_is_10000() {
    let store = EventStore::new();

    for _ in 0..10_000 {
        store.record(create_completed_event(100));
    }

    assert_eq!(store.len(), 10_000);
}

// ============================================================================
// Event Recording - Basic
// ============================================================================

#[test]
fn test_record_single_event_increments_count() {
    let store = EventStore::new();

    store.record(create_completed_event(100));

    assert_eq!(store.len(), 1);
    assert!(!store.is_empty());
}

#[test]
fn test_record_multiple_events() {
    let store = EventStore::new();

    store.record(create_completed_event(100));
    store.record(create_completed_event(200));
    store.record(create_completed_event(300));

    assert_eq!(store.len(), 3);
}

// ============================================================================
// Event Recording - Different Event Types
// ============================================================================

#[test]
fn test_record_llm_request_completed() {
    let store = EventStore::new();

    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: Some("ip_openai-001".into()),
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 150,
        output_tokens: 50,
        cost_micros: 6000,
    })));

    assert_eq!(store.len(), 1);

    let stats = store.stats();
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.failed_requests, 0);
}

#[test]
fn test_record_llm_request_failed() {
    let store = EventStore::new();

    store.record(AnalyticsEvent::new(EventPayload::LlmRequestFailed(LlmFailureData {
        meta: LlmModelMeta {
            provider_id: Some("ip_openai-001".into()),
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        error_code: None,
        error_message: None,
    })));

    assert_eq!(store.len(), 1);

    let stats = store.stats();
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.failed_requests, 1);
}

#[test]
fn test_record_budget_threshold_reached() {
    let store = EventStore::new();

    store.record(AnalyticsEvent::new(EventPayload::BudgetThresholdReached {
        threshold_percent: 80,
        current_spend_micros: 8_000_000,
        budget_micros: 10_000_000,
    }));

    assert_eq!(store.len(), 1);

    // Budget events don't count as requests
    let stats = store.stats();
    assert_eq!(stats.total_requests, 0);
}

// ============================================================================
// Buffer Bounds (Pilot Strategy: Fixed Memory + Drop on Full)
// ============================================================================

#[test]
fn test_buffer_bounded_by_capacity() {
    let store = EventStore::with_capacity(3);

    store.record(create_completed_event(1000));
    store.record(create_completed_event(2000));
    store.record(create_completed_event(3000));
    store.record(create_completed_event(4000)); // Dropped
    store.record(create_completed_event(5000)); // Dropped

    let events = store.snapshot_events();
    assert_eq!(events.len(), 3);
}

#[test]
fn test_dropped_count_tracks_overflow() {
    let store = EventStore::with_capacity(3);

    store.record(create_completed_event(1000));
    store.record(create_completed_event(2000));
    store.record(create_completed_event(3000));
    store.record(create_completed_event(4000));
    store.record(create_completed_event(5000));

    assert_eq!(store.dropped_count(), 2);
}

#[test]
fn test_dropped_count_starts_at_zero() {
    let store = EventStore::new();
    assert_eq!(store.dropped_count(), 0);
}

#[test]
fn test_no_drops_when_buffer_has_space() {
    let store = EventStore::with_capacity(100);

    for _ in 0..50 {
        store.record(create_completed_event(100));
    }

    assert_eq!(store.dropped_count(), 0);
    assert_eq!(store.len(), 50);
}

#[test]
fn test_atomic_counters_track_all_events() {
    let store = EventStore::with_capacity(3);

    for i in 1..=5 {
        store.record(create_completed_event(i * 1000));
    }

    // Atomic counters track ALL events (including dropped)
    let stats = store.stats();
    assert_eq!(stats.total_requests, 5);
    assert_eq!(stats.total_cost_micros, 15000); // 1000+2000+3000+4000+5000
}

#[test]
fn test_record_never_blocks() {
    let store = EventStore::with_capacity(10);

    let start = std::time::Instant::now();

    for _ in 0..1000 {
        store.record(create_completed_event(100));
    }

    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 100, "Recording took too long: {:?}", elapsed);
    assert_eq!(store.dropped_count(), 990);
}

// ============================================================================
// Sync State Tracking
// ============================================================================

#[test]
fn test_new_events_are_unsynced() {
    let store = EventStore::new();

    store.record(create_completed_event(100));
    store.record(create_completed_event(200));

    assert_eq!(store.unsynced_count(), 2);
}

#[test]
fn test_unsynced_events_returns_only_unsynced() {
    let store = EventStore::new();

    store.record(create_completed_event(100));
    store.record(create_completed_event(200));

    let unsynced = store.unsynced_events();
    assert_eq!(unsynced.len(), 2);

    for event in &unsynced {
        assert!(!event.is_synced());
    }
}

#[test]
fn test_mark_synced_updates_count() {
    let store = EventStore::new();

    store.record(create_completed_event(100));
    store.record(create_completed_event(200));

    let unsynced = store.unsynced_events();
    let ids: Vec<EventId> = unsynced.iter().map(|e| e.event_id()).collect();

    // Mark first event as synced
    store.mark_synced(&[ids[0]]);

    assert_eq!(store.unsynced_count(), 1);
}

#[test]
fn test_mark_synced_all_events() {
    let store = EventStore::new();

    store.record(create_completed_event(100));
    store.record(create_completed_event(200));
    store.record(create_completed_event(300));

    let unsynced = store.unsynced_events();
    let ids: Vec<EventId> = unsynced.iter().map(|e| e.event_id()).collect();

    store.mark_synced(&ids);

    assert_eq!(store.unsynced_count(), 0);
}

// ============================================================================
// Event Snapshot and Drain
// ============================================================================

#[test]
fn test_snapshot_events_returns_copy() {
    let store = EventStore::new();

    store.record(create_completed_event(100));
    store.record(create_completed_event(200));

    let snapshot1 = store.snapshot_events();
    let snapshot2 = store.snapshot_events();

    assert_eq!(snapshot1.len(), 2);
    assert_eq!(snapshot2.len(), 2);
    assert_eq!(store.len(), 2);
}

#[test]
fn test_drain_all_removes_events() {
    let store = EventStore::new();

    store.record(create_completed_event(100));
    store.record(create_completed_event(200));

    let drained = store.drain_all();

    assert_eq!(drained.len(), 2);
    assert!(store.is_empty());
}

// ============================================================================
// Event Streaming (Channel)
// ============================================================================

#[test]
fn test_with_streaming_creates_channel() {
    let (store, receiver) = EventStore::with_streaming(100, 10);

    store.record(create_completed_event(100));

    let event = receiver.try_recv();
    assert!(event.is_ok());
}

#[test]
fn test_streaming_sends_all_events() {
    let (store, receiver) = EventStore::with_streaming(100, 10);

    store.record(create_completed_event(100));
    store.record(create_completed_event(200));
    store.record(create_completed_event(300));

    assert!(receiver.try_recv().is_ok());
    assert!(receiver.try_recv().is_ok());
    assert!(receiver.try_recv().is_ok());
    assert!(receiver.try_recv().is_err());
}

#[test]
fn test_streaming_nonblocking_when_channel_full() {
    let (store, _receiver) = EventStore::with_streaming(100, 2);

    let start = std::time::Instant::now();

    for _ in 0..100 {
        store.record(create_completed_event(100));
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 100, "Streaming blocked: {:?}", elapsed);

    assert_eq!(store.len(), 100);
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_completed_event(cost_micros: u64) -> AnalyticsEvent {
    AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 100,
        output_tokens: 50,
        cost_micros,
    }))
}