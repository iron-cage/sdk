# Security: Isolation Layers

## ⚠️ CRITICAL: Isolation Architecture Context

**This document describes the multi-layer isolation architecture for Server Execution Mode (future, 5% of deployments).**

**In Local Execution Mode (default, 95% of deployments):**
- Agents run as local processes on developer machine
- No container isolation (agents run in developer's environment)
- Standard OS-level process isolation applies
- Safety guaranteed by local Safety Service checking all LLM calls before sending
- Agent code executes with developer's permissions and access

**This multi-layer isolation architecture (containers, seccomp, Landlock, network policies) applies ONLY to Server Execution Mode** where agents run in cloud infrastructure (Kubernetes) and need strong isolation from each other and the host system.

---

### Scope

This document defines the four-layer isolation architecture for Iron Runtime's agent execution environment. It specifies isolation technologies, isolation modes, and escape prevention strategies through defense-in-depth design.

**In scope**:
- Four isolation layers (Network, Filesystem, Syscall, Process)
- Layer technologies (iptables, Landlock, seccomp-bpf, containers/cgroups)
- Three isolation modes (Minimal, Standard, Maximum)
- Escape prevention through layered defense
- Layer independence and failure containment
- Use case mapping to isolation modes

**Out of scope**:
- Detailed configuration of isolation technologies (see deployment guides)
- Specific threat scenarios and attack vectors (see Security 001: Threat Model)
- Credential flow and token security (see Security 003: Credential Flow)
- Audit logging implementation (see Security 004: Audit Model)
- Container runtime selection and setup (see deployment documentation)
- Performance impact benchmarks (see Constraints 004: Trade-offs)

### Purpose

**User Need:** Contain agent failures and attacks through layered isolation.

**Solution:** Four isolation layers, each independent:

```
+-----------------------------------------+
| Layer 4: Network Isolation              | <-- Blocks external access
+-----------------------------------------+
| Layer 3: Filesystem Isolation           | <-- Read-only system
+-----------------------------------------+
| Layer 2: Syscall Filtering (seccomp)    | <-- Blocks dangerous calls
+-----------------------------------------+
| Layer 1: Process Isolation (container)  | <-- Separate namespace
+-----------------------------------------+
         Agent Code Runs Here
```

**Key Insight:** Each layer is independent - breaching one layer doesn't compromise others. This is defense in depth, not perimeter security. No single vulnerability can compromise all layers.

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

### Layer Details

| Layer | Technology | Blocks |
|-------|------------|--------|
| Process | Container/cgroups | Resource exhaustion |
| Syscall | seccomp-bpf | Dangerous system calls |
| Filesystem | Landlock | Unauthorized file access |
| Network | iptables/policy | Unauthorized connections |

### Isolation Modes

| Mode | Layers | Use Case |
|------|--------|----------|
| **Minimal** | Process only | Trusted agents, max performance |
| **Standard** | Process + Syscall | Default for most agents |
| **Maximum** | All 4 layers | Untrusted code, sandboxed execution |

### Escape Prevention

- Each layer is independent (breach one, others hold)
- No single vulnerability compromises all layers
- Defense in depth, not perimeter security

#### Threat→Layer Mapping

| Threat Category | Mitigated By | How |
|----------------|--------------|-----|
| Resource exhaustion (CPU, memory) | Layer 1: Process | cgroups limits prevent DoS |
| Malicious syscalls (ptrace, reboot) | Layer 2: Syscall | seccomp blocks dangerous calls |
| Unauthorized file access | Layer 3: Filesystem | Landlock restricts file paths |
| External network attacks | Layer 4: Network | iptables blocks unauthorized connections |
| Container escape attempts | Layers 1+2+3 combined | Multiple barriers prevent breakout |

#### Trust Boundaries

```
+---------------------------------------+
| UNTRUSTED: Agent code, user prompts   |  <-- Layer 1-4 isolation applied
+---------------------------------------+
| VALIDATED: Isolated runtime           |  <-- Contained within boundaries
+---------------------------------------+
| TRUSTED: Host system, infrastructure  |  <-- Protected by all layers
+---------------------------------------+
```

**Boundary Enforcement:**
- Agents operate in UNTRUSTED zone
- All four isolation layers enforce boundary
- Host system remains TRUSTED even if agent compromised

### Cross-References

**Related Security Documents:**
- [001_threat_model.md](001_threat_model.md) - Security threats requiring isolation mitigation
- [003_credential_flow.md](003_credential_flow.md) - Credential protection within isolated environments
- [004_audit_model.md](004_audit_model.md) - Audit logging from isolated agent processes

**Used By:**
- Architecture 002: [Layer Model](../architecture/002_layer_model.md) - References isolation layers for Input/Output Safety implementation
- Capabilities 004: [AI Safety Guardrails](../capabilities/004_ai_safety_guardrails.md) - Safety features implemented via isolation

**Dependencies:**
- Security 001: [Threat Model](001_threat_model.md) - Defines threats that isolation layers mitigate
- Deployment documentation: Provides configuration guides for isolation technologies
- Constraints 004: [Trade-offs](../constraints/004_trade_offs.md) - Performance impact of isolation modes

**Implementation:**
- Source: Agent execution environment (runtime isolation implementation)
- Source: Container orchestration layer (process isolation)
- Source: Security policy enforcement (seccomp, Landlock configuration)
- Tests: Isolation layer verification tests (module paths TBD)
