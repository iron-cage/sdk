# Feature Documentation

**Purpose:** Detailed documentation for specific features implemented in iron_runtime.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 001 | **001_cli_architecture.md** | Document CLI tools architecture (wrapper pattern, binary discovery, tool responsibilities) |
| 002 | **002_token_management.md** | Explain token management feature overview (concepts, user workflows, integration points) |
| 003 | **003_token_management_api_reference.md** | Document token management API endpoints (request/response schemas, error codes, authentication) |
| 004 | **004_token_management_cli_api_parity.md** | Define CLI and API parity requirements (command mapping, coverage gaps, implementation status) |
| 005 | **005_token_management_implementation_plan.md** | Provide token management implementation roadmap (TDD phases, milestones, technical approach) |

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
3. Update this readme.md with new entries in the Responsibility Table (3 columns: ID, Entity, Responsibility)
4. Add cross-references in file's "Related" section at bottom

---

**Last Updated:** 2025-12-08
