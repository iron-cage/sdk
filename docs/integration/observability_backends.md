# Observability Backends

**Purpose:** Export metrics and traces to existing monitoring tools.

---

## User Need

See Iron Cage data in existing monitoring dashboards, not a separate system.

## Core Idea

**OpenTelemetry as universal export protocol:**

```
Iron Cage --OTLP--> OTEL Collector --export--> Datadog
                          |              --> New Relic
                          |              --> Grafana
                          +------------> Custom
```

## Supported Backends

| Backend | Metrics | Traces | Logs |
|---------|---------|--------|------|
| Datadog | Yes | Yes | Yes |
| New Relic | Yes | Yes | Yes |
| Grafana/Prometheus | Yes | Yes | Yes |
| AWS CloudWatch | Yes | No | Yes |
| Custom OTLP | Yes | Yes | Yes |

## Iron Cage Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `ironcage.llm.tokens` | Counter | Tokens consumed |
| `ironcage.llm.cost_usd` | Counter | Cost in dollars |
| `ironcage.safety.violations` | Counter | Safety blocks |
| `ironcage.agent.latency_ms` | Histogram | Request latency |

## Configuration

```yaml
observability:
  exporter: otlp
  otlp:
    endpoint: https://otel.company.com:4317
    headers:
      api-key: ${OTEL_API_KEY}
```

---

*Related: [identity_providers.md](identity_providers.md)*
