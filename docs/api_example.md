# Iron Cage API — v1 (Self-Hosted)

## Overview

Self-hosted solution with full features: multi-user, budgets, rate limits.

| Group | Endpoints | Auth |
|-------|-----------|------|
| Health | 1 | - |
| Auth | 3 | - / JWT |
| Users | 3 | JWT (admin) |
| Projects | 5 | JWT |
| IC Keys | 3 | JWT |
| Vault | 2 | JWT |
| Library | 2 | IC Key |

**19 endpoints total.**

---

# Health

```
GET /api/v1/health
```

**Response 200:**

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

---

# Auth

## Register

```
POST /api/v1/auth/register
```

**Body:**

```json
{
  "email": "dev@company.com",
  "password": "securepassword123",
  "name": "John Doe"
}
```

**Response 201:**

```json
{
  "success": true,
  "user": {
    "id": "uuid",
    "email": "dev@company.com",
    "name": "John Doe",
    "role": "developer"
  },
  "token": "eyJhbGciOiJIUzI1NiIs..."
}
```

> First user becomes `admin`, others are `developer`.

---

## Login

```
POST /api/v1/auth/login
```

**Body:**

```json
{
  "email": "dev@company.com",
  "password": "securepassword123"
}
```

**Response 200:**

```json
{
  "success": true,
  "user": {
    "id": "uuid",
    "email": "dev@company.com",
    "role": "admin"
  },
  "token": "eyJhbGciOiJIUzI1NiIs..."
}
```

---

## Get Current User

```
GET /api/v1/auth/me
```

**Headers:** `Authorization: Bearer <token>`

**Response 200:**

```json
{
  "success": true,
  "user": {
    "id": "uuid",
    "email": "dev@company.com",
    "name": "John Doe",
    "role": "admin",
    "created_at": "2025-01-15T10:30:00Z"
  }
}
```

---

# Users

## List Users

```
GET /api/v1/users
```

**Headers:** `Authorization: Bearer <token>`

**Required role:** `admin`

**Response 200:**

```json
{
  "success": true,
  "users": [
    {
      "id": "uuid",
      "email": "admin@company.com",
      "name": "Admin",
      "role": "admin",
      "created_at": "2025-01-15T10:30:00Z"
    },
    {
      "id": "uuid",
      "email": "dev@company.com",
      "name": "Developer",
      "role": "developer",
      "created_at": "2025-01-16T09:00:00Z"
    }
  ]
}
```

---

## Update User

```
PATCH /api/v1/users/:id
```

**Headers:** `Authorization: Bearer <token>`

**Required role:** `admin`

**Body:**

```json
{
  "role": "admin",
  "name": "New Name"
}
```

**Response 200:**

```json
{
  "success": true,
  "user": {
    "id": "uuid",
    "email": "dev@company.com",
    "name": "New Name",
    "role": "admin"
  }
}
```

---

## Delete User

```
DELETE /api/v1/users/:id
```

**Headers:** `Authorization: Bearer <token>`

**Required role:** `admin`

**Response 200:**

```json
{
  "success": true
}
```

---

# Projects

## List Projects

```
GET /api/v1/projects
```

**Headers:** `Authorization: Bearer <token>`

**Response 200:**

```json
{
  "success": true,
  "projects": [
    {
      "id": "uuid",
      "name": "My Agent",
      "budget_usd": 100.00,
      "spent_usd": 45.30,
      "keys_count": 3,
      "providers": ["openai", "anthropic"],
      "created_at": "2025-01-15T10:30:00Z"
    }
  ]
}
```

---

## Create Project

```
POST /api/v1/projects
```

**Headers:** `Authorization: Bearer <token>`

**Required role:** `admin`

**Body:**

```json
{
  "name": "My Agent",
  "budget_usd": 100.00
}
```

**Response 201:**

```json
{
  "success": true,
  "project": {
    "id": "uuid",
    "name": "My Agent",
    "budget_usd": 100.00,
    "spent_usd": 0,
    "created_at": "2025-01-15T10:30:00Z"
  }
}
```

---

## Get Project

```
GET /api/v1/projects/:id
```

**Headers:** `Authorization: Bearer <token>`

**Response 200:**

```json
{
  "success": true,
  "project": {
    "id": "uuid",
    "name": "My Agent",
    "budget_usd": 100.00,
    "spent_usd": 45.30,
    "remaining_usd": 54.70,
    "keys_count": 3,
    "providers": ["openai", "anthropic"],
    "created_at": "2025-01-15T10:30:00Z"
  }
}
```

---

## Update Project

```
PATCH /api/v1/projects/:id
```

**Headers:** `Authorization: Bearer <token>`

**Required role:** `admin`

**Body:**

```json
{
  "name": "Updated Name",
  "budget_usd": 200.00
}
```

**Response 200:**

```json
{
  "success": true,
  "project": {
    "id": "uuid",
    "name": "Updated Name",
    "budget_usd": 200.00,
    "spent_usd": 45.30
  }
}
```

---

## Delete Project

```
DELETE /api/v1/projects/:id
```

**Headers:** `Authorization: Bearer <token>`

**Required role:** `admin`

**Response 200:**

```json
{
  "success": true
}
```

---

# IC Keys

## List IC Keys

```
GET /api/v1/projects/:project_id/keys
```

**Headers:** `Authorization: Bearer <token>`

**Response 200:**

```json
{
  "success": true,
  "keys": [
    {
      "id": "uuid",
      "name": "Dev Key 1",
      "prefix": "ic_a1b2c3d4",
      "is_active": true,
      "created_by": "admin@company.com",
      "created_at": "2025-01-15T10:30:00Z"
    }
  ]
}
```

---

## Generate IC Key

```
POST /api/v1/projects/:project_id/keys
```

**Headers:** `Authorization: Bearer <token>`

**Body:**

```json
{
  "name": "Dev Key 1"
}
```

**Response 201:**

```json
{
  "success": true,
  "key": {
    "id": "uuid",
    "name": "Dev Key 1",
    "prefix": "ic_a1b2c3d4",
    "full_key": "ic_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"
  }
}
```

> ⚠️ `full_key` shown only once!

---

## Revoke IC Key

```
DELETE /api/v1/keys/:id
```

**Headers:** `Authorization: Bearer <token>`

**Response 200:**

```json
{
  "success": true
}
```

---

# Vault

## Add Provider Key

```
POST /api/v1/projects/:project_id/vault
```

**Headers:** `Authorization: Bearer <token>`

**Required role:** `admin`

**Body:**

```json
{
  "provider": "openai",
  "key": "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
}
```

**Response 201:**

```json
{
  "success": true,
  "provider": {
    "id": "uuid",
    "provider": "openai",
    "key_hint": "sk-...xxx"
  }
}
```

**Providers:**

| Provider | Key prefix |
|----------|------------|
| `openai` | `sk-` |
| `anthropic` | `sk-ant-` |

---

## Remove Provider Key

```
DELETE /api/v1/vault/:id
```

**Headers:** `Authorization: Bearer <token>`

**Required role:** `admin`

**Response 200:**

```json
{
  "success": true
}
```

---

# Library

## Get Provider Keys

Called by `iron_cage` Python library.

```
POST /api/v1/lib/keys
```

**Headers:** `X-IC-Key: ic_a1b2c3d4e5f6...`

**Response 200:**

```json
{
  "success": true,
  "keys": {
    "openai": "sk-xxx",
    "anthropic": "sk-ant-xxx"
  },
  "budget": {
    "limit_usd": 100.00,
    "spent_usd": 45.30,
    "remaining_usd": 54.70
  }
}
```

**Errors:**

| Code | Status | Description |
|------|--------|-------------|
| `INVALID_KEY` | 401 | Key wrong or missing |
| `KEY_REVOKED` | 403 | Key deactivated |
| `BUDGET_EXCEEDED` | 402 | No budget remaining |
| `NO_PROVIDERS` | 404 | No keys in vault |

---

## Report Usage

Called by library after each request to track spending.

```
POST /api/v1/lib/usage
```

**Headers:** `X-IC-Key: ic_a1b2c3d4e5f6...`

**Body:**

```json
{
  "provider": "openai",
  "model": "gpt-4",
  "input_tokens": 150,
  "output_tokens": 450,
  "cost_usd": 0.025
}
```

**Response 200:**

```json
{
  "success": true,
  "budget": {
    "spent_usd": 45.33,
    "remaining_usd": 54.67
  }
}
```

---

# Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `POST /lib/keys` | 60 | per minute |
| `POST /lib/usage` | 300 | per minute |
| All other | 100 | per minute |

**Response 429:**

```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMITED",
    "message": "Too many requests",
    "retry_after": 45
  }
}
```

**Headers:**

```
Retry-After: 45
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1705312800
```

---

# Error Format

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Description"
  }
}
```

## Error Codes

| Code | Status | Description |
|------|--------|-------------|
| `INVALID_KEY` | 401 | IC key wrong |
| `KEY_REVOKED` | 403 | IC key deactivated |
| `INVALID_CREDENTIALS` | 401 | Wrong email/password |
| `UNAUTHORIZED` | 401 | Missing/invalid JWT |
| `FORBIDDEN` | 403 | Not enough permissions |
| `EMAIL_EXISTS` | 409 | Email taken |
| `NOT_FOUND` | 404 | Resource not found |
| `BUDGET_EXCEEDED` | 402 | No budget left |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

---

# Auth

## JWT (Dashboard)

```
Authorization: Bearer eyJ...
```

Expiry: 24 hours

## IC Key (Library)

```
X-IC-Key: ic_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
```

---

# Roles

| Role | Permissions |
|------|-------------|
| `admin` | All actions |
| `developer` | View projects, use keys |

---

# Config (.env)

```bash
# Database
DATABASE_URL=postgres://user:pass@localhost/iron_cage

# Encryption
MASTER_KEY=32-byte-key-for-aes-encryption!!

# JWT
JWT_SECRET=another-32-byte-secret-key-here
JWT_EXPIRY_HOURS=24

# Rate Limits
RATE_LIMIT_LIB_KEYS=60
RATE_LIMIT_LIB_USAGE=300
RATE_LIMIT_DEFAULT=100
```

---

# Summary

| Method | Endpoint | Auth | Role | Description |
|--------|----------|------|------|-------------|
| GET | `/health` | - | - | Health |
| POST | `/auth/register` | - | - | Register |
| POST | `/auth/login` | - | - | Login |
| GET | `/auth/me` | JWT | any | Current user |
| GET | `/users` | JWT | admin | List users |
| PATCH | `/users/:id` | JWT | admin | Update user |
| DELETE | `/users/:id` | JWT | admin | Delete user |
| GET | `/projects` | JWT | any | List projects |
| POST | `/projects` | JWT | admin | Create project |
| GET | `/projects/:id` | JWT | any | Get project |
| PATCH | `/projects/:id` | JWT | admin | Update project |
| DELETE | `/projects/:id` | JWT | admin | Delete project |
| GET | `/projects/:id/keys` | JWT | any | List IC keys |
| POST | `/projects/:id/keys` | JWT | any | Generate IC key |
| DELETE | `/keys/:id` | JWT | any | Revoke IC key |
| POST | `/projects/:id/vault` | JWT | admin | Add provider |
| DELETE | `/vault/:id` | JWT | admin | Remove provider |
| POST | `/lib/keys` | IC Key | - | Get provider keys |
| POST | `/lib/usage` | IC Key | - | Report usage |
