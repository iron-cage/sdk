# Docker Compose Deployment

**Purpose:** Docker Compose architecture for pilot Control Panel deployment (Package #1).

---

## User Need

Deploy Control Panel (API + Dashboard) for pilot/development use with minimal operational complexity and persistent data storage.

## Core Idea

**2-service architecture** (Backend API + Frontend nginx) with persistent SQLite storage provides production-ready deployment without database server complexity.

## Architecture Overview

```
┌─────────────────────────────────────────┐
│         Browser (localhost:8080)        │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  Frontend (nginx)                       │
│  - Serve Vue static files               │
│  - Proxy /api/* → backend:3000          │
│  Port: 80 (mapped to 8080)              │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  Backend (iron_control_api_server)      │
│  - REST API (50 endpoints)              │
│  - JWT authentication                   │
│  - SQLite database                      │
│  Port: 3000 (internal)                  │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  Persistent Volume (sqlite_data)        │
│  - /app/data/iron.db                    │
│  - Docker-managed volume                │
└─────────────────────────────────────────┘
```

### Traffic Flow

1. **User → Frontend**: Browser hits `http://localhost:8080`
2. **Frontend → User**: nginx serves Vue static files (index.html, *.js, *.css)
3. **Frontend → Backend**: nginx proxies `/api/*` requests to `http://backend:3000`
4. **Backend → SQLite**: API connects to `sqlite:///app/data/iron.db?mode=rwc`

## Service Breakdown

### 1. Backend API (iron_control_api)

**Build**: Multi-stage Dockerfile (rust:1-slim-bookworm → debian:bookworm-slim)

**Purpose**: REST API server for Control Panel backend with embedded SQLite database

**Why Multi-Stage Build**:
- Small runtime image (~50MB vs ~2GB with build tools)
- Security (no build tools in production image)
- Fast deployment (smaller image = faster pulls)

**Key Features**:
- Health check: `curl http://localhost:3000/api/health`
- Non-root user (UID 1000)
- Persistent SQLite volume: `sqlite_data:/app/data`
- Automatic restart: `restart: unless-stopped`

**Database Configuration**:
- **Type**: SQLite (embedded, ACID-compliant)
- **Location**: `/app/data/iron.db` (persistent Docker volume)
- **Connection**: `sqlite:///app/data/iron.db?mode=rwc`
- **Persistence**: Data survives container restarts via named volume

### 2. Frontend (iron_dashboard)

**Build**: Multi-stage Dockerfile (node:20-alpine → nginx:1.27-alpine)

**Purpose**: Serve Vue application and proxy API requests

**Why nginx Reverse Proxy**:
- Single entry point (eliminates CORS complexity)
- Static file serving (efficient for Vue SPA)
- API proxying (frontend doesn't know backend location)
- SSL termination point (future HTTPS support)

**Key Features**:
- Serves Vue SPA with fallback routing (try_files $uri /index.html)
- Proxies /api/* to backend:3000
- WebSocket support (/ws endpoint)
- Security headers (X-Frame-Options, X-Content-Type-Options)
- Static asset caching (1 year for immutable assets)

## Design Decisions

### Why This Architecture?

| Decision | Rationale | Alternative Rejected |
|----------|-----------|---------------------|
| **nginx as reverse proxy** | Single entry point, SSL termination, static file serving | Direct backend exposure (CORS complexity, no SSL, no static serving) |
| **SQLite for all deployment modes** | Embedded database, zero configuration, ACID guarantees, sufficient for pilot scale | PostgreSQL (requires separate service, more complex, overkill for pilot workloads) |
| **Persistent Docker volume** | Data survives container restarts, Docker-managed backups | Ephemeral storage (data loss on restart) or bind mounts (permission issues) |
| **Multi-stage Docker builds** | Smaller images (~50MB runtime vs ~2GB with build tools) | Single-stage (bloated images with build dependencies) |
| **No Redis** | Not currently used in codebase (premature optimization) | Add Redis now (YAGNI violation, extra complexity) |
| **Bridge network** | Simple, predictable DNS, isolated from host | Host network (breaks container portability) |
| **Named volumes** | Persistent data, Docker-managed backups | Bind mounts (host path coupling, permission issues) |
| **Health checks** | Automated recovery, orchestration readiness | Manual monitoring (no automated restart) |
| **Non-root user** | Security best practice (principle of least privilege) | Root user (security risk if container compromised) |

### SQLite Design Choice

**Why SQLite for Pilot Deployment**:

| Aspect | SQLite Advantage | PostgreSQL Disadvantage |
|--------|------------------|------------------------|
| **Setup Complexity** | Zero configuration (embedded) | Requires separate service, credentials, health checks |
| **Resource Usage** | ~50MB total (no DB server) | +250MB for postgres container, +512MB RAM |
| **Operational Burden** | No DB server to manage | Database backups, connection pooling, version upgrades |
| **Deployment Simplicity** | 2 services instead of 3 | Additional service dependency, startup ordering |
| **Pilot Scale Sufficiency** | Handles <100 concurrent users | Overkill for pilot workloads |
| **ACID Guarantees** | Full ACID compliance | Same (no advantage) |
| **Data Persistence** | Docker volume (same as PostgreSQL) | Same (no advantage) |

**When SQLite is Sufficient**:
- Pilot/development deployments
- <100 concurrent users
- <1000 requests/second
- Small teams (<50 users)
- Single backend instance

**When to Migrate to PostgreSQL** (Future):
- >100 concurrent users
- Need for horizontal backend scaling (multiple instances)
- >10GB database size
- Replication requirements
- Advanced query optimization needs

### Simplifications Made

**What's Missing (Intentionally)**:

1. **No PostgreSQL**: SQLite is sufficient for pilot scale (add PostgreSQL when scaling requirements emerge)
2. **No Redis**: Code doesnt use Redis yet (WebSocket stub only)
3. **No Secrets Manager**: Uses `.env` file (acceptable for pilot/small deployments)
4. **No Load Balancer**: Single backend instance (scaling requires Kubernetes)
5. **No Monitoring**: No Prometheus/Grafana (add later if needed)
6. **No Log Aggregation**: Uses Docker logs (ELK stack is overkill for pilot)
7. **No HTTPS**: HTTP-only (add Let's Encrypt before production)

## Resource Requirements

### Minimum (Development)

- **CPU**: 1 core
- **RAM**: 2GB
- **Disk**: 5GB

### Recommended (Pilot Production)

- **CPU**: 2 cores
- **RAM**: 4GB
- **Disk**: 20GB (including SQLite data growth)

### Per-Service Breakdown

| Service | CPU | RAM | Disk | Notes |
|---------|-----|-----|------|-------|
| Backend | 1 core | 1GB | 5GB | Includes SQLite database storage |
| Frontend | 0.1 core | 128MB | 50MB | nginx is lightweight |
| **Total** | **1.1 cores** | **1.1GB** | **5GB** | Minimal viable setup |

**Comparison to PostgreSQL Architecture** (Not Implemented):
- **CPU Savings**: -0.5 core (no postgres process)
- **RAM Savings**: -512MB (no database server)
- **Disk Savings**: Minimal (SQLite file vs postgres data dir)
- **Complexity Savings**: -1 service (simpler deployment)

## Security Considerations

### Implemented

- ✅ Non-root user in containers (UID 1000)
- ✅ Environment variable secrets (not hardcoded)
- ✅ Health checks for automated recovery
- ✅ Internal network (backend not exposed to host, only frontend)
- ✅ nginx security headers (X-Frame-Options, X-Content-Type-Options)
- ✅ SQLite database in persistent volume (data integrity)

### Missing (Future Work)

- ❌ **TLS/HTTPS**: nginx doesnt have SSL cert (add via Let's Encrypt + certbot)
- ❌ **Secrets Management**: `.env` file not encrypted (use HashiCorp Vault or cloud secrets)
- ❌ **Container Scanning**: No vulnerability scanning in CI/CD (add Trivy/Snyk)
- ❌ **Network Policies**: Docker bridge network not segmented (add in Kubernetes)
- ❌ **Rate Limiting**: nginx doesnt have rate limiting (add `limit_req` directive)
- ❌ **SQLite Encryption**: Database file not encrypted at rest (add SQLCipher extension if needed)

## Deployment Modes Comparison

### Development Mode (Docker Compose - Clean State)

- **Database**: SQLite in bind mount (`./-dev_data/iron.db`)
- **Behavior**: Database wiped automatically on every startup
- **Deployment**: `docker compose -f docker-compose.dev.yml up`
- **Use Case**: Testing, debugging, clean state verification
- **Environment**: `IRON_DEPLOYMENT_MODE=development`
- **Secrets**: Hardcoded development secrets (insecure, for testing only)
- **Ports**: Frontend :5173 (Vite HMR), Backend :3000
- **Build**: Development build with source mounting (hot reload)
- **Persistence**: None (data deleted on startup)
- **⚠️ WARNING**: Never use in production - all data is lost on restart

### Production Mode (Docker Compose - Data Persistence)

- **Database**: SQLite in Docker volume (`sqlite_data:/app/data/iron.db`)
- **Behavior**: Database persists across restarts
- **Deployment**: `docker compose up -d`
- **Use Case**: Production deployment with data retention
- **Environment**: `IRON_DEPLOYMENT_MODE=production`
- **Secrets**: Required via `.env` file (secure)
- **Ports**: Frontend :8080 (nginx production build)
- **Build**: Production build with optimizations
- **Persistence**: Full (survives container restarts)
- **Data Safety**: Backed by Docker named volume

### Native SQLite (Local Development - Deprecated)

- **Database**: SQLite file (`./iron.db`)
- **Concurrency**: Single process only
- **Deployment**: `cargo run --bin iron_control_api_server`
- **Use Case**: Local development (superseded by development mode)
- **Limitation**: File locking prevents concurrent backend instances
- **Note**: Use development mode instead for better experience

### Quick Comparison: Development vs Production

| Aspect | Development Mode | Production Mode |
|--------|------------------|-----------------|
| **Command** | `docker compose -f docker-compose.dev.yml up` | `docker compose up -d` |
| **Data Persistence** | ❌ Wiped on startup | ✅ Persists forever |
| **Configuration** | Zero config (hardcoded) | Requires .env file |
| **Secrets** | Insecure (hardcoded) | Secure (from .env) |
| **Use Case** | Testing, debugging | Production |
| **Frontend** | Vite dev server (:5173) | nginx production (:8080) |
| **Backend** | Hot reload enabled | Pre-built optimized |

### Kubernetes + PostgreSQL (Future Production)

- **Database**: Managed PostgreSQL (RDS, Cloud SQL)
- **Concurrency**: Unlimited (horizontal pod autoscaling)
- **Deployment**: `kubectl apply -f manifests/`
- **Use Case**: Production (>100 users, high availability)
- **Limitation**: Operational complexity (requires K8s expertise, PostgreSQL expertise)
- **Migration Required**: Code refactoring to support PostgreSQL (currently SQLite-only)

**Current Status**: Backend codebase is SQLite-only (68 hardcoded `Pool<Sqlite>` references). PostgreSQL support requires code refactoring.

## Scaling Limitations

| Metric | Current Limit | Bottleneck | Solution |
|--------|---------------|------------|----------|
| **Concurrent Users** | ~100 | SQLite + single backend instance | Migrate to PostgreSQL + add load balancer (requires code refactoring) |
| **Requests/Second** | ~500 | Single backend instance | Add backend replicas (requires PostgreSQL migration first) |
| **Database Connections** | ~100 | SQLite serialized writes | Migrate to PostgreSQL with connection pooler (PgBouncer) |
| **Storage** | ~10GB practical | SQLite performance degrades >10GB | Migrate to PostgreSQL or partition data |
| **Write Throughput** | ~1000 writes/sec | SQLite serialized transactions | Migrate to PostgreSQL for concurrent writes |

**When to Upgrade Architecture**:
- More than 100 concurrent users
- More than 500 requests/second sustained
- Database size exceeds 5GB
- Need for multiple backend instances (horizontal scaling)
- Need for 99.9%+ uptime (HA requirements)
- Multi-region deployment

**Migration Path**: Pilot SQLite → PostgreSQL migration (requires backend code refactoring) → Kubernetes orchestration

## Trade-offs

### Simplicity vs Production-Readiness

| Aspect | Simplicity Choice | Production Choice | Current Decision |
|--------|-------------------|-------------------|------------------|
| **Database** | SQLite (embedded) | PostgreSQL cluster | ✅ SQLite (simple, sufficient for pilot) |
| **Secrets** | .env file | HashiCorp Vault | ✅ .env (simple) |
| **Monitoring** | Docker logs | Prometheus + Grafana | ✅ Docker logs (simple) |
| **HTTPS** | HTTP only | Let's Encrypt + certbot | ✅ HTTP (simple, add HTTPS before prod) |
| **Load Balancing** | Single instance | Multiple instances + LB | ✅ Single instance (simple) |
| **Orchestration** | Docker Compose | Kubernetes | ✅ Docker Compose (simple) |

**Philosophy**: Start simple (Docker Compose + SQLite), migrate only when metrics show bottlenecks (>100 users, >5GB data).

## Relationship to Existing Documentation

- **Extends**: [001_package_model.md](001_package_model.md) - Implements Package #1 deployment
- **Complements**: [deployment_guide.md](../deployment_guide.md) - This is pilot deployment, guide describes operational procedures
- **Referenced By**: [getting_started.md](../getting_started.md) - Quickstart links here for architecture details

## Operational Procedures

For step-by-step deployment instructions, see:
- [Getting Started Guide](../getting_started.md) § Deploy Control Panel - 5-minute quickstart
- [Deployment Guide](../deployment_guide.md) § Pilot Deployment - Complete operational procedures

## Future Considerations

### PostgreSQL Migration (When Needed)

**Code Changes Required**:
1. Replace `Pool<Sqlite>` with generic database pool in all route states (13 files)
2. Replace SQLite-specific queries (`sqlite_master` checks) with database-agnostic migrations
3. Update `iron_token_manager` module (55 SQLite references)
4. Add PostgreSQL feature flag to Cargo.toml
5. Create database abstraction layer

**Infrastructure Changes**:
1. Add postgres service to docker-compose.yml
2. Add PostgreSQL credentials to .env
3. Update backend DATABASE_URL to PostgreSQL
4. Add database migration tooling
5. Create backup/restore procedures for PostgreSQL

**Estimated Effort**: 3-5 days of development + testing

---

*Related: [001_package_model.md](001_package_model.md) | [003_distribution_strategy.md](003_distribution_strategy.md) | [004_scaling_patterns.md](004_scaling_patterns.md)*
