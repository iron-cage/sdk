# Deployment

**Purpose:** Conceptual overview of how Iron Cage is packaged and distributed.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 001 | **001_package_model.md** | Define the 5 deployment packages (Control Panel, Marketing Site, Agent Runtime, Sandbox, CLI Tools) and their characteristics |
| 002 | **002_actor_model.md** | Document system actors and interactions (human actors, software actors, interaction patterns) |
| 003 | **003_distribution_strategy.md** | Explain package distribution channels (Docker Hub, CDN, PyPI, GitHub Releases, versioning, compatibility) |
| 004 | **004_scaling_patterns.md** | Describe horizontal scaling approach (stateless services, shared state, load balancing, K8s HPA) |
| 005 | **005_module_distribution.md** | Map modules to deployment packages (which modules in which packages, runtime dependencies) |
| 006 | **006_docker_compose_deployment.md** | Docker Compose architecture for pilot Control Panel deployment (2 services: Backend API with SQLite, Frontend nginx) |

---

## Deployment Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [Package Model](001_package_model.md) | 5 deployment packages |
| 002 | [Actor Model](002_actor_model.md) | Who/what interacts with system |
| 003 | [Distribution Strategy](003_distribution_strategy.md) | How packages reach users |
| 004 | [Scaling Patterns](004_scaling_patterns.md) | Horizontal scaling approach |
| 005 | [Module Distribution](005_module_distribution.md) | Which modules in which package |
| 006 | [Docker Compose Deployment](006_docker_compose_deployment.md) | Pilot deployment architecture |

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
<!-- TODO: Add module package matrix documentation -->
