# Protocol: Projects API

Provides read-only access to project information in the Iron Control Panel.

### Scope

#### In Scope

- Read-only project access (list, get details)
- Single "Master Project" implementation for Pilot phase
- Project attributes (name, description, counts, budgets, settings)
- User membership in Master Project (all users automatically included)
- Project-level resource counts (users, agents, providers)
- Project-level budget aggregation

#### Out of Scope

- Project creation, update, deletion (POST-PILOT feature)
- Multi-project support (POST-PILOT feature)
- Project membership management (POST-PILOT feature)
- Project-specific roles and permissions (POST-PILOT feature)
- Project-level resource isolation (POST-PILOT feature)
- Project-scoped API tokens (POST-PILOT feature)
- Cross-project resource sharing (POST-PILOT feature)

### Purpose

**User Need:** Users need visibility into their project context including resource counts, budget allocation, and project settings. Applications need to retrieve project information to display context, validate resource ownership, and enforce project-level constraints. Analytics systems need project metadata for aggregation and reporting.

**Solution:** Protocol 015 provides GET endpoints for listing projects and retrieving project details. In the Pilot phase, a single "Master Project" exists with all users automatically included, simplifying implementation while providing necessary project context. The read-only API ensures data consistency during Pilot while the architecture supports future multi-project expansion.

**Key Insight:** The single-project Pilot design balances simplicity with future scalability. By implementing the Projects API early (even for one project), client code can be written project-aware from day one, avoiding costly refactoring when multi-project support is added POST-PILOT. The Master Project ID (`proj_master_001`) serves as a known constant for configuration and testing during Pilot phase.

---

**Status:** Certain (Required for project context awareness)
**Version:** 1.1.0
**Last Updated:** 2025-12-14
**Priority:** MUST-HAVE

### Standards Compliance

This protocol defines the following ID formats:

- `project_id`: `proj_<alphanumeric>` (e.g., `proj_master_001`)
  - Pattern: `^proj_[a-z0-9_]{3,32}$`
  - Usage: Database entity identifier for projects
  - Master Project ID: `proj_master_001` (constant in Pilot phase)

- `user_id`: `user_<alphanumeric>` (e.g., `user_xyz789`)
  - Pattern: `^user_[a-z0-9_]{3,32}$`
  - Source: Protocol 007 (Authentication API)
  - Usage: Project member references

**Data Format Standards:**
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45Z`)
- Counts: Integer (e.g., `12` users, `8` agents)
- Currency: Decimal numbers with 2 decimal places (e.g., `100.50`)
- Booleans: JSON boolean `true`/`false` (not strings)

**Error Format Standards:**
- Consistent error response structure with `error.code` and `error.message`
- HTTP status codes: 200, 400, 401, 404

### Pilot Phase Design

**Single Project Model:**
- All users and resources belong to the "Master Project" (`proj_master_001`)
- No project creation, update, or deletion operations
- All authenticated users have read access to Master Project
- Simplifies authorization (no multi-project access control needed)

**Future Expansion (POST-PILOT):**
- Multiple projects per organization
- Project-level resource isolation
- Project-specific roles and permissions
- Project creation and management endpoints
- Project-level budget allocation
- User membership management

### Endpoints

#### List Projects

**Endpoint:** `GET /api/v1/projects`

**Description:** Retrieves all projects accessible to the authenticated user. In Pilot, returns the single "Master Project".

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```http
GET /api/v1/projects?page=1&per_page=50
Authorization: Bearer <user-token or api-token>
```

##### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page` | integer | No | Page number (default: 1) |
| `per_page` | integer | No | Items per page (default: 50, max: 100) |

##### Success Response

```json
HTTP 200 OK
Content-Type: application/json

{
  "data": [
    {
      "id": "proj_master_001",
      "name": "Master Project",
      "description": "Default project for Iron Control Panel Pilot",
      "user_count": 12,
      "agent_count": 8,
      "created_at": "2025-01-15T08:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total_items": 1,
    "total_pages": 1
  }
}
```

##### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Project ID (format: `proj_<alphanumeric>`) |
| `name` | string | Project name |
| `description` | string | Project description |
| `user_count` | integer | Number of users in project |
| `agent_count` | integer | Number of agents in project |
| `created_at` | string | ISO 8601 timestamp with Z |

##### Empty Results

```json
HTTP 200 OK
{
  "data": [],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total_items": 0,
    "total_pages": 0
  }
}
```

**Note:** In Pilot, this will never return empty results (Master Project always exists). POST-PILOT, new users without project membership may see empty results.

##### Error Responses

```json
HTTP 401 Unauthorized
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication required"
  }
}
```

```json
HTTP 401 Unauthorized
{
  "error": {
    "code": "TOKEN_EXPIRED",
    "message": "Authentication token has expired"
  }
}
```

##### Authorization

- **Any authenticated user:** Can list projects
- **Pilot:** All users see Master Project
- **POST-PILOT:** Users only see projects they have access to

##### Audit Log

No (read operation)

#### Get Project Details

**Endpoint:** `GET /api/v1/projects/{id}`

**Description:** Retrieves detailed information about a specific project.

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```http
GET /api/v1/projects/proj_master_001
Authorization: Bearer <user-token or api-token>
```

##### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Project ID (format: `proj_<alphanumeric>`) |

##### Success Response

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "proj_master_001",
  "name": "Master Project",
  "description": "Default project for Iron Control Panel Pilot",
  "user_count": 12,
  "agent_count": 8,
  "provider_count": 3,
  "total_budget": 1500.00,
  "total_spent": 892.45,
  "created_at": "2025-01-15T08:00:00Z",
  "settings": {
    "default_agent_budget": 100.00,
    "max_agents_per_user": 10,
    "allowed_providers": ["openai", "anthropic", "google"]
  }
}
```

##### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Project ID |
| `name` | string | Project name |
| `description` | string | Project description |
| `user_count` | integer | Number of users in project |
| `agent_count` | integer | Number of active agents |
| `provider_count` | integer | Number of configured providers |
| `total_budget` | number | Sum of all agent budgets |
| `total_spent` | number | Total spending across all agents |
| `created_at` | string | ISO 8601 timestamp with Z |
| `settings` | object | Project-level settings |
| `settings.default_agent_budget` | number | Default budget for new agents |
| `settings.max_agents_per_user` | integer | Maximum agents per user |
| `settings.allowed_providers` | array | List of allowed AI providers |

##### Error Responses

```json
HTTP 401 Unauthorized
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication required"
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "PROJECT_NOT_FOUND",
    "message": "Project not found"
  }
}
```

**Note:** In Pilot, 404 only occurs if user requests wrong project ID. In POST-PILOT, 404 also returned if user lacks access to requested project.

##### Authorization

- **Any authenticated user:** Can view project details
- **Pilot:** All users can view Master Project
- **POST-PILOT:** Users can only view projects they belong to

##### Audit Log

No (read operation)

### Data Models

#### Project Object (Summary)

```json
{
  "id": "proj_master_001",
  "name": "Master Project",
  "description": "Default project for Iron Control Panel Pilot",
  "user_count": 12,
  "agent_count": 8,
  "created_at": "2025-01-15T08:00:00Z"
}
```

#### Project Object (Details)

```json
{
  "id": "proj_master_001",
  "name": "Master Project",
  "description": "Default project for Iron Control Panel Pilot",
  "user_count": 12,
  "agent_count": 8,
  "provider_count": 3,
  "total_budget": 1500.00,
  "total_spent": 892.45,
  "created_at": "2025-01-15T08:00:00Z",
  "settings": {
    "default_agent_budget": 100.00,
    "max_agents_per_user": 10,
    "allowed_providers": ["openai", "anthropic", "google"]
  }
}
```

### Pilot Phase Implementation

#### Master Project Configuration

**Constant Values:**
- Project ID: `proj_master_001` (hardcoded)
- Project Name: "Master Project"
- Description: "Default project for Iron Control Panel Pilot"

**User Membership:**
- All users automatically belong to Master Project
- No membership management needed in Pilot
- POST-PILOT: Implement project_members table

**Resource Association:**
- All agents belong to Master Project
- All API tokens belong to Master Project
- All provider configurations belong to Master Project

#### Performance Optimizations

**Caching:**
- List endpoint response cached (5-minute TTL)
- Single project = minimal cache invalidation complexity
- POST-PILOT: Invalidate cache on project modifications

**Computed Fields:**
- `user_count`: Real-time count from users table
- `agent_count`: Real-time count from agents table
- `provider_count`: Real-time count from providers table
- POST-PILOT: Denormalize counters for multi-project performance

**Database Schema (Reference):**
```sql
-- Pilot: Single row in projects table
CREATE TABLE projects (
  id VARCHAR(50) PRIMARY KEY,        -- 'proj_master_001'
  name VARCHAR(255) NOT NULL,        -- 'Master Project'
  description TEXT,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

-- POST-PILOT: Add user-project relationships
CREATE TABLE project_members (
  project_id VARCHAR(50) NOT NULL,
  user_id VARCHAR(50) NOT NULL,
  role VARCHAR(50) NOT NULL,         -- 'admin', 'owner', 'user'
  joined_at TIMESTAMP NOT NULL,
  PRIMARY KEY (project_id, user_id),
  FOREIGN KEY (project_id) REFERENCES projects(id),
  FOREIGN KEY (user_id) REFERENCES users(id)
);
```

### Security

#### Access Control (Pilot)

**Authorization Model:**
- Any authenticated user can list projects
- Any authenticated user can view Master Project details
- No write operations = no write authorization needed

**Project Existence Checks:**
- Return 404 for non-existent project IDs (prevents enumeration)
- POST-PILOT: Return 404 for projects user doesn't have access to (prevents information leakage)

#### Access Control (POST-PILOT)

**Planned Authorization:**
- Users can only view projects they belong to
- Project admins can manage project settings
- Organization admins can create/delete projects
- Fine-grained project-level permissions (read, write, admin)

### Future Enhancements (POST-PILOT)

#### Project Management Endpoints

- `POST /api/v1/projects` - Create new project (Admin only)
- `PUT /api/v1/projects/{id}` - Update project (Owner/Admin)
- `DELETE /api/v1/projects/{id}` - Delete project (Admin only)
- `GET /api/v1/projects/{id}/members` - List project members
- `POST /api/v1/projects/{id}/members` - Add user to project
- `DELETE /api/v1/projects/{id}/members/{user_id}` - Remove user
- `PUT /api/v1/projects/{id}/members/{user_id}/role` - Update user role

#### Multi-Project Features

**User Experience:**
- Project switcher in dashboard
- Context switching between projects
- Project-scoped API tokens
- Project-level analytics and reporting

**Resource Management:**
- Project-level resource isolation
- Cross-project resource sharing (with permissions)
- Project-level budget allocation
- Project-specific provider configurations

**Architecture Changes:**
- Project context in all API requests
- Project-scoped authentication tokens
- Project-level audit logs
- Project-specific rate limits

### Cross-References

#### Related Principles Documents

None

#### Related Architecture Documents

None

#### Used By

- Dashboard applications (project context display)
- CLI tools (project switching, resource scoping)
- Analytics systems (project-level aggregation)
- Admin tools (project management, user assignment)

#### Dependencies

- Protocol 007: Authentication API (User authentication, token management)
- Protocol 010: Agents API (Agent creation within projects)
- Protocol 011: Providers API (Provider configuration within projects)
- Protocol 016: Settings API (Project-level settings management, POST-PILOT)
- Protocol 002: REST API Protocol (General REST standards, pagination)

#### Implementation

**Status:** Specified (Not yet implemented)

**Planned Files:**
- `module/iron_control_api/src/routes/projects.rs` - Endpoint implementation
- `module/iron_control_api/src/services/project_service.rs` - Project business logic
- `module/iron_control_api/tests/projects/endpoints.rs` - Integration tests
- `module/iron_control_api/tests/projects/pilot.rs` - Pilot phase specific tests

**Database Migration:**
- Create projects table with Master Project row
- Seed Master Project data (`proj_master_001`)
- POST-PILOT: Create project_members table and foreign keys
