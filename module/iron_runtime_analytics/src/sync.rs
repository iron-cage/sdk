//! Analytics sync - background sync of events to Control API.
//!
//! Provides a clean API for syncing EventStore events to the server.
//!
//! # Example
//!
//! ```ignore
//! use iron_runtime_analytics::{EventStore, SyncClient, SyncConfig};
//!
//! let store = Arc::new(EventStore::new());
//! let config = SyncConfig::new("https://api.example.com", "ic_token");
//! let client = SyncClient::new(store.clone(), config);
//!
//! // Start background sync
//! let handle = client.start();
//!
//! // ... use store ...
//!
//! // Stop and flush
//! handle.stop().await;
//! ```

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use serde::Serialize;
use crate::{EventStore, AnalyticsEvent, EventPayload};

/// Configuration for analytics sync
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Control API server URL
    pub server_url: String,
    /// IC token for authentication
    pub ic_token: String,
    /// Sync interval (default: 30s)
    pub sync_interval: Duration,
    /// Batch threshold - sync immediately when this many events are pending
    pub batch_threshold: usize,
    /// Request timeout
    pub timeout: Duration,
}

impl SyncConfig {
    /// Create new sync config
    pub fn new(server_url: impl Into<String>, ic_token: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            ic_token: ic_token.into(),
            sync_interval: Duration::from_secs(30),
            batch_threshold: 10,
            timeout: Duration::from_secs(30),
        }
    }

    /// Set sync interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.sync_interval = interval;
        self
    }

    /// Set batch threshold
    pub fn with_batch_threshold(mut self, threshold: usize) -> Self {
        self.batch_threshold = threshold;
        self
    }
}

/// Handle to control a running sync task
pub struct SyncHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl SyncHandle {
    /// Stop the sync task and flush remaining events
    pub fn stop(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for SyncHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Analytics sync client
pub struct SyncClient {
    event_store: Arc<EventStore>,
    config: SyncConfig,
    http_client: reqwest::Client,
}

impl SyncClient {
    /// Create new sync client
    pub fn new(event_store: Arc<EventStore>, config: SyncConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            event_store,
            config,
            http_client,
        }
    }

    /// Start background sync task
    ///
    /// Returns a handle that can be used to stop the sync.
    /// The sync task will flush remaining events when stopped.
    ///
    /// Uses the provided runtime handle to spawn the background task.
    pub fn start(self, runtime_handle: &tokio::runtime::Handle) -> SyncHandle {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        runtime_handle.spawn(self.run_loop(shutdown_rx));

        SyncHandle {
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Run sync loop
    async fn run_loop(self, mut shutdown_rx: oneshot::Receiver<()>) {
        if self.config.server_url.is_empty() {
            tracing::debug!("Analytics sync disabled: no server URL");
            return;
        }

        let mut ticker = tokio::time::interval(self.config.sync_interval);

        tracing::info!(
            "Analytics sync started (interval: {:?}, threshold: {})",
            self.config.sync_interval,
            self.config.batch_threshold
        );

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.sync_events().await;
                }
                _ = &mut shutdown_rx => {
                    tracing::info!("Analytics sync shutting down, flushing...");
                    self.sync_events().await;
                    break;
                }
            }

            // Check batch threshold
            if self.event_store.unsynced_count() >= self.config.batch_threshold as u64 {
                self.sync_events().await;
            }
        }

        tracing::info!("Analytics sync stopped");
    }

    /// Sync unsynced events to server
    pub async fn sync_events(&self) -> usize {
        let events = self.event_store.unsynced_events();
        if events.is_empty() {
            return 0;
        }

        // Filter to only LLM events (server only accepts these)
        // NOTE: RouterStarted, RouterStopped, and BudgetThresholdReached events are
        // intentionally NOT synced. They remain in the EventStore with unsynced status.
        // This is a pilot limitation - these events are logged locally but not sent to server.
        // Future: Add server endpoints for lifecycle/budget events if dashboard needs them.
        let llm_events: Vec<_> = events.into_iter().filter(|e| {
            matches!(e.payload, EventPayload::LlmRequestCompleted(_) | EventPayload::LlmRequestFailed(_))
        }).collect();

        if llm_events.is_empty() {
            return 0;
        }

        let url = format!("{}/api/v1/analytics/events", self.config.server_url);
        let mut synced_ids = Vec::new();
        let total = llm_events.len();

        for event in llm_events {
            let event_id = event.event_id();
            let request = event_to_request(&event, &self.config.ic_token);

            match self.http_client.post(&url).json(&request).send().await {
                Ok(resp) if resp.status().is_success() => {
                    synced_ids.push(event_id);
                }
                Ok(resp) => {
                    let status = resp.status();
                    // Don't retry 4xx errors
                    if status.is_client_error() {
                        synced_ids.push(event_id);
                    }
                    tracing::warn!("Event sync failed: {}", status);
                }
                Err(e) => {
                    tracing::warn!("Event sync error: {}", e);
                }
            }
        }

        if !synced_ids.is_empty() {
            self.event_store.mark_synced(&synced_ids);
            tracing::debug!("Synced {}/{} events", synced_ids.len(), total);
        }

        synced_ids.len()
    }
}

/// Event payload for POST /api/v1/analytics/events
/// Matches Protocol 012 AnalyticsEventRequest format
#[derive(Debug, Serialize)]
struct EventRequest {
    ic_token: String,
    event_id: String,
    timestamp_ms: i64,  // Server expects i64
    event_type: String,
    model: String,
    provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_tokens: Option<i64>,  // Server expects i64
    #[serde(skip_serializing_if = "Option::is_none")]
    output_tokens: Option<i64>,  // Server expects i64
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_micros: Option<i64>,  // Server expects i64
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
}

fn event_to_request(event: &AnalyticsEvent, ic_token: &str) -> EventRequest {
    match &event.payload {
        EventPayload::LlmRequestCompleted(data) => EventRequest {
            ic_token: ic_token.to_string(),
            event_id: event.event_id().to_string(),
            timestamp_ms: event.timestamp_ms() as i64,
            event_type: "llm_request_completed".to_string(),
            model: data.meta.model.to_string(),
            provider: data.meta.provider.to_string(),
            input_tokens: Some(data.input_tokens as i64),
            output_tokens: Some(data.output_tokens as i64),
            cost_micros: Some(data.cost_micros as i64),
            provider_id: data.meta.provider_id.as_ref().map(|s| s.to_string()),
            error_code: None,
            error_message: None,
        },
        EventPayload::LlmRequestFailed(data) => EventRequest {
            ic_token: ic_token.to_string(),
            event_id: event.event_id().to_string(),
            timestamp_ms: event.timestamp_ms() as i64,
            event_type: "llm_request_failed".to_string(),
            model: data.meta.model.to_string(),
            provider: data.meta.provider.to_string(),
            input_tokens: None,
            output_tokens: None,
            cost_micros: None,
            provider_id: data.meta.provider_id.as_ref().map(|s| s.to_string()),
            error_code: data.error_code.clone(),
            error_message: data.error_message.clone(),
        },
        _ => EventRequest {
            ic_token: ic_token.to_string(),
            event_id: event.event_id().to_string(),
            timestamp_ms: event.timestamp_ms() as i64,
            event_type: "unknown".to_string(),
            model: String::new(),
            provider: "unknown".to_string(),
            input_tokens: None,
            output_tokens: None,
            cost_micros: None,
            provider_id: None,
            error_code: None,
            error_message: None,
        },
    }
}
