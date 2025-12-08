# Capability 4: Safe Execution Environment - Product Specification

**Version:** 1.0.0
**Last Updated:** 2025-01-20
**Status:** Draft - Platform Component Specification
**Build Priority:** Platform Component (55/100 standalone viability - build as component via E2B partnership)

---

### Scope

**Responsibility:** Product specification for Safe Execution Environment capability (Capability 4 of 8 - platform component, 55/100, E2B partnership)

**In Scope:**
- Market context (AI sandbox market, E2B market leader, part of $27B AI security market)
- Strategic approach (PARTNER WITH E2B, do NOT build sandboxes in-house, wrap E2B with policy layer)
- Problem statement (unsafe code execution in production, security risks, no resource limits, no isolation)
- Solution architecture (E2B integration with filesystem isolation, network restrictions, timeout enforcement, audit trails)
- Build recommendation (integrate E2B SDK + add Iron Cage policy layer, 2-3 months, 1-2 engineers)
- Platform integration (included in $100K-300K/year, not sold separately)
- Feature specifications (sandbox isolation, resource limits, security controls, compliance audit)
- Competitive positioning (E2B partnership vs building in-house)
- Standalone viability score (55/100 - build as component via partnership)

**Out of Scope:**
- Other 7 capabilities (see other capability specs)
- Strategic analysis (see `/business/strategy/executive_summary.md`)
- Warsaw pilot (see `../pilot/spec.md`)
- Implementation details (see `/docs/architecture.md`)

---

## Executive Summary

This specification defines the requirements for Iron Cage's Safe Execution Environment capability - sandboxed code execution for AI agents with resource limits, network isolation, and security controls.

**Market Opportunity:** AI sandbox market (E2B dominates, no clear TAM - part of broader $27B AI security market)
**Strategic Approach:** Platform component via E2B partnership (NOT build from scratch)
**Build Timeline:** 2-3 months, integrate E2B SDK + add Iron Cage policy layer
**Platform Pricing:** Included in $100K-300K/year Iron Cage platform (not sold separately)

**Core Value Proposition:** Replace unsafe code execution (agents running arbitrary code in production environment, security risks, no resource limits) with isolated sandboxes providing filesystem isolation, network restrictions, timeout enforcement, and audit trails.

**Strategic Recommendation:** PARTNER WITH E2B (market leader, proven tech). Do NOT build sandboxes in-house (reinventing wheel, high security risk). Wrap E2B with Iron Cage policy layer (authorization, resource limits, audit logs).

---

## 1. Product Overview

### 1.1 Problem Statement

AI agents executing code without isolation:

```
CURRENT STATE: Unsafe Code Execution
┌─────────────────────────────────────────────────────┐
│  AGENT: "Execute this Python code"                  │
│         subprocess.run(["python", "script.py"])     │
│                                                      │
│  RISKS:                                             │
│  ❌ Runs in production environment (no isolation)   │
│  ❌ Can access production database                  │
│  ❌ Can read secrets/credentials                    │
│  ❌ Can make external API calls                     │
│  ❌ No resource limits (runaway CPU/memory)         │
│  ❌ No timeout (infinite loops)                     │
│  ❌ No audit trail                                  │
└─────────────────────────────────────────────────────┘
```

### 1.2 Solution: Iron Cage Safe Execution (E2B Integration)

```
IRON CAGE SOLUTION: Sandboxed Execution
┌─────────────────────────────────────────────────────┐
│  AGENT: "Execute this Python code"                  │
│         ↓                                            │
│  IRON CAGE TOOL PROXY (Cap 4)                       │
│         ↓ (authorization check)                     │
│  E2B SANDBOX                                        │
│  ┌─────────────────────────────────────────────┐   │
│  │  ISOLATED ENVIRONMENT                        │   │
│  │  ✅ Filesystem: tmpfs (ephemeral, isolated)  │   │
│  │  ✅ Network: Whitelisted domains only        │   │
│  │  ✅ Resources: 1 CPU, 2GB RAM (configurable) │   │
│  │  ✅ Timeout: 30s (configurable)              │   │
│  │  ✅ Read-only system files                   │   │
│  │  ✅ No access to production data             │   │
│  └─────────────────────────────────────────────┘   │
│         ↓                                            │
│  AUDIT LOG (execution time, output, errors)         │
└─────────────────────────────────────────────────────┘
```

### 1.3 Strategic Positioning

**NOT:** "Code Execution Sandbox" (E2B dominates, DockerSandbox and Fly.io Machines are alternatives)

**YES:** "Integrated Safe Execution for AI Agents" (Part of unified governance platform, not standalone)

**Partnership Strategy:**
- Integrate E2B SDK (market leader, proven sandboxes)
- Add Iron Cage policy layer (authorization via Cap 4, audit logs via Cap 7)
- Position as platform component ($100K-300K/year platform), not point tool

---

## 2. Functional Requirements

### 2.1 Sandbox Runtime Support

**Requirement:** Support multiple programming languages/runtimes.

**Priority 1 (Launch):**
- Python 3.11+ (most common for AI/ML)
- Node.js 20+ (JavaScript/TypeScript)
- Shell (bash, basic commands)

**Priority 2 (Month 6):**
- Go 1.21+
- Rust (via rustc + cargo)
- R (data science)

**Implementation:**
```rust
// src/sandbox/executor.rs

pub struct SandboxExecutor
{
  e2b_client: Arc< E2bClient >,
  policy_engine: Arc< PolicyEngine >, // From Cap 4
}

impl SandboxExecutor
{
  pub async fn execute
  (
    &self,
    agent_id: &str,
    code: ExecutionRequest,
  ) -> Result< ExecutionResult >
  {
    // 1. Authorization check (via Cap 4 Tool Proxy)
    let auth_result = self.policy_engine
      .authorize_code_execution( agent_id, &code )
      .await?;

    if !auth_result.allowed
    {
      return Ok( ExecutionResult::Denied
      {
        reason: auth_result.reason,
      });
    }

    // 2. Create E2B sandbox
    let sandbox_config = SandboxConfig
    {
      language: code.language,
      timeout: code.timeout.unwrap_or( Duration::from_secs( 30 ) ),
      memory_limit_mb: code.memory_limit_mb.unwrap_or( 2048 ),
      cpu_limit: code.cpu_limit.unwrap_or( 1.0 ),
      network_policy: code.network_policy.unwrap_or( NetworkPolicy::Restricted ),
    };

    let sandbox = self.e2b_client
      .create_sandbox( sandbox_config )
      .await?;

    // 3. Execute code
    let start_time = Instant::now();

    let result = sandbox
      .execute( &code.code )
      .await?;

    let duration = start_time.elapsed();

    // 4. Clean up sandbox
    sandbox.terminate().await?;

    // 5. Audit log (async, non-blocking)
    tokio::spawn( async move
    {
      // Log execution to audit trail (Cap 7)
    });

    Ok( ExecutionResult::Success
    {
      stdout: result.stdout,
      stderr: result.stderr,
      exit_code: result.exit_code,
      duration_ms: duration.as_millis() as u64,
    })
  }
}

pub struct ExecutionRequest
{
  pub language: Language,
  pub code: String,
  pub timeout: Option< Duration >,
  pub memory_limit_mb: Option< usize >,
  pub cpu_limit: Option< f64 >, // CPU cores (0.5, 1.0, 2.0)
  pub network_policy: Option< NetworkPolicy >,
}

pub enum Language
{
  Python,
  NodeJs,
  Bash,
  Go,
  Rust,
  R,
}

pub enum NetworkPolicy
{
  Isolated, // No network access
  Restricted, // Whitelisted domains only
  Unrestricted, // Full network access (requires explicit approval)
}

pub enum ExecutionResult
{
  Success
  {
    stdout: String,
    stderr: String,
    exit_code: i32,
    duration_ms: u64,
  },
  Denied
  {
    reason: String,
  },
  Timeout,
  ResourceExceeded,
}
```

### 2.2 Resource Limits

**Requirement:** Enforce CPU, memory, timeout limits to prevent runaway execution.

**Default Limits:**
- CPU: 1 core
- Memory: 2GB RAM
- Timeout: 30 seconds
- Disk: 1GB tmpfs (ephemeral)

**Configurable Limits (per agent policy):**
- CPU: 0.5-4 cores
- Memory: 512MB-8GB
- Timeout: 10s-300s (5 minutes max)
- Disk: 1GB-10GB

**Enforcement:**
- E2B provides resource limits (CPU, memory, timeout)
- Iron Cage policy engine determines limits per agent
- Exceeded limits → sandbox terminates, returns `ResourceExceeded`

### 2.3 Filesystem Isolation

**Requirement:** Isolated, ephemeral filesystem with no access to production data.

**Filesystem Structure:**
```
/sandbox
├── /tmp         (writable, tmpfs, 1GB, ephemeral)
├── /home/user   (writable, tmpfs, 1GB, ephemeral)
├── /usr         (read-only, system binaries)
├── /lib         (read-only, system libraries)
├── /etc         (read-only, system config)
└── [NO ACCESS]  /production_data, /secrets, /credentials
```

**Key Properties:**
- **Ephemeral:** All data deleted when sandbox terminates
- **Read-only system:** Cannot modify system files
- **No production access:** Cannot read production database, secrets, credentials

### 2.4 Network Restrictions

**Requirement:** Control network access to prevent data exfiltration and unauthorized API calls.

**Network Policies:**

**1. Isolated (Default):**
- No network access whatsoever
- Use for: Untrusted code, data processing

**2. Restricted (Whitelist):**
- Only whitelisted domains allowed
- Example whitelist: `api.openai.com`, `api.anthropic.com`, `api.company.com`
- Use for: AI agents calling approved external APIs

**3. Unrestricted:**
- Full network access (requires explicit approval via Cap 4 human-in-loop)
- Use for: Trusted internal tools, research environments

**Implementation:**
```rust
// src/sandbox/network_policy.rs

pub struct NetworkPolicy
{
  pub mode: NetworkMode,
  pub whitelist: Vec< String >, // Domain whitelist (for Restricted mode)
}

pub enum NetworkMode
{
  Isolated,
  Restricted,
  Unrestricted,
}
```

### 2.5 Audit Logging

**Requirement:** Complete audit trail for all code execution (compliance: SOC2, HIPAA).

**Audit Log Fields:**
- Timestamp
- Agent ID
- User ID
- Tenant ID
- Language (Python, NodeJs, etc.)
- Code (sanitized, PII/secrets redacted)
- Execution result (success, denied, timeout, resource_exceeded)
- stdout (first 10KB)
- stderr (first 10KB)
- Exit code
- Duration (ms)
- Resource usage (CPU, memory peak)

**Retention:**
- PostgreSQL: 90 days (hot storage)
- S3: 7 years (cold storage, compliance)

---

## 3. Non-Functional Requirements

### 3.1 Performance

**Latency:**
- Sandbox creation: p50 < 500ms, p99 < 2s
- Code execution: p50 < 1s, p99 < 5s (excluding user code time)
- Sandbox termination: p50 < 100ms, p99 < 500ms

**Throughput:**
- 1000 concurrent sandboxes per deployment
- 10K executions/minute

### 3.2 Reliability

**Availability:**
- 99.9% uptime SLA
- E2B SLA: 99.9% (per E2B docs)

**Error Handling:**
- Timeout enforcement (hard 30s default, configurable)
- Resource limit enforcement (memory, CPU)
- Automatic sandbox cleanup (on success, error, timeout)

### 3.3 Security

**Isolation Guarantees:**
- Filesystem isolation (tmpfs, no production access)
- Network isolation (default: no network)
- Process isolation (no access to other sandboxes, host processes)
- User isolation (no root access, read-only system)

**Secrets Protection:**
- Code is sanitized before logging (PII/secrets redacted via Cap 4)
- No secrets passed to sandbox (use temporary tokens with short TTL)

---

## 4. Technical Architecture

### 4.1 Technology Stack

**Base Platform:** E2B SDK (Python/TypeScript, proven sandboxes)

**Iron Cage Wrapper:**
- Rust (sandbox executor, authorization integration)
- PostgreSQL (audit logs, execution history)
- Integration with Cap 4 (Tool Proxy for authorization)
- Integration with Cap 7 (Observability for metrics/logs)

**E2B Infrastructure:**
- Managed by E2B (no ops burden for Iron Cage)
- Multi-region support (US, EU)
- Auto-scaling (handles load spikes)

### 4.2 Deployment Architecture

```
┌─────────────────────────────────────────────────────┐
│         SAFE EXECUTION (Iron Cage)                  │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │       SANDBOX EXECUTOR (Rust)               │   │
│  │  - Authorization integration (Cap 4)         │   │
│  │  - Policy enforcement (resource limits)      │   │
│  │  - Audit logging (Cap 7)                     │   │
│  └─────────────────┬───────────────────────────┘   │
│                    │                                 │
│                    │ (E2B SDK)                       │
│                    ▼                                 │
│  ┌─────────────────────────────────────────────┐   │
│  │       E2B SANDBOXES (Managed)               │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │   │
│  │  │ Python   │  │ Node.js  │  │  Bash    │  │   │
│  │  │ Sandbox  │  │ Sandbox  │  │ Sandbox  │  │   │
│  │  └──────────┘  └──────────┘  └──────────┘  │   │
│  │  - Isolated filesystem (tmpfs)              │   │
│  │  - Network restrictions                     │   │
│  │  - Resource limits (CPU, memory, timeout)   │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 5. Integration with Other Capabilities

### 5.1 Capability 4 (Safety Guardrails) - Tool Proxy

**Integration:** All code execution requests go through Tool Proxy for authorization.

**Flow:**
1. Agent requests code execution
2. Tool Proxy (Cap 4) intercepts request
3. Check authorization (whitelist/blacklist, parameter validation)
4. If approved → Forward to Sandbox Executor (Cap 3)
5. Sandbox Executor calls E2B, returns result
6. Tool Proxy logs execution to audit trail

### 5.2 Capability 7 (Observability)

**Integration:** Sandbox Executor reports metrics to observability service.

**Metrics:**
- Execution rate (executions/second by language, agent)
- Duration (p50, p95, p99)
- Success rate (% successful executions)
- Timeout rate (% executions timing out)
- Resource usage (CPU, memory peak)

---

## 6. Build Roadmap

### Phase 1: E2B Integration (Months 12-13)

- ✅ Integrate E2B SDK (Python/TypeScript)
- ✅ Wrap with Rust sandbox executor
- ✅ Basic authorization (Cap 4 integration)
- ✅ Python + Node.js + Bash support

### Phase 2: Policy Layer (Months 14-15)

- ✅ Resource limit policies (per agent, per tenant)
- ✅ Network policies (isolated, restricted, unrestricted)
- ✅ Audit logging (PostgreSQL + S3)
- ✅ Human-in-loop for unrestricted network (Cap 4)

### Phase 3: Additional Runtimes (Month 15)

- ✅ Go support
- ✅ Rust support (optional)
- ✅ R support (optional, for data science use cases)

---

## 7. Success Metrics

### Product Metrics (Month 15)

**Adoption:**
- 100% of agents requiring code execution use sandboxes (mandatory)
- 1000+ executions/day per deployment

**Performance:**
- p99 sandbox creation < 2s
- p99 execution latency < 5s (excluding user code time)
- 99.9% uptime

**Security:**
- Zero sandbox escapes (production data access)
- Zero unauthorized network access

---

## 8. Pricing & E2B Partnership

### E2B Pricing (Pass-Through)

**E2B Pricing Model:**
- Free tier: 100 hours/month sandbox time (for testing)
- Pay-as-you-go: $0.001/second ($3.60/hour)
- Enterprise: Custom pricing (volume discounts)

**Iron Cage Approach:**
- Include E2B costs in platform pricing ($100K-300K/year)
- Estimate 10,000 hours/month usage = $36K/year E2B cost
- Bundle into platform (don't charge separately)

### Partnership Terms

**Potential Partnership with E2B:**
- Volume discount (enterprise pricing)
- White-label integration (Iron Cage branding)
- SLA guarantees (99.9% uptime)
- Priority support

---

## 9. Open Questions

1. **E2B vs Self-Hosted:** Use managed E2B (easier) vs self-host sandboxes (more control, lower cost)?

2. **Sandbox Lifecycle:** Create sandbox per execution (slower, safer) vs pool of warm sandboxes (faster, slightly less isolation)?

3. **Network Whitelist Granularity:** Domain-level (e.g., `*.openai.com`) vs URL-level (e.g., `https://api.openai.com/v1/chat/completions`)?

4. **Code Size Limits:** Max code size to prevent abuse (1KB? 10KB? 100KB)?

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-01-20 | Platform Engineering | Initial product specification for Capability 3 (Safe Execution Environment). Defines functional requirements (Python/Node.js/Bash sandboxes, resource limits, filesystem isolation, network restrictions, audit logging), non-functional requirements (performance <2s sandbox creation, 99.9% uptime, security isolation), technical architecture (E2B SDK integration, Rust wrapper), integration with Cap 4 (Tool Proxy authorization) and Cap 7 (Observability metrics), build roadmap (2-3 months), success metrics. Strategic recommendation: PARTNER WITH E2B (market leader), wrap with Iron Cage policy layer. Ready for engineering review. |
