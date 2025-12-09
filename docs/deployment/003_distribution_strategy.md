# Distribution Strategy

**Purpose:** How packages reach users through different channels.

---

## User Need

Know where to get each package and how updates are delivered.

## Core Idea

**Multiple channels optimized for each package type:**

| Package | Channel | URL |
|---------|---------|-----|
| Control Panel | Docker Hub | `ironcage/control-panel` |
| Marketing Site | CDN | `ironcage.ai` |
| Agent Runtime | PyPI | `pip install iron-cage` |
| Sandbox | PyPI | `pip install iron-sandbox` |
| CLI Tools | GitHub + PyPI | Binary + `pip install iron-cli` |

## Channel Characteristics

| Channel | Auto-Update | Versioning | Rollback |
|---------|-------------|------------|----------|
| Docker Hub | Pull latest | Tags | Yes |
| CDN | Automatic | Immutable | CDN cache |
| PyPI | `pip install --upgrade` | Semantic | Pin version |
| GitHub + PyPI | Manual + pip | Releases + Semantic | Download older / Pin |

## Version Compatibility

```
Agent Runtime 1.x --- compatible ---> Control Panel 1.x
                 +-- compatible ---> CLI Tools 1.x
```

- Major versions must match across packages
- Minor versions are forward-compatible
- Patch versions are always compatible

---

*Related: [001_package_model.md](001_package_model.md) | [004_scaling_patterns.md](004_scaling_patterns.md)*
