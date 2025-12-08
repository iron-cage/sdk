# iron_telemetry

Structured logging and tracing for all Iron Cage crates.

### Scope

**Responsibilities:**
Provides centralized logging abstraction using the `tracing` crate. Outputs structured, timestamped, colored logs for terminal display and machine-readable JSON for storage. Injects agent context into all log events for traceability.

**In Scope:**
- Structured logging via `tracing` crate
- Terminal output formatting (timestamp, colors, human-readable)
- Log levels (DEBUG, INFO, WARN, ERROR, CRIT, OK)
- Agent context injection (agent_id in all events)
- Specialized logging functions (PII detections, budget warnings, circuit breaker events)
- Environment-based log level configuration
- JSON export for audit compliance

**Out of Scope:**
- Log aggregation to external services (Datadog, Splunk, Grafana)
- Log sampling and filtering (future)
- Distributed tracing (OpenTelemetry integration)
- Log rotation and archival (future)
- Metrics collection (Prometheus, StatsD)
- Custom log formatters (future)
- Dashboard log display (see iron_dashboard)

## Installation

```toml
[dependencies]
iron_telemetry = { path = "../iron_telemetry" }
```

## Example

```rust
use iron_telemetry::{init_logging, log_agent_event, log_pii_detection};

// Initialize logging with default configuration
init_logging()?;

// Log agent lifecycle events
log_agent_event("agent-001", "Agent started processing leads");

// Log PII detection (specialized format)
log_pii_detection("agent-001", "email", "john@example.com");

// Output: [14:32:05] INFO  agent-001 | Agent started processing leads
// Output: [14:32:06] WARN  agent-001 | PII DETECTED: email redacted
```

## License

Apache-2.0
