# Protocol: Settings API

### Scope

#### In Scope

- User-level settings (personal preferences)
- Project-level settings (project defaults and configurations)
- System-level settings (global defaults)
- Settings hierarchy and inheritance (system → project → user)
- Partial update support (PATCH-like behavior with PUT)
- Settings categories (Display, Notifications, Operational, Security)
- Authorization by role (Admin for system, Owner/Admin for project, any user for own settings)
- Settings validation and type checking
- Audit logging for all settings changes

#### Out of Scope

- Settings versioning and history (POST-PILOT feature)
- Settings templates or presets (POST-PILOT feature)
- Bulk settings import/export (POST-PILOT feature)
- Settings diff/comparison tools (POST-PILOT feature)
- Settings rollback functionality (POST-PILOT feature)
- Cross-project settings replication (POST-PILOT feature)
- Notification delivery (covered by separate notification service)
- Actual enforcement of settings values (enforced by respective APIs)

### Purpose

**User Need:** Users need ability to customize their experience through personal preferences (UI theme, notifications, defaults), while project owners need project-level configuration (budget defaults, provider settings, team policies), and administrators need system-wide control over operational parameters, security policies, and resource limits. These settings must cascade hierarchically (system → project → user) to provide sensible defaults while allowing local overrides.

**Solution:** Protocol 016 provides three-tier settings hierarchy with 7 endpoints: users manage personal preferences (GET/PUT/DELETE /api/v1/settings/user), project owners configure project settings (GET/PUT /api/v1/settings/project/{id}), and administrators control system defaults (GET/PUT /api/v1/settings/system). All update endpoints support partial updates (only specified fields modified), settings cascade through inheritance chain with tracking of source level (system/project/user), and comprehensive validation ensures setting values meet type and range constraints.

**Key Insight:** The hierarchical settings model separates concerns across three distinct needs: system administrators define organizational policy (security, limits, defaults), project owners configure team-specific behavior (budgets, providers, notifications), and users personalize their interface (theme, layout, alerts). This separation enables both top-down governance (admin enforces password policy) and bottom-up customization (user chooses dark theme) within the same framework. The inheritance tracking ("this setting came from project level") provides transparency and helps users understand why settings have certain values.

---

**Status:** POST-PILOT (Specification)
**Version:** 1.0.0
**Last Updated:** 2025-12-14
**Priority:** POST-PILOT

### Standards Compliance

This protocol defines the following ID formats:

- `user_id`: `user_<alphanumeric>` (e.g., `user_abc123`, `user_admin001`)
  - Pattern: `^user_[a-z0-9_]{3,32}$`
  - Source: Protocol 007 (Authentication API)
  - Usage: User identifier for user settings and audit logs

- `project_id`: `proj_<alphanumeric>` (e.g., `proj_master_001`)
  - Pattern: `^proj_[a-z0-9_]{3,32}$`
  - Source: Protocol 015 (Projects API)
  - Usage: Project identifier for project settings scope
  - Master Project ID: `proj_master_001` (constant in Pilot phase)

**Data Format Standards:**
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45Z`)
- Booleans: JSON boolean `true`/`false` (not strings)
- Numeric ranges: Validated against documented min/max values
- Currency: Decimal numbers with 2 decimal places (e.g., `100.00`)
- Settings values: Type varies by setting (string, number, boolean, object, array)

**Error Format Standards:**
- Consistent error response structure with `error.code` and `error.message`
- Optional `error.field` for validation errors (dot-notation path)
- HTTP status codes: 200, 400, 401, 403, 404

**API Design Standards:**
- URL structure: `/api/v1/settings/{scope}` where scope is user|project|system
- Partial updates: Only specified fields modified (PATCH-like with PUT)
- Inheritance tracking: Response includes source level for each setting

### Settings Hierarchy

Settings are organized in three scopes with inheritance:

#### System Settings (Global)

- Administrator-only configuration
- Default values for all projects
- Examples: Max agents per project, allowed provider types, rate limits, security policies

#### Project Settings (Project-scoped)

- Owner/Admin can configure
- Override system defaults for specific project
- Examples: Default agent budget, max agents per user, notification preferences, provider configurations

#### User Settings (User-scoped)

- User-specific preferences
- Override project defaults for individual user
- Examples: UI theme, email notifications, dashboard layout, timezone

#### Settings Inheritance Chain

Settings cascade from system → project → user:

```
System Default: max_agents_per_user = 20
  ↓
Project Override: max_agents_per_user = 10
  ↓
User Setting: (inherits project value = 10)
```

Each GET response includes `inheritance` object tracking source level for each setting.

### Endpoints

#### Get User Settings

Retrieves current user's personal settings with inherited values from project and system levels.

**Endpoint:** `GET /api/v1/settings/user`

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```http
GET /api/v1/settings/user
Authorization: Bearer <user-token or api-token>
```

##### Success Response

```json
HTTP 200 OK
Content-Type: application/json

{
  "user_id": "user_abc123",
  "settings": {
    "display": {
      "theme": "dark",
      "dashboard_layout": "grid",
      "default_date_range": "last-7-days",
      "currency_format": "USD"
    },
    "notifications": {
      "email_enabled": true,
      "budget_alerts": true,
      "agent_status_alerts": true,
      "notification_frequency": "immediate"
    },
    "operational": {
      "default_agent_budget": 100.00,
      "auto_pause_threshold": 95.0
    }
  },
  "inheritance": {
    "display.theme": "user",
    "display.dashboard_layout": "user",
    "operational.default_agent_budget": "project",
    "operational.auto_pause_threshold": "system"
  }
}
```

##### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | string | User ID |
| `settings` | object | Current settings (merged from system/project/user) |
| `inheritance` | object | Source of each setting value (system/project/user) |

##### Authorization

- Any authenticated user (can only view own settings)

##### Audit Log

No (read operation)

#### Update User Settings

Updates current user's personal settings. Only specified fields are updated (partial update).

**Endpoint:** `PUT /api/v1/settings/user`

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```http
PUT /api/v1/settings/user
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "display": {
    "theme": "light",
    "dashboard_layout": "list"
  },
  "notifications": {
    "email_enabled": false
  }
}
```

##### Request Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `display` | object | No | Display preferences |
| `notifications` | object | No | Notification preferences |
| `operational` | object | No | Operational preferences |

##### Success Response

```json
HTTP 200 OK
Content-Type: application/json

{
  "user_id": "user_abc123",
  "settings": {
    "display": {
      "theme": "light",
      "dashboard_layout": "list",
      "default_date_range": "last-7-days",
      "currency_format": "USD"
    },
    "notifications": {
      "email_enabled": false,
      "budget_alerts": true,
      "agent_status_alerts": true,
      "notification_frequency": "immediate"
    },
    "operational": {
      "default_agent_budget": 100.00,
      "auto_pause_threshold": 95.0
    }
  },
  "updated_at": "2025-12-10T10:30:45Z"
}
```

##### Authorization

- Any authenticated user (can only update own settings)

##### Error Responses

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "INVALID_SETTING_VALUE",
    "message": "Invalid theme value. Must be one of: light, dark, auto",
    "field": "display.theme"
  }
}
```

##### Audit Log

Yes - User ID, timestamp, setting scope (user), previous values, new values

#### Reset User Settings

Resets user settings to project/system defaults.

**Endpoint:** `DELETE /api/v1/settings/user`

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```http
DELETE /api/v1/settings/user
Authorization: Bearer <user-token or api-token>
```

##### Success Response

```json
HTTP 200 OK
Content-Type: application/json

{
  "message": "User settings reset to defaults",
  "settings": {
    "display": {
      "theme": "auto",
      "dashboard_layout": "grid",
      "default_date_range": "last-7-days",
      "currency_format": "USD"
    },
    "notifications": {
      "email_enabled": true,
      "budget_alerts": true,
      "agent_status_alerts": true,
      "notification_frequency": "immediate"
    },
    "operational": {
      "default_agent_budget": 100.00,
      "auto_pause_threshold": 95.0
    }
  }
}
```

##### Authorization

- Any authenticated user (can only reset own settings)

##### Audit Log

Yes - User ID, timestamp, setting scope (user), action (reset to defaults)

#### Get Project Settings

Retrieves project-level settings. Requires Owner or Admin role.

**Endpoint:** `GET /api/v1/settings/project/{project_id}`

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```http
GET /api/v1/settings/project/proj_master_001
Authorization: Bearer <user-token or api-token>
```

##### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project ID (format: `proj_<alphanumeric>`) |

##### Success Response

```json
HTTP 200 OK
Content-Type: application/json

{
  "project_id": "proj_master_001",
  "settings": {
    "operational": {
      "default_agent_budget": 100.00,
      "max_agents_per_user": 10,
      "auto_pause_threshold": 95.0,
      "budget_alert_levels": [50, 80, 95]
    },
    "providers": {
      "allowed_providers": ["openai", "anthropic", "google"],
      "default_provider": "openai"
    },
    "notifications": {
      "webhook_url": "https://hooks.example.com/iron-alerts",
      "email_from": "alerts@example.com"
    },
    "security": {
      "session_timeout_minutes": 480,
      "api_token_max_age_days": 90,
      "require_2fa": false
    }
  },
  "inheritance": {
    "operational.default_agent_budget": "project",
    "operational.max_agents_per_user": "project",
    "operational.auto_pause_threshold": "system",
    "security.session_timeout_minutes": "system"
  }
}
```

##### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `project_id` | string | Project ID |
| `settings` | object | Current project settings (merged with system defaults) |
| `inheritance` | object | Source of each setting (system/project) |

##### Authorization

- Owner or Admin role in the project

##### Error Responses

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Owner or Admin role required to view project settings"
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

##### Audit Log

No (read operation)

#### Update Project Settings

Updates project-level settings. Requires Owner or Admin role.

**Endpoint:** `PUT /api/v1/settings/project/{project_id}`

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```http
PUT /api/v1/settings/project/proj_master_001
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "operational": {
    "default_agent_budget": 150.00,
    "max_agents_per_user": 15
  },
  "notifications": {
    "webhook_url": "https://hooks.example.com/new-endpoint"
  }
}
```

##### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project ID (format: `proj_<alphanumeric>`) |

##### Request Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `operational` | object | No | Operational settings |
| `providers` | object | No | Provider settings |
| `notifications` | object | No | Notification settings |
| `security` | object | No | Security settings (Admin only) |

##### Success Response

```json
HTTP 200 OK
Content-Type: application/json

{
  "project_id": "proj_master_001",
  "settings": {
    "operational": {
      "default_agent_budget": 150.00,
      "max_agents_per_user": 15,
      "auto_pause_threshold": 95.0,
      "budget_alert_levels": [50, 80, 95]
    },
    "providers": {
      "allowed_providers": ["openai", "anthropic", "google"],
      "default_provider": "openai"
    },
    "notifications": {
      "webhook_url": "https://hooks.example.com/new-endpoint",
      "email_from": "alerts@example.com"
    },
    "security": {
      "session_timeout_minutes": 480,
      "api_token_max_age_days": 90,
      "require_2fa": false
    }
  },
  "updated_at": "2025-12-10T10:30:45Z",
  "updated_by": "user_abc123"
}
```

##### Authorization

- Owner or Admin role in the project
- Security settings require Admin role

##### Error Responses

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "INVALID_SETTING_VALUE",
    "message": "default_agent_budget must be at least 0.01",
    "field": "operational.default_agent_budget"
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Admin role required to modify security settings"
  }
}
```

##### Audit Log

Yes - User ID, timestamp, project ID, setting scope (project), previous values, new values

#### Get System Settings

Retrieves system-wide default settings. Requires Admin role.

**Endpoint:** `GET /api/v1/settings/system`

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```http
GET /api/v1/settings/system
Authorization: Bearer <user-token or api-token>
```

##### Success Response

```json
HTTP 200 OK
Content-Type: application/json

{
  "settings": {
    "operational": {
      "default_agent_budget": 100.00,
      "max_agents_per_user": 20,
      "max_agents_per_project": 100,
      "auto_pause_threshold": 95.0,
      "budget_alert_levels": [50, 80, 95]
    },
    "providers": {
      "allowed_provider_types": ["openai", "anthropic", "google", "azure"],
      "default_rate_limit": 100,
      "default_timeout_seconds": 30
    },
    "security": {
      "session_timeout_minutes": 480,
      "api_token_max_age_days": 90,
      "password_min_length": 12,
      "require_2fa_for_admin": true,
      "ip_whitelist_enabled": false
    },
    "audit": {
      "retention_days": 90,
      "log_mutations_only": true
    }
  }
}
```

##### Authorization

- Admin role only

##### Error Responses

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Admin role required to view system settings"
  }
}
```

##### Audit Log

No (read operation)

#### Update System Settings

Updates system-wide default settings. Requires Admin role.

**Endpoint:** `PUT /api/v1/settings/system`

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```http
PUT /api/v1/settings/system
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "operational": {
    "max_agents_per_project": 200
  },
  "security": {
    "require_2fa_for_admin": true,
    "ip_whitelist_enabled": true
  }
}
```

##### Request Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `operational` | object | No | Operational defaults |
| `providers` | object | No | Provider defaults |
| `security` | object | No | Security policies |
| `audit` | object | No | Audit logging configuration |

##### Success Response

```json
HTTP 200 OK
Content-Type: application/json

{
  "settings": {
    "operational": {
      "default_agent_budget": 100.00,
      "max_agents_per_user": 20,
      "max_agents_per_project": 200,
      "auto_pause_threshold": 95.0,
      "budget_alert_levels": [50, 80, 95]
    },
    "providers": {
      "allowed_provider_types": ["openai", "anthropic", "google", "azure"],
      "default_rate_limit": 100,
      "default_timeout_seconds": 30
    },
    "security": {
      "session_timeout_minutes": 480,
      "api_token_max_age_days": 90,
      "password_min_length": 12,
      "require_2fa_for_admin": true,
      "ip_whitelist_enabled": true
    },
    "audit": {
      "retention_days": 90,
      "log_mutations_only": true
    }
  },
  "updated_at": "2025-12-10T10:30:45Z",
  "updated_by": "user_admin001"
}
```

##### Authorization

- Admin role only

##### Error Responses

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "INVALID_SETTING_VALUE",
    "message": "max_agents_per_project must be greater than max_agents_per_user",
    "field": "operational.max_agents_per_project"
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Admin role required to modify system settings"
  }
}
```

##### Audit Log

Yes - User ID, timestamp, setting scope (system), previous values, new values

### Settings Schema

#### Display Settings

```json
{
  "theme": "light|dark|auto",
  "dashboard_layout": "grid|list|compact",
  "default_date_range": "today|yesterday|last-7-days|last-30-days|all-time",
  "currency_format": "USD|EUR|GBP",
  "timezone": "UTC|America/New_York|Europe/London|..."
}
```

#### Notification Settings

```json
{
  "email_enabled": true,
  "budget_alerts": true,
  "agent_status_alerts": true,
  "notification_frequency": "immediate|daily|weekly",
  "webhook_url": "https://...",
  "email_from": "alerts@example.com"
}
```

#### Operational Settings

```json
{
  "default_agent_budget": 100.00,
  "max_agents_per_user": 10,
  "max_agents_per_project": 100,
  "auto_pause_threshold": 95.0,
  "budget_alert_levels": [50, 80, 95]
}
```

#### Security Settings

```json
{
  "session_timeout_minutes": 480,
  "api_token_max_age_days": 90,
  "password_min_length": 12,
  "require_2fa": false,
  "require_2fa_for_admin": true,
  "ip_whitelist_enabled": false,
  "allowed_ip_ranges": ["192.168.1.0/24"]
}
```

### Common Patterns

#### Partial Updates

All update endpoints support partial updates - only specified fields are modified:

```json
// Only updates theme, other display settings unchanged
{
  "display": {
    "theme": "dark"
  }
}
```

#### Settings Inheritance

Settings cascade from system → project → user:

1. System settings provide global defaults
2. Project settings override system defaults for that project
3. User settings override project defaults for that user

The `inheritance` field in GET responses tracks the source level for each setting.

#### Validation Rules

- Budget values must be ≥ 0.01
- Thresholds must be 0-100 (percentage)
- Max agents per project ≥ max agents per user
- Alert levels must be ascending order
- URLs must be valid HTTPS endpoints
- IP ranges must use valid CIDR notation

### Implementation Notes

#### Pilot Phase

In Pilot, settings are hardcoded and not exposed via API:

- Default agent budget: $100.00
- Max agents per user: 10
- Auto-pause threshold: 95%
- Budget alert levels: [50%, 80%, 95%]
- Session timeout: 8 hours
- Theme: Auto (system preference)

#### Database Schema (Reference)

```sql
-- System-level settings (single row)
CREATE TABLE system_settings (
  id INTEGER PRIMARY KEY DEFAULT 1,
  settings JSONB NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  updated_by VARCHAR(50),
  CONSTRAINT single_row CHECK (id = 1)
);

-- Project-level settings
CREATE TABLE project_settings (
  project_id VARCHAR(50) PRIMARY KEY,
  settings JSONB NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  updated_by VARCHAR(50),
  FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- User-level settings
CREATE TABLE user_settings (
  user_id VARCHAR(50) PRIMARY KEY,
  settings JSONB NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);
```

#### Settings Resolution Algorithm

```python
def resolve_settings(user_id, project_id):
  system_settings = get_system_settings()
  project_settings = get_project_settings(project_id)
  user_settings = get_user_settings(user_id)

  # Deep merge: user > project > system
  resolved = deep_merge(
    system_settings,
    project_settings,
    user_settings
  )

  # Track inheritance for each setting
  inheritance = {}
  for key in all_setting_keys:
    if key in user_settings:
      inheritance[key] = "user"
    elif key in project_settings:
      inheritance[key] = "project"
    else:
      inheritance[key] = "system"

  return resolved, inheritance
```

#### Caching Strategy

- System settings: Cache globally (1 hour TTL)
- Project settings: Cache per project (15 minutes TTL)
- User settings: Cache per user (5 minutes TTL)
- Invalidate cache on update

### Security Considerations

#### Authorization Matrix

| Endpoint | User | Owner | Admin |
|----------|------|-------|-------|
| GET /settings/user | Own only | Own only | Own only |
| PUT /settings/user | Own only | Own only | Own only |
| DELETE /settings/user | Own only | Own only | Own only |
| GET /settings/project/{id} | ❌ | ✅ | ✅ |
| PUT /settings/project/{id} | ❌ | ✅ (non-security) | ✅ |
| GET /settings/system | ❌ | ❌ | ✅ |
| PUT /settings/system | ❌ | ❌ | ✅ |

#### Sensitive Settings

Security settings have additional restrictions:

- Only Admin can modify system security settings
- Only Admin can modify project security settings
- Owner can view but not modify security settings
- Two-factor authentication cannot be disabled if required by policy

#### Validation

- All numeric ranges validated (min/max)
- URL endpoints validated and tested on update
- Email addresses validated with regex
- IP ranges validated with CIDR notation check

### Cross-References

#### Related Principles Documents

None

#### Related Architecture Documents

None

#### Used By

- Dashboard applications (display user preferences, project configurations)
- CLI tools (respect user settings for output formatting)
- Admin tools (system settings management interface)

#### Dependencies

- Protocol 002: REST API Protocol (General REST standards, partial update patterns)
- Protocol 007: Authentication API (User authentication, role-based authorization)
- Protocol 008: User Management API (User roles for authorization checks)
- Protocol 010: Agents API (Operational settings affect agent creation)
- Protocol 015: Projects API (Project-level settings scope)

#### Implementation

**Status:** POST-PILOT (Not yet implemented)

**Planned Files:**
- `module/iron_control_api/src/routes/settings.rs` - Endpoint implementation
- `module/iron_control_api/src/services/settings_service.rs` - Settings resolution and inheritance logic
- `module/iron_control_api/tests/settings/endpoints.rs` - Integration tests
- `module/iron_control_api/tests/settings/inheritance.rs` - Inheritance chain tests
- `module/iron_control_api/tests/settings/authorization.rs` - Authorization tests
- `module/iron_control_api/tests/settings/validation.rs` - Validation tests

**Database Migration:**
- Create system_settings, project_settings, user_settings tables
- Seed system_settings with defaults
- Migrate hardcoded Pilot values to database
