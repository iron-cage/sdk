# Isolation Layers

**Purpose:** Defense in depth through multiple isolation boundaries.

---

## User Need

Contain agent failures and attacks through layered isolation.

## Core Idea

**Four isolation layers, each independent:**

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

## Layer Details

| Layer | Technology | Blocks |
|-------|------------|--------|
| Process | Container/cgroups | Resource exhaustion |
| Syscall | seccomp-bpf | Dangerous system calls |
| Filesystem | Landlock | Unauthorized file access |
| Network | iptables/policy | Unauthorized connections |

## Isolation Modes

| Mode | Layers | Use Case |
|------|--------|----------|
| **Minimal** | Process only | Trusted agents, max performance |
| **Standard** | Process + Syscall | Default for most agents |
| **Maximum** | All 4 layers | Untrusted code, sandboxed execution |

## Escape Prevention

- Each layer is independent (breach one, others hold)
- No single vulnerability compromises all layers
- Defense in depth, not perimeter security

---

*Related: [threat_model.md](threat_model.md) | [credential_flow.md](credential_flow.md)*
