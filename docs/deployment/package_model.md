# Package Model

**Purpose:** The 6 deployment packages and their purposes.

---

## User Need

Understand what ships together and how to deploy each component.

## Core Idea

**Group by deployment unit, not by technology.** A package can contain Rust + TypeScript if they deploy together.

## The Six Packages

| Package | Contents | Artifact | Install |
|---------|----------|----------|---------|
| **1. Control Panel** | API + Dashboard | Docker image | `docker pull` |
| **2. Marketing Site** | Static website | HTML/CSS/JS | CDN deploy |
| **3. Agent Runtime** | SDK + core services | PyPI wheel | `pip install` |
| **4. Sandbox** | OS isolation | PyPI wheel | `pip install` |
| **5. CLI Tool** | Token management | Binary | Download |
| **6. Python CLI** | CLI alternative | PyPI wheel | `pip install` |

## Package Characteristics

| Package | Language | Size | Updates |
|---------|----------|------|---------|
| Control Panel | Rust + Vue | ~100MB | Weekly |
| Marketing Site | Vue | ~5MB | Monthly |
| Agent Runtime | Rust + Python | ~50MB | Weekly |
| Sandbox | Rust + Python | ~20MB | Monthly |
| CLI Tool | Rust | ~10MB | Monthly |
| Python CLI | Python | ~1MB | Monthly |

## Key Principle

- **Control Panel** = always deployed together (API needs dashboard)
- **Agent Runtime** = single `pip install` for all protection features
- **Sandbox** = optional, security-focused teams only

---

*Related: [module_distribution.md](module_distribution.md) | [distribution_strategy.md](distribution_strategy.md)*
