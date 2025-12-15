//! Tests for atomic statistics and computed stats

use iron_runtime_analytics::event::*;
use iron_runtime_analytics::event_storage::EventStore;

// ============================================================================
// Total Counters (Atomic, O(1))
// ============================================================================

#[test]
fn test_total_requests_counter() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 200, 100, 30000));
    store.record(failed_event("gpt-4", "openai"));

    let stats = store.stats();
    assert_eq!(stats.total_requests, 3);
}

#[test]
fn test_failed_requests_counter() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(failed_event("gpt-4", "openai"));
    store.record(failed_event("claude-3", "anthropic"));
    store.record(completed_event("claude-3", "anthropic", 100, 50, 10000));

    let stats = store.stats();
    assert_eq!(stats.total_requests, 4);
    assert_eq!(stats.failed_requests, 2);
}

#[test]
fn test_total_input_tokens_counter() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 200, 100, 30000));
    store.record(completed_event("claude-3", "anthropic", 150, 75, 20000));

    let stats = store.stats();
    assert_eq!(stats.total_input_tokens, 450); // 100 + 200 + 150
}

#[test]
fn test_total_output_tokens_counter() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 200, 100, 30000));
    store.record(completed_event("claude-3", "anthropic", 150, 75, 20000));

    let stats = store.stats();
    assert_eq!(stats.total_output_tokens, 225); // 50 + 100 + 75
}

#[test]
fn test_total_cost_micros_counter() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 200, 100, 30000));
    store.record(completed_event("claude-3", "anthropic", 150, 75, 20000));

    let stats = store.stats();
    assert_eq!(stats.total_cost_micros, 65000); // 15000 + 30000 + 20000
}

#[test]
fn test_failed_requests_dont_add_tokens_or_cost() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(failed_event("gpt-4", "openai")); // No tokens or cost

    let stats = store.stats();
    assert_eq!(stats.total_requests, 2);
    assert_eq!(stats.total_input_tokens, 100);
    assert_eq!(stats.total_output_tokens, 50);
    assert_eq!(stats.total_cost_micros, 15000);
}

// ============================================================================
// Per-Model Stats
// ============================================================================

#[test]
fn test_stats_by_model_single_model() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 200, 100, 30000));

    let stats = store.stats();

    assert_eq!(stats.by_model.len(), 1);
    assert!(stats.by_model.contains_key("gpt-4"));

    let gpt4_stats = &stats.by_model["gpt-4"];
    assert_eq!(gpt4_stats.request_count, 2);
    assert_eq!(gpt4_stats.input_tokens, 300);
    assert_eq!(gpt4_stats.output_tokens, 150);
    assert_eq!(gpt4_stats.cost_micros, 45000);
}

#[test]
fn test_stats_by_model_multiple_models() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("claude-3-opus", "anthropic", 200, 100, 20000));
    store.record(completed_event("gemini-pro", "google", 150, 75, 10000));

    let stats = store.stats();

    assert_eq!(stats.by_model.len(), 3);

    assert_eq!(stats.by_model["gpt-4"].request_count, 2);
    assert_eq!(stats.by_model["gpt-4"].cost_micros, 30000);

    assert_eq!(stats.by_model["claude-3-opus"].request_count, 1);
    assert_eq!(stats.by_model["claude-3-opus"].cost_micros, 20000);

    assert_eq!(stats.by_model["gemini-pro"].request_count, 1);
    assert_eq!(stats.by_model["gemini-pro"].cost_micros, 10000);
}

#[test]
fn test_stats_by_model_tracks_failed_requests() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(failed_event("gpt-4", "openai"));

    let stats = store.stats();

    // Model should be tracked even for failed requests
    assert!(stats.by_model.contains_key("gpt-4"));
}

// ============================================================================
// Per-Provider Stats
// ============================================================================

#[test]
fn test_stats_by_provider_single_provider() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-3.5-turbo", "openai", 50, 25, 5000));

    let stats = store.stats();

    assert_eq!(stats.by_provider.len(), 1);
    assert!(stats.by_provider.contains_key("openai"));

    let openai_stats = &stats.by_provider["openai"];
    assert_eq!(openai_stats.request_count, 2);
    assert_eq!(openai_stats.input_tokens, 150);
    assert_eq!(openai_stats.cost_micros, 20000);
}

#[test]
fn test_stats_by_provider_multiple_providers() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("claude-3", "anthropic", 200, 100, 20000));

    let stats = store.stats();

    assert_eq!(stats.by_provider.len(), 2);

    assert_eq!(stats.by_provider["openai"].request_count, 2);
    assert_eq!(stats.by_provider["openai"].cost_micros, 30000);

    assert_eq!(stats.by_provider["anthropic"].request_count, 1);
    assert_eq!(stats.by_provider["anthropic"].cost_micros, 20000);
}

// ============================================================================
// ComputedStats Methods
// ============================================================================

#[test]
fn test_total_cost_usd_conversion() {
    let store = EventStore::new();

    // 1 USD = 1,000,000 micros
    store.record(completed_event("gpt-4", "openai", 100, 50, 1_500_000)); // $1.50

    let stats = store.stats();
    assert!((stats.total_cost_usd() - 1.5).abs() < 0.0001);
}

#[test]
fn test_total_cost_usd_small_amounts() {
    let store = EventStore::new();

    // $0.006 = 6000 micros
    store.record(completed_event("gpt-4", "openai", 100, 50, 6000));

    let stats = store.stats();
    assert!((stats.total_cost_usd() - 0.006).abs() < 0.000001);
}

#[test]
fn test_success_rate_all_success() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));

    let stats = store.stats();
    assert!((stats.success_rate() - 100.0).abs() < 0.01);
}

#[test]
fn test_success_rate_with_failures() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(failed_event("gpt-4", "openai")); // 1 failure out of 4

    let stats = store.stats();
    assert!((stats.success_rate() - 75.0).abs() < 0.01); // 3/4 = 75%
}

#[test]
fn test_success_rate_no_requests() {
    let store = EventStore::new();

    let stats = store.stats();
    assert!((stats.success_rate() - 100.0).abs() < 0.01); // Default to 100%
}

#[test]
fn test_avg_cost_per_request_usd() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 1_000_000)); // $1.00
    store.record(completed_event("gpt-4", "openai", 100, 50, 2_000_000)); // $2.00
    store.record(completed_event("gpt-4", "openai", 100, 50, 3_000_000)); // $3.00

    let stats = store.stats();
    // Average: $6.00 / 3 = $2.00
    assert!((stats.avg_cost_per_request_usd() - 2.0).abs() < 0.0001);
}

#[test]
fn test_avg_cost_per_request_no_requests() {
    let store = EventStore::new();

    let stats = store.stats();
    assert!((stats.avg_cost_per_request_usd() - 0.0).abs() < 0.0001);
}

#[test]
fn test_total_tokens() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000));
    store.record(completed_event("gpt-4", "openai", 200, 100, 30000));

    let stats = store.stats();
    assert_eq!(stats.total_tokens(), 450); // (100+200) + (50+100)
}

#[test]
fn test_avg_tokens_per_request() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 15000)); // 150 tokens
    store.record(completed_event("gpt-4", "openai", 200, 100, 30000)); // 300 tokens

    let stats = store.stats();
    // Average: 450 / 2 = 225
    assert!((stats.avg_tokens_per_request() - 225.0).abs() < 0.01);
}

#[test]
fn test_avg_tokens_per_request_no_requests() {
    let store = EventStore::new();

    let stats = store.stats();
    assert!((stats.avg_tokens_per_request() - 0.0).abs() < 0.01);
}

// ============================================================================
// ModelStats Methods
// ============================================================================

#[test]
fn test_model_stats_cost_usd() {
    let store = EventStore::new();

    store.record(completed_event("gpt-4", "openai", 100, 50, 1_500_000)); // $1.50

    let stats = store.stats();
    let gpt4_stats = &stats.by_model["gpt-4"];

    assert!((gpt4_stats.cost_usd() - 1.5).abs() < 0.0001);
}

// ============================================================================
// Stats Consistency
// ============================================================================

#[test]
fn test_stats_are_consistent_snapshot() {
    let store = EventStore::new();

    for i in 0..100 {
        store.record(completed_event("gpt-4", "openai", 100, 50, 1000 * (i + 1)));
    }

    let stats = store.stats();

    // Total should equal sum of per-model
    let model_total: u64 = stats.by_model.values().map(|s| s.cost_micros).sum();
    assert_eq!(stats.total_cost_micros, model_total);

    // Total should equal sum of per-provider
    let provider_total: u64 = stats.by_provider.values().map(|s| s.cost_micros).sum();
    assert_eq!(stats.total_cost_micros, provider_total);
}

// ============================================================================
// Helper Functions
// ============================================================================

fn completed_event(
    model: &str,
    provider: &str,
    input_tokens: u64,
    output_tokens: u64,
    cost_micros: u64,
) -> AnalyticsEvent {
    AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: provider.into(),
            model: model.into(),
        },
        input_tokens,
        output_tokens,
        cost_micros,
    }))
}

fn failed_event(model: &str, provider: &str) -> AnalyticsEvent {
    AnalyticsEvent::new(EventPayload::LlmRequestFailed(LlmFailureData {
        meta: LlmModelMeta {
            provider_id: None,
            provider: provider.into(),
            model: model.into(),
        },
        error_code: None,
        error_message: None,
    }))
}