# Feature Documentation

**Purpose:** Detailed documentation for specific features implemented in iron_runtime.

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| 001 | **001_cli_architecture.md** | CLI tools architecture overview | Architecture question → Tool selection guidance | CLI responsibilities, wrapper pattern, binary discovery | iron_cli implementation (→ module/iron_cli/spec.md), iron_cli_py implementation (→ module/iron_cli_py/spec.md), Architecture decision rationale (→ pilot/decisions/002-cli-architecture.md) |
| 002 | **002_token_management.md** | Token management feature overview | User need → Feature description | Feature concepts, user workflows, integration points | API details (→ token_management_api_reference.md), CLI specifics (→ token_management_cli_api_parity.md), Implementation (→ token_management_implementation_plan.md) |
| 003 | **003_token_management_api_reference.md** | Token API endpoint reference | API call → Response schema | REST endpoints, request/response formats, error codes | Feature concepts (→ token_management.md), CLI commands (→ token_management_cli_api_parity.md), Implementation plan (→ token_management_implementation_plan.md) |
| 004 | **004_token_management_cli_api_parity.md** | CLI/API parity matrix | CLI command → API equivalent | Command mapping, coverage gaps, implementation status | API details (→ token_management_api_reference.md), Feature overview (→ token_management.md), Implementation timeline (→ token_management_implementation_plan.md) |
| 005 | **005_token_management_implementation_plan.md** | Implementation roadmap | Requirement → Implementation steps | Phases, milestones, technical approach | API reference (→ token_management_api_reference.md), Feature overview (→ token_management.md), CLI parity (→ token_management_cli_api_parity.md) |

---

## File Relationships

Token Management documentation follows a layered structure:

1. **Overview** (`token_management.md`) - Start here for concepts
2. **API Reference** (`token_management_api_reference.md`) - For developers using the API
3. **CLI Parity** (`token_management_cli_api_parity.md`) - For CLI users
4. **Implementation** (`token_management_implementation_plan.md`) - For contributors

```
                    ┌────────────────────────┐
                    │  token_management.md   │
                    │     (Overview)         │
                    └───────────┬────────────┘
                                │
            ┌───────────────────┼───────────────────┐
            │                   │                   │
            ▼                   ▼                   ▼
┌───────────────────┐ ┌────────────────────┐ ┌─────────────────────┐
│ api_reference.md  │ │ cli_api_parity.md  │ │ implementation_     │
│   (REST API)      │ │  (CLI Commands)    │ │    plan.md          │
└───────────────────┘ └────────────────────┘ └─────────────────────┘
```

---

## Adding New Feature Documentation

When adding documentation for a new feature:

1. Create `feature_name.md` as the primary overview document
2. Add supporting documents as needed (api_reference, implementation_plan, etc.)
3. Update this readme.md with new entries in the Responsibility Table
4. Ensure Out of Scope column has 3+ cross-references

---

**Last Updated:** 2025-12-08
