# Capability 7: Zero-Config MCP - Product Specification

**Version:** 1.0.0
**Last Updated:** 2025-01-20
**Status:** Draft - Thin Component Specification
**Build Priority:** Thin Component (30/100 standalone viability - integration layer only)

---

### Scope

**Responsibility:** Product specification for Zero-Config MCP capability (Capability 7 of 8 - thin component, 30/100, integration layer)

**In Scope:**
- Market context (Model Context Protocol ecosystem $4.5B, Anthropic-backed standard)
- Strategic approach (thin integration layer, leverage existing MCP servers, 3-5 weeks, 1 engineer)
- Problem statement (manual MCP configuration, fragmented server setup, no centralized management)
- Solution architecture (zero-config MCP discovery, automatic server provisioning, unified server management)
- Platform integration (included in $100K-300K/year)
- Standalone viability score (30/100 - build as thin component only)

**Out of Scope:**
- Other capabilities, strategic analysis, pilot specs, implementation details

---

## Executive Summary

This specification defines the requirements for Iron Cage's Zero-Config MCP capability - discovery, deployment, and management of Model Context Protocol (MCP) servers with security scanning and governance.

**Market Opportunity:** MCP ecosystem $1.2B → $4.5B (55% CAGR), dominated by GitHub MCP Registry (official, free)
**Strategic Approach:** Integration layer on top of GitHub Registry (NOT competing with discovery)
**Build Timeline:** 3-5 weeks, 1 engineer
**Platform Pricing:** Included in $100K-300K/year Iron Cage platform (not sold separately)

**Core Value Proposition:** Replace manual MCP server discovery and deployment (browse GitHub, copy YAML, deploy manually, no security scanning) with automated discovery, one-click deployment, security scanning (43% have vulnerabilities), and compliance governance.

**Strategic Recommendation:** DO NOT BUILD MCP REGISTRY. Integrate GitHub MCP Registry (official, Microsoft-backed, free) and ADD Iron Cage governance layer (security scanning, approval workflows, credential integration).

---

## 1. Product Overview

### 1.1 Problem Statement

Manual MCP server management:

```
CURRENT STATE: Manual MCP Discovery & Deployment
┌─────────────────────────────────────────────────────┐
│  1. DISCOVER                                         │
│     - Browse GitHub MCP Registry                    │
│     - Read documentation                            │
│     - No security assessment                        │
│                                                      │
│  2. CONFIGURE                                        │
│     - Copy YAML configuration                       │
│     - Add API keys manually                         │
│     - No credential management                      │
│                                                      │
│  3. DEPLOY                                           │
│     - kubectl apply -f mcp-server.yaml              │
│     - Manual troubleshooting                        │
│     - No automated deployment                       │
│                                                      │
│  PAIN POINTS:                                       │
│  ❌ Manual discovery (time-consuming)               │
│  ❌ No security scanning (43% have vulnerabilities!)│
│  ❌ Manual credential management (error-prone)      │
│  ❌ Manual deployment (kubectl, YAML complexity)    │
└─────────────────────────────────────────────────────┘
```

### 1.2 Solution: Iron Cage MCP Integration

```
IRON CAGE SOLUTION: Automated MCP Management
┌─────────────────────────────────────────────────────┐
│  IRON CAGE MCP SERVICE                              │
│  ┌─────────────────────────────────────────────┐   │
│  │   MCP CATALOG (Web UI)                       │   │
│  │   ┌─────────────────────────────────────┐   │   │
│  │   │ GitHub MCP Registry (1000+ servers)  │   │   │
│  │   │ ✅ Security: Scanned, 57% Safe       │   │   │
│  │   │ ✅ Popular: 500+ stars                │   │   │
│  │   │ ✅ Maintained: Updated last week      │   │   │
│  │   │ [Deploy] button                       │   │   │
│  │   └─────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────┘   │
│                    │                                 │
│                    │ (One-click)                     │
│                    ▼                                 │
│  ┌─────────────────────────────────────────────┐   │
│  │   DEPLOYMENT SERVICE                         │   │
│  │   1. Security scan (vulnerabilities?)        │   │
│  │   2. Fetch credentials (Cap 5 integration)   │   │
│  │   3. Deploy to K8s (Helm chart)              │   │
│  │   4. Health check (is it running?)           │   │
│  └─────────────────────────────────────────────┘   │
│                    │                                 │
│                    ▼                                 │
│  ┌─────────────────────────────────────────────┐   │
│  │   MCP SERVERS (Running in K8s)              │   │
│  │   ┌──────────┐  ┌──────────┐  ┌──────────┐ │   │
│  │   │  GitHub  │  │  Slack   │  │PostgreSQL│ │   │
│  │   │   MCP    │  │   MCP    │  │   MCP    │ │   │
│  │   └──────────┘  └──────────┘  └──────────┘ │   │
│  └─────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 2. Functional Requirements

### 2.1 MCP Discovery (GitHub Registry Integration)

**Requirement:** Pull MCP server metadata from GitHub MCP Registry (official).

**GitHub Registry API:**
- Endpoint: `https://github.com/modelcontextprotocol/servers` (public repo)
- Metadata: Server name, description, GitHub stars, last updated, maintainer
- Installation: Docker image, npm package, or source code

**Implementation:**
```rust
// src/mcp/discovery.rs

pub struct McpDiscoveryService
{
  github_client: Arc< GithubClient >,
}

impl McpDiscoveryService
{
  /// Fetch all MCP servers from GitHub Registry
  pub async fn fetch_registry( &self ) -> Result< Vec< McpServer > >
  {
    // 1. Clone GitHub MCP Registry repo (or use API)
    let registry_url = "https://api.github.com/repos/modelcontextprotocol/servers/contents";
    let response = self.github_client.get( registry_url ).await?;

    // 2. Parse server metadata
    let servers: Vec< McpServer > = response
      .into_iter()
      .map( | entry | self.parse_server_metadata( entry ) )
      .collect();

    Ok( servers )
  }

  fn parse_server_metadata( &self, entry: GithubRepoEntry ) -> McpServer
  {
    McpServer
    {
      id: entry.name.clone(),
      name: entry.name,
      description: entry.description,
      github_url: entry.html_url,
      stars: entry.stargazers_count,
      last_updated: entry.updated_at,
      maintainer: entry.owner.login,
      installation: self.detect_installation_method( &entry ),
    }
  }
}

pub struct McpServer
{
  pub id: String,
  pub name: String,
  pub description: String,
  pub github_url: String,
  pub stars: usize,
  pub last_updated: DateTime< Utc >,
  pub maintainer: String,
  pub installation: InstallationMethod,
}

pub enum InstallationMethod
{
  Docker { image: String },
  Npm { package: String },
  Source { repo_url: String },
}
```

### 2.2 Security Scanning

**Requirement:** Scan MCP servers for vulnerabilities before deployment.

**Vulnerability Categories:**
- **Command Injection:** 43% of community MCP servers have this vulnerability (Equixly assessment)
- **Dependency Vulnerabilities:** Outdated npm packages, Docker base images
- **Secrets in Code:** Hardcoded API keys, passwords
- **Malicious Code:** Obfuscated code, suspicious network calls

**Scanning Tools:**
- **Docker:** Trivy (container vulnerability scanner)
- **npm:** npm audit (dependency vulnerabilities)
- **Code:** Semgrep (static analysis for command injection, secrets)

**Implementation:**
```rust
// src/mcp/security_scanner.rs

pub struct McpSecurityScanner
{
  trivy_client: Arc< TrivyClient >,
  semgrep_client: Arc< SemgrepClient >,
}

impl McpSecurityScanner
{
  pub async fn scan_mcp_server
  (
    &self,
    server: &McpServer,
  ) -> Result< ScanResult >
  {
    let mut vulnerabilities = Vec::new();

    // 1. Docker image scan (if Docker-based)
    if let InstallationMethod::Docker { image } = &server.installation
    {
      let trivy_results = self.trivy_client.scan_image( image ).await?;
      vulnerabilities.extend( trivy_results );
    }

    // 2. Dependency scan (if npm package)
    if let InstallationMethod::Npm { package } = &server.installation
    {
      let npm_audit_results = self.npm_audit( package ).await?;
      vulnerabilities.extend( npm_audit_results );
    }

    // 3. Code scan (static analysis)
    let semgrep_results = self.semgrep_client
      .scan_repo( &server.github_url )
      .await?;
    vulnerabilities.extend( semgrep_results );

    // 4. Compute risk score
    let risk_score = self.compute_risk_score( &vulnerabilities );

    Ok( ScanResult
    {
      server_id: server.id.clone(),
      vulnerabilities,
      risk_score,
      scanned_at: Utc::now(),
    })
  }

  fn compute_risk_score( &self, vulns: &[ Vulnerability ] ) -> RiskScore
  {
    let critical_count = vulns.iter().filter( | v | v.severity == Severity::Critical ).count();
    let high_count = vulns.iter().filter( | v | v.severity == Severity::High ).count();

    if critical_count > 0
    {
      RiskScore::Critical
    }
    else if high_count > 2
    {
      RiskScore::High
    }
    else if high_count > 0
    {
      RiskScore::Medium
    }
    else
    {
      RiskScore::Low
    }
  }
}

pub enum RiskScore
{
  Critical, // Do not deploy (requires fixes)
  High, // Requires approval
  Medium, // Warning, proceed with caution
  Low, // Safe to deploy
}
```

### 2.3 One-Click Deployment

**Requirement:** Deploy MCP server to Kubernetes with single button click.

**Deployment Flow:**
1. User clicks "Deploy" button in Iron Cage UI
2. Iron Cage checks security scan (if High/Critical risk → require approval)
3. Iron Cage fetches credentials from Cap 5 (if MCP server needs API keys)
4. Iron Cage generates Helm chart (K8s deployment)
5. Iron Cage deploys to K8s cluster
6. Iron Cage runs health check (is server responding?)
7. Iron Cage reports status to user (deployed, failed, needs approval)

**Implementation:**
```rust
// src/mcp/deployment.rs

pub struct McpDeploymentService
{
  k8s_client: Arc< KubernetesClient >,
  credential_service: Arc< CredentialService >, // Cap 5
  security_scanner: Arc< McpSecurityScanner >,
}

impl McpDeploymentService
{
  pub async fn deploy_mcp_server
  (
    &self,
    server: &McpServer,
    user_id: &str,
  ) -> Result< DeploymentResult >
  {
    // 1. Security scan (cached if already scanned)
    let scan_result = self.security_scanner
      .scan_mcp_server( server )
      .await?;

    // 2. Risk assessment
    if scan_result.risk_score == RiskScore::Critical
    {
      return Ok( DeploymentResult::Denied
      {
        reason: "Critical vulnerabilities detected. Cannot deploy.".into(),
      });
    }

    if scan_result.risk_score == RiskScore::High
    {
      // Require human approval (via Cap 4 human-in-loop)
      let approval = self.request_approval( server, user_id ).await?;
      if !approval.approved
      {
        return Ok( DeploymentResult::Denied
        {
          reason: "Deployment requires approval. High-risk vulnerabilities detected.".into(),
        });
      }
    }

    // 3. Fetch credentials (if needed)
    let credentials = self.fetch_credentials_for_mcp( server ).await?;

    // 4. Generate Helm chart
    let helm_chart = self.generate_helm_chart( server, &credentials )?;

    // 5. Deploy to K8s
    self.k8s_client
      .deploy_helm_chart( &helm_chart )
      .await?;

    // 6. Health check
    let health_status = self.check_mcp_health( server ).await?;

    if health_status.healthy
    {
      Ok( DeploymentResult::Success
      {
        mcp_url: health_status.url,
      })
    }
    else
    {
      Ok( DeploymentResult::Failed
      {
        reason: health_status.error_message,
      })
    }
  }
}

pub enum DeploymentResult
{
  Success { mcp_url: String },
  Denied { reason: String },
  Failed { reason: String },
}
```

### 2.4 Credential Integration (Cap 5)

**Requirement:** Automatically inject credentials for MCP servers requiring API keys.

**Example:** GitHub MCP server needs GitHub API token.

**Flow:**
1. MCP server metadata specifies required credentials: `{ "github_token": "required" }`
2. Iron Cage fetches `github_token` from credential service (Cap 5)
3. Iron Cage injects token as environment variable: `GITHUB_TOKEN=<token>`
4. MCP server uses token for GitHub API calls

---

## 3. Non-Functional Requirements

### 3.1 Performance

**Discovery:**
- Refresh GitHub Registry: Every 24 hours (background job)
- Discovery latency: < 2s (cached registry data)

**Deployment:**
- Deployment time: p50 < 60s, p99 < 120s (from button click to running server)

### 3.2 Reliability

**Availability:**
- 99.9% uptime (MCP service)
- MCP servers: 99.5% uptime (varies by server quality)

**Error Handling:**
- Deployment failures: Rollback K8s deployment, notify user
- Health check failures: Mark server as unhealthy, alert admin

---

## 4. Technical Architecture

### 4.1 Technology Stack

**Backend:**
- Rust (MCP discovery, deployment, security scanning)
- Kubernetes (MCP server deployment)
- Helm (deployment templating)

**Security Scanning:**
- Trivy (Docker image scanning)
- Semgrep (code scanning)
- npm audit (dependency scanning)

**Frontend:**
- React (MCP catalog UI)

### 4.2 Deployment Architecture

```
┌─────────────────────────────────────────────────────┐
│         MCP INTEGRATION (Iron Cage)                 │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │   MCP DISCOVERY SERVICE (Rust)              │   │
│  │   - Fetch GitHub Registry (daily)            │   │
│  │   - Cache server metadata (PostgreSQL)       │   │
│  └─────────────────┬───────────────────────────┘   │
│                    │                                 │
│  ┌─────────────────▼───────────────────────────┐   │
│  │   SECURITY SCANNER (Trivy, Semgrep)         │   │
│  │   - Scan Docker images                       │   │
│  │   - Scan code for vulnerabilities            │   │
│  │   - Compute risk score                       │   │
│  └─────────────────┬───────────────────────────┘   │
│                    │                                 │
│  ┌─────────────────▼───────────────────────────┐   │
│  │   DEPLOYMENT SERVICE (Rust + K8s)           │   │
│  │   - Generate Helm charts                     │   │
│  │   - Deploy to Kubernetes                     │   │
│  │   - Health check servers                     │   │
│  └─────────────────┬───────────────────────────┘   │
│                    │                                 │
│  ┌─────────────────▼───────────────────────────┐   │
│  │   MCP SERVERS (Kubernetes Pods)             │   │
│  │   - GitHub MCP, Slack MCP, PostgreSQL MCP   │   │
│  │   - Credential injection (Cap 5)             │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 5. Integration with Other Capabilities

### 5.1 Capability 5 (Credential Management)

**Integration:** MCP servers fetch credentials from credential service.

**Example:** GitHub MCP server needs GitHub token.

**Flow:**
1. MCP Deployment Service queries Cap 5: `GET /credentials/github/token`
2. Cap 5 returns token (from Vault)
3. Deployment Service injects token as env var: `GITHUB_TOKEN=<token>`
4. MCP server uses token for GitHub API calls

### 5.2 Capability 7 (Observability)

**Integration:** MCP servers report usage metrics to observability service.

**Metrics:**
- MCP server request rate (requests/second)
- MCP server latency (p50, p99)
- MCP server error rate
- Deployment success rate

---

## 6. Build Roadmap

### Phase 1: Discovery & Security (Months 16-17)

- ✅ GitHub Registry integration (fetch server metadata)
- ✅ Security scanning (Trivy, Semgrep)
- ✅ Risk scoring (Critical, High, Medium, Low)

### Phase 2: Deployment (Month 18)

- ✅ One-click deployment (Helm charts, K8s)
- ✅ Credential integration (Cap 5)
- ✅ Health checks
- ✅ MCP catalog UI (React)

---

## 7. Success Metrics

### Product Metrics (Month 18)

**Adoption:**
- 10+ MCP servers deployed per tenant
- 100+ total MCP servers deployed across platform

**Security:**
- 100% of MCP servers scanned before deployment
- Zero Critical vulnerability deployments (blocked)

**Performance:**
- Deployment time < 60s (p50)
- 99.5% MCP server uptime

---

## 8. Open Questions

1. **Security Scanning Depth:** Scan on every deployment (slower, safer) vs cache scans for 7 days (faster, slightly less safe)?

2. **MCP Server Updates:** Auto-update MCP servers when new versions released (risky) vs manual update (safer, requires user action)?

3. **Custom MCP Servers:** Allow users to upload custom MCP servers (higher risk, more flexibility) vs only GitHub Registry (safer, less flexibility)?

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-01-20 | Platform Engineering | Initial product specification for Capability 6 (Zero-Config MCP). Defines functional requirements (GitHub Registry integration, security scanning with 43% vulnerability baseline, one-click deployment, credential integration), non-functional requirements (performance <60s deployment, 99.9% uptime), technical architecture (Rust, Kubernetes, Helm, Trivy, Semgrep), integration with Cap 5 (Credentials) and Cap 7 (Observability), build roadmap (3-5 weeks, 1 engineer), success metrics. Strategic recommendation: INTEGRATE GitHub Registry (don't compete), ADD security & governance layer. Ready for engineering review. |
