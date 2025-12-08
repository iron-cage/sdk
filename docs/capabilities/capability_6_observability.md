# Capability 6: Comprehensive Observability - Product Specification

**Version:** 1.0.0
**Last Updated:** 2025-01-20
**Status:** Draft - Platform Component Specification
**Build Priority:** Platform Component (35/100 standalone viability - multi-partner integration)

---

### Scope

**Responsibility:** Product specification for Observability capability (Capability 6 of 8 - platform component, 35/100, multi-partner integration)

**In Scope:**
- Market context (AI observability $1.4B→$10.7B, dominated by Datadog/New Relic/Langfuse)
- Strategic approach (multi-partner integration, do NOT build from scratch, integrate existing tools)
- Problem statement (fragmented observability, no unified AI-specific monitoring)
- Solution architecture (unified observability layer integrating metrics/traces/logs with AI-specific features)
- Platform integration (included in $100K-300K/year)
- Standalone viability score (35/100 - build as component via partnerships)

**Out of Scope:**
- Other capabilities, strategic analysis, pilot specs, implementation details

---

## Executive Summary

This specification defines the requirements for Iron Cage's Comprehensive Observability capability - unified monitoring, tracing, and logging for all platform components with AI-specific metrics (LLM cost, agent behavior, guardrail violations).

**Market Opportunity:** AI observability $1.4B → $10.7B (49% CAGR), LLMOps market $608M → $3.1B (38% CAGR)
**Strategic Approach:** Multi-partner integration (Prometheus, OpenTelemetry, Grafana) + AI-specific dashboards
**Build Timeline:** 2-3 months, leverage open-source tools + build custom dashboards
**Platform Pricing:** Included in $100K-300K/year Iron Cage platform (not sold separately)

**Core Value Proposition:** Replace scattered monitoring (separate tools for metrics, traces, logs, LLM costs) with unified observability platform providing single pane of glass for agent health, LLM spend, security violations, and performance.

**Strategic Recommendation:** LEVERAGE BEST-OF-BREED TOOLS (Prometheus, OpenTelemetry, Grafana, CloudWatch). Build Iron Cage-specific dashboards and AI-focused metrics. Do NOT build custom monitoring infrastructure.

---

## 1. Product Overview

### 1.1 Problem Statement

Scattered observability across multiple tools:

```
CURRENT STATE: Fragmented Monitoring
┌─────────────────────────────────────────────────────┐
│  Metrics (Prometheus)                                │
│  - CPU, memory, network                             │
│  - Generic API metrics                              │
│  - No LLM-specific metrics                          │
│                                                      │
│  Traces (Jaeger)                                    │
│  - Distributed tracing                              │
│  - No agent workflow visibility                     │
│                                                      │
│  Logs (CloudWatch)                                  │
│  - Application logs                                 │
│  - No guardrail violation logs                      │
│                                                      │
│  LLM Cost (Spreadsheets)                            │
│  - Manual tracking                                  │
│  - No real-time visibility                          │
│                                                      │
│  PAIN POINTS:                                       │
│  ❌ 4+ separate tools (no unified view)             │
│  ❌ No AI-specific metrics (LLM cost, agent health) │
│  ❌ No correlation (traces + metrics + logs)        │
│  ❌ Manual cost tracking (outdated, error-prone)    │
└─────────────────────────────────────────────────────┘
```

### 1.2 Solution: Iron Cage Unified Observability

```
IRON CAGE SOLUTION: Unified Observability Platform
┌─────────────────────────────────────────────────────┐
│  IRON CAGE OBSERVABILITY                            │
│  ┌─────────────────────────────────────────────┐   │
│  │   UNIFIED DASHBOARD (Grafana)                │   │
│  │   ┌─────────┐  ┌─────────┐  ┌─────────┐   │   │
│  │   │ Agent   │  │ LLM Cost│  │ Security│   │   │
│  │   │ Health  │  │ $5.2K/mo│  │ 12 Viola│   │   │
│  │   │ 98.5%   │  │ GPT-4   │  │ Prompt  │   │   │
│  │   │ Uptime  │  │ $3.1K   │  │ Inject. │   │   │
│  │   └─────────┘  └─────────┘  └─────────┘   │   │
│  └─────────────────────────────────────────────┘   │
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │   METRICS (Prometheus)                       │   │
│  │   - Agent uptime, latency, errors            │   │
│  │   - LLM cost (by model, agent, tenant)       │   │
│  │   - Guardrail violations (by type)           │   │
│  │   - Vector DB queries (rate, latency)        │   │
│  └─────────────────────────────────────────────┘   │
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │   TRACES (OpenTelemetry + Jaeger)           │   │
│  │   - End-to-end agent workflows               │   │
│  │   - Service dependencies (call graph)        │   │
│  │   - Bottleneck identification                │   │
│  └─────────────────────────────────────────────┘   │
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │   LOGS (CloudWatch or ELK)                   │   │
│  │   - Structured JSON logs                     │   │
│  │   - Log aggregation across services          │   │
│  │   - Query interface (Kibana)                 │   │
│  └─────────────────────────────────────────────┘   │
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │   ALERTING (PagerDuty)                       │   │
│  │   - Threshold alerts (latency > 5s)          │   │
│  │   - Anomaly detection (ML-based)             │   │
│  │   - Incident management                      │   │
│  └─────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 2. Functional Requirements

### 2.1 Metrics Collection (Prometheus)

**Requirement:** Collect and store time-series metrics from all platform components.

**Golden Signals (All Services):**
- **Latency:** p50, p95, p99 response times
- **Traffic:** Requests/second
- **Errors:** Error rate (4xx, 5xx)
- **Saturation:** CPU, memory, disk, network utilization

**AI-Specific Metrics:**
- **LLM Cost:**
  - Total spend ($/day, $/week, $/month)
  - Cost by model (GPT-4, Claude, Gemini)
  - Cost by agent
  - Cost by tenant
  - Cost per request (average)

- **Agent Health:**
  - Uptime percentage
  - Success rate (% of successful requests)
  - Average latency
  - Error rate by type

- **Guardrail Violations:**
  - Count by type (prompt injection, PII, secrets)
  - Violation rate (% of requests blocked)
  - False positive rate (user-reported)

- **Vector DB Performance:**
  - Query latency (p50, p99)
  - Query rate (queries/second)
  - Index size (GB)
  - Embedding cache hit rate

**Implementation:**
```rust
// src/observability/metrics.rs

pub struct MetricsCollector
{
  registry: prometheus::Registry,
}

impl MetricsCollector
{
  pub fn new() -> Self
  {
    let registry = prometheus::Registry::new();

    // Register AI-specific metrics
    registry.register( Box::new( LLM_COST_GAUGE.clone() ) ).unwrap();
    registry.register( Box::new( AGENT_UPTIME_GAUGE.clone() ) ).unwrap();
    registry.register( Box::new( GUARDRAIL_VIOLATION_COUNTER.clone() ) ).unwrap();

    Self { registry }
  }

  pub fn record_llm_cost
  (
    &self,
    model: &str,
    agent_id: &str,
    cost_usd: f64,
  )
  {
    LLM_COST_GAUGE
      .with_label_values( &[ model, agent_id ] )
      .set( cost_usd );
  }

  pub fn record_guardrail_violation
  (
    &self,
    violation_type: &str, // "prompt_injection", "pii", "secret"
    agent_id: &str,
  )
  {
    GUARDRAIL_VIOLATION_COUNTER
      .with_label_values( &[ violation_type, agent_id ] )
      .inc();
  }
}

// Prometheus metrics
lazy_static!
{
  static ref LLM_COST_GAUGE: prometheus::GaugeVec = prometheus::register_gauge_vec!
  (
    "iron_cage_llm_cost_usd",
    "LLM cost in USD",
    &[ "model", "agent_id" ]
  ).unwrap();

  static ref AGENT_UPTIME_GAUGE: prometheus::GaugeVec = prometheus::register_gauge_vec!
  (
    "iron_cage_agent_uptime_percentage",
    "Agent uptime percentage",
    &[ "agent_id" ]
  ).unwrap();

  static ref GUARDRAIL_VIOLATION_COUNTER: prometheus::IntCounterVec = prometheus::register_int_counter_vec!
  (
    "iron_cage_guardrail_violations_total",
    "Total guardrail violations",
    &[ "violation_type", "agent_id" ]
  ).unwrap();
}
```

### 2.2 Distributed Tracing (OpenTelemetry + Jaeger)

**Requirement:** End-to-end distributed tracing for agent workflows.

**Trace Context Propagation:**
- Every request gets unique `trace_id` (UUID)
- Spans created for each service call
- Trace context propagated via HTTP headers (`traceparent`)

**Example Trace (User Query → Agent → LLM → Data Access):**
```
Trace ID: abc123-def456-ghi789
│
├─ api_gateway (5ms)
│
├─ input_firewall (42ms)
│   ├─ prompt_injection_detection (38ms)
│   └─ pii_detection (4ms)
│
├─ agent_runtime (100ms)
│   └─ langchain_agent (100ms)
│       │
│       ├─ data_access_query (500ms)
│       │   ├─ generate_embedding (50ms)
│       │   ├─ vector_search (400ms)
│       │   └─ rerank (50ms)
│       │
│       └─ llm_gateway (3000ms)
│           ├─ semantic_cache_lookup (10ms) [MISS]
│           └─ openai_api_call (2990ms)
│
└─ output_firewall (18ms)
    ├─ secret_scanning (10ms)
    └─ pii_redaction (8ms)

Total: 5.7 seconds
```

**Implementation:**
```rust
// src/observability/tracing.rs

use opentelemetry::trace::{ Tracer, TracerProvider };

pub struct TracingService
{
  tracer: Box< dyn Tracer >,
}

impl TracingService
{
  pub fn create_span
  (
    &self,
    name: &str,
    parent_context: Option< Context >,
  ) -> Span
  {
    self.tracer
      .span_builder( name )
      .with_parent( parent_context )
      .start( &self.tracer )
  }

  pub fn record_attribute
  (
    &self,
    span: &Span,
    key: &str,
    value: &str,
  )
  {
    span.set_attribute( opentelemetry::KeyValue::new( key, value ) );
  }
}
```

### 2.3 Logging (CloudWatch or ELK Stack)

**Requirement:** Centralized logging with structured JSON format.

**Log Fields:**
- `timestamp` (ISO 8601)
- `level` (INFO, WARN, ERROR, CRITICAL)
- `service` (api_gateway, llm_gateway, etc.)
- `trace_id` (for correlation with traces)
- `agent_id`
- `user_id`
- `tenant_id`
- `message`
- `metadata` (arbitrary JSON)

**Example Log:**
```json
{
  "timestamp": "2025-01-20T10:00:00.123Z",
  "level": "ERROR",
  "service": "input_firewall",
  "trace_id": "abc123-def456-ghi789",
  "agent_id": "agent-001",
  "user_id": "user-123",
  "tenant_id": "tenant-xyz",
  "message": "Prompt injection detected",
  "metadata": {
    "confidence": 0.97,
    "attack_type": "direct_injection"
  }
}
```

### 2.4 Alerting (PagerDuty + Slack)

**Requirement:** Automated alerts for critical issues.

**Critical Alerts (PagerDuty, immediate response):**
- **High error rate:** > 5% for 5 minutes
- **High latency:** p99 > 10s for 5 minutes
- **Service down:** Health check failing for 3 consecutive checks
- **Budget exceeded:** LLM spend > budget limit
- **Secret detected:** Secret found in agent output

**Warning Alerts (Slack, investigate within 1 hour):**
- **Elevated error rate:** > 1% for 10 minutes
- **High latency:** p95 > 5s for 10 minutes
- **High LLM cost:** Spend exceeds forecast by 20%
- **Guardrail violations:** > 100 violations/hour

**Alert Configuration:**
```yaml
# alerts.yml

- name: HighErrorRate
  condition: error_rate > 0.05
  duration: 5m
  severity: critical
  channel: pagerduty

- name: HighLatency
  condition: p99_latency > 10s
  duration: 5m
  severity: critical
  channel: pagerduty

- name: BudgetExceeded
  condition: llm_cost > budget_limit
  duration: 1m
  severity: critical
  channel: pagerduty + slack

- name: HighGuardrailViolations
  condition: guardrail_violations > 100/hour
  duration: 10m
  severity: warning
  channel: slack
```

### 2.5 Unified Dashboard (Grafana)

**Requirement:** Single pane of glass for all observability data.

**Dashboards:**

**1. Platform Overview Dashboard:**
- Total requests/second (all services)
- Total error rate
- Total LLM cost (last 24h, last 7d, last 30d)
- Agent health summary (uptime %, success rate)
- Top 5 most expensive models
- Recent alerts (last 24h)

**2. Agent Dashboard:**
- Per-agent metrics:
  - Uptime percentage
  - Average latency
  - Success rate
  - LLM cost breakdown (by model)
  - Guardrail violation count
- Agent workflow trace (example trace)

**3. LLM Cost Dashboard:**
- Cost trends (daily, weekly, monthly)
- Cost by model (pie chart)
- Cost by agent (bar chart)
- Cost by tenant (bar chart)
- Cost forecast (next 30 days)
- Budget utilization (% of budget spent)

**4. Security Dashboard:**
- Guardrail violations (by type, over time)
- Blocked requests (count, percentage)
- PII detections (count, by agent)
- Secret detections (count, by agent)
- Failed authorization attempts (count, by agent)

---

## 3. Non-Functional Requirements

### 3.1 Performance

**Metrics Collection:**
- Overhead: < 1% CPU per service
- Latency: Metrics collection adds < 1ms to request time

**Log Shipping:**
- Latency: Logs shipped to CloudWatch/ELK within 10 seconds
- Throughput: 100K logs/second

**Dashboard Query:**
- Load time: p50 < 2s, p99 < 5s (for Grafana dashboards)

### 3.2 Reliability

**Availability:**
- Prometheus: 99.9% uptime
- Grafana: 99.9% uptime
- Jaeger: 99.9% uptime

**Data Retention:**
- Metrics: 90 days (Prometheus)
- Traces: 7 days (Jaeger)
- Logs: 90 days (hot storage), 7 years (S3 cold storage)

---

## 4. Technical Architecture

### 4.1 Technology Stack

**Metrics:**
- Prometheus (metrics storage, time-series DB)
- Grafana (dashboards, visualization)

**Tracing:**
- OpenTelemetry SDK (trace instrumentation)
- Jaeger (trace storage, visualization)

**Logging:**
- CloudWatch (AWS) OR
- ELK Stack (Elasticsearch, Logstash, Kibana)

**Alerting:**
- PagerDuty (incident management)
- Slack (notifications)

### 4.2 Deployment Architecture

```
┌─────────────────────────────────────────────────────┐
│        OBSERVABILITY STACK (Kubernetes)             │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │     PROMETHEUS (Metrics)                     │   │
│  │     - 2 replicas (HA)                        │   │
│  │     - 90 days retention                      │   │
│  │     - 10s scrape interval                    │   │
│  └─────────────────────────────────────────────┘   │
│                       │                              │
│  ┌────────────────────▼────────────────────────┐   │
│  │     GRAFANA (Dashboards)                     │   │
│  │     - 2 replicas (HA)                        │   │
│  │     - Iron Cage custom dashboards            │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │     JAEGER (Traces)                          │   │
│  │     - OpenTelemetry collector                │   │
│  │     - Cassandra backend (7 days retention)   │   │
│  └─────────────────────────────────────────────┘   │
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │     CLOUDWATCH (Logs)                        │   │
│  │     - Structured JSON logs                   │   │
│  │     - 90 days hot storage                    │   │
│  │     - S3 archival (7 years)                  │   │
│  └─────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 5. Integration with Other Capabilities

**All capabilities report to observability:**
- Cap 1 (Runtime): Agent uptime, latency, errors
- Cap 2 (LLM Gateway): LLM cost, request rate, cache hit rate
- Cap 3 (Sandbox): Execution count, duration, resource usage
- Cap 4 (Guardrails): Violation count, false positive rate
- Cap 5 (Credentials): Secret access count, rotation status
- Cap 6 (MCP): MCP server usage, deployment status
- Cap 8 (Data Access): Vector DB query rate, embedding cost

---

## 6. Build Roadmap

### Phase 1: Core Infrastructure (Months 15-16)

- ✅ Deploy Prometheus + Grafana
- ✅ Deploy OpenTelemetry + Jaeger
- ✅ Deploy CloudWatch (or ELK)
- ✅ Configure PagerDuty integration

### Phase 2: AI-Specific Dashboards (Month 17)

- ✅ LLM cost dashboard
- ✅ Agent health dashboard
- ✅ Security dashboard (guardrail violations)
- ✅ Platform overview dashboard

### Phase 3: Advanced Features (Month 18)

- ✅ Anomaly detection (ML-based alerts)
- ✅ Cost forecasting (predictive analytics)
- ✅ Custom alert rules (per-tenant)

---

## 7. Success Metrics

### Product Metrics (Month 18)

**Adoption:**
- 100% of services reporting metrics
- 100% of requests traced
- 100% of logs centralized

**Performance:**
- Dashboard load time < 2s
- Alert latency < 60s (from issue to notification)

**Reliability:**
- 99.9% uptime (Prometheus, Grafana, Jaeger)

---

## 8. Open Questions

1. **Log Storage:** CloudWatch (managed, $X/GB) vs ELK Stack (self-hosted, lower cost, more ops)?

2. **Trace Sampling:** Sample 100% of traces (high storage cost) vs 10% sampling (miss rare issues)?

3. **Alerting Logic:** Rule-based (simple, deterministic) vs ML-based anomaly detection (catches novel issues, false positives)?

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-01-20 | Platform Engineering | Initial product specification for Capability 7 (Comprehensive Observability). Defines functional requirements (metrics collection with AI-specific metrics, distributed tracing, centralized logging, alerting, unified dashboards), non-functional requirements (performance <1% CPU overhead, 99.9% uptime), technical architecture (Prometheus, Grafana, OpenTelemetry, Jaeger, CloudWatch/ELK, PagerDuty), integration with all capabilities, build roadmap (2-3 months), success metrics. Strategic recommendation: LEVERAGE BEST-OF-BREED TOOLS (don't rebuild monitoring infrastructure), build Iron Cage-specific dashboards. Ready for engineering review. |
