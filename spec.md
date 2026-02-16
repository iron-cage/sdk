# System Specification

- **Name:** Iron Cage - AI Agent Governance SDK
- **Version:** 0.1.0
- **Date:** 2026-02-12
- **Status:** PILOT

### **Table of Contents**

- [System Specification](#system-specification)
    - [**Part I: Public Contract (Mandatory Requirements)**](#part-i-public-contract-mandatory-requirements)
        - [1. Vision \& Scope](#1-vision--scope)
            - [1.1. Project Vision](#11-project-vision)
            - [1.2. Problem Solved](#12-problem-solved)
            - [1.3. In Scope (Pilot)](#13-in-scope-pilot)
            - [1.4. In Scope (Production)](#14-in-scope-production)
            - [1.5. Out of Scope](#15-out-of-scope)
        - [2. Ubiquitous Language](#2-ubiquitous-language)
        - [3. System Actors](#3-system-actors)
        - [4. Functional Requirements](#4-functional-requirements)
        - [5. Non-Functional Requirements](#5-non-functional-requirements)
        - [6. External System Interfaces](#6-external-system-interfaces)
    - [**Part II: Internal Design (Design Recommendations)**](#part-ii-internal-design-design-recommendations)

---

## **Part I: Public Contract (Mandatory Requirements)**

### 1. Vision & Scope

#### 1.1. Project Vision

To build an open-source AI agent governance platform that provides production-grade safety controls, cost enforcement, and reliability patterns for AI agents in regulated industries — while maintaining complete data privacy through 100% local execution.

Most AI frameworks assume R&D. Iron Cage assumes production in regulated industries.

#### 1.2. Problem Solved

Organizations deploying AI agents in production face critical risks that existing frameworks ignore:

* **Runaway costs:** A single misconfigured agent can burn through thousands of dollars in LLM API calls within minutes, with no centralized kill switch.
* **Data leaks:** Agents processing sensitive data may inadvertently include PII in prompts or receive it in responses, creating compliance violations.
* **Vendor lock-in:** Switching between LLM providers (OpenAI, Anthropic, Google) requires rewriting agent code, raising migration costs.
* **No audit trail:** Regulated industries (finance, healthcare, legal) require complete accountability for every AI decision, but most frameworks provide no structured logging.
* **Cascading failures:** When an LLM provider goes down, agents fail catastrophically instead of gracefully degrading.

#### 1.3. In Scope (Pilot)

The Pilot phase delivers a functional single-developer environment with core governance features.

* **Agent Runtime:** Local LLM proxy (Gateway) with transparent API key injection and multi-provider routing (OpenAI, Anthropic).
* **Safety:** Regex-based PII detection (email, US phone) with automatic redaction on both input and output.
* **Cost Control:** Microdollar-precision budget tracking with atomic reservation system. Budget borrowing from Control Panel in $10 portions.
* **Reliability:** Circuit breaker pattern (Closed/Open/HalfOpen) with per-service isolation.
* **Token Management:** Two-token system — IC Token (agent→Control Panel) and IP Token (Control Panel→LLM Provider). BCrypt-hashed storage.
* **Secrets:** AES-256-GCM encrypted secret storage with Argon2id key derivation.
* **Control API:** REST API (Axum) for agent management, budget status, provider configuration, and authentication (JWT + API tokens).
* **Dashboard:** Vue 3 web dashboard for real-time monitoring.
* **CLI:** Command-line interface for token management and configuration.
* **Python SDK:** PyO3-based bindings (`pip install iron-cage`) exposing `LlmRouter` to Python agents.
* **Data Store:** SQLite for all persistent state (single-process, zero-config).

#### 1.4. In Scope (Production)

Production extends Pilot with enterprise-grade scalability and multi-tenant support.

* All Pilot features.
* **Multi-Tenant:** Full organizational hierarchy (Master Budget → Project Budget → Agent Budget) with role-based access control.
* **Horizontal Scaling:** Stateless API servers behind load balancer, PostgreSQL replacing SQLite.
* **Advanced Safety:** ML-based PII detection, prompt injection detection, credential exposure scanning.
* **Provider Expansion:** Google Gemini support, custom provider adapters.
* **Advanced Reliability:** Automatic failover between providers, retry with exponential backoff.
* **Analytics:** Usage forecasting, cost trend analysis, anomaly detection.
* **Compliance:** SOC 2 audit trail, GDPR data handling controls, immutable audit logs.
* **Deployment:** Docker Compose and Kubernetes-ready packaging.

#### 1.5. Out of Scope

* Hosting or managing LLM models (Iron Cage is a governance layer, not an inference platform).
* Agent code execution sandboxing (filesystem/process isolation is planned post-production).
* Multi-cloud deployment orchestration.
* Building AI agents themselves — Iron Cage wraps existing agents.
* GUI agent builder or visual workflow editor.

### 2. Ubiquitous Language

Canonical glossary of 80+ project terms covering actors, tokens, budgets, processing layers, and architectural concepts.

* **[docs/vocabulary.md](docs/vocabulary.md)**

### 3. System Actors

18 actors across three types (Human, Software, Service) with roles, responsibilities, access levels, and communication protocols. Defines the three-role RBAC model (Admin, User, Viewer) and service failure modes (fail-safe vs fail-open).

* **[docs/deployment/002_actor_model.md](docs/deployment/002_actor_model.md)** — complete actor taxonomy
* **[docs/architecture/006_roles_and_permissions.md](docs/architecture/006_roles_and_permissions.md)** — RBAC permission matrix

### 4. Functional Requirements

43 functional requirements covering Core Runtime, Safety, Cost Control, Reliability, Credential Management, Observability, Configuration, and CLI. Each requirement includes acceptance criteria and priority (MUST/SHOULD/COULD).

* **[spec/requirements.md](spec/requirements.md)**

### 5. Non-Functional Requirements

Measurable quality targets across five attributes: Performance (<5ms gateway overhead), Reliability (99.9% availability, fail-safe defaults), Scalability (10K+ agents), Security (defense-in-depth, BCrypt/AES-256-GCM/Argon2id), and Usability (Pythonic API, zero-config defaults).

* **[docs/principles/002_quality_attributes.md](docs/principles/002_quality_attributes.md)** — quality attribute targets
* **[docs/constraints/004_trade_offs.md](docs/constraints/004_trade_offs.md)** — latency budget and trade-off rationale

### 6. External System Interfaces

Protocol specifications for all external interfaces: LLM Provider APIs (OpenAI, Anthropic, Google), Control Panel REST API, WebSocket real-time events, token management, authentication, and budget control.

* **[docs/protocol/readme.md](docs/protocol/readme.md)** — all 15 protocol specifications
* **[docs/integration/001_llm_providers.md](docs/integration/001_llm_providers.md)** — LLM provider integration patterns

---

## **Part II: Internal Design (Design Recommendations)**

Architecture, design philosophy, crate catalog, data stores, and security model for the Iron Cage platform. Covers the three-boundary model (Developer Zone / Control Plane / Provider Zone), six processing layers, 11-step request flow, and dual-token security architecture.

* **[docs/architecture/000_high_level_overview.md](docs/architecture/000_high_level_overview.md)** — comprehensive architecture with diagrams
* **[docs/architecture/002_layer_model.md](docs/architecture/002_layer_model.md)** — six processing layers
* **[docs/architecture/004_data_flow.md](docs/architecture/004_data_flow.md)** — 11-step request flow
* **[docs/security/readme.md](docs/security/readme.md)** — threat model, isolation, credential flow, audit
* **[docs/principles/001_design_philosophy.md](docs/principles/001_design_philosophy.md)** — seven guiding principles
* **[codestyle.md](codestyle.md)** — Rust formatting standards
