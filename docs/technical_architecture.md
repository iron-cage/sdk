# Iron Cage Platform - Technical Architecture

**Version:** 1.0.0
**Last Updated:** 2025-01-20
**Status:** Draft - System Design Document

### Scope

**Responsibility:** Complete system architecture defining HOW all 8 capabilities are built and integrated.

**In Scope:**
- System architecture (control plane, data plane, agent runtime)
- Component interactions and request flows
- Technology stack and data layer decisions
- Security model and scalability targets

**Out of Scope:**
- Deployment procedures (see `deployment_guide.md`)
- Product specifications (see `/spec/`)
- Business strategy (see `/business/`)

### Deployment Mode Note

**⚠️ ARCHITECTURE SCOPE:** This document describes the **full production platform architecture** integrating all 8 capabilities (K8s deployment, multi-region, PostgreSQL + Redis infrastructure).

**For current pilot implementation architecture** (subset of capabilities, single-process deployment), see [docs/deployment_packages.md](../../docs/deployment_packages.md) § Deployment Modes and [pilot/spec.md](../..../pilot/spec.md) for pilot scope.

**Relationship:**
- **This Document:** Full 8-capability production platform vision
- **Pilot Mode (Current):** Subset implementation (capabilities 1, 2, 4 only) for conference demo
- **Production Mode (Future):** Will implement full architecture described in this document

---

## Executive Summary

This document defines the technical architecture for the Iron Cage platform - a unified AI governance platform integrating 8 capabilities into a cohesive system. It covers system design, component interactions, data flows, deployment architecture, and technology stack decisions.

**Platform Capabilities:**
1. **Production Agent Runtime** (Cap 1) - K8s-based agent orchestration
2. **Unified LLM Access Control** (Cap 2) - Multi-provider LLM gateway
3. **Safe Execution Environment** (Cap 3) - Sandboxed code execution
4. **AI Safety Guardrails** (Cap 4) - Input/output/tool security [BUILD SECOND]
5. **Credential Management** (Cap 5) - Secrets management
6. **Zero-Config MCP** (Cap 6) - Model Context Protocol integration
7. **Comprehensive Observability** (Cap 7) - Monitoring & tracing
8. **Enterprise Data Access** (Cap 8) - RAG & vector DB [BUILD FIRST]

**Architecture Goals:**
- **Modularity:** Each capability is independently deployable
- **Composability:** Capabilities integrate seamlessly (unified governance)
- **Scalability:** Support 10K concurrent agents, 1M requests/minute
- **Reliability:** 99.9% uptime SLA, multi-region deployment
- **Security:** Zero-trust architecture, encryption at rest/transit

---

## 1. High-Level System Architecture

### 1.1 Component Overview

```
┌───────────────────────────────────────────────────────────────────────────┐
│                          IRON CAGE PLATFORM                                │
│                        (Unified AI Governance)                             │
├───────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  ┌──────────────────────────────────────────────────────────────────┐    │
│  │                      CONTROL PLANE                                │    │
│  │  - API Gateway (Axum, authentication, rate limiting)             │    │
│  │  - Admin Control Panel (React SPA, policy management)                │    │
│  │  - Scheduler (agent lifecycle, cron jobs)                        │    │
│  └──────────────────────────────────────────────────────────────────┘    │
│                                 │                                          │
│  ┌──────────────────────────────┼──────────────────────────────────────┐ │
│  │                      DATA PLANE                                     │ │
│  ├────────────────┬────────────────┬────────────────┬────────────────┤ │
│  │                │                │                │                │ │
│  │  CAPABILITY 8  │  CAPABILITY 4  │  CAPABILITY 2  │  CAPABILITY 7  │ │
│  │   (Data        │   (Safety      │   (LLM         │   (Observ-     │ │
│  │    Access)     │    Guardrails) │    Gateway)    │    ability)    │ │
│  │                │                │                │                │ │
│  │ - Connectors   │ - Input FW     │ - Multi-LLM    │ - Metrics      │ │
│  │ - ETL Pipeline │ - Output FW    │ - Cost Track   │ - Traces       │ │
│  │ - Vector DB    │ - Tool Proxy   │ - Rate Limit   │ - Logs         │ │
│  │ - RAG Query    │ - Audit Log    │ - Caching      │ - Alerts       │ │
│  └────────────────┴────────────────┴────────────────┴────────────────┘ │
│                                 │                                          │
│  ┌──────────────────────────────┼──────────────────────────────────────┐ │
│  │                      AGENT RUNTIME (Capability 1)                   │ │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐               │ │
│  │  │ Agent 1 │  │ Agent 2 │  │ Agent 3 │  │ Agent N │               │ │
│  │  │(LangChain)  │(CrewAI) │  │(AutoGen)│  │(Custom) │               │ │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘               │ │
│  │         │            │            │            │                     │ │
│  │         └────────────┴────────────┴────────────┘                     │ │
│  │                         │                                             │ │
│  │  ┌──────────────────────▼─────────────────────────────────────────┐ │ │
│  │  │  SHARED INFRASTRUCTURE                                          │ │ │
│  │  │  - Capability 3: Safe Execution (E2B, sandboxes)               │ │ │
│  │  │  - Capability 5: Credential Management (Vault)                 │ │ │
│  │  │  - Capability 6: MCP Integration (discovery, deployment)       │ │ │
│  │  └─────────────────────────────────────────────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                            │
└───────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Request Flow (End-to-End)

**Scenario:** User asks agent to query enterprise data and execute code

```
1. USER REQUEST
   │
   ├─▶ API Gateway (authentication, rate limiting)
   │
2. CAPABILITY 4: INPUT FIREWALL
   │
   ├─▶ Prompt injection detection (95%+ accuracy, <50ms)
   ├─▶ privacy protection (detect, optionally block)
   ├─▶ Content moderation (OpenAI API)
   │
3. CAPABILITY 1: AGENT RUNTIME
   │
   ├─▶ Route to agent (LangChain, CrewAI, AutoGen)
   │
4. AGENT DECISION: Query data
   │
   ├─▶ CAPABILITY 8: DATA ACCESS
   │   │
   │   ├─▶ Generate query embedding (OpenAI, with caching)
   │   ├─▶ Vector search (Pinecone/Weaviate/Qdrant)
   │   ├─▶ Apply access control (row-level security)
   │   └─▶ Return results to agent
   │
5. AGENT DECISION: Execute code
   │
   ├─▶ CAPABILITY 4: TOOL PROXY
   │   │
   │   ├─▶ Authorization check (whitelist/blacklist)
   │   ├─▶ Parameter validation
   │   ├─▶ Human-in-loop approval (if required)
   │   │
   │   └─▶ CAPABILITY 3: SAFE EXECUTION
   │       │
   │       ├─▶ E2B sandbox (isolated environment)
   │       └─▶ Execute code, return result
   │
6. AGENT GENERATES RESPONSE
   │
   ├─▶ CAPABILITY 2: LLM GATEWAY
   │   │
   │   ├─▶ Route to provider (OpenAI, Anthropic, etc.)
   │   ├─▶ Cost tracking (tokens, $)
   │   └─▶ Response caching (60% cost reduction)
   │
7. CAPABILITY 4: OUTPUT FIREWALL
   │
   ├─▶ Secret scanning (API keys, passwords)
   ├─▶ PII redaction (SSN, credit card)
   │
8. CAPABILITY 7: OBSERVABILITY
   │
   ├─▶ Log request/response (audit trail)
   ├─▶ Record metrics (latency, cost, violations)
   ├─▶ Distributed tracing (end-to-end span)
   │
9. RETURN RESPONSE TO USER
```

**Total Latency Budget:**
- API Gateway: 5ms
- Input Firewall: 50ms
- Agent Runtime: 100ms
- Data Access (Cap 8): 500ms (vector search + access control)
- Tool Proxy + Sandbox (Cap 3+4): 2000ms (code execution)
- LLM Gateway (Cap 2): 3000ms (model inference)
- Output Firewall: 30ms
- Observability (async): 0ms (non-blocking)
- **Total: ~5.7 seconds** (acceptable for complex agent workflow)

---

## 2. Component Architecture (Detailed)

### 2.1 Capability 1: Production Agent Runtime

**Purpose:** Orchestrate agent lifecycle (deploy, scale, monitor, terminate).

**Architecture:**
```
┌─────────────────────────────────────────────────────┐
│            AGENT RUNTIME (Kubernetes)                │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │         RUNTIME CONTROLLER                   │   │
│  │  - Agent deployment (Helm charts)            │   │
│  │  - Scaling (HPA based on CPU/memory)         │   │
│  │  - Health checks (liveness, readiness)       │   │
│  └─────────────────────────────────────────────┘   │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         AGENT PODS (Kubernetes)              │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │  │
│  │  │ LangChain│  │  CrewAI  │  │ AutoGen  │  │  │
│  │  │  Agent   │  │  Agent   │  │  Agent   │  │  │
│  │  └──────────┘  └──────────┘  └──────────┘  │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Key Design Decisions:**
- **Leverage Kubernetes:** Don't rebuild orchestration (use K8s primitives)
- **Framework-agnostic:** Support LangChain, CrewAI, AutoGen, custom agents
- **Minimal wrapper:** K8s Helm charts + health check endpoints (8-12 weeks)

**Integration Points:**
- Cap 2 (LLM Gateway): Agents call LLM API through gateway
- Cap 3 (Sandbox): Agents execute code via sandbox API
- Cap 4 (Guardrails): All agent inputs/outputs flow through firewalls
- Cap 7 (Observability): Runtime controller reports metrics to observability service

**Technology Stack:**
- Kubernetes (EKS, GKE, or AKS)
- Helm (agent deployment charts)
- Go or Rust (runtime controller)

---

### 2.2 Capability 2: Unified LLM Access Control

**Purpose:** Centralized gateway for all LLM API calls with cost tracking, rate limiting, caching.

**Architecture:**
```
┌─────────────────────────────────────────────────────┐
│              LLM GATEWAY (Fork of LiteLLM)          │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │         ROUTING LAYER                        │   │
│  │  - Provider routing (OpenAI, Anthropic, etc.)│   │
│  │  - Load balancing (round-robin, least-loaded)│   │
│  │  - Fallback logic (if primary fails)         │   │
│  └─────────────────────────────────────────────┘   │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         COST TRACKING                        │  │
│  │  - Token counting (input, output)            │  │
│  │  - Cost calculation ($0.01/1K tokens)        │  │
│  │  - Attribution (by agent, user, tenant)      │  │
│  └──────────────────────────────────────────────┘  │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         SEMANTIC CACHING                     │  │
│  │  - Cache frequent queries (60% hit rate)     │  │
│  │  - Similarity search (cosine distance)       │  │
│  │  - TTL management (24 hours default)         │  │
│  └──────────────────────────────────────────────┘  │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         RATE LIMITING                        │  │
│  │  - Per-user limits (1K requests/hour)        │  │
│  │  - Per-agent limits (10K requests/hour)      │  │
│  │  - Per-tenant limits (100K requests/hour)    │  │
│  └──────────────────────────────────────────────┘  │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         PROVIDER CLIENTS                     │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐     │  │
│  │  │ OpenAI  │  │Anthropic│  │  Cohere │     │  │
│  │  └─────────┘  └─────────┘  └─────────┘     │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Key Design Decisions:**
- **Fork LiteLLM:** Open-source gateway (MIT license, active community)
- **Add Iron Cage features:** Cost attribution, access control integration, observability hooks
- **Timeline:** 2-3 months (fork + customize)

**Integration Points:**
- Cap 1 (Runtime): Agents use gateway for all LLM calls
- Cap 4 (Guardrails): Gateway passes requests through input/output firewalls
- Cap 5 (Credentials): Gateway fetches API keys from credential service
- Cap 7 (Observability): Gateway reports metrics (cost, latency, provider)

**Technology Stack:**
- Python (LiteLLM is Python-based)
- Redis (caching, rate limiting)
- PostgreSQL (cost tracking, audit logs)

---

### 2.3 Capability 3: Safe Execution Environment

**Purpose:** Sandboxed code execution for agents (Python, JavaScript, shell commands).

**Architecture:**
```
┌─────────────────────────────────────────────────────┐
│         SAFE EXECUTION (E2B Partnership)            │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │         EXECUTION MANAGER                    │   │
│  │  - Request validation                        │   │
│  │  - Timeout enforcement (30s default)         │   │
│  │  - Resource limits (1 CPU, 2GB RAM)          │   │
│  └─────────────────────────────────────────────┘   │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         E2B SANDBOXES                        │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │  │
│  │  │ Python   │  │JavaScript│  │  Shell   │  │  │
│  │  │ Sandbox  │  │ Sandbox  │  │ Sandbox  │  │  │
│  │  └──────────┘  └──────────┘  └──────────┘  │  │
│  │  - Isolated filesystem (tmpfs)              │  │
│  │  - No network access (except whitelisted)   │  │
│  │  - Read-only system files                   │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Key Design Decisions:**
- **Partner with E2B:** Don't build sandboxes in-house (E2B dominates market)
- **White-label integration:** E2B provides SDK, we wrap with Iron Cage policies
- **Timeline:** 2-3 months (integration + policy layer)

**Integration Points:**
- Cap 4 (Guardrails): Tool proxy authorizes code execution before calling sandbox
- Cap 7 (Observability): Sandbox reports execution metrics (duration, resource usage)

**Technology Stack:**
- E2B SDK (Python, TypeScript)
- Rust wrapper (execution manager)

---

### 2.4 Capability 4: AI Safety Guardrails

**Purpose:** Input validation, output filtering, tool authorization. **See spec/capability_4_ai_safety_guardrails.md for full details.**

**Architecture:**
```
┌─────────────────────────────────────────────────────┐
│              AI SAFETY GUARDRAILS                    │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │         LAYER 1: INPUT FIREWALL              │   │
│  │  - Prompt injection detection (95%+ acc)     │   │
│  │  - privacy protection (regex + NER)               │   │
│  │  - Content moderation (OpenAI API)           │   │
│  └─────────────────────────────────────────────┘   │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         LAYER 2: OUTPUT FIREWALL             │  │
│  │  - Secret scanning (TruffleHog patterns)     │  │
│  │  - PII redaction (full/partial/hash)         │  │
│  └──────────────────────────────────────────────┘  │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         LAYER 3: TOOL PROXY                  │  │
│  │  - Authorization (whitelist/blacklist)       │  │
│  │  - Parameter validation (constraints)        │  │
│  │  - Human-in-loop (Slack approval)            │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Key Design Decisions:**
- **Complete stack:** Input + output + tool (NO competitor has all 3)
- **Agent-specific:** Tool authorization is unique to Iron Cage
- **Timeline:** 4-6 months (BUILD SECOND priority)

**Integration Points:**
- Cap 1 (Runtime): All agent inputs/outputs routed through guardrails
- Cap 3 (Sandbox): Tool proxy intercepts code execution requests
- Cap 7 (Observability): Guardrails log all violations, approvals

**Technology Stack:**
- Rust (firewall services)
- Python (ML models: BERT for prompt injection, spaCy for NER)
- PostgreSQL (policies, audit logs)
- Redis (cache, rate limiting)

---

### 2.5 Capability 5: Credential Management

**Purpose:** Centralized secrets management for agents (API keys, database passwords).

**Note:** Pilot uses custom implementation (iron_secrets), full platform uses HashiCorp Vault. See `../pilot/spec.md` for pilot approach and `spec/capability_5_credential_management.md` for migration path.

**Architecture:**
```
┌─────────────────────────────────────────────────────┐
│         CREDENTIAL MANAGEMENT (Vault)               │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │         CREDENTIAL SERVICE                   │   │
│  │  - CRUD API for secrets                      │   │
│  │  - Rotation reminders (90 days)              │   │
│  │  - Audit logs (who accessed what)            │   │
│  └─────────────────────────────────────────────┘   │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         HASHICORP VAULT                      │  │
│  │  - Encrypted storage (AES-256)               │  │
│  │  - Dynamic secrets (generate on-demand)      │  │
│  │  - Lease management (TTL, renewal)           │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Key Design Decisions:**
- **Leverage Vault:** HashiCorp Vault is de facto standard (don't rebuild)
- **Thin wrapper:** API layer for Iron Cage-specific features (rotation reminders, LLM control panel)
- **Timeline:** 1-2 months (thin component)

**Integration Points:**
- Cap 2 (LLM Gateway): Fetches LLM API keys from credential service
- Cap 8 (Data Access): Fetches data source credentials (Salesforce OAuth, database passwords)

**Technology Stack:**
- HashiCorp Vault (secrets storage)
- Rust (credential service wrapper)

---

### 2.6 Capability 6: Zero-Config MCP

**Purpose:** Discover, deploy, and manage Model Context Protocol (MCP) servers.

**Architecture:**
```
┌─────────────────────────────────────────────────────┐
│         MCP INTEGRATION (GitHub Registry)           │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │         MCP DISCOVERY SERVICE                │   │
│  │  - Pull metadata from GitHub MCP Registry    │   │
│  │  - Security scanning (43% have vulns!)       │   │
│  │  - Governance (approval workflows)           │   │
│  └─────────────────────────────────────────────┘   │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         MCP DEPLOYMENT                       │  │
│  │  - One-click deploy (Docker/K8s)             │  │
│  │  - Configuration management (env vars)       │  │
│  │  - Integration with credential service       │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Key Design Decisions:**
- **Don't compete with GitHub Registry:** Integrate, don't replace
- **Add governance layer:** Security hardening, compliance checks
- **Timeline:** 3-5 weeks (thin component)

**Integration Points:**
- Cap 5 (Credentials): MCP servers fetch API keys from credential service
- Cap 7 (Observability): MCP usage metrics reported to observability

**Technology Stack:**
- Rust (MCP discovery service)
- Kubernetes (MCP server deployment)

---

### 2.7 Capability 7: Comprehensive Observability

**Purpose:** Unified monitoring, tracing, logging for all platform components.

**Architecture:**
```
┌─────────────────────────────────────────────────────┐
│         OBSERVABILITY (Multi-Partner)               │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │         METRICS (Prometheus + Grafana)       │   │
│  │  - Request rate (QPS)                        │   │
│  │  - Latency (p50, p95, p99)                   │   │
│  │  - Error rate (5xx)                          │   │
│  │  - Cost (LLM spend, vector DB queries)       │   │
│  └─────────────────────────────────────────────┘   │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         TRACING (OpenTelemetry + Jaeger)     │  │
│  │  - Distributed traces (end-to-end spans)     │  │
│  │  - Service dependencies (call graph)         │  │
│  │  - Bottleneck identification                 │  │
│  └──────────────────────────────────────────────┘  │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         LOGGING (CloudWatch or ELK)          │  │
│  │  - Structured JSON logs                      │  │
│  │  - Log aggregation (across services)         │  │
│  │  - Query interface (Kibana)                  │  │
│  └──────────────────────────────────────────────┘  │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         ALERTING (PagerDuty)                 │  │
│  │  - Threshold alerts (latency > 5s)           │  │
│  │  - Anomaly detection (ML-based)              │  │
│  │  - Incident management (on-call rotation)    │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Key Design Decisions:**
- **Multi-partner approach:** Leverage best-of-breed tools (Prometheus, OpenTelemetry, CloudWatch)
- **Unified control panel:** Single pane of glass for all observability data
- **Timeline:** 2-3 months (integration + control panel)

**Integration Points:**
- **All capabilities:** Every component reports metrics, traces, logs to observability service

**Technology Stack:**
- Prometheus (metrics storage)
- Grafana (metrics visualization)
- OpenTelemetry (tracing SDK)
- Jaeger (tracing backend)
- CloudWatch or ELK (logging)
- PagerDuty (alerting)

---

### 2.8 Capability 8: Enterprise Data Access for AI

**Purpose:** Connect enterprise data sources to AI agents via ETL, vectorization, RAG. **See spec/capability_8_enterprise_data_access.md for full details.**

**Architecture:**
```
┌─────────────────────────────────────────────────────┐
│         ENTERPRISE DATA ACCESS (RAG Platform)       │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │         DATA CONNECTORS (20+)                │   │
│  │  - Salesforce, Jira, Confluence, GSuite...  │   │
│  │  - OAuth2 authentication                     │   │
│  │  - Webhook-driven real-time sync             │   │
│  └─────────────────────────────────────────────┘   │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         PROCESSING PIPELINE                  │  │
│  │  - Document extraction (PDF, DOCX, etc.)     │  │
│  │  - Chunking (512 tokens, semantic)           │  │
│  │  - Embedding generation (OpenAI, Cohere)     │  │
│  │  - Semantic caching (60% cost reduction)     │  │
│  └──────────────────────────────────────────────┘  │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         VECTOR DATABASE (Multi-vendor)       │  │
│  │  - Pinecone (default, hosted)                │  │
│  │  - Weaviate (hybrid search)                  │  │
│  │  - Qdrant (self-hosted, performance)         │  │
│  └──────────────────────────────────────────────┘  │
│                      │                               │
│  ┌───────────────────▼──────────────────────────┐  │
│  │         RAG QUERY SERVICE                    │  │
│  │  - Semantic search (vector similarity)       │  │
│  │  - Access control (row-level security)       │  │
│  │  - Reranking (cross-encoder)                 │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Key Design Decisions:**
- **Complete end-to-end platform:** ETL + vectorization + RAG (NO competitor has all)
- **Real-time sync:** Webhook-driven (NOT batch ETL like competitors)
- **Timeline:** 6-9 months (BUILD FIRST priority)

**Integration Points:**
- Cap 1 (Runtime): Agents query RAG service for enterprise data
- Cap 4 (Guardrails): Data access policies enforced via access control engine
- Cap 5 (Credentials): Data connectors fetch credentials from credential service
- Cap 7 (Observability): Data pipeline reports metrics (sync status, embedding cost)

**Technology Stack:**
- Rust (connector service, processing pipeline)
- Python (document processing: Unstructured.io, spaCy)
- PostgreSQL (metadata, sync state)
- Redis (semantic cache)
- Pinecone/Weaviate/Qdrant (vector databases)

---

## 3. Data Architecture

### 3.1 Databases

**PostgreSQL (Primary Metadata Store):**
- **Schema: `agents`** - Agent configurations, deployments
- **Schema: `policies`** - Guardrail policies, tool authorization rules
- **Schema: `audit_logs`** - All requests, violations, approvals
- **Schema: `cost_tracking`** - LLM spend by agent/user/tenant
- **Schema: `data_sources`** - Connector configs, sync state
- **Replication:** Multi-AZ (master-replica) for high availability

**Redis (Caching & Queuing):**
- **Database 0:** Semantic cache (LLM responses, embeddings)
- **Database 1:** Rate limiting (per-user, per-agent)
- **Database 2:** Session storage (admin control panel)
- **Streams:** Webhook queue, background jobs
- **Cluster:** Redis Cluster (3 masters, 3 replicas)

**Vector Databases (Multi-Vendor):**
- **Pinecone:** Default hosted option (serverless, easiest setup)
- **Weaviate:** Self-hosted or cloud (hybrid search, better performance)
- **Qdrant:** Self-hosted (cost optimization, highest performance)
- **Collection per tenant:** Logical isolation (collection naming: `tenant_<id>_data`)

**S3 (Object Storage):**
- **Bucket: `iron-cage-audit-logs`** - Long-term audit log archival (7 years retention)
- **Bucket: `iron-cage-documents`** - Raw documents from data connectors
- **Bucket: `iron-cage-backups`** - Database backups (daily)

### 3.2 Data Flow Diagram

```
┌─────────────────────────────────────────────────────┐
│                  DATA SOURCES                        │
│  (Salesforce, Jira, Confluence, GSuite, etc.)       │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ (1) Webhook or polling
                  ▼
┌─────────────────────────────────────────────────────┐
│         DATA CONNECTORS (Capability 8)              │
│  - Fetch records (paginated)                        │
│  - Store raw documents in S3                        │
│  - Update sync state in PostgreSQL                  │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ (2) Process documents
                  ▼
┌─────────────────────────────────────────────────────┐
│         PROCESSING PIPELINE (Capability 8)          │
│  - Extract text (Unstructured.io)                   │
│  - Chunk (512 tokens, semantic boundaries)          │
│  - Generate embeddings (OpenAI API)                 │
│  - Check semantic cache (Redis)                     │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ (3) Upsert vectors
                  ▼
┌─────────────────────────────────────────────────────┐
│         VECTOR DATABASE (Pinecone/Weaviate/Qdrant) │
│  - Upsert vectors (with metadata)                   │
│  - Build index (ANN: HNSW, IVF)                     │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ (4) Agent queries data
                  ▼
┌─────────────────────────────────────────────────────┐
│         RAG QUERY SERVICE (Capability 8)            │
│  - Generate query embedding                         │
│  - Apply access control (row-level security)        │
│  - Vector search (kNN)                              │
│  - Rerank results (cross-encoder)                   │
│  - Return to agent                                  │
└─────────────────────────────────────────────────────┘
```

---

## 4. Security Architecture

### 4.1 Zero-Trust Principles

**Authentication:**
- **External API:** JWT tokens (issued by API Gateway)
- **Inter-service:** mTLS (mutual TLS with certificate rotation)
- **Admin Control Panel:** OAuth2 (Google, GitHub, SAML for enterprise)

**Authorization:**
- **API Gateway:** RBAC (role-based access control)
- **Guardrails:** Policy engine (whitelist/blacklist, parameter constraints)
- **Data Access:** Row-level security (enforced at query time)

**Network Security:**
- **Zero-trust network:** All traffic encrypted (TLS 1.3)
- **Service mesh:** Istio (for mTLS, traffic management)
- **Firewall:** Network policies (K8s NetworkPolicy)

### 4.2 Data Protection

**Encryption at Rest:**
- **PostgreSQL:** AES-256 (AWS RDS encryption)
- **S3:** AES-256 (server-side encryption)
- **Vector DB:** Provider-specific encryption (Pinecone: AES-256)

**Encryption in Transit:**
- **All traffic:** TLS 1.3
- **Inter-service:** mTLS (certificate rotation every 90 days)

**Secret Management:**
- **Credentials:** HashiCorp Vault (AES-256 encryption)
- **Rotation:** Automated rotation every 90 days (with reminders)

**PII Protection:**
- **Input firewall:** Detect PII, optionally block/redact
- **Output firewall:** Redact PII before logging
- **Audit logs:** Redact PII/secrets before writing to S3

### 4.3 Compliance

**SOC2 Type II:**
- Access controls (authentication, authorization)
- Audit logs (all requests logged to S3)
- Encryption (at rest, in transit)
- Incident response (PagerDuty, runbooks)

**GDPR:**
- Right to be forgotten (API for data deletion)
- Data portability (export user data to JSON)
- Consent management (explicit opt-in for data processing)

**HIPAA (for healthcare customers):**
- Business Associate Agreement (BAA)
- PHI encryption (at rest, in transit)
- Access controls (role-based, audit logs)
- Breach notification (24-hour SLA)

---

## 5. Deployment Architecture

### 5.1 Multi-Region Deployment

```
┌─────────────────────────────────────────────────────┐
│                   US-EAST-1 (Primary)                │
├─────────────────────────────────────────────────────┤
│  - EKS Cluster (10 nodes, t3.xlarge)                │
│  - PostgreSQL RDS (db.r5.xlarge, Multi-AZ)          │
│  - Redis Cluster (3 masters, 3 replicas)            │
│  - S3 Buckets (audit logs, documents, backups)      │
│  - Pinecone Serverless (us-east-1 region)           │
└─────────────────────────────────────────────────────┘
                       │
                       │ (Replication)
                       ▼
┌─────────────────────────────────────────────────────┐
│                   EU-WEST-1 (Secondary)              │
├─────────────────────────────────────────────────────┤
│  - EKS Cluster (10 nodes, t3.xlarge)                │
│  - PostgreSQL RDS Read Replica                      │
│  - Redis Cluster (3 masters, 3 replicas)            │
│  - S3 Buckets (cross-region replication)            │
│  - Pinecone Serverless (eu-west-1 region)           │
└─────────────────────────────────────────────────────┘
```

**Geographic Distribution:**
- **Primary region:** US-EAST-1 (North America customers)
- **Secondary region:** EU-WEST-1 (European customers, GDPR compliance)
- **Tertiary region:** AP-SOUTHEAST-1 (APAC customers, future)

**Failover Strategy:**
- **DNS-based failover:** Route53 health checks (failover to secondary in 60 seconds)
- **Database:** Read replicas promoted to primary (RPO: 5 minutes)
- **Stateless services:** Horizontal scaling (K8s HPA)

### 5.2 Kubernetes Architecture

**Namespaces:**
- `control-plane` - API Gateway, Admin Control Panel, Scheduler
- `data-access` - Capability 8 (connectors, processing, vector DB)
- `guardrails` - Capability 4 (input/output firewalls, tool proxy)
- `llm-gateway` - Capability 2 (LLM routing, caching)
- `observability` - Capability 7 (Prometheus, Grafana, Jaeger)
- `agent-runtime` - Capability 1 (agent pods)

**Resource Allocation:**
- **Control Plane:** 5 nodes (t3.xlarge, 4 vCPU, 16GB RAM)
- **Data Access:** 10 nodes (c5.2xlarge, 8 vCPU, 16GB RAM)
- **Guardrails:** 5 nodes (t3.xlarge, 4 vCPU, 16GB RAM)
- **LLM Gateway:** 3 nodes (t3.xlarge, 4 vCPU, 16GB RAM)
- **Agent Runtime:** 20 nodes (c5.2xlarge, 8 vCPU, 16GB RAM, autoscaling 10-50)

**Total Cost (AWS):**
- EKS cluster: $0.10/hour × 43 nodes × 730 hours = $3,139/month
- Node instances: $0.1664/hour × 43 nodes × 730 hours = $5,218/month
- PostgreSQL RDS: $1.36/hour × 730 hours = $993/month
- Redis Cluster: $0.68/hour × 6 nodes × 730 hours = $2,979/month
- **Total: ~$12K/month** (+ S3, data transfer, LLM API costs)

### 5.3 Disaster Recovery

**Backup Strategy:**
- **PostgreSQL:** Daily snapshots (retained 30 days), automated backups to S3
- **Redis:** AOF persistence (append-only file), daily snapshots to S3
- **Vector DB:** Daily exports to S3 (JSON format)
- **Audit logs:** Real-time replication to S3 (99.999999999% durability)

**Recovery Time Objectives:**
- **RTO (Recovery Time Objective):** 1 hour (time to restore service)
- **RPO (Recovery Point Objective):** 5 minutes (max data loss)

**DR Drills:**
- Monthly failover tests (switch to secondary region)
- Quarterly full DR simulation (complete region outage)

---

## 6. Monitoring & Observability

### 6.1 Key Metrics

**Golden Signals:**
- **Latency:** p50, p95, p99 (per endpoint, per capability)
- **Traffic:** Requests/second (by endpoint, agent, tenant)
- **Errors:** Error rate (4xx, 5xx), error types
- **Saturation:** CPU, memory, disk, network utilization

**Business Metrics:**
- **LLM Cost:** $ spent per agent/user/tenant (tracked in real-time)
- **Agent Uptime:** % of time agents are healthy (target: 99.9%)
- **Guardrail Violations:** # of blocked requests (by type: prompt injection, PII, etc.)
- **Data Sync Status:** # of connectors syncing, last sync time, errors

### 6.2 Alerting Rules

**Critical Alerts (PagerDuty, immediate response):**
- **High Error Rate:** > 5% for 5 minutes
- **High Latency:** p99 > 10s for 5 minutes
- **Service Down:** Health check failing for 3 consecutive checks
- **Database Unavailable:** PostgreSQL connection pool exhausted
- **Secret Detected:** Secret found in agent output (immediate incident)

**Warning Alerts (Slack, investigate within 1 hour):**
- **Elevated Error Rate:** > 1% for 10 minutes
- **High Latency:** p95 > 5s for 10 minutes
- **High Cost:** LLM spend exceeds budget by 20%
- **Sync Failures:** Data connector failing for 3+ consecutive attempts

### 6.3 Distributed Tracing

**Trace Context Propagation:**
- Every request gets unique `trace_id` (UUID)
- Spans created for each service call (e.g., "input_firewall", "llm_gateway", "vector_search")
- Trace context propagated via HTTP headers (`traceparent`, OpenTelemetry standard)

**Example Trace (End-to-End):**
```
Trace ID: abc123-def456-ghi789
│
├─ api_gateway (5ms)
│   └─ input_firewall (42ms)
│       ├─ prompt_injection_detection (38ms)
│       └─ pii_detection (4ms)
│
├─ agent_runtime (100ms)
│   └─ langchain_agent (100ms)
│       ├─ data_access (500ms)
│       │   ├─ generate_embedding (50ms)
│       │   ├─ vector_search (400ms)
│       │   └─ rerank (50ms)
│       │
│       └─ llm_gateway (3000ms)
│           ├─ semantic_cache (10ms) [MISS]
│           └─ openai_api (2990ms)
│
└─ output_firewall (18ms)
    ├─ secret_scanning (10ms)
    └─ pii_redaction (8ms)

Total: 5.7 seconds
```

---

## 7. Technology Stack Summary

| Layer | Technology | Justification |
|-------|------------|---------------|
| **Backend Services** | Rust | High performance, memory safety, async/await (tokio) |
| **LLM Gateway** | Python (LiteLLM fork) | Leverage open-source LiteLLM, active community |
| **ML Models** | Python (HuggingFace, spaCy) | Best ML ecosystem for NLP tasks |
| **API Framework** | Axum (Rust) | Fast, ergonomic, type-safe web framework |
| **Database** | PostgreSQL | ACID compliance, rich SQL features, proven reliability |
| **Cache/Queue** | Redis | In-memory speed, pub/sub, streams for background jobs |
| **Vector DB** | Pinecone/Weaviate/Qdrant | Multi-vendor support, best-of-breed options |
| **Container Orchestration** | Kubernetes | Industry standard, 96% production adoption |
| **Service Mesh** | Istio | mTLS, traffic management, observability |
| **Metrics** | Prometheus + Grafana | Open-source, rich ecosystem, powerful queries |
| **Tracing** | OpenTelemetry + Jaeger | Vendor-neutral, distributed tracing standard |
| **Logging** | CloudWatch or ELK | Managed service or self-hosted, structured logs |
| **CI/CD** | GitHub Actions | Integrated with GitHub, easy YAML config |
| **Infrastructure as Code** | Terraform | Multi-cloud support, declarative, mature tooling |
| **Secrets Management** | HashiCorp Vault | Industry standard, dynamic secrets, audit logs |
| **Frontend (Admin Control Panel)** | React + TypeScript | Rich ecosystem, type safety, component reusability |

---

## 8. Build Roadmap Integration

### 8.1 Phase 1: Core Standalone Products (Months 1-15)

**Capability 8: Enterprise Data Access (Months 1-9, 3-4 engineers)**
- Build: Connectors (20+), processing pipeline, vector DB integration, RAG query
- Integration: Standalone API, later integrate with Cap 1 (Runtime) and Cap 4 (Guardrails)

**Capability 4: AI Safety Guardrails (Months 10-15, 2-3 engineers)**
- Build: Input firewall, output firewall, tool proxy
- Integration: Standalone API, later integrate with Cap 1 (Runtime) and Cap 8 (Data Access)

### 8.2 Phase 2: Platform Integration (Months 6-18)

**Capability 2: LLM Gateway (Months 6-9, fork LiteLLM)**
- Build: Fork LiteLLM, add cost tracking, semantic caching, Iron Cage hooks
- Integration: Cap 1 (agents use gateway), Cap 4 (firewalls intercept), Cap 7 (metrics)

**Capability 3: Safe Execution (Months 12-15, E2B partnership)**
- Build: E2B SDK integration, execution manager, policy layer
- Integration: Cap 4 (tool proxy authorizes execution), Cap 7 (metrics)

**Capability 7: Observability (Months 15-18, multi-partner)**
- Build: Prometheus setup, OpenTelemetry SDKs, Grafana control panels
- Integration: All capabilities report metrics/traces/logs

### 8.3 Phase 3: Convenience Features (Months 12-18)

**Capability 5: Credentials (Months 12-14, 1 engineer)**
- Build: Vault wrapper, rotation reminders, LLM control panel
- Integration: Cap 2 (LLM keys), Cap 8 (data source credentials)

**Capability 6: MCP (Months 16-18, 1 engineer)**
- Build: GitHub Registry integration, security scanning, deployment
- Integration: Cap 5 (credentials), Cap 7 (metrics)

**Capability 1: Runtime (Months 12-15, minimal K8s wrapper)**
- Build: Helm charts, health checks, runtime controller
- Integration: Cap 2 (LLM gateway), Cap 4 (guardrails), Cap 8 (data access)

---

## 9. Open Questions & Decisions

1. **Multi-Tenancy Strategy:** Shared K8s cluster (namespaces per tenant) vs dedicated clusters (higher isolation, higher cost)?

2. **Vector DB Selection:** Default to Pinecone (easiest) vs Weaviate (better features) vs Qdrant (lowest cost)?

3. **LLM Gateway Deployment:** Standalone service (separate from agents) vs sidecar (co-located with agents)?

4. **Observability Data Retention:** How long to retain metrics (30 days?), traces (7 days?), logs (90 days?)?

5. **Disaster Recovery Automation:** Fully automated failover (risk of false positives) vs manual failover (slower, safer)?

6. **Pricing Model:** Per-agent pricing (simple) vs usage-based pricing (aligns with cost)?

7. **On-Premise Deployment:** Support air-gapped on-premise (high effort) or cloud-only (easier)?

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-01-20 | Platform Engineering | Initial technical architecture document. Defines high-level system architecture (control plane + data plane), component architecture for all 8 capabilities, data architecture (PostgreSQL, Redis, vector DBs, S3), security architecture (zero-trust, encryption, compliance), deployment architecture (multi-region, K8s, disaster recovery), monitoring & observability (golden signals, alerting, distributed tracing), technology stack summary, build roadmap integration. Ready for engineering review and stakeholder approval. |
