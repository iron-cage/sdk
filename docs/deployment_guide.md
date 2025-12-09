# Iron Cage Runtime - Deployment Guide

### Scope

**Responsibility:** Installation procedures and deployment configuration for all Iron Cage environments.

**In Scope:**
- 3 deployment modes (Local, On-Premise, SaaS)
- Installation steps and infrastructure requirements
- Docker Compose and Kubernetes manifests
- Operational procedures

**Out of Scope:**
- System architecture design (see `architecture.md`)
- Product specifications (see `/spec/`)
- Source code (see `/runtime/src/`)

### Deployment Mode Note

**⚠️ DEPLOYMENT SCOPE:** This guide describes **future production deployment architecture** with 3 modes (Local, On-Premise, SaaS), centralized iron_cage-runtime, K8s orchestration, PostgreSQL + Redis infrastructure.

**For current pilot implementation deployment** (single-process localhost architecture), see [docs/deployment_packages.md](../../docs/deployment_packages.md) § Deployment Modes.

**Relationship:**
- **This Guide:** Future production deployment options (Local/On-Premise/SaaS)
- **Pilot Mode (Current):** Single-process localhost deployment for conference demo
- **Production Mode (Future):** Will use deployment architecture described in this guide

**Key Differences:**
- **This Guide:** Centralized runtime, PostgreSQL + Redis, Docker Compose/K8s, Local and Server execution
- **Current Pilot:** Distributed packages, SQLite (iron_state), single process, localhost only

## Document Purpose

**Responsibility:** Installation procedures, deployment modes, infrastructure configuration, and operational setup for all environments (development, on-premise, SaaS).

**Audience:** DevOps engineers, system administrators, infrastructure teams, operations staff

---

## Table of Contents

1. [Deployment Modes Overview](#1-deployment-modes-overview)
2. [Local Development Deployment](#2-local-development-deployment)
3. [On-Premise Enterprise Deployment](#3-on-premise-enterprise-deployment)
4. [SaaS Multi-Tenant Deployment](#4-saas-multi-tenant-deployment)
5. [System Components](#5-system-components)
6. [Execution Models](#6-execution-models)
7. [Infrastructure Requirements](#7-infrastructure-requirements)
8. [Network Architecture](#8-network-architecture)

---

## 1. Deployment Modes Overview

Iron Cage supports three deployment modes, each optimized for different use cases:

| Deployment Mode | Use Case | Infrastructure | Multi-Tenancy | Operational Complexity |
|-----------------|----------|----------------|---------------|------------------------|
| **Local** | Development, testing | Developer's laptop/desktop | Single user | Low (Docker Compose) |
| **On-Premise** | Enterprise production | Customer's data center | Single tenant | Medium (Kubernetes) |
| **SaaS** | Managed service | Cloud (AWS/GCP/Azure) | Multi-tenant | High (K8s + isolation) |

### Decision Matrix

**Choose Local when:**
- Developing/debugging Iron Cage itself
- Testing agent integrations before production
- Running occasional batch workloads
- No 24/7 uptime requirements

**Choose On-Premise when:**
- Enterprise requires data residency (GDPR, HIPAA)
- High security/compliance needs (air-gapped networks)
- Need full infrastructure control
- Budget for dedicated ops team

**Choose SaaS when:**
- Fast time-to-market (< 1 week)
- No ops team or infrastructure expertise
- Variable workloads (scale 0-1000+ agents)
- Prefer OpEx over CapEx

---

## 2. Local Development Deployment

### 2.1 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    DEVELOPER LAPTOP                          │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Docker Compose Stack                       │ │
│  │                                                          │ │
│  │  ┌──────────────────────────────────────────────────┐  │ │
│  │  │ iron_cage-runtime (Port 8080)                     │  │ │
│  │  │ - REST API (Axum)                                 │  │ │
│  │  │ - gRPC API (Tonic)                                │  │ │
│  │  │ - WebSocket (Axum)                                │  │ │
│  │  └───────────────┬──────────────────────────────────┘  │ │
│  │                  │                                       │ │
│  │  ┌───────────────┴──────────┬────────────────────────┐  │ │
│  │  │                           │                        │  │ │
│  │  │ redis:7-alpine            │ postgres:15-alpine     │  │ │
│  │  │ (Port 6379)               │ (Port 5432)            │  │ │
│  │  │ - Ephemeral state         │ - Durable state        │  │ │
│  │  │ - Checkpoints             │ - Audit logs           │  │ │
│  │  └───────────────────────────┴────────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Python Agent (your_agent.py)                          │   │
│  │ - Runs directly on laptop (not in Docker)            │   │
│  │ - Calls http://localhost:8080 REST API               │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Installation Steps

### Prerequisites
- Docker 24+ with Docker Compose
- 4 GB RAM available
- 10 GB disk space
- Ports 8080, 6379, 5432 available

### Step 1: Download docker-compose.yaml

```yaml
# docker-compose.yaml
version: '3.8'

services:
  iron_cage-runtime:
    image: iron_cage/runtime:latest
    container_name: iron_cage-runtime
    ports:
      - "8080:8080"   # REST API + WebSocket
      - "50051:50051" # gRPC API
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
      - DATABASE_URL=postgres://postgres:postgres@postgres:5432/iron_cage
      - JWT_SECRET=dev-secret-change-in-production
      - MAX_AGENTS_PER_INSTANCE=100
      - CIRCUIT_BREAKER_THRESHOLD=5
      - CIRCUIT_BREAKER_TIMEOUT=60s
    depends_on:
      - redis
      - postgres
    volumes:
      - ./config:/etc/iron_cage
      - ./logs:/var/log/iron_cage
    restart: unless-stopped
    networks:
      - iron_cage-network

  redis:
    image: redis:7-alpine
    container_name: iron_cage-redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    restart: unless-stopped
    networks:
      - iron_cage-network

  postgres:
    image: postgres:15-alpine
    container_name: iron_cage-postgres
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=iron_cage
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d
    restart: unless-stopped
    networks:
      - iron_cage-network

  # Optional: Observability stack
  prometheus:
    image: prom/prometheus:latest
    container_name: iron_cage-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    networks:
      - iron_cage-network

  grafana:
    image: grafana/grafana:latest
    container_name: iron_cage-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana
    networks:
      - iron_cage-network

volumes:
  redis-data:
  postgres-data:
  prometheus-data:
  grafana-data:

networks:
  iron_cage-network:
    driver: bridge
```

### Step 2: Start Services

```bash
# Download and start Iron Cage
docker-compose up -d

# Verify all containers are running
docker-compose ps

# Check logs
docker-compose logs -f iron_cage-runtime

# Health check
curl http://localhost:8080/health
# Expected: {"status":"healthy","version":"0.1.0","uptime_seconds":42}
```

### Step 3: Configure Python Agent

```python
# your_agent.py
from iron_cage import IronCageClient, SafetyConfig

# Initialize client
client = IronCageClient(
  api_url="http://localhost:8080",
  api_key="dev-key",  # For local dev, weak key is fine
)

# Register agent
agent_id = client.register_agent(
  name="my-agent",
  runtime="python",
  version="3.11",
  config=SafetyConfig(
    pii_detection=True,
    cost_limit_usd=10.0,
    max_tokens_per_request=4096,
  )
)

# Start agent
client.start_agent(agent_id)

# Your agent logic here
while True:
  task = client.get_next_task(agent_id)
  result = process_task(task)
  client.submit_result(agent_id, result)
```

### Step 4: Verify Deployment

```bash
# Check agent registration
curl http://localhost:8080/api/v1/agents

# View metrics
open http://localhost:9090  # Prometheus
open http://localhost:3000  # Grafana (admin/admin)

# Stop services
docker-compose down
```

### 2.3 Local Development Best Practices

- **Data Persistence:** Docker volumes persist data across restarts
- **Configuration:** Mount `./config` for custom guardrail rules
- **Hot Reload:** For Rust development, build outside Docker and mount binary
- **Testing:** Use separate compose file for integration tests (`docker-compose.test.yaml`)

---

## 3. On-Premise Enterprise Deployment

### 3.1 Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                    KUBERNETES CLUSTER                             │
│                  (Customer Data Center)                           │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                      Namespace: iron_cage                   │  │
│  │                                                             │  │
│  │  ┌─────────────────────────────────────────────────────┐   │  │
│  │  │ Ingress (nginx-ingress)                              │   │  │
│  │  │ - TLS termination (mTLS for API)                     │   │  │
│  │  │ - Rate limiting (10k req/s)                          │   │  │
│  │  └───────────────────┬──────────────────────────────────┘   │  │
│  │                      │                                       │  │
│  │  ┌───────────────────┴──────────────────────────────────┐   │  │
│  │  │ iron_cage-runtime (Deployment)                        │   │  │
│  │  │ - Replicas: 3 (auto-scale 3-10)                      │   │  │
│  │  │ - Resources: 2 CPU, 4 GB RAM per pod                 │   │  │
│  │  │ - Liveness/Readiness probes                          │   │  │
│  │  └───────────────────┬──────────────────────────────────┘   │  │
│  │                      │                                       │  │
│  │  ┌───────────────────┴─────────────┬──────────────────────┐ │  │
│  │  │                                 │                      │ │  │
│  │  │ redis-ha (StatefulSet)          │ postgres-ha          │ │  │
│  │  │ - Master + 2 replicas           │ (StatefulSet)        │ │  │
│  │  │ - Sentinel for failover         │ - Primary + standby  │ │  │
│  │  └─────────────────────────────────┴──────────────────────┘ │  │
│  │                                                             │  │
│  │  ┌─────────────────────────────────────────────────────┐   │  │
│  │  │ Observability (Namespace: monitoring)                │   │  │
│  │  │ - Prometheus (metrics)                               │   │  │
│  │  │ - Grafana (control panels)                               │   │  │
│  │  │ - Jaeger (distributed tracing)                       │   │  │
│  │  └─────────────────────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

### 3.2 Installation Steps

### Prerequisites
- Kubernetes 1.24+ cluster
- kubectl configured with admin access
- Helm 3.10+
- Storage class for persistent volumes
- Load balancer or Ingress controller

### Step 1: Create Namespace

```bash
kubectl create namespace iron_cage
kubectl create namespace monitoring  # For observability stack
```

### Step 2: Deploy PostgreSQL (StatefulSet)

```yaml
# postgres-statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: iron_cage
spec:
  serviceName: postgres
  replicas: 2  # Primary + standby
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        ports:
        - containerPort: 5432
          name: postgres
        env:
        - name: POSTGRES_USER
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: username
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        - name: POSTGRES_DB
          value: iron_cage
        - name: PGDATA
          value: /var/lib/postgresql/data/pgdata
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            cpu: "1"
            memory: "2Gi"
          limits:
            cpu: "2"
            memory: "4Gi"
        livenessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 5
          periodSeconds: 5
  volumeClaimTemplates:
  - metadata:
      name: postgres-storage
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 100Gi
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: iron_cage
spec:
  type: ClusterIP
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
---
apiVersion: v1
kind: Secret
metadata:
  name: postgres-secret
  namespace: iron_cage
type: Opaque
stringData:
  username: postgres
  password: CHANGE-THIS-IN-PRODUCTION  # Use HashiCorp Vault or AWS Secrets Manager
```

### Step 3: Deploy Redis (StatefulSet with Sentinel)

```yaml
# redis-statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: redis
  namespace: iron_cage
spec:
  serviceName: redis
  replicas: 3  # 1 master + 2 replicas
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        ports:
        - containerPort: 6379
          name: redis
        command:
        - redis-server
        - "--appendonly"
        - "yes"
        - "--requirepass"
        - "$(REDIS_PASSWORD)"
        env:
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: redis-secret
              key: password
        volumeMounts:
        - name: redis-storage
          mountPath: /data
        resources:
          requests:
            cpu: "500m"
            memory: "1Gi"
          limits:
            cpu: "1"
            memory: "2Gi"
  volumeClaimTemplates:
  - metadata:
      name: redis-storage
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 20Gi
---
apiVersion: v1
kind: Service
metadata:
  name: redis
  namespace: iron_cage
spec:
  type: ClusterIP
  selector:
    app: redis
  ports:
  - port: 6379
    targetPort: 6379
---
apiVersion: v1
kind: Secret
metadata:
  name: redis-secret
  namespace: iron_cage
type: Opaque
stringData:
  password: CHANGE-THIS-IN-PRODUCTION
```

### Step 4: Deploy Iron Cage Runtime

```yaml
# iron_cage-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: iron_cage-runtime
  namespace: iron_cage
spec:
  replicas: 3
  selector:
    matchLabels:
      app: iron_cage-runtime
  template:
    metadata:
      labels:
        app: iron_cage-runtime
    spec:
      containers:
      - name: iron_cage-runtime
        image: iron_cage/runtime:0.1.0
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 50051
          name: grpc
        env:
        - name: RUST_LOG
          value: "info"
        - name: REDIS_URL
          value: "redis://:$(REDIS_PASSWORD)@redis:6379"
        - name: DATABASE_URL
          value: "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@postgres:5432/iron_cage"
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: iron_cage-secret
              key: jwt-secret
        - name: MAX_AGENTS_PER_INSTANCE
          value: "1000"
        - name: CIRCUIT_BREAKER_THRESHOLD
          value: "10"
        - name: CIRCUIT_BREAKER_TIMEOUT
          value: "120s"
        envFrom:
        - secretRef:
            name: redis-secret
        - secretRef:
            name: postgres-secret
        volumeMounts:
        - name: config
          mountPath: /etc/iron_cage
        resources:
          requests:
            cpu: "2"
            memory: "4Gi"
          limits:
            cpu: "4"
            memory: "8Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: iron_cage-config
---
apiVersion: v1
kind: Service
metadata:
  name: iron_cage-runtime
  namespace: iron_cage
spec:
  type: ClusterIP
  selector:
    app: iron_cage-runtime
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: grpc
    port: 50051
    targetPort: 50051
---
apiVersion: v1
kind: Secret
metadata:
  name: iron_cage-secret
  namespace: iron_cage
type: Opaque
stringData:
  jwt-secret: GENERATE-256-BIT-SECRET-HERE
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: iron_cage-config
  namespace: iron_cage
data:
  guardrails.yaml: |
    pii_detection:
      enabled: true
      patterns:
        - ssn
        - credit_card
        - email
        - phone_number
    cost_limits:
      default_per_agent_usd: 100.0
      max_per_request_usd: 10.0
    circuit_breakers:
      default_threshold: 10
      default_timeout_seconds: 120
```

### Step 5: Deploy Ingress

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: iron_cage-ingress
  namespace: iron_cage
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/backend-protocol: "GRPC"
    nginx.ingress.kubernetes.io/proxy-body-size: "50m"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - iron_cage.your-company.com
    secretName: iron_cage-tls
  rules:
  - host: iron_cage.your-company.com
    http:
      paths:
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: iron_cage-runtime
            port:
              number: 8080
      - path: /grpc
        pathType: Prefix
        backend:
          service:
            name: iron_cage-runtime
            port:
              number: 50051
```

### Step 6: Deploy Autoscaling

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: iron_cage-runtime-hpa
  namespace: iron_cage
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: iron_cage-runtime
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Pods
        value: 1
        periodSeconds: 120
```

### Step 7: Apply All Manifests

```bash
# Apply in dependency order
kubectl apply -f postgres-statefulset.yaml
kubectl apply -f redis-statefulset.yaml
kubectl apply -f iron_cage-deployment.yaml
kubectl apply -f ingress.yaml
kubectl apply -f hpa.yaml

# Wait for all pods to be ready
kubectl wait --for=condition=ready pod -l app=postgres -n iron_cage --timeout=300s
kubectl wait --for=condition=ready pod -l app=redis -n iron_cage --timeout=300s
kubectl wait --for=condition=ready pod -l app=iron_cage-runtime -n iron_cage --timeout=300s

# Verify deployment
kubectl get pods -n iron_cage
kubectl get svc -n iron_cage
kubectl get ingress -n iron_cage

# Check logs
kubectl logs -f deployment/iron_cage-runtime -n iron_cage
```

### 3.3 On-Premise Best Practices

- **High Availability:** Run 3+ replicas across availability zones
- **Backups:** Configure automated PostgreSQL backups (pg_dump + WAL archiving)
- **Secrets Management:** Use HashiCorp Vault or AWS Secrets Manager (not K8s Secrets)
- **mTLS:** Enable mutual TLS for API authentication in production
- **Resource Limits:** Set proper CPU/memory limits to prevent OOMKill
- **Monitoring:** Deploy Prometheus/Grafana in separate namespace
- **Upgrades:** Use rolling updates with `maxUnavailable: 1`

---

## 4. SaaS Multi-Tenant Deployment

### 4.1 Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│                      CLOUD PROVIDER (AWS/GCP/Azure)                 │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                   Application Load Balancer                   │  │
│  │  - TLS termination                                            │  │
│  │  - DDoS protection (AWS Shield / Cloudflare)                  │  │
│  │  - Rate limiting per tenant (1000 req/s)                      │  │
│  └────────────────────────┬─────────────────────────────────────┘  │
│                           │                                         │
│  ┌────────────────────────┴─────────────────────────────────────┐  │
│  │          Kubernetes Cluster (Multi-AZ)                        │  │
│  │                                                               │  │
│  │  ┌────────────────────────────────────────────────────────┐  │  │
│  │  │ iron_cage-runtime (Deployment)                          │  │  │
│  │  │ - Replicas: 10-50 (auto-scale based on load)           │  │  │
│  │  │ - Tenant isolation via JWT claims                       │  │  │
│  │  │ - Resource quotas per tenant                            │  │  │
│  │  └───────────────────┬────────────────────────────────────┘  │  │
│  │                      │                                         │  │
│  │  ┌───────────────────┴────────────┬────────────────────────┐  │  │
│  │  │                                │                        │  │  │
│  │  │ ElastiCache (Redis)            │ RDS PostgreSQL         │  │  │
│  │  │ - Cluster mode enabled         │ - Multi-AZ             │  │  │
│  │  │ - 3-node cluster               │ - Read replicas (3)    │  │  │
│  │  │ - Automatic failover           │ - Automated backups    │  │  │
│  │  └────────────────────────────────┴────────────────────────┘  │  │
│  │                                                               │  │
│  │  ┌────────────────────────────────────────────────────────┐  │  │
│  │  │ Observability                                           │  │  │
│  │  │ - CloudWatch / Stackdriver (metrics, logs)             │  │  │
│  │  │ - Jaeger (distributed tracing)                         │  │  │
│  │  │ - PagerDuty (alerting)                                 │  │  │
│  │  └────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

### 4.2 Tenant Isolation Strategy

Iron Cage uses **namespace-based isolation** for multi-tenancy:

| Isolation Layer | Mechanism | Example |
|-----------------|-----------|---------|
| **Network** | Virtual Private Cloud (VPC) per tier | `vpc-tier-enterprise`, `vpc-tier-startup` |
| **Compute** | Kubernetes namespace per tenant tier | `namespace-tier-enterprise` |
| **Data (Redis)** | Key prefix per tenant | `tenant:acme-corp:agent:123` |
| **Data (PostgreSQL)** | Schema per tenant | `CREATE SCHEMA tenant_acme_corp` |
| **API** | JWT claim `tenant_id` validated on every request | `{"tenant_id": "acme-corp"}` |

### 4.3 Multi-Tenant Configuration

```yaml
# Tenant-aware deployment configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: iron_cage-config
  namespace: iron_cage-saas
data:
  tenants.yaml: |
    tenants:
      - id: acme-corp
        tier: enterprise
        quotas:
          max_agents: 1000
          max_requests_per_second: 500
          cost_limit_usd_per_month: 50000
        features:
          pii_detection: true
          custom_guardrails: true
          dedicated_support: true
          sla_uptime: 99.99

      - id: startup-xyz
        tier: startup
        quotas:
          max_agents: 50
          max_requests_per_second: 50
          cost_limit_usd_per_month: 1000
        features:
          pii_detection: true
          custom_guardrails: false
          dedicated_support: false
          sla_uptime: 99.9
```

### 4.4 SaaS Deployment Best Practices

- **Tenant Isolation:** Strict namespace isolation in Redis/PostgreSQL
- **Rate Limiting:** Per-tenant quotas enforced at API gateway
- **Cost Tracking:** Real-time tracking per tenant (bill monthly)
- **Zero-Downtime Deploys:** Blue/green deployments for runtime updates
- **Backups:** Automated daily backups with 30-day retention
- **Compliance:** SOC 2 Type II, GDPR, HIPAA compliance
- **Scaling:** Auto-scale based on tenant growth (10-50 pods)

---

## 5. System Components

All deployment modes share these core components:

| Component | Purpose | Technology | Scaling Strategy |
|-----------|---------|------------|------------------|
| **iron_cage-runtime** | Core orchestration engine | Rust (Tokio, Axum) | Horizontal (stateless) |
| **redis** | Ephemeral state, checkpoints | Redis 7 | Master-replica + Sentinel |
| **postgres** | Durable state, audit logs | PostgreSQL 15 | Primary-standby replication |
| **prometheus** | Metrics collection | Prometheus | Single instance (federated) |
| **grafana** | Metrics visualization | Grafana | Single instance |
| **jaeger** | Distributed tracing | Jaeger | Collector + query service |

### 5.1 Component Responsibilities

**iron_cage-runtime:**
- agent management management (Start/Stop/Pause/Resume)
- safety cutoff execution
- privacy protection (98% accuracy)
- Cost tracking (real-time)
- WebSocket streaming
- gRPC bidirectional streaming

**redis:**
- Agent ephemeral state (current status, active connections)
- safety cutoff state (open/closed/half-open)
- Checkpoints for pause/resume (up to 1 GB per agent)
- Cache for hot data (LLM responses, guardrail rules)

**postgres:**
- Agent metadata (name, owner, config)
- Audit logs (every API call, compliance requirement)
- Cost history (billing, analytics)
- User accounts and permissions (RBAC)

**prometheus:**
- Metrics: API latency, agent count, circuit breaker trips, privacy protections
- Alerting: High error rate, pod restarts, disk full

**grafana:**
- Control Panels: System health, per-tenant usage, cost breakdown
- Visualization: Time-series graphs, heatmaps

**jaeger:**
- Distributed tracing: End-to-end request flow (API → runtime → LLM)
- Performance debugging: Identify bottlenecks

---

## 6. Execution Models

Iron Cage supports two execution models depending on where the Python agent runs:

### 6.1 Local Execution (Client-Side, Most Common)

**Agent runs on user's infrastructure, calls Iron Cage API for safety/monitoring:**

```
┌──────────────────────────┐
│   User's Laptop/Server   │
│                          │
│  ┌────────────────────┐  │       ┌─────────────────────────┐
│  │ Python Agent       │  │       │  Iron Cage Runtime      │
│  │ (your_agent.py)    │◄─┼──────►│  (REST/gRPC API)        │
│  │                    │  │ HTTPS │                         │
│  │ - Business logic   │  │       │ - privacy protection         │
│  │ - LLM calls        │  │       │ - safety cutoffs      │
│  │ - Data processing  │  │       │ - Cost tracking         │
│  └────────────────────┘  │       │ - Audit logs            │
│                          │       └─────────────────────────┘
└──────────────────────────┘

Data Flow:
1. Agent calls Iron Cage API: "Check this prompt for PII"
2. Iron Cage validates → returns "safe" or "blocked"
3. Agent sends prompt to OpenAI (if safe)
4. Agent sends response to Iron Cage: "Track $0.03 cost"
5. Iron Cage updates metrics
```

**Use Cases:**
- Development/testing
- Batch workloads (run once, shut down)
- Data stays on user's infrastructure
- User controls when agent runs

### 6.2 Server Execution (Enterprise 24/7)

**Agent uploaded to Iron Cage, runs 24/7 in Iron Cage's infrastructure:**

```
┌─────────────────────────────────────────────────────────────┐
│               Iron Cage Runtime (K8s Cluster)                │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Agent Executor (PyO3 FFI)                               │ │
│  │                                                         │ │
│  │  ┌──────────────────┐  ┌──────────────────┐           │ │
│  │  │ Python Agent 1   │  │ Python Agent 2   │  ... (N)  │ │
│  │  │ (customer.py)    │  │ (customer2.py)   │           │ │
│  │  │                  │  │                  │           │ │
│  │  │ - Runs in        │  │ - Isolated       │           │ │
│  │  │   sandboxed env  │  │   namespace      │           │ │
│  │  └──────────────────┘  └──────────────────┘           │ │
│  │                                                         │ │
│  │  Safety guardrails, circuit breakers, cost tracking    │ │
│  │  applied automatically to all agents                   │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

Data Flow:
1. User uploads agent.py via API
2. Iron Cage validates code (no malware, resource limits)
3. Iron Cage starts agent in sandboxed container
4. Agent runs 24/7, all LLM calls proxied through Iron Cage
5. User monitors via control panel
```

**Use Cases:**
- Production 24/7 agents (customer support, monitoring)
- SaaS customers without infrastructure
- Maximum safety (all traffic monitored)
- Managed infrastructure (no ops required)

### 6.3 Execution Model Comparison

| Aspect | Local Execution | Server Execution |
|--------|----------------------|----------------------|
| **Agent Location** | User's laptop/server | Iron Cage K8s cluster |
| **Uptime** | User-controlled | 24/7 managed |
| **Data Residency** | User's infrastructure | Iron Cage infrastructure |
| **Safety Enforcement** | API calls (opt-in) | Automatic (proxied) |
| **Pricing** | Pay-per-API-call | Pay-per-agent-hour + API calls |
| **Setup Complexity** | Low (uv add) | Medium (upload agent code) |

---

## 7. Infrastructure Requirements

### 7.1 Hardware Requirements

### Local Development
- **CPU:** 4 cores (Intel/AMD x86_64 or ARM64)
- **RAM:** 8 GB (4 GB for Docker containers + 4 GB for OS)
- **Disk:** 20 GB SSD (10 GB for images + 10 GB for data)
- **Network:** 100 Mbps (for Docker image downloads)

### On-Premise Production (per node)
- **CPU:** 16-32 cores (x86_64, AVX2 support recommended)
- **RAM:** 64-128 GB (ECC recommended)
- **Disk:** 500 GB NVMe SSD (1 TB+ for high-throughput workloads)
- **Network:** 10 Gbps (low-latency networking for K8s)

### SaaS Production (AWS example)
- **Compute:** EC2 c6i.4xlarge (16 vCPU, 32 GB RAM) x 10-50 nodes
- **Database:** RDS db.r6g.2xlarge (8 vCPU, 64 GB RAM, Multi-AZ)
- **Cache:** ElastiCache r6g.xlarge (4 vCPU, 26 GB RAM, cluster mode)
- **Storage:** EBS gp3 (3000 IOPS, 125 MB/s throughput)

### 7.2 Software Requirements

### All Deployments
- **Operating System:** Linux (Ubuntu 22.04 LTS, RHEL 8+, or Debian 12)
- **Container Runtime:** Docker 24+ or containerd 1.6+
- **Orchestration:** Kubernetes 1.24+ (for production deployments)
- **TLS Certificates:** Let's Encrypt or commercial CA

### Development Only
- **Python:** 3.9+ (for running agents)
- **Rust:** 1.61+ (for building Iron Cage from source)

### 7.3 Network Requirements

| Port | Protocol | Component | Purpose |
|------|----------|-----------|---------|
| 8080 | HTTP/HTTPS | REST API + WebSocket | Agent communication |
| 50051 | gRPC | gRPC API | High-performance streaming |
| 6379 | TCP | Redis | Internal (cluster-only) |
| 5432 | TCP | PostgreSQL | Internal (cluster-only) |
| 9090 | HTTP | Prometheus | Metrics (internal/VPN) |
| 3000 | HTTP | Grafana | Control Panels (internal/VPN) |

**Firewall Rules:**
- Inbound: Allow 8080, 50051 from trusted sources only
- Outbound: Allow 443 (HTTPS) for LLM API calls (OpenAI, Anthropic, etc.)
- Internal: Allow all traffic within K8s cluster (pod-to-pod)

### 7.4 Sandboxing Requirements (Server-Side Agents)

For **server execution**, Iron Cage requires Linux kernel features to isolate and secure agent tool execution. These requirements ONLY apply when agents are uploaded to Iron Cage and run on Iron Cage infrastructure (not required for local execution).

### Kernel Features

**Required Linux Kernel Version:** 5.4+

**Required Kernel Features:**
- **cgroups v2:** Resource limits (CPU, memory, disk, processes)
- **seccomp-bpf:** Syscall filtering (whitelist/blacklist system calls)
- **Linux namespaces:** Network, PID, mount isolation
- **AppArmor or SELinux:** Mandatory access control (optional but recommended)

**Verify Kernel Support:**

```bash
# Check kernel version
uname -r
# Expected: 5.4.0 or higher

# Check cgroups v2 support
mount | grep cgroup2
# Expected: cgroup2 on /sys/fs/cgroup type cgroup2 (rw,nosuid,nodev,noexec,relatime)

# Check seccomp support
grep CONFIG_SECCOMP /boot/config-$(uname -r)
# Expected: CONFIG_SECCOMP=y

# Check namespace support
ls /proc/self/ns/
# Expected: cgroup ipc mnt net pid pid_for_children user uts

# Check AppArmor (Ubuntu/Debian)
aa-status
# Expected: apparmor module is loaded

# Or check SELinux (RHEL/CentOS)
getenforce
# Expected: Enforcing or Permissive
```

### Resource Limits (cgroups v2)

Iron Cage uses cgroups v2 to enforce per-tool resource limits:

| Resource | Default Limit | Configurable | Enforcement |
|----------|--------------|--------------|-------------|
| **CPU** | 2 cores (200%) | Yes (0.5-8 cores) | Throttled when exceeded |
| **Memory** | 1 GB | Yes (256 MB - 8 GB) | OOMKilled when exceeded |
| **Disk (tmp)** | 100 MB | Yes (10 MB - 1 GB) | Write fails when exceeded |
| **Processes** | 100 PIDs | Yes (10 - 500) | Fork fails when exceeded |
| **Execution Time** | 60 seconds | Yes (10s - 600s) | Killed when exceeded |

**cgroups Configuration (Kubernetes):**

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: iron_cage-runtime
spec:
  containers:
  - name: runtime
    image: iron_cage/runtime:latest
    resources:
      requests:
        cpu: "2"
        memory: "4Gi"
      limits:
        cpu: "4"
        memory: "8Gi"
    securityContext:
      capabilities:
        add:
        - SYS_ADMIN  # Required for cgroup management
        - SYS_RESOURCE  # Required for setrlimit
```

**Verify cgroups v2:**

```bash
# Check if cgroups v2 is mounted
mount | grep cgroup2

# Enable cgroups v2 if not mounted (Ubuntu 22.04+)
echo 'GRUB_CMDLINE_LINUX="systemd.unified_cgroup_hierarchy=1"' | \
  sudo tee -a /etc/default/grub
sudo update-grub
sudo reboot

# Create test cgroup
sudo mkdir -p /sys/fs/cgroup/iron_cage-test
echo "200000" | sudo tee /sys/fs/cgroup/iron_cage-test/cpu.max
# Expected: 200000 (= 2 cores @ 100000 microseconds per 100ms)
```

### Syscall Filtering (seccomp-bpf)

Iron Cage uses seccomp to whitelist safe syscalls and block dangerous ones:

**Whitelisted Syscalls (Safe):**
- `read`, `write`, `open`, `close`, `stat`, `fstat`, `lstat`
- `mmap`, `munmap`, `brk`, `mprotect`
- `futex`, `nanosleep`, `getpid`, `gettid`
- `socket`, `connect`, `send`, `recv` (if network allowed)

**Blocked Syscalls (Dangerous):**
- `exec`, `execve`, `execveat` - Execute arbitrary binaries (code injection)
- `fork`, `clone`, `vfork` - Create processes (fork bombs)
- `chroot`, `pivot_root` - Escape sandbox
- `mount`, `umount`, `unshare` - Privilege escalation
- `reboot`, `kexec_load` - System DoS
- `ptrace` - Attach debugger (escape sandbox)
- `bpf`, `perf_event_open` - Kernel exploitation

**seccomp Policy Example:**

```json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "architectures": ["SCMP_ARCH_X86_64"],
  "syscalls": [
    {
      "names": ["read", "write", "open", "close", "stat", "fstat"],
      "action": "SCMP_ACT_ALLOW"
    },
    {
      "names": ["mmap", "munmap", "brk", "mprotect"],
      "action": "SCMP_ACT_ALLOW"
    },
    {
      "names": ["exec", "execve", "fork", "clone", "chroot"],
      "action": "SCMP_ACT_KILL"
    }
  ]
}
```

**Test seccomp:**

```bash
# Check if seccomp is enabled
grep Seccomp /proc/self/status
# Expected: Seccomp: 0 (disabled) or 2 (enabled)

# Test with Docker (blocks fork)
docker run --rm -it --security-opt seccomp=seccomp-policy.json ubuntu:22.04 bash
# Inside container:
bash
# Expected: fork() error if fork is blocked
```

### Network Isolation

Iron Cage uses Linux network namespaces to isolate agent network access:

**Network Policies:**
- **Isolated (default):** No network access (local loopback only)
- **Whitelisted Domains:** Only allowed domains (e.g., `api.openai.com`)
- **Full Access:** All domains (development/testing only)

**Network Namespace Configuration:**

```bash
# Create isolated network namespace
sudo ip netns add iron_cage-sandbox

# Verify isolation (no network interfaces except loopback)
sudo ip netns exec iron_cage-sandbox ip addr
# Expected: Only lo (loopback)

# Add veth pair for whitelisted access
sudo ip link add veth0 type veth peer name veth1
sudo ip link set veth1 netns iron_cage-sandbox
sudo ip netns exec iron_cage-sandbox ip addr add 10.200.1.2/24 dev veth1
sudo ip netns exec iron_cage-sandbox ip link set veth1 up

# Add iptables rules for domain whitelist
sudo iptables -A OUTPUT -d api.openai.com -j ACCEPT
sudo iptables -A OUTPUT -j DROP
```

**Kubernetes NetworkPolicy:**

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: iron_cage-sandbox-policy
  namespace: iron_cage
spec:
  podSelector:
    matchLabels:
      app: iron_cage-runtime
  policyTypes:
  - Egress
  egress:
  # Allow DNS
  - to:
    - namespaceSelector:
        matchLabels:
          name: kube-system
    ports:
    - protocol: UDP
      port: 53

  # Allow LLM APIs only
  - to:
    - podSelector: {}
    ports:
    - protocol: TCP
      port: 443
    # Whitelist IPs (OpenAI, Anthropic)
    # Note: Use external DNS or service mesh for domain-based filtering
```

### Sandboxing Tools

**Recommended Tools:**

1. **gVisor (runsc):** Lightweight container sandbox
   - Intercepts syscalls in userspace
   - Stronger isolation than seccomp alone
   - Compatible with Kubernetes (RuntimeClass)

2. **Kata Containers:** VM-based sandboxing
   - Each container runs in dedicated micro-VM
   - Hardware-level isolation
   - Higher overhead but maximum security

3. **Firecracker:** Lightweight micro-VMs (AWS Lambda uses this)
   - 125ms cold start
   - 5 MB memory overhead per VM
   - Strong isolation

**gVisor Installation (Kubernetes):**

```bash
# Install gVisor runtime
wget https://storage.googleapis.com/gvisor/releases/release/latest/x86_64/runsc
chmod +x runsc
sudo mv runsc /usr/local/bin/

# Configure containerd to use gVisor
cat <<EOF | sudo tee -a /etc/containerd/config.toml
[plugins."io.containerd.grpc.v1.cri".containerd.runtimes.runsc]
  runtime_type = "io.containerd.runsc.v1"
EOF

sudo systemctl restart containerd

# Create RuntimeClass
cat <<EOF | kubectl apply -f -
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: gvisor
handler: runsc
EOF

# Use in deployment
apiVersion: v1
kind: Pod
spec:
  runtimeClassName: gvisor  # Use gVisor for this pod
  containers:
  - name: iron_cage-runtime
    image: iron_cage/runtime:latest
```

### Monitoring and Alerting

**Sandbox Violations to Monitor:**

| Violation | Alert Threshold | Action |
|-----------|----------------|--------|
| **Memory Limit Exceeded** | Any OOMKill | Alert + suspend agent |
| **CPU Quota Exceeded** | Sustained >100% | Alert + throttle |
| **Forbidden Syscall** | Any attempt | Alert + kill immediately |
| **Network Violation** | Unauthorized domain | Alert + block |
| **Disk Quota Exceeded** | >90% of quota | Warning |

**Prometheus Metrics:**

```yaml
# Sandbox violation metrics
sandbox_violations_total{agent_id, violation_type}
sandbox_oom_kills_total{agent_id}
sandbox_cpu_throttle_seconds_total{agent_id}
sandbox_syscall_denials_total{agent_id, syscall}
sandbox_network_blocks_total{agent_id, domain}
```

**Alert Example (Prometheus):**

```yaml
groups:
- name: iron_cage_sandbox
  rules:
  - alert: SandboxOOMKill
    expr: rate(sandbox_oom_kills_total[5m]) > 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Agent {{ $labels.agent_id }} killed due to memory limit"

  - alert: ForbiddenSyscallAttempt
    expr: rate(sandbox_syscall_denials_total{syscall="exec"}[1m]) > 0
    for: 0s
    labels:
      severity: critical
    annotations:
      summary: "Agent {{ $labels.agent_id }} attempted forbidden syscall: {{ $labels.syscall }}"
```

### Compliance Requirements

**Sandboxing for Compliance:**

| Standard | Requirement | Iron Cage Implementation |
|----------|-------------|--------------------------|
| **SOC 2** | Multi-tenant isolation | cgroups + network namespaces |
| **HIPAA** | PHI data isolation | Separate cgroups per tenant |
| **PCI-DSS** | Cardholder data isolation | Encrypted volumes + sandboxing |
| **GDPR** | Data processing isolation | Per-tenant sandboxes |

**Audit Logging:**

All sandbox violations must be logged:

```json
{
  "event_type": "sandbox_violation",
  "timestamp": "2025-01-17T10:34:52Z",
  "agent_id": "agent-12345",
  "tenant_id": "tenant-67890",
  "violation_type": "memory_limit_exceeded",
  "limit": "1 GB",
  "actual_usage": "1.2 GB",
  "action_taken": "OOMKilled",
  "alert_sent": true
}
```

---

## 8. Network Architecture

### 8.1 Local Development Network Flow

```
┌────────────────┐
│ Developer's    │
│ Laptop         │
│                │
│  localhost:8080├──┐
└────────────────┘  │
                    │ Docker bridge network (172.18.0.0/16)
┌───────────────────┼────────────────────────────────────────┐
│                   │                                         │
│  ┌────────────────▼──────────┐                             │
│  │ iron_cage-runtime         │                             │
│  │ IP: 172.18.0.2            │                             │
│  └───┬────────────────────┬──┘                             │
│      │                    │                                │
│  ┌───▼──────────┐    ┌────▼─────────┐                     │
│  │ redis        │    │ postgres     │                     │
│  │ 172.18.0.3   │    │ 172.18.0.4   │                     │
│  └──────────────┘    └──────────────┘                     │
└────────────────────────────────────────────────────────────┘
```

### 8.2 On-Premise Enterprise Network Flow

```
┌─────────────────────────────────────────────────────────────┐
│                      CORPORATE NETWORK                       │
│                                                              │
│  ┌────────────────────┐                                     │
│  │ Corporate Firewall │                                     │
│  │ (only port 443)    │                                     │
│  └────────┬───────────┘                                     │
│           │                                                  │
│  ┌────────▼──────────────────────────────────────────────┐  │
│  │  Kubernetes Cluster (10.0.0.0/16)                     │  │
│  │                                                        │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │ Ingress Controller (10.0.1.10)                   │  │  │
│  │  │ - TLS termination                                │  │  │
│  │  │ - mTLS client cert validation                    │  │  │
│  │  └───────────┬─────────────────────────────────────┘  │  │
│  │              │                                          │  │
│  │  ┌───────────▼─────────────────────────────────────┐  │  │
│  │  │ iron_cage-runtime (Service: 10.0.2.100)         │  │  │
│  │  │ - Pods: 10.0.2.1, 10.0.2.2, 10.0.2.3            │  │  │
│  │  └───┬───────────────────────┬─────────────────────┘  │  │
│  │      │                       │                         │  │
│  │  ┌───▼─────────────┐    ┌────▼──────────────┐         │  │
│  │  │ redis           │    │ postgres          │         │  │
│  │  │ (10.0.3.100)    │    │ (10.0.4.100)      │         │  │
│  │  └─────────────────┘    └───────────────────┘         │  │
│  └────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 8.3 SaaS Multi-Tenant Network Flow

```
┌─────────────────────────────────────────────────────────────┐
│                         INTERNET                             │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│  Application Load Balancer (AWS ALB / GCP Load Balancer)    │
│  - DDoS protection                                           │
│  - Rate limiting per tenant                                  │
│  - TLS termination                                           │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                  Kubernetes Cluster (VPC)                    │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  iron_cage-runtime (10 replicas)                       │ │
│  │  - Each pod handles multiple tenants                   │ │
│  │  - Tenant ID extracted from JWT                        │ │
│  └──────┬─────────────────────────┬───────────────────────┘ │
│         │                         │                          │
│  ┌──────▼──────────────┐   ┌──────▼──────────────┐          │
│  │ ElastiCache         │   │ RDS PostgreSQL      │          │
│  │ (Redis cluster)     │   │ (Multi-AZ)          │          │
│  │ - Tenant key prefix │   │ - Tenant schema     │          │
│  └─────────────────────┘   └─────────────────────┘          │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  CloudWatch / Stackdriver                              │ │
│  │  - Per-tenant metrics                                  │ │
│  │  - Cost tracking                                       │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2025-01-17 | Initial deployment guide created with 3 deployment modes, K8s manifests, network diagrams |
