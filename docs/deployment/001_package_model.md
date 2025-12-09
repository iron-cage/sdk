# Package Model

**Purpose:** The 5 deployment packages and their purposes.

---

## User Need

Understand what ships together and how to deploy each component.

## Core Idea

**Group by deployment unit, not by technology.** A package can contain Rust + TypeScript if they deploy together.

## The Five Packages

| Package | Contents | Artifact | Install |
|---------|----------|----------|---------|
| **1. Control Panel** | API + Dashboard | Docker image | `docker pull` |
| **2. Marketing Site** | Static website | HTML/CSS/JS | CDN deploy |
| **3. Agent Runtime** | SDK + core services | PyPI wheel | `pip install iron-sdk` |
| **4. Sandbox** | OS isolation | PyPI wheel | `pip install` |
| **5. CLI Tools** | Token management + wrapper | Binary + PyPI | Download + pip |

## Package Characteristics

| Package | Language | Size | Updates |
|---------|----------|------|---------|
| Control Panel | Rust + Vue | ~100MB | Weekly |
| Marketing Site | Vue | ~5MB | Monthly |
| Agent Runtime | Rust + Python | ~50MB | Weekly |
| Sandbox | Rust + Python | ~20MB | Monthly |
| CLI Tools | Rust + Python | ~11MB | Monthly |

## Key Principle

- **Control Panel** = REQUIRED for all deployments (admin service, always standalone)
- **Agent Runtime** = single `pip install iron-sdk` for all protection features (uses Library mode - runtime embedded via PyO3)
- **Sandbox** = optional, security-focused teams only
- **CLI Tools** = binary + Python wrapper, installed together

**Deployment Mode:**
- **Pilot:** Library mode (runtime embedded in SDK, single process, PyO3)
- **Production:** Library mode default, Router mode optional for multi-framework support

**See:** [architecture/008: Runtime Modes](../architecture/008_runtime_modes.md) for Router vs Library comparison.

---

*Related: [005_module_distribution.md](005_module_distribution.md) | [003_distribution_strategy.md](003_distribution_strategy.md)*
