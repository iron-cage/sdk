# Directory: src

## Responsibility Table

| File | Responsibility |
|------|----------------|
| event.rs | Defines analytics event types and payloads |
| event_storage.rs | Implements lock-free event storage with counters |
| lib.rs | Exports analytics module public API |
| provider_utils.rs | Provides utility functions for analytics |
| recording.rs | Provides high-level recording API for events |
| stats.rs | Defines statistics types for aggregation |
| sync.rs | Syncs events to Control API background |

## Validation

One-Second Test: Scan the Responsibility column - if any two entries sound similar, they overlap and must be reconsolidated.
