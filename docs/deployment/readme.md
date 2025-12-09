# Deployment

**Purpose:** Conceptual overview of how Iron Cage is packaged and distributed.

---

## Directory Responsibilities

| ID | Entity | Responsibility | Input → Output | Scope | Out of Scope |
|----|--------|----------------|----------------|-------|--------------|
| 001 | **001_package_model.md** | Define the 5 deployment packages | Packaging question → Package definitions | Five packages (Control Panel, Marketing Site, Agent Runtime, Sandbox, CLI Tools), package characteristics, grouping rationale | NOT module mappings (→ 005), NOT distribution channels (→ 003), NOT actors (→ 002) |
| 002 | **002_actor_model.md** | Document system actors and interactions | Actor question → Actor taxonomy | Human actors (developer, operations, security, admin, visitor), software actors (agent, browser, terminal, LLM API, database), interaction patterns | NOT packages (→ 001), NOT distribution (→ 003), NOT scaling (→ 004) |
| 003 | **003_distribution_strategy.md** | Explain package distribution channels | Distribution question → Channel strategy | Docker Hub, CDN, PyPI, GitHub Releases, versioning, compatibility matrix, update mechanisms | NOT package contents (→ 001), NOT module mappings (→ 005), NOT scaling (→ 004) |
| 004 | **004_scaling_patterns.md** | Describe horizontal scaling approach | Scaling question → Scaling architecture | Stateless services, shared state (PostgreSQL, Redis), load balancing, K8s HPA, replica management | NOT packages (→ 001), NOT distribution (→ 003), NOT actors (→ 002) |
| 005 | **005_module_distribution.md** | Map modules to deployment packages | Module location question → Package assignment | Which modules in which packages, runtime dependencies, module grouping rationale | NOT package definitions (→ 001), NOT distribution channels (→ 003), NOT scaling (→ 004) |

---

## Deployment Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [Package Model](001_package_model.md) | 5 deployment packages |
| 002 | [Actor Model](002_actor_model.md) | Who/what interacts with system |
| 003 | [Distribution Strategy](003_distribution_strategy.md) | How packages reach users |
| 004 | [Scaling Patterns](004_scaling_patterns.md) | Horizontal scaling approach |
| 005 | [Module Distribution](005_module_distribution.md) | Which modules in which package |

## Package Overview

```
+-------------+  +-------------+  +-------------+
|  Package 1  |  |  Package 2  |  |  Package 3  |
|  Control    |  |  Marketing  |  |   Agent     |
|  Panel      |  |  Site       |  |   Runtime   |
|  (Docker)   |  |  (Static)   |  |   (PyPI)    |
+-------------+  +-------------+  +-------------+

+-------------+  +-------------+
|  Package 4  |  |  Package 5  |
|  Sandbox    |  |   CLI       |
|  (PyPI)     |  |  Tools      |
|             |  | (Bin + PyPI)|
+-------------+  +-------------+
```

*For operational procedures, see [deployment_guide.md](../deployment_guide.md)*
*For module mappings, see [module_package_matrix.md](../module_package_matrix.md)*
