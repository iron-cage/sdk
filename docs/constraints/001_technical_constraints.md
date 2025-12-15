# Technical Constraints

**Purpose:** Technology limitations and requirements constraining implementation choices.

---

## User Need

Understand what technical constraints limit design options and why.

## Core Idea

**Technology choices impose hard constraints on what's possible:**

## Language and Runtime

| Constraint | Minimum Version | Rationale |
|------------|----------------|-----------|
| **Python** | 3.8+ | Typing support, async/await maturity |
| **Rust** | 1.70+ | MSRV for error_tools, mod_interface |
| **PyO3** | 0.22 | Latest stable for Python 3.12 support |
| **Linux Kernel** | 5.15+ | Landlock LSM support for sandboxing |

## Platform Support

| Platform | Status | Reason |
|----------|--------|--------|
| **Linux** | Full support | Landlock, seccomp available |
| **macOS** | Partial | No Landlock, limited sandboxing |
| **Windows** | Not supported | No Landlock, seccomp unavailable |

**Implication:** Sandboxing features (Package 4) are Linux-only.

## Database Constraints

| Database | Mode | Limitation |
|----------|------|------------|
| **SQLite** | Pilot, Agent Runtime | Single-writer, local only |
| **PostgreSQL** | Production Control Panel | Requires external infrastructure |
| **Redis** | Production | Optional, for rate limiter state |

**Implication:** Pilot mode uses SQLite (simpler), production requires PostgreSQL setup.

## FFI Constraints

**PyO3 limitations:**
- Platform-specific wheels needed (Linux, macOS, Windows)
- Cross-compilation complexity
- Debugging across Rust-Python boundary harder
- GIL (Global Interpreter Lock) affects threading

**Mitigation:** Pre-built wheels, extensive integration tests, rich error messages.

## Networking Constraints

**Local Execution (Primary):**
- Runtime runs on developer machine
- Network access controlled by developer's firewall
- HTTPS to Control Panel may be blocked

**Server Execution (Future, Post-Pilot):**
- Requires Iron Cage infrastructure
- Network policies control agent egress

---

*Related: [002_business_constraints.md](002_business_constraints.md) | [../technology/001_why_rust.md](../technology/001_why_rust.md)*
