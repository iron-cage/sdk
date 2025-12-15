# Documentation Map

**Purpose:** Guide to Iron Cage documentation hierarchy and navigation paths.

**Last Updated:** 2025-12-10

---

## Documentation Hierarchy

```
Iron Cage Documentation
│
├── Standards (Normative Layer)
│   ├── ID Format Standards          → Entity ID format (`prefix_uuid`)
│   ├── Error Format Standards       → Error response structure
│   ├── Data Format Standards        → Timestamps, currency, booleans
│   ├── API Design Standards         → Pagination, sorting, versioning
│   └── URL Standards                → URL structure patterns
│
├── Specification (Requirements Layer)
│   └── spec/requirements.md
│       ├── FR-1.1 through FR-1.7    → Core functional requirements
│       └── FR-1.8 REST API          → REST API requirements (NEW)
│           ├── FR-1.8.1  API Architecture & Standards
│           ├── FR-1.8.2  Authentication & Authorization
│           ├── FR-1.8.3  Agent Management API
│           ├── FR-1.8.4  Provider Management API
│           ├── FR-1.8.5  API Token Management
│           ├── FR-1.8.6  Analytics & Monitoring API
│           ├── FR-1.8.7  Budget Control API
│           ├── FR-1.8.8  Data Format Standards
│           ├── FR-1.8.9  Error Handling & Rate Limiting
│           ├── FR-1.8.10 API Versioning & Deprecation
│           ├── FR-1.8.11 Audit Logging
│           └── FR-1.8.12 CLI-API Parity
│
└── Protocols (API Contracts Layer)
    ├── Core Protocols (Pilot-Critical)
    │   ├── 002: REST API Protocol           → HTTP REST API overview
    │   ├── 003: WebSocket Protocol          → Real-time message format
    │   ├── 005: Budget Control Protocol     → Two-token system, budget borrowing
    │   ├── 006: Token Management API        → IC Token CRUD
    │   ├── 007: Authentication API          → User login/logout/refresh
    │   └── 008: User Management API         → Admin user account management
    │
    ├── Extended API Protocols (MUST-HAVE)
    │   ├── 010: Agents API                  → Agent CRUD operations
    │   ├── 011: Providers API               → Provider CRUD operations
    │   ├── 012: Analytics API               → Usage and spending metrics
    │   ├── 013: Budget Limits API           → Budget modification (admin)
    │   ├── 014: API Tokens API              → API token management
    │   ├── 015: Projects API                → Project access (read-only Pilot)
    │   └── 017: Budget Requests API         → Budget request/approval workflow
    │
    └── POST-PILOT Protocols
        ├── 004: MCP Integration Protocol    → Model Context Protocol (POST-PILOT)
        └── 016: Settings API                → Settings management (POST-PILOT)
```

---

## Standards Documents

All standards are located in `/docs/standards/` and serve as the normative reference for all API implementations.

### ID Format Standards
**File:** `docs/standards/id_format_standards.md`

**Purpose:** Define entity identifier format for all Iron Cage resources

**Key Specifications:**
- Format: `prefix_uuid` with underscore separator
- Entity prefixes: `agent_`, `provider_`, `token_`, `user_`, `project_`, etc.
- Validation: UUID v4 format after prefix
- Example: `agent_550e8400-e29b-41d4-a716-446655440000`

**Referenced By:** All 15 protocol documents

---

### Error Format Standards
**File:** `docs/standards/error_format_standards.md`

**Purpose:** Define consistent error response structure across all APIs

**Key Specifications:**
- Standard JSON structure with `error.code` and `error.message`
- Machine-readable error codes: `VALIDATION_ERROR`, `UNAUTHORIZED`, etc.
- HTTP status codes: 200, 201, 400, 401, 403, 404, 409, 429, 500, 503
- Field-level validation details in `error.fields` object

**Referenced By:** All 15 protocol documents

---

### Data Format Standards
**File:** `docs/standards/data_format_standards.md`

**Purpose:** Define data type conventions for all API requests/responses

**Key Specifications:**
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Currency: Decimal with exactly 2 decimal places (e.g., `100.50`)
- Booleans: JSON boolean `true`/`false` (not strings)
- Nulls: Omit optional fields when empty (not `null`)
- Arrays: Empty array `[]` when no items (not `null`)

**Referenced By:** All 15 protocol documents

---

### API Design Standards
**File:** `docs/standards/api_design_standards.md`

**Purpose:** Define REST API design patterns and conventions

**Key Specifications:**
- Pagination: Offset-based with `?page=N&per_page=M` (default 50 items/page)
- Sorting: Optional `?sort=field` (ascending) or `?sort=-field` (descending)
- Filtering: Resource-specific query parameters
- Versioning: URL-based `/api/v1/`, `/api/v2/`
- Deprecation: 6-month notice with `X-API-Deprecation` headers

**Referenced By:** 12 REST API protocol documents (excludes WebSocket, MCP, Budget Control)

---

### URL Standards
**File:** `docs/standards/url_standards.md`

**Purpose:** Define URL structure and naming conventions

**Referenced By:** Protocol implementations

---

## Protocol Documents

All protocols are located in `/docs/protocol/` and define specific API contracts.

### Core Protocols (Pilot-Critical)

#### Protocol 002: REST API Protocol
**File:** `docs/protocol/002_rest_api_protocol.md`

**Purpose:** HTTP REST API overview and common patterns

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- Resource organization (Entity, Operation, Analytics, Configuration, System)
- Authentication types (IC Token, User Token, None)
- Common patterns (pagination, filtering, sorting)
- API versioning strategy
- References to all resource-specific protocols (006-017)

---

#### Protocol 003: WebSocket Protocol
**File:** `docs/protocol/003_websocket_protocol.md`

**Purpose:** Real-time dashboard message format

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards

**Key Content:**
- Message types: STATE_UPDATE, AGENT_EVENT, COST_ALERT, HEARTBEAT
- Connection lifecycle
- Reconnection strategy (exponential backoff)

---

#### Protocol 004: MCP Integration Protocol (POST-PILOT)
**File:** `docs/protocol/004_mcp_integration_protocol.md`

**Status:** POST-PILOT (Deferred Implementation)

**Purpose:** Model Context Protocol tool integration

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards

**Key Content:**
- Tool discovery (tools/list)
- Tool invocation (tools/call)
- Iron Cage governance layer (budget tracking, safety validation)
- Error mapping (Iron Cage ↔ MCP)

---

#### Protocol 005: Budget Control Protocol
**File:** `docs/protocol/005_budget_control_protocol.md`

**Purpose:** Two-token system (IC/IP), budget borrowing, token handshake

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards

**Key Content:**
- IC Token (agent authentication) vs IP Token (provider credentials)
- Budget borrowing protocol (handshake, usage reporting, refresh)
- Token format (JWT structure)
- Budget types (restrictive vs informative)
- Security model (IP Token encryption)

---

#### Protocol 006: Token Management API
**File:** `docs/protocol/006_token_management_api.md`

**Purpose:** IC Token CRUD endpoints

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- List IC Tokens (pagination, filtering)
- Create IC Token
- Get IC Token details
- Delete IC Token
- Rotate IC Token (regenerate)

---

#### Protocol 007: Authentication API
**File:** `docs/protocol/007_authentication_api.md`

**Purpose:** User login/logout/refresh endpoints

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- Login (email/password → User Token)
- Logout (invalidate User Token)
- Refresh (extend User Token expiration)
- Validate (check User Token validity)
- User Token format (JWT, 30 days lifetime)

---

#### Protocol 008: User Management API
**File:** `docs/protocol/008_user_management_api.md`

**Purpose:** Admin user account management endpoints

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- Create user (admin-only)
- List users (filtering, search)
- Get user details
- Suspend/activate user
- Soft delete user
- Role management
- Password reset (admin-initiated)
- Audit logging

---

### Extended API Protocols (MUST-HAVE)

#### Protocol 010: Agents API
**File:** `docs/protocol/010_agents_api.md`

**Purpose:** Agent CRUD operations

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- Create agent (with budget and provider assignments)
- List agents (pagination, filtering)
- Get agent details
- Update agent (PATCH semantics)
- Delete agent (cascade to IC Token, budget requests)

---

#### Protocol 011: Providers API
**File:** `docs/protocol/011_providers_api.md`

**Purpose:** Provider CRUD operations

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- Create provider (with credentials)
- List providers
- Get provider details
- Update provider (credential rotation)
- Delete provider (validation for agent usage)

---

#### Protocol 012: Analytics API
**File:** `docs/protocol/012_analytics_api.md`

**Purpose:** Usage and spending analytics

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- 8 critical analytics use cases
- Total spend, agent spend, provider spend
- Request counts, error rates
- Budget status
- Real-time + daily aggregations (today, yesterday, last-7-days, last-30-days, all-time)

---

#### Protocol 013: Budget Limits API
**File:** `docs/protocol/013_budget_limits_api.md`

**Purpose:** Direct budget modification (admin-only)

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- Modify agent budget (PATCH)
- Force flag for decreases (prevent accidental shutdowns)
- Emergency budget increases
- Audit logging

---

#### Protocol 014: API Tokens API
**File:** `docs/protocol/014_api_tokens_api.md`

**Purpose:** API token management (dashboard, automation)

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- Create API token (SAME-AS-USER scope)
- List API tokens
- Get API token details
- Revoke API token
- Token format: `at_<random_base64_32chars>`
- Security: Token value shown ONLY on creation (GitHub pattern)

---

#### Protocol 015: Projects API
**File:** `docs/protocol/015_projects_api.md`

**Purpose:** Project access (read-only in Pilot)

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- List projects (Pilot: returns single "Master Project")
- Get project details
- Future: Full CRUD for multi-project support (POST-PILOT)

---

#### Protocol 016: Settings API
**File:** `docs/protocol/016_settings_api.md`

**Status:** POST-PILOT

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- Settings hierarchy (System, Project, User)
- Settings inheritance (cascade from system → project → user)
- Settings categories (Operational, Notification, Display, Security)

---

#### Protocol 017: Budget Requests API
**File:** `docs/protocol/017_budget_requests_api.md`

**Purpose:** Budget request/approval workflow

**Standards Applied:**
- ✅ ID Format Standards
- ✅ Data Format Standards
- ✅ Error Format Standards
- ✅ API Design Standards

**Key Content:**
- Create budget request (developer)
- Approve/reject request (admin)
- List budget requests (filtering by status, agent, requester)
- Get request details
- Cancel request
- State machine: pending → approved/rejected/cancelled
- Integration with budget modification (automatic updates on approval)

---

## Navigation Paths

### By Use Case

**"I need to implement agent CRUD operations"**
1. Start: `spec/requirements.md` § FR-1.8.3 (Agent Management API)
2. Design: `docs/protocol/010_agents_api.md`
3. Standards:
   - `docs/standards/id_format_standards.md` (entity IDs)
   - `docs/standards/data_format_standards.md` (timestamps, currency)
   - `docs/standards/error_format_standards.md` (error responses)
   - `docs/standards/api_design_standards.md` (pagination, filtering)

**"I need to understand error handling"**
1. Start: `docs/standards/error_format_standards.md`
2. Apply: See "Error Format Standards" section in any protocol
3. Specification: `spec/requirements.md` § FR-1.8.9 (Error Handling & Rate Limiting)

**"I need to add a new API endpoint"**
1. Standards: Read all 4 core standards (ID, Data, Error, API Design)
2. Pattern: Review `docs/protocol/002_rest_api_protocol.md` (REST API overview)
3. Reference: Find similar endpoint in protocols 006-017
4. Specification: Add to `spec/requirements.md` § FR-1.8

**"I need to understand budget control"**
1. Specification: `spec/requirements.md` § FR-1.8.7 (Budget Control API)
2. Protocol: `docs/protocol/005_budget_control_protocol.md` (two-token system)
3. Admin API: `docs/protocol/013_budget_limits_api.md` (direct modification)
4. Developer API: `docs/protocol/017_budget_requests_api.md` (request/approval)

---

## Validation Rules

All documentation follows strict hierarchical validation:

**Standards Layer (Normative)**
- Defines the "HOW" (format, structure, conventions)
- Zero implementation-specific details
- Must be stable and version-controlled

**Specification Layer (Requirements)**
- Defines the "WHAT" (functionality, constraints)
- References standards (NEVER duplicates)
- Includes acceptance criteria and pending decisions

**Protocol Layer (Contracts)**
- Defines the "WHAT EXACTLY" (endpoints, schemas, messages)
- References standards (NEVER duplicates)
- Includes Standards Compliance section
- Provides examples using standards-compliant formats

**Duplication Rule:** Content defined in higher layers MUST NOT be duplicated in lower layers. Lower layers MUST reference higher layers.

**Consistency Rule:** All protocols must use identical terminology, formats, and examples when referencing the same standard.

---

## Pending Decisions

See `/dev/-default_topic/-pending_decisions_for_user.md` for 16 open REST API design decisions (Q20-Q36) awaiting user confirmation.

**Categories:**
- Agent API (Q20-Q24): Required fields, mutable fields, partial updates, name uniqueness, deletion
- Provider API (Q25-Q28): Credential validation, key rotation, deletion warning
- API Token Management (Q29-Q31): Token format, permissions, revocation
- Rate Limiting (Q32-Q33): Scope, values
- Audit Logging (Q35): Logging scope
- CLI-API Parity (Q36): Parity enforcement

**Impact:** All pending decisions are documented in `spec/requirements.md` with recommended answers marked as "(Pending: QXX)". Work can proceed using recommendations while awaiting user confirmation.

---

## Recent Changes

**2025-12-10:**
- Created 5 standards documents (ID Format, Error Format, Data Format, API Design, URL Standards)
- Extended `spec/requirements.md` with FR-1.8 section (12 subsections covering complete REST API requirements)
- Updated 15 protocol documents with Standards Compliance sections
- Created documentation map (this file)

---

## Related Documentation

**Architecture:** See `/docs/architecture/` for system architecture, data flow, service integration
**Capabilities:** See `/docs/capabilities/` for feature capabilities enabled by protocols
**Features:** See `/docs/features/` for user-facing feature specifications
**Constraints:** See `/docs/constraints/` for design constraints and trade-offs

---

**Last Updated:** 2025-12-10
**Document Version:** 1.0
**Status:** Complete
