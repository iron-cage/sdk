# Agent Runtime

**Concept:** Lifecycle management layer for deploying, scaling, and recovering AI agent workloads.

---

## ⚠️ CRITICAL: Default Execution Location

**By default (95% of deployments), agents run LOCALLY on the developer machine, NOT in the cloud.**

- **Local Execution (Default, 95%):** Agents run as local processes on developer machine. No Kubernetes, no containers, no cloud deployment. The Runtime intercepts LLM calls via iron_sdk and runs all safety/cost/audit checks locally before sending prompts to LLM providers.
- **Server Execution (Future, 5%):** Agents run in cloud (Kubernetes pods) for hosted execution. This document describes the server execution architecture.

**This document describes the Kubernetes-based orchestration architecture for server execution mode (future capability). The default local execution mode does not use Kubernetes - agents simply run as local processes.**

**Key Privacy Guarantee:** In local mode (default), no data leaves the developer machine - all checks happen before sending prompts to LLM providers.

---

## User Need

Running AI agents in production requires:
- Starting, stopping, pausing agents reliably
- Scaling to handle varying load
- Recovering from failures automatically
- Updating agents without downtime

## Core Idea (Server Execution Mode Only)

Provide a **thin orchestration layer** on top of Kubernetes that:
1. Manages agent lifecycle (deploy, scale, terminate)
2. Monitors health and restarts failed agents
3. Supports rolling updates and rollbacks
4. Integrates with Python agent frameworks (LangChain, CrewAI)

The insight: Runtime is **infrastructure** - leverage Kubernetes (don't rebuild it). The value is **integration** with governance capabilities.

## Key Components

- **Lifecycle Manager** - Start/stop/pause/resume agents
- **Health Monitor** - Detect failures, trigger restarts
- **Scaling Controller** - Horizontal scaling based on load
- **Framework Bridge** - PyO3 integration with Python agent code

## Related Capabilities

- [Observability](007_observability.md) - Monitors runtime health
- [Safe Execution](003_safe_execution.md) - Agents run in sandboxes
- [LLM Access Control](002_llm_access_control.md) - Governs agent LLM usage
