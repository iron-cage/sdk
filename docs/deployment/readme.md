# Deployment

**Purpose:** Conceptual overview of how Iron Cage is packaged and distributed.

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **package_model.md** | Define the 6 deployment packages | Packaging question → Package definitions | Six packages (Control Panel, Marketing Site, Agent Runtime, Sandbox, CLI Tool, Python CLI), package characteristics, grouping rationale | NOT module mappings (→ module_distribution.md), NOT distribution channels (→ distribution_strategy.md), NOT actors (→ actor_model.md) |
| **actor_model.md** | Document system actors and interactions | Actor question → Actor taxonomy | Human actors (developer, operations, security, admin, visitor), software actors (agent, browser, terminal, LLM API, database), interaction patterns | NOT packages (→ package_model.md), NOT distribution (→ distribution_strategy.md), NOT scaling (→ scaling_patterns.md) |
| **distribution_strategy.md** | Explain package distribution channels | Distribution question → Channel strategy | Docker Hub, CDN, PyPI, GitHub Releases, versioning, compatibility matrix, update mechanisms | NOT package contents (→ package_model.md), NOT module mappings (→ module_distribution.md), NOT scaling (→ scaling_patterns.md) |
| **scaling_patterns.md** | Describe horizontal scaling approach | Scaling question → Scaling architecture | Stateless services, shared state (PostgreSQL, Redis), load balancing, K8s HPA, replica management | NOT packages (→ package_model.md), NOT distribution (→ distribution_strategy.md), NOT actors (→ actor_model.md) |
| **module_distribution.md** | Map modules to deployment packages | Module location question → Package assignment | Which modules in which packages, runtime dependencies, module grouping rationale | NOT package definitions (→ package_model.md), NOT distribution channels (→ distribution_strategy.md), NOT scaling (→ scaling_patterns.md) |

---

## The Five Deployment Concepts

| # | Concept | Core Idea |
|---|---------|-----------|
| 1 | [Package Model](package_model.md) | 6 deployment packages |
| 2 | [Actor Model](actor_model.md) | Who/what interacts with system |
| 3 | [Distribution Strategy](distribution_strategy.md) | How packages reach users |
| 4 | [Scaling Patterns](scaling_patterns.md) | Horizontal scaling approach |
| 5 | [Module Distribution](module_distribution.md) | Which modules in which package |

## Package Overview

```
+-------------+  +-------------+  +-------------+
|  Package 1  |  |  Package 2  |  |  Package 3  |
|  Control    |  |  Marketing  |  |   Agent     |
|  Panel      |  |  Site       |  |   Runtime   |
|  (Docker)   |  |  (Static)   |  |   (PyPI)    |
+-------------+  +-------------+  +-------------+

+-------------+  +-------------+  +-------------+
|  Package 4  |  |  Package 5  |  |  Package 6  |
|  Sandbox    |  |  CLI Tool   |  |  Python CLI |
|  (PyPI)     |  |  (Binary)   |  |  (PyPI)     |
+-------------+  +-------------+  +-------------+
```

*For operational procedures, see [deployment_guide.md](../deployment_guide.md)*
*For module mappings, see [module_package_matrix.md](../module_package_matrix.md)*
