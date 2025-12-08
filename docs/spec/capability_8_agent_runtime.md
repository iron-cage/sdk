# Capability 8: Production Agent Runtime - Product Specification

**Version:** 1.0.0
**Last Updated:** 2025-01-20
**Status:** Draft - Minimal Component Specification
**Build Priority:** Minimal K8s Wrapper (18/100 standalone viability - DO NOT build standalone)

---

### Scope

**Responsibility:** Product specification for Production Agent Runtime capability (Capability 8 of 8 - LOWEST priority, minimal K8s wrapper only)

**In Scope:**
- Strategic recommendation to NOT build proprietary runtime (leverage Kubernetes, LangChain/CrewAI/AutoGen instead)
- Minimal K8s wrapper requirements (automated deployment, health monitoring, governance integration)
- Market opportunity analysis (container orchestration $679M→$2.37B at 14.7% CAGR, Kubernetes 96% dominance)
- Problem statement (manual agent deployment with no governance integration, YAML complexity, no observability)
- Solution architecture (Kubernetes + minimal controller in Go or Rust, 8-12 weeks build)
- Platform integration (included in $100K-300K/year Iron Cage platform, NOT sold separately)
- Standalone viability score (18/100 - LOWEST of all 8 capabilities, DO NOT build standalone)
- Competitive positioning (avoid competing with Kubernetes, avoid rebuilding orchestration)

**Out of Scope:**
- Other 7 capabilities (see `capability_1_enterprise_data_access.md` through `capability_7_mcp_integration.md`)
- Comprehensive strategic analysis (see `/business/strategy/executive_summary.md` for all 8 capabilities ranked)
- System architecture implementation (see `/docs/architecture.md` for HOW to build)
- Warsaw pilot specifications (see `../pilot/spec.md` for 28 pilot features focusing on safety/cost, not full orchestration)
- Implementation guide (see `/runtime/PILOT_GUIDE.md` for step-by-step build instructions)
- Rust crate dependencies (see `../pilot/crates.md` for dependency specifications)
- Technology stack (see `../pilot/tech_stack.md` for Rust/Python/React setup)
- Business model and licensing (see `/business/business_model.md` for open core strategy)
- Competitor research (see `/research/competitors/capability_8_competitors_2025.md` for analysis)

---

## Executive Summary

This specification defines the requirements for Iron Cage's Production Agent Runtime capability - a lightweight Kubernetes wrapper for orchestrating AI agent lifecycles with health checks and integration hooks for other Iron Cage capabilities.

**Market Opportunity:** Container orchestration $679M → $2.37B (14.7% CAGR), dominated by Kubernetes (96% production adoption)
**Strategic Approach:** Minimal K8s wrapper (NOT proprietary runtime, NOT competing with Kubernetes)
**Build Timeline:** 8-12 weeks, leverage Kubernetes + Helm + minimal controller
**Platform Pricing:** Included in $100K-300K/year Iron Cage platform (not sold separately)

**Core Value Proposition:** Replace manual agent deployment (kubectl apply, manual scaling, no governance integration) with managed runtime providing automated deployment, health monitoring, and unified governance (integrated with Caps 2-8).

**Strategic Recommendation:** DO NOT BUILD PROPRIETARY RUNTIME. Leverage Kubernetes (don't rebuild orchestration), leverage LangChain/CrewAI/AutoGen (don't rebuild frameworks). Build MINIMAL wrapper (8-12 weeks) integrating Iron Cage governance.

---

## 1. Product Overview

### 1.1 Problem Statement

Manual agent deployment without governance:

```
CURRENT STATE: Manual Agent Deployment
┌─────────────────────────────────────────────────────┐
│  DEPLOY AGENT                                        │
│  1. Write Kubernetes YAML (50+ lines)               │
│  2. kubectl apply -f agent.yaml                     │
│  3. Manually configure:                             │
│     - LLM API endpoints (hardcoded URLs)            │
│     - API keys (env vars, insecure)                 │
│     - No guardrails integration                     │
│     - No observability hooks                        │
│  4. Manual scaling (kubectl scale)                  │
│  5. Manual health checks                            │
│                                                      │
│  PAIN POINTS:                                       │
│  ❌ No governance integration (LLM gateway, safety) │
│  ❌ Manual deployment (YAML complexity)             │
│  ❌ No unified observability                        │
│  ❌ No cost attribution                             │
└─────────────────────────────────────────────────────┘
```

### 1.2 Solution: Iron Cage Agent Runtime

```
IRON CAGE SOLUTION: Managed Agent Runtime
┌─────────────────────────────────────────────────────┐
│  IRON CAGE RUNTIME (Kubernetes + Controller)        │
│  ┌─────────────────────────────────────────────┐   │
│  │   RUNTIME CONTROLLER (Go or Rust)           │   │
│  │   - Agent deployment (Helm charts)           │   │
│  │   - Auto-scaling (HPA based on CPU/memory)   │   │
│  │   - Health checks (liveness, readiness)      │   │
│  │   - Governance hooks (integrate Caps 2-8)    │   │
│  └─────────────────┬───────────────────────────┘   │
│                    │                                 │
│  ┌─────────────────▼───────────────────────────┐   │
│  │   AGENT PODS (Kubernetes)                    │   │
│  │   ┌──────────┐  ┌──────────┐  ┌──────────┐ │   │
│  │   │LangChain │  │ CrewAI   │  │ AutoGen  │ │   │
│  │   │  Agent   │  │  Agent   │  │  Agent   │ │   │
│  │   │          │  │          │  │          │ │   │
│  │   │ ✅ LLM   │  │ ✅ LLM   │  │ ✅ LLM   │ │   │
│  │   │ Gateway  │  │ Gateway  │  │ Gateway  │ │   │
│  │   │ ✅ Guard │  │ ✅ Guard │  │ ✅ Guard │ │   │
│  │   │ rails    │  │ rails    │  │ rails    │ │   │
│  │   │ ✅ Obs.  │  │ ✅ Obs.  │  │ ✅ Obs.  │ │   │
│  │   └──────────┘  └──────────┘  └──────────┘ │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  GOVERNANCE INTEGRATION                              │
│  - Cap 2: LLM Gateway (cost tracking, caching)      │
│  - Cap 3: Sandbox (code execution)                  │
│  - Cap 4: Guardrails (input/output/tool security)   │
│  - Cap 5: Credentials (API keys from Vault)         │
│  - Cap 7: Observability (metrics, traces, logs)     │
│  - Cap 8: Data Access (RAG, vector DB)              │
└─────────────────────────────────────────────────────┘
```

---

## 2. Functional Requirements

### 2.1 Agent Deployment (Helm Charts)

**Requirement:** Deploy agents via simple API call (not kubectl apply).

**API:**
```rust
// src/runtime/deployment.rs

pub struct RuntimeController
{
  k8s_client: Arc< KubernetesClient >,
  helm_client: Arc< HelmClient >,
}

impl RuntimeController
{
  pub async fn deploy_agent
  (
    &self,
    config: AgentConfig,
  ) -> Result< AgentDeployment >
  {
    // 1. Generate Helm chart
    let helm_chart = self.generate_helm_chart( &config )?;

    // 2. Inject Iron Cage environment variables
    let env_vars = self.build_env_vars( &config )?;

    // 3. Deploy via Helm
    let deployment = self.helm_client
      .install( &helm_chart, &env_vars )
      .await?;

    // 4. Wait for readiness
    self.wait_for_ready( &deployment.name, Duration::from_secs( 300 ) ).await?;

    Ok( AgentDeployment
    {
      id: deployment.name,
      status: DeploymentStatus::Running,
      endpoints: deployment.endpoints,
    })
  }

  fn build_env_vars( &self, config: &AgentConfig ) -> Result< HashMap< String, String > >
  {
    let mut env = HashMap::new();

    // Iron Cage integration endpoints
    env.insert( "LLM_API_BASE_URL".into(), "https://llm-gateway.ironcage.svc".into() ); // Cap 2
    env.insert( "GUARDRAILS_API_URL".into(), "https://guardrails.ironcage.svc".into() ); // Cap 4
    env.insert( "SANDBOX_API_URL".into(), "https://sandbox.ironcage.svc".into() ); // Cap 3
    env.insert( "DATA_ACCESS_API_URL".into(), "https://data-access.ironcage.svc".into() ); // Cap 8

    // Agent-specific config
    env.insert( "AGENT_ID".into(), config.id.clone() );
    env.insert( "AGENT_NAME".into(), config.name.clone() );
    env.insert( "TENANT_ID".into(), config.tenant_id.clone() );

    // Observability (Cap 7)
    env.insert( "OTEL_EXPORTER_OTLP_ENDPOINT".into(), "http://otel-collector.observability.svc".into() );

    Ok( env )
  }
}

pub struct AgentConfig
{
  pub id: String,
  pub name: String,
  pub tenant_id: String,
  pub framework: AgentFramework, // LangChain, CrewAI, AutoGen, Custom
  pub image: String, // Docker image
  pub resources: ResourceLimits,
  pub replicas: usize, // Initial replica count
  pub auto_scaling: Option< AutoScalingConfig >,
}

pub enum AgentFramework
{
  LangChain { version: String },
  CrewAI { version: String },
  AutoGen { version: String },
  Custom, // User-provided Docker image
}

pub struct ResourceLimits
{
  pub cpu: String, // e.g., "1000m" (1 CPU)
  pub memory: String, // e.g., "2Gi"
}

pub struct AutoScalingConfig
{
  pub min_replicas: usize,
  pub max_replicas: usize,
  pub target_cpu_utilization: usize, // e.g., 70 (70%)
}
```

### 2.2 Health Monitoring

**Requirement:** Continuous health checks for all agents.

**Health Check Types:**
- **Liveness Probe:** Is agent process alive? (HTTP GET `/healthz`)
- **Readiness Probe:** Is agent ready to serve requests? (HTTP GET `/ready`)
- **Startup Probe:** Did agent start successfully? (HTTP GET `/startup`)

**Health Status:**
- **Healthy:** All probes passing
- **Degraded:** Some probes failing (warnings)
- **Unhealthy:** Liveness probe failing (restart agent)

**Implementation:**
```rust
// src/runtime/health.rs

pub struct HealthMonitor
{
  k8s_client: Arc< KubernetesClient >,
}

impl HealthMonitor
{
  pub async fn check_agent_health
  (
    &self,
    agent_id: &str,
  ) -> Result< HealthStatus >
  {
    // 1. Get pod status from Kubernetes
    let pods = self.k8s_client
      .get_pods_for_agent( agent_id )
      .await?;

    let mut healthy_pods = 0;
    let mut total_pods = pods.len();

    for pod in pods
    {
      if pod.status == "Running" && pod.ready
      {
        healthy_pods += 1;
      }
    }

    // 2. Compute health status
    let health_percentage = ( healthy_pods as f64 / total_pods as f64 ) * 100.0;

    let status = if health_percentage >= 80.0
    {
      HealthStatus::Healthy
    }
    else if health_percentage >= 50.0
    {
      HealthStatus::Degraded
    }
    else
    {
      HealthStatus::Unhealthy
    };

    Ok( status )
  }
}

pub enum HealthStatus
{
  Healthy,
  Degraded,
  Unhealthy,
}
```

### 2.3 Auto-Scaling (Kubernetes HPA)

**Requirement:** Automatically scale agents based on CPU/memory utilization.

**Scaling Triggers:**
- CPU utilization > 70% → Scale up
- CPU utilization < 30% → Scale down
- Memory utilization > 80% → Scale up
- Custom metrics (request rate, queue depth)

**Implementation:** Leverage Kubernetes Horizontal Pod Autoscaler (HPA)

**Example HPA Configuration:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: agent-001-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: agent-001
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### 2.4 Agent Lifecycle Management

**Requirement:** Support full agent lifecycle (create, start, stop, restart, delete).

**API:**
```rust
// src/runtime/lifecycle.rs

impl RuntimeController
{
  /// Create agent (deploy to K8s)
  pub async fn create_agent( &self, config: AgentConfig ) -> Result< AgentDeployment >
  {
    self.deploy_agent( config ).await
  }

  /// Stop agent (scale to 0 replicas, preserve config)
  pub async fn stop_agent( &self, agent_id: &str ) -> Result< () >
  {
    self.k8s_client
      .scale_deployment( agent_id, 0 )
      .await
  }

  /// Start agent (scale from 0 to configured replicas)
  pub async fn start_agent( &self, agent_id: &str ) -> Result< () >
  {
    let config = self.get_agent_config( agent_id ).await?;
    self.k8s_client
      .scale_deployment( agent_id, config.replicas )
      .await
  }

  /// Restart agent (rolling restart of all pods)
  pub async fn restart_agent( &self, agent_id: &str ) -> Result< () >
  {
    self.k8s_client
      .rollout_restart( agent_id )
      .await
  }

  /// Delete agent (remove K8s deployment)
  pub async fn delete_agent( &self, agent_id: &str ) -> Result< () >
  {
    self.k8s_client
      .delete_deployment( agent_id )
      .await
  }
}
```

---

## 3. Non-Functional Requirements

### 3.1 Performance

**Deployment Time:**
- Agent deployment: p50 < 60s, p99 < 120s (from API call to running)
- Health check: p50 < 100ms, p99 < 500ms

**Scaling:**
- Scale up: < 60s (from trigger to new pods running)
- Scale down: < 30s (from trigger to pods terminating)

### 3.2 Reliability

**Availability:**
- Runtime controller: 99.9% uptime
- Agents: 99.5% uptime (varies by agent quality)

**Fault Tolerance:**
- Automatic pod restart on failure (K8s liveness probe)
- Multi-AZ deployment (pods spread across availability zones)

### 3.3 Scalability

**Agent Capacity:**
- 1000+ concurrent agents per cluster
- 10K+ total agent pods

**Throughput:**
- 100 agent deployments/minute
- 1000 health checks/second

---

## 4. Technical Architecture

### 4.1 Technology Stack

**Runtime Controller:**
- Go (K8s operator pattern, strong K8s client library) OR
- Rust (if integrating with other Iron Cage Rust services)

**Orchestration:**
- Kubernetes (EKS, GKE, or AKS)
- Helm (deployment templating)

**Monitoring:**
- Prometheus (metrics collection)
- Kubernetes Metrics Server (auto-scaling metrics)

### 4.2 Deployment Architecture

```
┌─────────────────────────────────────────────────────┐
│      AGENT RUNTIME (Kubernetes Cluster)             │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │   RUNTIME CONTROLLER (Go/Rust)              │   │
│  │   - API server (REST API for agent CRUD)    │   │
│  │   - Kubernetes operator (watch K8s events)  │   │
│  │   - Health monitor (periodic checks)         │   │
│  └─────────────────┬───────────────────────────┘   │
│                    │                                 │
│  ┌─────────────────▼───────────────────────────┐   │
│  │   KUBERNETES CONTROL PLANE                   │   │
│  │   - Deployments (agent pods)                 │   │
│  │   - Services (agent endpoints)               │   │
│  │   - HPA (auto-scaling)                       │   │
│  │   - ConfigMaps (agent config)                │   │
│  └─────────────────┬───────────────────────────┘   │
│                    │                                 │
│  ┌─────────────────▼───────────────────────────┐   │
│  │   AGENT PODS (Worker Nodes)                 │   │
│  │   ┌──────────┐  ┌──────────┐  ┌──────────┐ │   │
│  │   │ Agent 1  │  │ Agent 2  │  │ Agent N  │ │   │
│  │   │(LangChain)  │(CrewAI)  │  │(AutoGen) │ │   │
│  │   └──────────┘  └──────────┘  └──────────┘ │   │
│  │   - CPU: 1000m (1 CPU)                       │   │
│  │   - Memory: 2Gi                              │   │
│  │   - Replicas: 2-10 (auto-scaled)             │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 5. Integration with Other Capabilities

**ALL capabilities integrate through agent runtime:**

- **Cap 2 (LLM Gateway):** Agents use gateway via env var `LLM_API_BASE_URL`
- **Cap 3 (Sandbox):** Agents execute code via sandbox API `SANDBOX_API_URL`
- **Cap 4 (Guardrails):** All agent I/O flows through guardrails `GUARDRAILS_API_URL`
- **Cap 5 (Credentials):** Agent fetches secrets from credential service (implicit, via Cap 2/8)
- **Cap 6 (MCP):** MCP servers deployed in same K8s cluster, agents discover via service DNS
- **Cap 7 (Observability):** Agents report metrics/traces via OpenTelemetry `OTEL_EXPORTER_OTLP_ENDPOINT`
- **Cap 8 (Data Access):** Agents query RAG service via `DATA_ACCESS_API_URL`

---

## 6. Build Roadmap

### Phase 1: Basic Runtime (Months 12-13)

- ✅ Runtime controller (Go/Rust, basic CRUD)
- ✅ Helm chart generation
- ✅ Agent deployment via API
- ✅ Health checks (liveness, readiness)

### Phase 2: Governance Integration (Months 14-15)

- ✅ Inject Iron Cage env vars (LLM Gateway, Guardrails, etc.)
- ✅ Auto-scaling (HPA configuration)
- ✅ Multi-tenant support (namespace isolation)

---

## 7. Success Metrics

### Product Metrics (Month 15)

**Adoption:**
- 100% of agents deployed via Iron Cage runtime (mandatory)
- 100+ agents deployed per deployment

**Performance:**
- Deployment time < 60s (p50)
- 99.5% agent uptime

**Governance:**
- 100% of agents integrated with LLM Gateway (Cap 2)
- 100% of agents integrated with Guardrails (Cap 4)

---

## 8. Open Questions

1. **Controller Language:** Go (better K8s ecosystem) vs Rust (consistency with other Iron Cage services)?

2. **Agent Isolation:** Shared K8s cluster (lower cost) vs dedicated cluster per tenant (higher isolation)?

3. **Agent Frameworks:** Pre-built images for LangChain/CrewAI/AutoGen (easier) vs user-provided Docker images only (more flexible)?

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-01-20 | Platform Engineering | Initial product specification for Capability 1 (Production Agent Runtime). Defines functional requirements (agent deployment via Helm, health monitoring, auto-scaling via K8s HPA, lifecycle management), non-functional requirements (performance <60s deployment, 99.5% uptime), technical architecture (K8s + Helm + minimal controller), integration with ALL other capabilities (Caps 2-8), build roadmap (8-12 weeks), success metrics. Strategic recommendation: MINIMAL K8S WRAPPER (don't rebuild Kubernetes, don't rebuild agent frameworks). Leverage K8s primitives, integrate Iron Cage governance. Ready for engineering review. |
