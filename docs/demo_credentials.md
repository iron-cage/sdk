# DEMO LOGIN CREDENTIALS

**URL:** https://dashboard.ironcage.ai

## Production Pilot Accounts

**Password (all users):** `IronDemo2025!`

| Email | Role | Notes |
|-------|------|-------|
| `admin@ironcage.ai` | admin | Full access |
| `demo@ironcage.ai` | user | Standard demo |
| `viewer@ironcage.ai` | user | Read-only |
| `tester@ironcage.ai` | user | Unlimited usage |
| `guest@ironcage.ai` | user | Minimal permissions |

## Demo Seeding for Local Development

- Default: Local databases start empty (no users are seeded automatically).
- Optional: Set `ENABLE_DEMO_SEED=true` to seed `admin@ironcage.ai` with password `IronDemo2025!`.
- If you previously relied on auto-seeded test users, enable the flag or create accounts manually.

---

**Note:** Production demo users available ONLY if `ENABLE_DEMO_SEED=true` set during deployment.
