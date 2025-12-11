use crate::event::{AnalyticsEvent, EventPayload, EventId};
use crate::stats::{AtomicModelStats, ComputedStats};
use crossbeam::queue::ArrayQueue;
use crossbeam::channel::{self, Sender, Receiver};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::iter;

const DEFAULT_CAPACITY: usize = 10_000;

#[derive(Debug, Default)]
struct GlobalStats {
    requests: AtomicU64,
    failed: AtomicU64,
    input_tokens: AtomicU64,
    output_tokens: AtomicU64,
    cost_micros: AtomicU64,
}

pub struct EventStore {
    // Storage
    buffer: ArrayQueue<AnalyticsEvent>,
    event_sender: Option<Sender<AnalyticsEvent>>,

    // Stats
    global: GlobalStats,
    by_model: DashMap<Arc<str>, AtomicModelStats>,
    by_provider: DashMap<Arc<str>, AtomicModelStats>,

    // State
    unsynced_count: AtomicU64,
    dropped_events: AtomicU64,
}

impl EventStore {
    pub fn new() -> Self { Self::with_capacity(DEFAULT_CAPACITY) }

    pub fn with_capacity(capacity: usize) -> Self { Self::init(capacity, None) }

    pub fn with_streaming(capacity: usize, chan_size: usize) -> (Self, Receiver<AnalyticsEvent>) {
        let (tx, rx) = channel::bounded(chan_size);
        (Self::init(capacity, Some(tx)), rx)
    }

    fn init(capacity: usize, event_sender: Option<Sender<AnalyticsEvent>>) -> Self {
        Self {
            buffer: ArrayQueue::new(capacity),
            event_sender,
            global: GlobalStats::default(),
            by_model: DashMap::new(),
            by_provider: DashMap::new(),
            unsynced_count: AtomicU64::new(0),
            dropped_events: AtomicU64::new(0),
        }
    }

    pub fn record(&self, event: AnalyticsEvent) {
        // 1. Stats
        self.update_stats(&event);

        // 2. Streaming: Idiomatic 'if let' -> .as_ref().map(...)
        if let Some(tx) = &self.event_sender {
            let _ = tx.try_send(event.clone());
        }

        // 3. Buffer: Optimistic update pattern
        let is_unsynced = !event.is_synced();
        if is_unsynced {
            self.unsynced_count.fetch_add(1, Ordering::Relaxed);
        }

        if self.buffer.push(event).is_err() {
            self.dropped_events.fetch_add(1, Ordering::Relaxed);
            // Rollback if push failed
            if is_unsynced {
                self.unsynced_count.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    fn update_stats(&self, event: &AnalyticsEvent) {
        match &event.payload {
            EventPayload::LlmRequestCompleted(data) => {
                // Global
                self.global.requests.fetch_add(1, Ordering::Relaxed);
                self.global.input_tokens.fetch_add(data.input_tokens, Ordering::Relaxed);
                self.global.output_tokens.fetch_add(data.output_tokens, Ordering::Relaxed);
                self.global.cost_micros.fetch_add(data.cost_micros, Ordering::Relaxed);

                // Maps: Use closure to stay DRY
                let update_map = |map: &DashMap<Arc<str>, AtomicModelStats>, key: Arc<str>| {
                    map.entry(key).or_default()
                        .add(data.input_tokens, data.output_tokens, data.cost_micros);
                };

                update_map(&self.by_model, data.meta.model.clone());
                update_map(&self.by_provider, data.meta.provider.clone());
            },
            EventPayload::LlmRequestFailed(data) => {
                self.global.requests.fetch_add(1, Ordering::Relaxed);
                self.global.failed.fetch_add(1, Ordering::Relaxed);

                // Ensure keys exist (for accurate "request count" even if 0 tokens)
                self.by_model.entry(data.meta.model.clone()).or_default();
                self.by_provider.entry(data.meta.provider.clone()).or_default();
            },
            _ => {}
        }
    }

    pub fn stats(&self) -> ComputedStats {
        let collect_map = |map: &DashMap<Arc<str>, AtomicModelStats>| {
            map.iter()
                .map(|r| (r.key().to_string(), r.value().into()))
                .collect()
        };

        ComputedStats {
            total_requests: self.global.requests.load(Ordering::Relaxed),
            failed_requests: self.global.failed.load(Ordering::Relaxed),
            total_input_tokens: self.global.input_tokens.load(Ordering::Relaxed),
            total_output_tokens: self.global.output_tokens.load(Ordering::Relaxed),
            total_cost_micros: self.global.cost_micros.load(Ordering::Relaxed),
            by_model: collect_map(&self.by_model),
            by_provider: collect_map(&self.by_provider),
        }
    }

    // === Buffer Access ===

    pub fn len(&self) -> usize { self.buffer.len() }
    pub fn is_empty(&self) -> bool { self.buffer.is_empty() }
    pub fn dropped_count(&self) -> u64 { self.dropped_events.load(Ordering::Relaxed) }
    pub fn unsynced_count(&self) -> u64 { self.unsynced_count.load(Ordering::Relaxed) }

    pub fn drain_all(&self) -> Vec<AnalyticsEvent> {
        self.unsynced_count.store(0, Ordering::Relaxed);
        iter::from_fn(|| self.buffer.pop()).collect()
    }

    pub fn snapshot_events(&self) -> Vec<AnalyticsEvent> {
        let events: Vec<_> = iter::from_fn(|| self.buffer.pop()).collect();
        // Repush to preserve state (best effort)
        events.iter().for_each(|e| { let _ = self.buffer.push(e.clone()); });
        events
    }

    pub fn unsynced_events(&self) -> Vec<AnalyticsEvent> {
        self.snapshot_events()
            .into_iter()
            .filter(|e| !e.is_synced())
            .collect()
    }

    /// Safely decrement the unsynced count (saturating at 0).
    pub fn mark_synced(&self, event_ids: &[EventId]) {
        let n = event_ids.len() as u64;
        // Robustness: CAS loop to ensure we never underflow if drain_all() was called concurrently
        let mut current = self.unsynced_count.load(Ordering::Relaxed);
        loop {
            let new_val = current.saturating_sub(n);
            match self.unsynced_count.compare_exchange_weak(
                current, new_val, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current = x,
            }
        }
    }
}

impl Default for EventStore {
    fn default() -> Self { Self::new() }
}