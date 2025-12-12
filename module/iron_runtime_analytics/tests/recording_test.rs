//! Tests for high-level recording API

use iron_cost::pricing::PricingManager;
use iron_runtime_analytics::event::EventPayload;
use iron_runtime_analytics::event_storage::EventStore;
use iron_runtime_analytics::provider_utils::Provider;

fn pricing() -> PricingManager {
    PricingManager::new().expect("LOUD FAILURE: Failed to create PricingManager")
}

// ============================================================================
// record_llm_completed
// ============================================================================

#[test]
fn test_record_llm_completed_basic() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed(&pricing, "gpt-4", 100, 50, None, None);

    let stats = store.stats();
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.total_input_tokens, 100);
    assert_eq!(stats.total_output_tokens, 50);
    // Cost should be > 0 for gpt-4
    assert!(stats.total_cost_micros > 0, "Expected non-zero cost for gpt-4");
}

#[test]
fn test_record_llm_completed_with_agent_id() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed(
        &pricing,
        "gpt-4",
        100,
        50,
        Some("agent_123"),
        None,
    );

    let events = store.drain_all();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].agent_id().map(|s| s.as_ref()), Some("agent_123"));
}

#[test]
fn test_record_llm_completed_with_provider_id() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed(
        &pricing,
        "gpt-4",
        100,
        50,
        None,
        Some("ip_openai-001"),
    );

    let events = store.drain_all();
    assert_eq!(events.len(), 1);
    if let EventPayload::LlmRequestCompleted(data) = &events[0].payload {
        assert_eq!(
            data.meta.provider_id.as_ref().map(|s| s.as_ref()),
            Some("ip_openai-001")
        );
    } else {
        panic!("Expected LlmRequestCompleted");
    }
}

#[test]
fn test_record_llm_completed_infers_openai_provider() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed(&pricing, "gpt-4-turbo", 100, 50, None, None);

    let events = store.drain_all();
    if let EventPayload::LlmRequestCompleted(data) = &events[0].payload {
        assert_eq!(data.meta.provider.as_ref(), "openai");
    } else {
        panic!("Expected LlmRequestCompleted");
    }
}

#[test]
fn test_record_llm_completed_infers_anthropic_provider() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed(&pricing, "claude-3-opus-20240229", 100, 50, None, None);

    let events = store.drain_all();
    if let EventPayload::LlmRequestCompleted(data) = &events[0].payload {
        assert_eq!(data.meta.provider.as_ref(), "anthropic");
    } else {
        panic!("Expected LlmRequestCompleted");
    }
}

#[test]
fn test_record_llm_completed_unknown_model_zero_cost() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed(&pricing, "nonexistent-model-xyz", 100, 50, None, None);

    let stats = store.stats();
    assert_eq!(stats.total_cost_micros, 0, "Unknown model should have zero cost");
}

#[test]
fn test_record_llm_completed_updates_by_model_stats() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed(&pricing, "gpt-4", 100, 50, None, None);
    store.record_llm_completed(&pricing, "gpt-4", 200, 100, None, None);

    let stats = store.stats();
    let model_stats = stats.by_model.get("gpt-4").expect("LOUD FAILURE: Should have gpt-4 stats");
    assert_eq!(model_stats.request_count, 2);
    assert_eq!(model_stats.input_tokens, 300);
    assert_eq!(model_stats.output_tokens, 150);
}

#[test]
fn test_record_llm_completed_updates_by_provider_stats() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed(&pricing, "gpt-4", 100, 50, None, None);
    store.record_llm_completed(&pricing, "gpt-3.5-turbo", 200, 100, None, None);

    let stats = store.stats();
    let provider_stats = stats.by_provider.get("openai").expect("LOUD FAILURE: Should have openai stats");
    assert_eq!(provider_stats.request_count, 2);
}

// ============================================================================
// record_llm_completed_with_provider
// ============================================================================

#[test]
fn test_record_llm_completed_with_explicit_provider() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed_with_provider(
        &pricing,
        "custom-model",
        Provider::OpenAI,
        100,
        50,
        None,
        None,
    );

    let events = store.drain_all();
    if let EventPayload::LlmRequestCompleted(data) = &events[0].payload {
        assert_eq!(data.meta.provider.as_ref(), "openai");
        assert_eq!(data.meta.model.as_ref(), "custom-model");
    } else {
        panic!("Expected LlmRequestCompleted");
    }
}

// ============================================================================
// record_llm_failed
// ============================================================================

#[test]
fn test_record_llm_failed_basic() {
    let store = EventStore::new();

    store.record_llm_failed("gpt-4", None, None, None, None);

    let stats = store.stats();
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.failed_requests, 1);
}

#[test]
fn test_record_llm_failed_with_error_details() {
    let store = EventStore::new();

    store.record_llm_failed(
        "gpt-4",
        Some("agent_456"),
        Some("ip_openai-002"),
        Some("rate_limit_exceeded"),
        Some("Too many requests"),
    );

    let events = store.drain_all();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].agent_id().map(|s| s.as_ref()), Some("agent_456"));

    if let EventPayload::LlmRequestFailed(data) = &events[0].payload {
        assert_eq!(data.error_code.as_deref(), Some("rate_limit_exceeded"));
        assert_eq!(data.error_message.as_deref(), Some("Too many requests"));
        assert_eq!(data.meta.provider.as_ref(), "openai");
    } else {
        panic!("Expected LlmRequestFailed");
    }
}

// ============================================================================
// record_budget_threshold
// ============================================================================

#[test]
fn test_record_budget_threshold_basic() {
    let store = EventStore::new();

    store.record_budget_threshold(80, 80_000_000, 100_000_000, None);

    let events = store.drain_all();
    assert_eq!(events.len(), 1);

    if let EventPayload::BudgetThresholdReached {
        threshold_percent,
        current_spend_micros,
        budget_micros,
    } = &events[0].payload
    {
        assert_eq!(*threshold_percent, 80);
        assert_eq!(*current_spend_micros, 80_000_000);
        assert_eq!(*budget_micros, 100_000_000);
    } else {
        panic!("Expected BudgetThresholdReached");
    }
}

#[test]
fn test_record_budget_threshold_with_agent() {
    let store = EventStore::new();

    store.record_budget_threshold(90, 90_000, 100_000, Some("agent_budget"));

    let events = store.drain_all();
    assert_eq!(events[0].agent_id().map(|s| s.as_ref()), Some("agent_budget"));
}

#[test]
fn test_record_budget_threshold_100_percent() {
    let store = EventStore::new();

    store.record_budget_threshold(100, 100_000_000, 100_000_000, None);

    let events = store.drain_all();
    if let EventPayload::BudgetThresholdReached {
        threshold_percent, ..
    } = &events[0].payload
    {
        assert_eq!(*threshold_percent, 100);
    } else {
        panic!("Expected BudgetThresholdReached");
    }
}

// ============================================================================
// record_router_started
// ============================================================================

#[test]
fn test_record_router_started() {
    let store = EventStore::new();

    store.record_router_started(8080);

    let events = store.drain_all();
    assert_eq!(events.len(), 1);

    if let EventPayload::RouterStarted { port } = &events[0].payload {
        assert_eq!(*port, 8080);
    } else {
        panic!("Expected RouterStarted");
    }
}

// ============================================================================
// record_router_stopped
// ============================================================================

#[test]
fn test_record_router_stopped_empty_stats() {
    let store = EventStore::new();

    store.record_router_stopped();

    let events = store.drain_all();
    assert_eq!(events.len(), 1);

    if let EventPayload::RouterStopped {
        total_requests,
        total_cost_micros,
    } = &events[0].payload
    {
        assert_eq!(*total_requests, 0);
        assert_eq!(*total_cost_micros, 0);
    } else {
        panic!("Expected RouterStopped");
    }
}

#[test]
fn test_record_router_stopped_with_stats() {
    let store = EventStore::new();
    let pricing = pricing();

    // Record some activity first
    store.record_llm_completed(&pricing, "gpt-4", 1000, 500, None, None);
    store.record_llm_completed(&pricing, "gpt-4", 2000, 1000, None, None);
    store.record_llm_failed("gpt-4", None, None, None, None);

    let stats_before = store.stats();
    store.record_router_stopped();

    let events = store.drain_all();
    let stopped_event = events.last().expect("LOUD FAILURE: Should have events");

    if let EventPayload::RouterStopped {
        total_requests,
        total_cost_micros,
    } = &stopped_event.payload
    {
        assert_eq!(*total_requests, stats_before.total_requests);
        assert_eq!(*total_cost_micros, stats_before.total_cost_micros);
    } else {
        panic!("Expected RouterStopped");
    }
}

// ============================================================================
// Integration / Edge Cases
// ============================================================================

#[test]
fn test_mixed_recording_types() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_router_started(8080);
    store.record_llm_completed(&pricing, "gpt-4", 100, 50, Some("agent1"), None);
    store.record_llm_failed("claude-3-opus", Some("agent2"), None, Some("error"), None);
    store.record_budget_threshold(50, 50_000, 100_000, None);
    store.record_router_stopped();

    let events = store.drain_all();
    assert_eq!(events.len(), 5);

    // Verify order and types
    assert!(matches!(events[0].payload, EventPayload::RouterStarted { .. }));
    assert!(matches!(events[1].payload, EventPayload::LlmRequestCompleted(_)));
    assert!(matches!(events[2].payload, EventPayload::LlmRequestFailed(_)));
    assert!(matches!(events[3].payload, EventPayload::BudgetThresholdReached { .. }));
    assert!(matches!(events[4].payload, EventPayload::RouterStopped { .. }));
}

#[test]
fn test_event_timestamps_are_populated() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed(&pricing, "gpt-4", 100, 50, None, None);

    let events = store.drain_all();
    assert!(events[0].timestamp_ms() > 0, "Timestamp should be set");
}

#[test]
fn test_event_ids_are_unique() {
    let store = EventStore::new();
    let pricing = pricing();

    store.record_llm_completed(&pricing, "gpt-4", 100, 50, None, None);
    store.record_llm_completed(&pricing, "gpt-4", 100, 50, None, None);

    let events = store.drain_all();
    assert_ne!(events[0].event_id(), events[1].event_id());
}
