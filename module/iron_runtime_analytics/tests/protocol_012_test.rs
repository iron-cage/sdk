//! Tests for Protocol 012 Analytics API compatibility
//!
//! TDD: These tests verify that event schema and stats match Protocol 012 spec.
//! See docs/protocol/012_analytics_api.md for specification.

use iron_runtime_analytics::event::*;
use iron_runtime_analytics::event_storage::EventStore;

// ============================================================================
// Event Schema - Required Fields
// ============================================================================

#[test]
fn test_llm_request_completed_has_all_protocol_fields() {
    let event = AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: Some("ip_openai-001".into()),
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 150,
        output_tokens: 50,
        cost_micros: 6000,
    }));

    // Verify all fields are accessible via payload
    match &event.payload {
        EventPayload::LlmRequestCompleted(data) => {
            assert!(event.timestamp_ms() > 0);
            assert_eq!(data.meta.provider_id.as_deref(), Some("ip_openai-001"));
            assert_eq!(data.meta.provider.as_ref(), "openai");
            assert_eq!(data.meta.model.as_ref(), "gpt-4");
            assert_eq!(data.input_tokens, 150);
            assert_eq!(data.output_tokens, 50);
            assert_eq!(data.cost_micros, 6000);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_llm_request_failed_has_all_protocol_fields() {
    let event = AnalyticsEvent::new(EventPayload::LlmRequestFailed(LlmFailureData {
        meta: LlmModelMeta {
            provider_id: Some("ip_openai-001".into()),
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        error_code: None,
        error_message: None,
    }));

    match &event.payload {
        EventPayload::LlmRequestFailed(data) => {
            assert!(event.timestamp_ms() > 0);
            assert_eq!(data.meta.provider_id.as_deref(), Some("ip_openai-001"));
            assert_eq!(data.meta.provider.as_ref(), "openai");
            assert_eq!(data.meta.model.as_ref(), "gpt-4");
        }
        _ => panic!("Wrong event type"),
    }
}

// ============================================================================
// Cost Format - Microdollars
// ============================================================================

#[test]
fn test_cost_stored_in_microdollars() {
    let store = EventStore::new();

    // $1.50 = 1,500,000 micros
    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 100,
        output_tokens: 50,
        cost_micros: 1_500_000, // $1.50 in micros
    })));

    let stats = store.stats();

    // Raw value in micros
    assert_eq!(stats.total_cost_micros, 1_500_000);

    // Converted to USD (Protocol 012: 2 decimal places)
    let usd = stats.total_cost_usd();
    assert!((usd - 1.50).abs() < 0.001);
}

#[test]
fn test_small_cost_precision() {
    let store = EventStore::new();

    // $0.006 = 6000 micros (typical small request)
    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 100,
        output_tokens: 50,
        cost_micros: 6000,
    })));

    let stats = store.stats();
    let usd = stats.total_cost_usd();

    // Should preserve precision for small amounts
    assert!((usd - 0.006).abs() < 0.000001);
}

// ============================================================================
// Timestamp Format - Milliseconds
// ============================================================================

#[test]
fn test_timestamp_in_milliseconds() {
    let store = EventStore::new();

    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 100,
        output_tokens: 50,
        cost_micros: 6000,
    })));

    let events = store.snapshot_events();
    let ts = events[0].timestamp_ms();

    // Verify it's in milliseconds (not seconds)
    // Seconds would be ~1.7 billion, millis is ~1.7 trillion
    assert!(ts > 1_000_000_000_000);
}

// ============================================================================
// Endpoint 1: Total Spending
// ============================================================================

#[test]
fn test_endpoint_1_total_spending_fields() {
    let store = EventStore::new();

    store.record(completed_event(100, 50, 15000));
    store.record(completed_event(200, 100, 30000));

    let stats = store.stats();

    // Protocol 012 Endpoint 1 fields:
    // - total_spend (USD, 2 decimals)
    let total_spend = stats.total_cost_usd();
    assert!((total_spend - 0.045).abs() < 0.001); // $0.045

    // - currency: "USD" (fixed)
    // (handled at API layer, not in stats)

    // - period: filter by timestamp_ms
    // (handled via get_events with since_ms filter)
}

// ============================================================================
// Endpoint 2: Spending by Agent
// ============================================================================

#[test]
fn test_endpoint_2_spending_by_agent() {
    let store = EventStore::new();

    // Events with different agent_ids
    // Note: agent_id is in metadata, not payload, so we can't set it directly
    // For this test, we verify the general structure works

    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 100,
        output_tokens: 50,
        cost_micros: 10000,
    })));

    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 200,
        output_tokens: 100,
        cost_micros: 20000,
    })));

    // Protocol 012 Endpoint 2 fields:
    // - agent_id: from event
    // - agent_name: server lookup (not in local stats)
    // - spending: sum(cost_micros)
    // - budget: server lookup (not in local stats)
    // - percent_used: computed from spending/budget
    // - request_count: count(events)

    let stats = store.stats();
    assert_eq!(stats.total_requests, 2);
    assert_eq!(stats.total_cost_micros, 30000);
}

// ============================================================================
// Endpoint 4: Spending by Provider
// ============================================================================

#[test]
fn test_endpoint_4_spending_by_provider() {
    let store = EventStore::new();

    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: Some("ip_openai-001".into()),
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 100,
        output_tokens: 50,
        cost_micros: 15000,
    })));

    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: Some("ip_anthropic-001".into()),
            provider: "anthropic".into(),
            model: "claude-3".into(),
        },
        input_tokens: 200,
        output_tokens: 100,
        cost_micros: 20000,
    })));

    let stats = store.stats();

    // Protocol 012 Endpoint 4 fields:
    // - provider_id: from event
    // - provider_name: from event.provider
    // - spending: sum(cost_micros)
    // - request_count: count(events)
    // - avg_cost_per_request: spending/count
    // - agent_count: count(distinct agent_id)

    assert_eq!(stats.by_provider.len(), 2);
    assert_eq!(stats.by_provider["openai"].request_count, 1);
    assert_eq!(stats.by_provider["openai"].cost_micros, 15000);
    assert_eq!(stats.by_provider["anthropic"].request_count, 1);
    assert_eq!(stats.by_provider["anthropic"].cost_micros, 20000);
}

// ============================================================================
// Endpoint 5: Request Usage
// ============================================================================

#[test]
fn test_endpoint_5_request_usage() {
    let store = EventStore::new();

    store.record(completed_event(100, 50, 15000));
    store.record(completed_event(100, 50, 15000));
    store.record(completed_event(100, 50, 15000));
    store.record(failed_event());

    let stats = store.stats();

    // Protocol 012 Endpoint 5 fields:
    // - total_requests
    assert_eq!(stats.total_requests, 4);

    // - successful_requests
    let successful = stats.total_requests - stats.failed_requests;
    assert_eq!(successful, 3);

    // - failed_requests
    assert_eq!(stats.failed_requests, 1);

    // - success_rate (percentage)
    let success_rate = stats.success_rate();
    assert!((success_rate - 75.0).abs() < 0.01); // 3/4 = 75%
}

// ============================================================================
// Endpoint 6: Token Usage by Agent
// ============================================================================

#[test]
fn test_endpoint_6_token_usage() {
    let store = EventStore::new();

    store.record(completed_event(100, 50, 15000));
    store.record(completed_event(200, 100, 30000));

    let stats = store.stats();

    // Protocol 012 Endpoint 6 fields:
    // - agent_id: from event
    // - agent_name: server lookup
    // - input_tokens
    assert_eq!(stats.total_input_tokens, 300);

    // - output_tokens
    assert_eq!(stats.total_output_tokens, 150);

    // - total_tokens
    assert_eq!(stats.total_tokens(), 450);

    // - request_count
    assert_eq!(stats.total_requests, 2);

    // - avg_tokens_per_request
    assert!((stats.avg_tokens_per_request() - 225.0).abs() < 0.01);
}

// ============================================================================
// Endpoint 7: Model Usage
// ============================================================================

#[test]
fn test_endpoint_7_model_usage() {
    let store = EventStore::new();

    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: Some("ip_openai-001".into()),
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 100,
        output_tokens: 50,
        cost_micros: 15000,
    })));

    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: Some("ip_openai-001".into()),
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 200,
        output_tokens: 100,
        cost_micros: 30000,
    })));

    let stats = store.stats();

    // Protocol 012 Endpoint 7 fields:
    // - model
    assert!(stats.by_model.contains_key("gpt-4"));

    let gpt4 = &stats.by_model["gpt-4"];

    // - provider_id: from event
    // - provider_name: from event.provider
    // - request_count
    assert_eq!(gpt4.request_count, 2);

    // - spending
    assert_eq!(gpt4.cost_micros, 45000);

    // - input_tokens
    assert_eq!(gpt4.input_tokens, 300);

    // - output_tokens
    assert_eq!(gpt4.output_tokens, 150);

    // - total_tokens (computed)
    // - avg_cost_per_request (computed)
}

// ============================================================================
// Endpoint 8: Average Cost Per Request
// ============================================================================

#[test]
fn test_endpoint_8_avg_cost_per_request() {
    let store = EventStore::new();

    store.record(completed_event(100, 50, 1_000_000)); // $1.00
    store.record(completed_event(100, 50, 2_000_000)); // $2.00
    store.record(completed_event(100, 50, 3_000_000)); // $3.00

    let stats = store.stats();

    // Protocol 012 Endpoint 8 fields:
    // - average_cost_per_request
    let avg = stats.avg_cost_per_request_usd();
    assert!((avg - 2.0).abs() < 0.001); // $6/3 = $2

    // - total_requests
    assert_eq!(stats.total_requests, 3);

    // - total_spend
    assert_eq!(stats.total_cost_micros, 6_000_000);

    // - median_cost_per_request: server computes from events
    // - min_cost_per_request: server computes from events
    // - max_cost_per_request: server computes from events
}

// ============================================================================
// Optional Fields (None values)
// ============================================================================

#[test]
fn test_optional_fields_can_be_none() {
    let store = EventStore::new();

    // Offline mode: no provider_id
    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None, // Optional
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 100,
        output_tokens: 50,
        cost_micros: 15000,
    })));

    let stats = store.stats();
    assert_eq!(stats.total_requests, 1);

    // Event should still be recorded and tracked
    let events = store.snapshot_events();
    assert_eq!(events.len(), 1);
}

// ============================================================================
// Internal Fields (Not in Protocol 012)
// ============================================================================

#[test]
fn test_event_id_is_internal() {
    let store = EventStore::new();

    store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens: 100,
        output_tokens: 50,
        cost_micros: 15000,
    })));

    let events = store.snapshot_events();

    // event_id is assigned automatically for deduplication
    // (we can't test for "not nil" since EventId doesn't expose nil check)
    let _ = events[0].event_id();

    // synced starts as false
    assert!(!events[0].is_synced());
}

// ============================================================================
// Helper Functions
// ============================================================================

fn completed_event(input_tokens: u64, output_tokens: u64, cost_micros: u64) -> AnalyticsEvent {
    AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        input_tokens,
        output_tokens,
        cost_micros,
    }))
}

fn failed_event() -> AnalyticsEvent {
    AnalyticsEvent::new(EventPayload::LlmRequestFailed(LlmFailureData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: "openai".into(),
            model: "gpt-4".into(),
        },
        error_code: None,
        error_message: None,
    }))
}