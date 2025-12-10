# 016 Settings API

**Status**: POST-PILOT
**Version**: 1.0
**Last Updated**: 2025-12-10

## Overview

The Settings API provides endpoints for managing system-wide, project-level, and user-level configuration settings in the Iron Control Panel. This API is planned for POST-PILOT implementation and enables customization of default behaviors, notification preferences, display options, and operational parameters.

**Note**: This specification describes the complete Settings API for future implementation. In the Pilot phase, settings are hardcoded and not exposed via API.

## Key Concepts

### Settings Hierarchy
Settings are organized in three scopes with inheritance:

1. **System Settings** (Global)
   - Administrator-only configuration
   - Default values for all projects
   - Examples: Max agents per project, allowed provider types, rate limits

2. **Project Settings** (Project-scoped)
   - Owner/Admin can configure
   - Override system defaults for specific project
   - Examples: Default agent budget, max agents per user, notification preferences

3. **User Settings** (User-scoped)
   - User-specific preferences
   - Override project defaults for individual user
   - Examples: UI theme, email notifications, dashboard layout

### Settings Inheritance
Settings cascade from system → project → user:
```
System Default: max_agents_per_user = 20
  ↓
Project Override: max_agents_per_user = 10
  ↓
User Setting: (inherits project value = 10)
```

### Settings Categories

**Operational Settings**:
- Default agent budget
- Max agents per user
- Auto-pause thresholds
- Budget alert levels

**Notification Settings**:
- Email notifications (budget alerts, agent status)
- Webhook URLs for events
- Notification frequency (immediate, daily digest, weekly)

**Display Settings**:
- UI theme (light, dark, auto)
- Dashboard layout preferences
- Default date ranges for analytics
- Currency display format

**Security Settings**:
- Session timeout duration
- API token expiration policy
- IP whitelist for API access
- Two-factor authentication requirements

## Base URL

```
https://api.iron-control.example.com/api/v1
```

## Authentication

All endpoints require authentication via:
- **User Token**: Short-lived session token from login
- **API Token**: Persistent authentication token

Authorization varies by settings scope:
- **System Settings**: Admin only
- **Project Settings**: Owner or Admin
- **User Settings**: Any authenticated user (own settings only)

## Endpoints

### 1. Get User Settings

Retrieves current user's personal settings with inherited values from project and system levels.

**Endpoint**: `GET /api/v1/settings/user`

**Request**:
```http
GET /api/v1/settings/user
Authorization: Bearer <user-token or api-token>
```

**Response** (HTTP 200 OK):
```json
{
  "user_id": "user-abc123",
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

**Response Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `user_id` | string | User ID |
| `settings` | object | Current settings (merged from system/project/user) |
| `inheritance` | object | Source of each setting value (system/project/user) |

**Authorization**:
- Any authenticated user (can only view own settings)

---

### 2. Update User Settings

Updates current user's personal settings. Only specified fields are updated (partial update).

**Endpoint**: `PUT /api/v1/settings/user`

**Request**:
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

**Request Fields**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `display` | object | No | Display preferences |
| `notifications` | object | No | Notification preferences |
| `operational` | object | No | Operational preferences |

**Response** (HTTP 200 OK):
```json
{
  "user_id": "user-abc123",
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

**Authorization**:
- Any authenticated user (can only update own settings)

**Error Responses**:

- **400 Bad Request**: Invalid setting value
```json
{
  "error": {
    "code": "INVALID_SETTING_VALUE",
    "message": "Invalid theme value. Must be one of: light, dark, auto",
    "field": "display.theme"
  }
}
```

---

### 3. Reset User Settings

Resets user settings to project/system defaults.

**Endpoint**: `DELETE /api/v1/settings/user`

**Request**:
```http
DELETE /api/v1/settings/user
Authorization: Bearer <user-token or api-token>
```

**Response** (HTTP 200 OK):
```json
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

**Authorization**:
- Any authenticated user (can only reset own settings)

---

### 4. Get Project Settings

Retrieves project-level settings. Requires Owner or Admin role.

**Endpoint**: `GET /api/v1/settings/project/{project_id}`

**Request**:
```http
GET /api/v1/settings/project/proj-master-001
Authorization: Bearer <user-token or api-token>
```

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project ID (format: `proj-*`) |

**Response** (HTTP 200 OK):
```json
{
  "project_id": "proj-master-001",
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

**Response Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `project_id` | string | Project ID |
| `settings` | object | Current project settings (merged with system defaults) |
| `inheritance` | object | Source of each setting (system/project) |

**Authorization**:
- Owner or Admin role in the project

**Error Responses**:

- **403 Forbidden**: Insufficient permissions
```json
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Owner or Admin role required to view project settings"
  }
}
```

- **404 Not Found**: Project not found
```json
{
  "error": {
    "code": "PROJECT_NOT_FOUND",
    "message": "Project not found"
  }
}
```

---

### 5. Update Project Settings

Updates project-level settings. Requires Owner or Admin role.

**Endpoint**: `PUT /api/v1/settings/project/{project_id}`

**Request**:
```http
PUT /api/v1/settings/project/proj-master-001
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

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project ID (format: `proj-*`) |

**Request Fields**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `operational` | object | No | Operational settings |
| `providers` | object | No | Provider settings |
| `notifications` | object | No | Notification settings |
| `security` | object | No | Security settings (Admin only) |

**Response** (HTTP 200 OK):
```json
{
  "project_id": "proj-master-001",
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
  "updated_by": "user-abc123"
}
```

**Authorization**:
- Owner or Admin role in the project
- Security settings require Admin role

**Error Responses**:

- **400 Bad Request**: Invalid setting value
```json
{
  "error": {
    "code": "INVALID_SETTING_VALUE",
    "message": "default_agent_budget must be at least 0.01",
    "field": "operational.default_agent_budget"
  }
}
```

- **403 Forbidden**: Insufficient permissions
```json
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Admin role required to modify security settings"
  }
}
```

---

### 6. Get System Settings

Retrieves system-wide default settings. Requires Admin role.

**Endpoint**: `GET /api/v1/settings/system`

**Request**:
```http
GET /api/v1/settings/system
Authorization: Bearer <user-token or api-token>
```

**Response** (HTTP 200 OK):
```json
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

**Authorization**:
- Admin role only

**Error Responses**:

- **403 Forbidden**: Insufficient permissions
```json
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Admin role required to view system settings"
  }
}
```

---

### 7. Update System Settings

Updates system-wide default settings. Requires Admin role.

**Endpoint**: `PUT /api/v1/settings/system`

**Request**:
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

**Request Fields**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `operational` | object | No | Operational defaults |
| `providers` | object | No | Provider defaults |
| `security` | object | No | Security policies |
| `audit` | object | No | Audit logging configuration |

**Response** (HTTP 200 OK):
```json
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
  "updated_by": "admin-user-001"
}
```

**Authorization**:
- Admin role only

**Error Responses**:

- **400 Bad Request**: Invalid setting value
```json
{
  "error": {
    "code": "INVALID_SETTING_VALUE",
    "message": "max_agents_per_project must be greater than max_agents_per_user",
    "field": "operational.max_agents_per_project"
  }
}
```

- **403 Forbidden**: Insufficient permissions
```json
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Admin role required to modify system settings"
  }
}
```

---

## Settings Schema

### Display Settings
```json
{
  "theme": "light|dark|auto",
  "dashboard_layout": "grid|list|compact",
  "default_date_range": "today|yesterday|last-7-days|last-30-days|all-time",
  "currency_format": "USD|EUR|GBP",
  "timezone": "UTC|America/New_York|Europe/London|..."
}
```

### Notification Settings
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

### Operational Settings
```json
{
  "default_agent_budget": 100.00,
  "max_agents_per_user": 10,
  "max_agents_per_project": 100,
  "auto_pause_threshold": 95.0,
  "budget_alert_levels": [50, 80, 95]
}
```

### Security Settings
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

## Common Patterns

### Partial Updates
All update endpoints support partial updates - only specified fields are modified:
```json
// Only updates theme, other display settings unchanged
{
  "display": {
    "theme": "dark"
  }
}
```

### Settings Inheritance
Settings cascade from system → project → user:
1. System settings provide global defaults
2. Project settings override system defaults for that project
3. User settings override project defaults for that user

### Validation Rules
- Budget values must be ≥ 0.01
- Thresholds must be 0-100 (percentage)
- Max agents per project ≥ max agents per user
- Alert levels must be ascending order
- URLs must be valid HTTPS endpoints

### Error Format
All errors follow the simple custom format:
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "field": "setting.path"  // Optional: for validation errors
  }
}
```

### Timestamps
All timestamps use ISO 8601 format with Z suffix:
- Format: `YYYY-MM-DDTHH:MM:SSZ`
- Example: `2025-12-10T10:30:45Z`

## Implementation Notes

### Pilot Phase
In Pilot, settings are hardcoded and not exposed via API:
- Default agent budget: $100.00
- Max agents per user: 10
- Auto-pause threshold: 95%
- Budget alert levels: [50%, 80%, 95%]
- Session timeout: 8 hours
- Theme: Auto (system preference)

### Database Schema (Reference)
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

### Settings Resolution Algorithm
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

### Caching Strategy
- System settings: Cache globally (1 hour TTL)
- Project settings: Cache per project (15 minutes TTL)
- User settings: Cache per user (5 minutes TTL)
- Invalidate cache on update

### Audit Logging
All settings changes are logged:
- User ID who made the change
- Timestamp of change
- Setting scope (system/project/user)
- Previous value and new value
- Change reason (optional)

## Security Considerations

### Authorization Matrix
| Endpoint | User | Owner | Admin |
|----------|------|-------|-------|
| GET /settings/user | Own only | Own only | Own only |
| PUT /settings/user | Own only | Own only | Own only |
| GET /settings/project/{id} | ❌ | ✅ | ✅ |
| PUT /settings/project/{id} | ❌ | ✅ (non-security) | ✅ |
| GET /settings/system | ❌ | ❌ | ✅ |
| PUT /settings/system | ❌ | ❌ | ✅ |

### Sensitive Settings
Security settings have additional restrictions:
- Only Admin can modify system security settings
- Only Admin can modify project security settings
- Owner can view but not modify security settings
- Two-factor authentication cannot be disabled if required by policy

### Validation
- All numeric ranges validated (min/max)
- URL endpoints validated and tested on update
- Email addresses validated with regex
- IP ranges validated with CIDR notation check

## Related APIs
- **001 Authentication API**: Session management settings
- **015 Projects API**: Project-level settings scope
- **010 Agents API**: Operational settings affect agent creation

## Changelog

### Version 1.0 (2025-12-10)
- Initial specification for POST-PILOT implementation
- Three-tier settings hierarchy (system/project/user)
- Full CRUD operations for all scopes
- Settings inheritance and resolution
- Comprehensive validation rules
