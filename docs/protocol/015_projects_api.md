# Protocol 015: Projects API

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10
**Priority:** NICE-TO-HAVE

---

## Overview

The Projects API provides read-only access to project information in the Iron Control Panel. In the Pilot phase, a single "Master Project" exists, and all users belong to this project. Full project CRUD operations (create, update, delete, multi-project support) are planned for POST-PILOT implementation.

## Key Concepts

### Project Structure
- **Single Project (Pilot)**: All users and resources belong to the "Master Project"
- **Project Attributes**: ID, name, description, creation date, user count, agent count
- **Resource Ownership**: All agents, providers, and tokens belong to the project
- **User Membership**: All users have access to the Master Project

### Authorization Model
- **List Projects**: Any authenticated user (User, Owner, Admin)
- **Get Project Details**: Any authenticated user (User, Owner, Admin)
- **Future CRUD**: POST-PILOT implementation will add role-based restrictions

### Future Expansion (POST-PILOT)
- Multiple projects per organization
- Project-level resource isolation
- Project-specific roles and permissions
- Project creation and management
- Project-level budget allocation

## Base URL

```
https://api.iron-control.example.com/api/v1
```

## Authentication

All endpoints require authentication via:
- **User Token**: Short-lived session token from login (`Authorization: Bearer <user-token>`)
- **API Token**: Persistent authentication token (`Authorization: Bearer <api-token>`)

## Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- All entity IDs use `prefix_uuid` format with underscore separator
- `project_id`: `project_<uuid>` (e.g., `project_550e8400-e29b-41d4-a716-446655440000`)
- `user_id`: `user_<uuid>`

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Counts: Integer (e.g., `42` users, `15` agents)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes: `VALIDATION_ERROR`, `UNAUTHORIZED`, `NOT_FOUND`
- HTTP status codes: 200, 400, 401, 404

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- Pagination: Offset-based with `?page=N&per_page=M` (default 50 items/page)
- URL structure: `/api/v1/projects`, `/api/v1/projects/{id}`

## Endpoints

### 1. List Projects

Retrieves all projects accessible to the authenticated user. In Pilot, returns the single "Master Project".

**Endpoint**: `GET /api/v1/projects`

**Request**:
```http
GET /api/v1/projects?page=1&per_page=50
Authorization: Bearer <user-token or api-token>
```

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page` | integer | No | Page number (default: 1) |
| `per_page` | integer | No | Items per page (default: 50, max: 100) |

**Response** (HTTP 200 OK):
```json
{
  "data": [
    {
      "id": "proj_master-001",
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

**Response Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Project ID (format: `proj-*`) |
| `name` | string | Project name |
| `description` | string | Project description |
| `user_count` | integer | Number of users in project |
| `agent_count` | integer | Number of agents in project |
| `created_at` | string | ISO 8601 timestamp with Z |

**Authorization**:
- Any authenticated user can list projects
- Users only see projects they have access to (Master Project in Pilot)

**Error Responses**:

- **401 Unauthorized**: Missing or invalid authentication token
```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication required"
  }
}
```

- **401 Unauthorized**: Expired authentication token
```json
{
  "error": {
    "code": "TOKEN_EXPIRED",
    "message": "Authentication token has expired"
  }
}
```

---

### 2. Get Project Details

Retrieves detailed information about a specific project.

**Endpoint**: `GET /api/v1/projects/{id}`

**Request**:
```http
GET /api/v1/projects/proj_master_001
Authorization: Bearer <user-token or api-token>
```

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Project ID (format: `proj-*`) |

**Response** (HTTP 200 OK):
```json
{
  "id": "proj_master-001",
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

**Response Fields**:
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

**Authorization**:
- Any authenticated user can view project details
- Users can only view projects they belong to

**Error Responses**:

- **401 Unauthorized**: Missing or invalid authentication token
```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication required"
  }
}
```

- **404 Not Found**: Project does not exist or user has no access
```json
{
  "error": {
    "code": "PROJECT_NOT_FOUND",
    "message": "Project not found"
  }
}
```

---

## Common Patterns

### Pagination
All list endpoints use offset pagination:
- Default: `page=1`, `per_page=50`
- Maximum: `per_page=100`
- Empty results return HTTP 200 with empty array

### Error Format
All errors follow the simple custom format:
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "field": "field_name"  // Optional: for validation errors
  }
}
```

### Timestamps
All timestamps use ISO 8601 format with Z suffix:
- Format: `YYYY-MM-DDTHH:MM:SSZ`
- Example: `2025-12-10T10:30:45Z`

### Currency
All monetary values use decimal numbers with 2 decimal places:
- Format: `number` (JSON number type)
- Example: `100.50`

## POST-PILOT Expansion

The following features are planned for POST-PILOT implementation:

### Project Management Endpoints (Future)
- `POST /api/v1/projects` - Create new project (Admin only)
- `PUT /api/v1/projects/{id}` - Update project (Owner/Admin)
- `DELETE /api/v1/projects/{id}` - Delete project (Admin only)

### User Management Endpoints (Future)
- `POST /api/v1/projects/{id}/users` - Add user to project
- `DELETE /api/v1/projects/{id}/users/{user_id}` - Remove user from project
- `PUT /api/v1/projects/{id}/users/{user_id}/role` - Update user role in project

### Resource Management (Future)
- Project-level resource isolation
- Cross-project resource sharing
- Project-level budget allocation
- Project-specific provider configurations

### Multi-Project Support (Future)
- Users can belong to multiple projects
- Context switching between projects
- Project-scoped API tokens
- Project-level analytics and reporting

## Implementation Notes

### Pilot Phase Constraints
- Single project (Master Project) hardcoded in system
- All users automatically belong to Master Project
- No project creation or deletion
- No multi-project switching
- Project ID is constant: `proj_master_001`

### Database Schema (Reference)
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
  PRIMARY KEY (project_id, user_id)
);
```

### Performance Considerations
- List endpoint cached (5-minute TTL in Pilot since single project)
- User count and agent count computed on-demand
- POST-PILOT: Implement denormalized counters for multi-project performance

### Security Considerations
- Users can only access projects they belong to
- Project existence checks prevent enumeration attacks
- POST-PILOT: Implement fine-grained project-level permissions

## Related APIs
- **001 Authentication API**: User login and token management
- **010 Agents API**: Agent creation and management within projects
- **011 Providers API**: Provider configuration within projects
- **016 Settings API**: Project-level settings management (POST-PILOT)

## Changelog

### Version 1.0 (2025-12-10)
- Initial specification for Pilot phase
- Read-only endpoints (List, Get)
- Single Master Project support
- Documented POST-PILOT expansion plans
