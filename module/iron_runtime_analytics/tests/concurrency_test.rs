//! Tests for lock-free concurrent access safety

use iron_runtime_analytics::event::*;
use iron_runtime_analytics::event_storage::EventStore;
use std::sync::Arc;
use std::thread;

// ============================================================================
// Multi-Threaded Recording
// ============================================================================

#[test]
fn test_concurrent_recording_10_threads() {
    let store = Arc::new(EventStore::new());
    let mut handles = vec![];

    for thread_id in 0..10 {
        let store = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                store.record(create_event(thread_id * 1000 + i));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = store.stats();
    assert_eq!(stats.total_requests, 1000);
}

#[test]
fn test_concurrent_recording_100_threads() {
    let store = Arc::new(EventStore::new());
    let mut handles = vec![];

    for thread_id in 0..100 {
        let store = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            for i in 0..10 {
                store.record(create_event(thread_id * 100 + i));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = store.stats();
    assert_eq!(stats.total_requests, 1000);
}

// ============================================================================
// Stats Consistency Under Concurrent Load
// ============================================================================

#[test]
fn test_atomic_counters_consistent_under_load() {
    let store = Arc::new(EventStore::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let store = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
                    meta: LlmModelMeta {
                        provider_id: None,
                        provider: "openai".into(),
                        model: "gpt-4".into(),
                    },
                    input_tokens: 10,
                    output_tokens: 5,
                    cost_micros: 100,
                })));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = store.stats();

    assert_eq!(stats.total_requests, 1000);
    assert_eq!(stats.total_input_tokens, 10000);
    assert_eq!(stats.total_output_tokens, 5000);
    assert_eq!(stats.total_cost_micros, 100000);
}

#[test]
fn test_per_model_stats_consistent_under_load() {
    let store = Arc::new(EventStore::new());
    let mut handles = vec![];

    for thread_id in 0..10 {
        let store = Arc::clone(&store);
        let model: Arc<str> = if thread_id % 2 == 0 { "gpt-4".into() } else { "claude-3".into() };

        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
                    meta: LlmModelMeta {
                        provider_id: None,
                        provider: "test".into(),
                        model: model.clone(),
                    },
                    input_tokens: 10,
                    output_tokens: 5,
                    cost_micros: 100,
                })));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = store.stats();

    assert_eq!(stats.by_model["gpt-4"].request_count, 500);
    assert_eq!(stats.by_model["claude-3"].request_count, 500);
}

// ============================================================================
// Lock-Free Guarantees
// ============================================================================

#[test]
fn test_no_deadlock_under_heavy_load() {
    let store = Arc::new(EventStore::new());
    let mut handles = vec![];

    for _ in 0..20 {
        let store = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            for _ in 0..500 {
                store.record(create_event(100));
                let _ = store.stats();
            }
        }));
    }

    let timeout = std::time::Duration::from_secs(10);
    let start = std::time::Instant::now();

    for handle in handles {
        handle.join().unwrap();
    }

    assert!(start.elapsed() < timeout, "Test took too long - possible deadlock");
}

#[test]
fn test_concurrent_record_and_read() {
    let store = Arc::new(EventStore::new());
    let store_writer = Arc::clone(&store);
    let store_reader = Arc::clone(&store);

    let writer = thread::spawn(move || {
        for i in 0..1000 {
            store_writer.record(create_event(i));
        }
    });

    let reader = thread::spawn(move || {
        let mut read_count = 0;
        for _ in 0..1000 {
            let stats = store_reader.stats();
            assert!(stats.total_requests <= 1000);
            read_count += 1;
        }
        read_count
    });

    writer.join().unwrap();
    let reads = reader.join().unwrap();

    assert!(reads > 0);

    let stats = store.stats();
    assert_eq!(stats.total_requests, 1000);
}

#[test]
fn test_concurrent_record_and_drain() {
    let store = Arc::new(EventStore::new());
    let store_writer = Arc::clone(&store);
    let store_drainer = Arc::clone(&store);

    let writer = thread::spawn(move || {
        for i in 0..1000 {
            store_writer.record(create_event(i));
            thread::yield_now();
        }
    });

    let drainer = thread::spawn(move || {
        let mut total_drained = 0;
        for _ in 0..100 {
            let drained = store_drainer.drain_all();
            total_drained += drained.len();
            thread::sleep(std::time::Duration::from_micros(100));
        }
        total_drained
    });

    writer.join().unwrap();
    let drained = drainer.join().unwrap();

    assert!(drained > 0);

    let stats = store.stats();
    assert_eq!(stats.total_requests, 1000);
}

// ============================================================================
// Sync State Under Concurrency
// ============================================================================

#[test]
fn test_concurrent_mark_synced() {
    let store = Arc::new(EventStore::new());

    for i in 0..100 {
        store.record(create_event(i));
    }

    let events = store.snapshot_events();
    let ids: Vec<EventId> = events.iter().map(|e| e.event_id()).collect();

    let ids1: Vec<EventId> = ids.iter().take(50).cloned().collect();
    let ids2: Vec<EventId> = ids.iter().skip(50).cloned().collect();

    let store1 = Arc::clone(&store);
    let store2 = Arc::clone(&store);

    let t1 = thread::spawn(move || {
        store1.mark_synced(&ids1);
    });

    let t2 = thread::spawn(move || {
        store2.mark_synced(&ids2);
    });

    t1.join().unwrap();
    t2.join().unwrap();

    assert_eq!(store.unsynced_count(), 0);
}

// ============================================================================
// Streaming Under Concurrency
// ============================================================================

#[test]
fn test_concurrent_streaming() {
    let (store, receiver) = EventStore::with_streaming(1000, 500);
    let store = Arc::new(store);

    let store_writer = Arc::clone(&store);

    let writer = thread::spawn(move || {
        for i in 0..100 {
            store_writer.record(create_event(i));
        }
    });

    let reader = thread::spawn(move || {
        let mut received = 0;
        while let Ok(_event) = receiver.recv_timeout(std::time::Duration::from_millis(100)) {
            received += 1;
        }
        received
    });

    writer.join().unwrap();
    let received = reader.join().unwrap();

    assert!(received > 0);
}

// ============================================================================
// Performance Under Load
// ============================================================================

#[test]
fn test_recording_throughput() {
    let store = EventStore::new();

    let start = std::time::Instant::now();

    for i in 0..10_000 {
        store.record(create_event(i));
    }

    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 1000, "Recording 10k events took {:?}", elapsed);

    let stats = store.stats();
    assert_eq!(stats.total_requests, 10_000);
}

#[test]
fn test_stats_retrieval_is_fast() {
    let store = EventStore::new();

    for i in 0..1000 {
        store.record(create_event(i));
    }

    let start = std::time::Instant::now();

    for _ in 0..1000 {
        let _ = store.stats();
    }

    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 200, "Getting stats 1000 times took {:?} (threshold allows for system variance)", elapsed);
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_event(cost_micros: u64) -> AnalyticsEvent {
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