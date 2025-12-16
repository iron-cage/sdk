# Safe Execution

**Concept:** Isolated sandbox environments preventing AI-generated code from escaping, consuming resources, or accessing secrets.

---

## ⚠️ CRITICAL: Sandbox Architecture Context

**This document describes the container-based sandbox architecture for Server Execution Mode (future, 5% of deployments).**

**In Local Execution Mode (default, 95% of deployments):**
- Agents run as local processes on developer machine
- No container isolation (agents run in developer's environment)
- Safety guaranteed by local Safety Service checking all LLM calls before sending
- Code execution happens in developer's local environment with standard OS-level process isolation

**This sandbox architecture applies ONLY to Server Execution Mode** where agents run in cloud (Kubernetes) and need strong isolation from each other and the host system.

---

## User Need

AI agents that execute code (Python, shell, SQL) pose risks:
- Code runs in production environment with full access
- No resource limits (runaway CPU/memory)
- Can read secrets, access databases, make network calls
- No audit trail of what was executed

## Core Idea (Server Execution Mode Only)

Execute untrusted code in **isolated containers** with:
1. Filesystem isolation (can't access host)
2. Network restrictions (whitelist allowed domains)
3. Resource limits (CPU, memory, time)
4. Syscall filtering (block dangerous operations)

The insight: Sandboxing is a **solved problem** (containers, seccomp, cgroups). The value is **integrating** sandboxing with AI-specific governance.

## Key Components

- **Container Runtime** - Docker/Kubernetes-based isolation
- **Resource Limits** - cgroups for CPU, memory, disk, processes
- **Network Policy** - Domain whitelisting, egress filtering
- **Syscall Filter** - seccomp profiles blocking dangerous calls

## Related Capabilities

- [AI Safety Guardrails](004_ai_safety_guardrails.md) - Validates before execution
- [Observability](007_observability.md) - Logs all sandbox activity
