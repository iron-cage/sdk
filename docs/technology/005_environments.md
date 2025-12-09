# Environments

**Purpose:** Document the runtime environment modes and their behaviors.

## Overview

The Iron Cage runtime adapts its behavior based on the deployment environment. This is controlled primarily by the `IRON_DEPLOYMENT_MODE` environment variable and various heuristic checks.

## Deployment Modes

The runtime supports the following modes:

| Mode | Trigger | Behavior | Use Case |
|------|---------|----------|----------|
| **Pilot** | Default (if no other signals) | Standard operation, uses local `iron.db` (SQLite) | Local development, demos |
| **Development** | `IRON_DEPLOYMENT_MODE=development` | **Clears `iron.db` on startup**, standard operation | Testing, CI/CD, fresh start development |
| **Production** | `IRON_DEPLOYMENT_MODE=production` | Standard operation, expects PostgreSQL/Redis | Production deployments |
| **ProductionUnconfirmed** | Heuristics (K8s, AWS, Release build) | Warnings on startup, sleeps 10s | Safety check for misconfigured production |

## Detailed Behaviors

### Development Mode

**Trigger:** Set `IRON_DEPLOYMENT_MODE=development` environment variable.

**Behavior:**
- On startup, the runtime **deletes the existing `iron.db` file** if it exists.
- A new, empty `iron.db` is created (handled by SQLite connection).
- This ensures a clean state for every run, which is useful for:
  - Integration testing
  - Rapid iteration where schema changes might conflict with old data
  - Verifying initialization logic

**Example:**
```bash
IRON_DEPLOYMENT_MODE=development cargo run --bin iron_api_server
```

### Pilot Mode

**Trigger:** Default when no production indicators are present.

**Behavior:**
- Uses `iron.db` (SQLite) in the current directory.
- Preserves data between runs.
- Suitable for local development where data persistence is desired.

### Production Mode

**Trigger:** Set `IRON_DEPLOYMENT_MODE=production`.

**Behavior:**
- Assumes a production environment.
- Does NOT clear any databases.
- Should be used with `DATABASE_URL` pointing to PostgreSQL and `REDIS_URL` pointing to Redis.

### ProductionUnconfirmed Mode

**Trigger:** Detected production environment (e.g., Kubernetes, AWS, Release build) but `IRON_DEPLOYMENT_MODE` is NOT set to `production`.

**Behavior:**
- Prints warning messages to stderr.
- Sleeps for 10 seconds to ensure warnings are visible in logs.
- Prevents accidental usage of default settings (like local SQLite) in production environments without explicit confirmation.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `IRON_DEPLOYMENT_MODE` | Explicitly sets the deployment mode (`development`, `production`, `pilot`) | Auto-detect |
| `DATABASE_URL` | Database connection string | `sqlite://./iron.db?mode=rwc` |
| `JWT_SECRET` | Secret for JWT signing | `dev-secret-change-in-production` |
| `KUBERNETES_SERVICE_HOST` | Heuristic for K8s environment | - |
| `AWS_EXECUTION_ENV` | Heuristic for AWS environment | - |
| `DYNO` | Heuristic for Heroku environment | - |
