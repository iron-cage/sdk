# Docker Compose Deployment

**Purpose:** Docker Compose architecture for pilot Control Panel deployment (Package #1).

---

## User Need

Deploy Control Panel (API + Dashboard) for pilot/development use with production-grade database (PostgreSQL) but minimal operational complexity.

## Core Idea

**3-service architecture** (PostgreSQL → Backend API → Frontend nginx) provides production-readiness without Kubernetes complexity.

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
│  Port: 3000 (internal)                  │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  PostgreSQL 16                          │
│  - Token management database            │
│  - Persistent volume                    │
│  Port: 5432 (internal)                  │
└─────────────────────────────────────────┘
```

### Traffic Flow

1. **User → Frontend**: Browser hits `http://localhost:8080`
2. **Frontend → User**: nginx serves Vue static files (index.html, *.js, *.css)
3. **Frontend → Backend**: nginx proxies `/api/*` requests to `http://backend:3000`
4. **Backend → Database**: API connects to `postgresql://postgres:5432/iron_tokens`

## Service Breakdown

### 1. PostgreSQL (Database)

**Image**: `postgres:16-alpine` (official, 250MB compressed)

**Purpose**: Production database for token management, usage tracking, authentication

**Why PostgreSQL**:
- Production-ready (ACID guarantees, concurrent writes)
- SQLite limitations (file locking, no concurrent writes, pilot-only)
- Industry standard for Control Panel deployments

**Key Features**:
- Health check: `pg_isready -U iron_user`
- Persistent volume: `postgres_data:/var/lib/postgresql/data`
- Automatic restart: `restart: unless-stopped`

### 2. Backend API (iron_control_api)

**Build**: Multi-stage Dockerfile (rust:1.75-slim-bookworm → debian:bookworm-slim)

**Purpose**: REST API server for Control Panel backend

**Why Multi-Stage Build**:
- Small runtime image (~50MB vs ~2GB with build tools)
- Security (no build tools in production image)
- Fast deployment (smaller image = faster pulls)

**Key Features**:
- Health check: `curl http://localhost:3000/api/health`
- Non-root user (UID 1000)
- Depends on PostgreSQL (waits for DB health check)

### 3. Frontend (iron_dashboard)

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
| **PostgreSQL instead of SQLite** | Production-ready, concurrent writes, ACID guarantees | SQLite (pilot only, no concurrent writes, file locking issues) |
| **Multi-stage Docker builds** | Smaller images (~50MB runtime vs ~2GB with build tools) | Single-stage (bloated images with build dependencies) |
| **No Redis** | Not currently used in codebase (premature optimization) | Add Redis now (YAGNI violation, extra complexity) |
| **Bridge network** | Simple, predictable DNS, isolated from host | Host network (breaks container portability) |
| **Named volumes** | Persistent data, Docker-managed backups | Bind mounts (host path coupling, permission issues) |
| **Health checks** | Automated recovery, orchestration readiness | Manual monitoring (no automated restart) |
| **Non-root user** | Security best practice (principle of least privilege) | Root user (security risk if container compromised) |

### Simplifications Made

**What's Missing (Intentionally)**:

1. **No Redis**: Code doesnt use Redis yet (WebSocket stub only)
2. **No Secrets Manager**: Uses `.env` file (acceptable for pilot/small deployments)
3. **No Load Balancer**: Single backend instance (scaling requires Kubernetes)
4. **No Monitoring**: No Prometheus/Grafana (add later if needed)
5. **No Log Aggregation**: Uses Docker logs (ELK stack is overkill for pilot)
6. **No HTTPS**: HTTP-only (add Let's Encrypt before production)

## Resource Requirements

### Minimum (Development)

- **CPU**: 2 cores
- **RAM**: 4GB
- **Disk**: 10GB

### Recommended (Production)

- **CPU**: 4 cores
- **RAM**: 8GB
- **Disk**: 50GB (including PostgreSQL data growth)

### Per-Service Breakdown

| Service | CPU | RAM | Disk | Notes |
|---------|-----|-----|------|-------|
| PostgreSQL | 0.5 core | 512MB | 5GB | Grows with usage data |
| Backend | 1 core | 1GB | 100MB | Rust binary is efficient |
| Frontend | 0.1 core | 128MB | 50MB | nginx is lightweight |
| **Total** | **1.6 cores** | **1.6GB** | **5.15GB** | Minimal viable setup |

## Security Considerations

### Implemented

- ✅ Non-root user in containers (UID 1000)
- ✅ Environment variable secrets (not hardcoded)
- ✅ Health checks for automated recovery
- ✅ Internal network (services not exposed to host except frontend)
- ✅ PostgreSQL credentials via env vars
- ✅ nginx security headers (X-Frame-Options, X-Content-Type-Options)

### Missing (Future Work)

- ❌ **TLS/HTTPS**: nginx doesnt have SSL cert (add via Let's Encrypt + certbot)
- ❌ **Secrets Management**: `.env` file not encrypted (use HashiCorp Vault or cloud secrets)
- ❌ **Container Scanning**: No vulnerability scanning in CI/CD (add Trivy/Snyk)
- ❌ **Network Policies**: Docker bridge network not segmented (add in Kubernetes)
- ❌ **Rate Limiting**: nginx doesnt have rate limiting (add `limit_req` directive)

## Deployment Modes Comparison

### Pilot SQLite (Current Development)

- **Database**: SQLite file (`./iron.db`)
- **Concurrency**: Single process only
- **Deployment**: `cargo run --bin iron_control_api_server`
- **Use Case**: Local development, testing
- **Limitation**: File locking prevents concurrent access

### Docker Compose PostgreSQL (This Architecture)

- **Database**: PostgreSQL 16 (container)
- **Concurrency**: 100+ concurrent connections
- **Deployment**: `docker compose up -d`
- **Use Case**: Pilot deployment, small teams (<50 users)
- **Limitation**: Single backend instance (no load balancing)

### Kubernetes (Future Production)

- **Database**: Managed PostgreSQL (RDS, Cloud SQL)
- **Concurrency**: Unlimited (horizontal scaling)
- **Deployment**: `kubectl apply -f manifests/`
- **Use Case**: Production (>100 users, high availability)
- **Limitation**: Operational complexity (requires K8s expertise)

## Scaling Limitations

| Metric | Current Limit | Bottleneck | Solution |
|--------|---------------|------------|----------|
| **Concurrent Users** | ~100 | Single backend instance | Add load balancer + horizontal pod autoscaling (K8s) |
| **Requests/Second** | ~1000 | Backend CPU | Add more backend replicas |
| **Database Connections** | ~100 | PostgreSQL max_connections | Add connection pooler (PgBouncer) |
| **Storage** | ~50GB | PostgreSQL volume size | Increase volume size or add read replicas |

**When to Upgrade to Kubernetes**:
- More than 100 concurrent users
- More than 1000 requests/second
- Need for 99.9%+ uptime (HA requirements)
- Multi-region deployment
- Compliance requirements (audit logging, encryption at rest)

## Trade-offs

### Simplicity vs Production-Readiness

| Aspect | Simplicity Choice | Production Choice | Current Decision |
|--------|-------------------|-------------------|------------------|
| **Database** | SQLite | PostgreSQL cluster | ✅ PostgreSQL (production-ready) |
| **Secrets** | .env file | HashiCorp Vault | ✅ .env (simple) |
| **Monitoring** | Docker logs | Prometheus + Grafana | ✅ Docker logs (simple) |
| **HTTPS** | HTTP only | Let's Encrypt + certbot | ✅ HTTP (simple, add HTTPS before prod) |
| **Load Balancing** | Single instance | Multiple instances + LB | ✅ Single instance (simple) |
| **Orchestration** | Docker Compose | Kubernetes | ✅ Docker Compose (simple) |

**Philosophy**: Start simple (Docker Compose), add complexity only when needed (metrics show bottlenecks).

## Relationship to Existing Documentation

- **Extends**: [001_package_model.md](001_package_model.md) - Implements Package #1 deployment
- **Complements**: [deployment_guide.md](../deployment_guide.md) - This is pilot deployment, guide describes future K8s
- **Referenced By**: [getting_started.md](../getting_started.md) - Quickstart links here for architecture details

## Operational Procedures

For step-by-step deployment instructions, see:
- [Getting Started Guide](../getting_started.md) § Deploy Control Panel - 5-minute quickstart
- [Deployment Guide](../deployment_guide.md) § Pilot Deployment - Complete operational procedures (coming soon)

---

*Related: [001_package_model.md](001_package_model.md) | [003_distribution_strategy.md](003_distribution_strategy.md) | [004_scaling_patterns.md](004_scaling_patterns.md)*
